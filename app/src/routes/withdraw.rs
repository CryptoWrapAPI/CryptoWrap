use crate::AppState;
use crate::entity::prelude::*;
use crate::entity::{tokens, withdrawals};
use crate::wallet::litecoin as litecoin_wallet_module;
use crate::wallet::litecoin_helper;
use crate::wallet::monero_helper;
use axum::extract::State;
use axum::response::Json;
use axum::{Router, routing::post};
use axum_extra::extract::cookie::PrivateCookieJar;
use chrono::Utc;
use hyper::StatusCode;
use rust_decimal::Decimal;
use sea_orm::{ActiveModelTrait, EntityTrait, Set};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct WithdrawRequest {
    coin_id: String,
    destination_address: String,
    amount: Decimal,
    auth_token: String,
    // coin_symbol: String,
}

#[derive(Debug, Serialize)]
pub struct WithdrawResponse {
    success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    transaction_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
}

#[derive(Debug, Serialize)]
struct ErrorResponse {
    error: String,
}

async fn create_withdraw(
    State(state): State<AppState>,
    jar: PrivateCookieJar,
    Json(req): Json<WithdrawRequest>,
) -> Result<Json<WithdrawResponse>, (StatusCode, Json<ErrorResponse>)> {
    let token_entry = get_authenticated_token(&state, &jar).await?;

    validate_auth_token(&state, &req.auth_token, &token_entry).await?;

    let coin_id = req.coin_id; // Bitcoin
    let destination_address = req.destination_address;
    let amount = req.amount;
    // let coin_symbol = req.coin_symbol; // BTC

    let resulted_tx_id; // only broadcasted/relayed txs are going in the withdrawals db table
    match coin_id.as_str() {
        "monero" => {
            let monero_account_index = token_entry.monero_major_index.ok_or_else(|| {
                (
                    StatusCode::BAD_REQUEST,
                    Json(ErrorResponse {
                        error: "Monero wallet not set up for this account".to_string(),
                    }),
                )
            })? as u32;

            let amount_str = amount.to_string();
            let amount_atomic = monero_helper::xmr_to_piconero(&amount_str).map_err(|e| {
                (
                    StatusCode::BAD_REQUEST,
                    Json(ErrorResponse {
                        error: format!("Invalid amount: {e}"),
                    }),
                )
            })?;

            // default fee is x4 weight/size (base fee) multiplier (e.g. Normal)
            // there is also x1 (unimportant) Low fee
            let tx_hash = monero_helper::transfer_xmr(
                &state.monero_wallet,
                &destination_address,
                amount_atomic,
                monero_account_index,
            )
            .await
            .map_err(|e| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ErrorResponse {
                        error: format!("Monero transfer failed: {e}"),
                    }),
                )
            })?;

            let withdrawal = withdrawals::ActiveModel {
                transaction_id: Set(Uuid::new_v4()),
                user_uuid: Set(token_entry.id),
                amount: Set(amount_str),
                coin_id: Set("monero".to_string()),
                destination_address: Set(destination_address),
                created_at: Set(Utc::now().naive_utc()),
                transaction_hash: Set(tx_hash.clone()),
            };

            withdrawal.insert(&state.conn).await.map_err(|e| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ErrorResponse {
                        error: format!("Failed to save withdrawal: {e}"),
                    }),
                )
            })?;

            resulted_tx_id = tx_hash;
        }
        "litecoin" => {
            let ltc_account_index = token_entry.litecoin_account_index.ok_or_else(|| {
                (
                    StatusCode::BAD_REQUEST,
                    Json(ErrorResponse {
                        error: "Litecoin wallet not set up for this account".to_string(),
                    }),
                )
            })? as u32;

            let amount_str = amount.to_string();
            let amount_litoshi = litecoin_wallet_module::ltc_to_litoshi(&amount_str).map_err(|e| {
                (
                    StatusCode::BAD_REQUEST,
                    Json(ErrorResponse {
                        error: format!("Invalid amount: {e}"),
                    }),
                )
            })?;

            let tx_hash = litecoin_helper::transfer_ltc(
                &state.litecoin_wallet,
                &destination_address,
                amount_litoshi,
                ltc_account_index,
                &state.conn,
            )
            .await
            .map_err(|e| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ErrorResponse {
                        error: format!("Litecoin transfer failed: {e}"),
                    }),
                )
            })?;

            let withdrawal = withdrawals::ActiveModel {
                transaction_id: Set(Uuid::new_v4()),
                user_uuid: Set(token_entry.id),
                amount: Set(amount_str),
                coin_id: Set("litecoin".to_string()),
                destination_address: Set(destination_address),
                created_at: Set(Utc::now().naive_utc()),
                transaction_hash: Set(tx_hash.clone()),
            };

            withdrawal.insert(&state.conn).await.map_err(|e| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ErrorResponse {
                        error: format!("Failed to save withdrawal: {e}"),
                    }),
                )
            })?;

            resulted_tx_id = tx_hash;
        }
        _ => {
            return Err((
                StatusCode::NOT_IMPLEMENTED,
                Json(ErrorResponse {
                    error: "Invalid coin ID".to_string(),
                }),
            ));
        }
    }

    Ok(Json(WithdrawResponse {
        success: true,
        transaction_id: Some(resulted_tx_id),
        error: None,
    }))
}

async fn get_authenticated_token(
    state: &AppState,
    jar: &PrivateCookieJar,
) -> Result<tokens::Model, (StatusCode, Json<ErrorResponse>)> {
    let user_id = jar.get("auth").ok_or_else(|| {
        (
            StatusCode::FORBIDDEN,
            Json(ErrorResponse {
                error: "Authentication required".to_string(),
            }),
        )
    })?;

    let token_id = user_id.value().parse::<Uuid>().map_err(|_| {
        (
            StatusCode::FORBIDDEN,
            Json(ErrorResponse {
                error: "Invalid token format".to_string(),
            }),
        )
    })?;

    Tokens::find_by_id(token_id)
        .one(&state.conn)
        .await
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: "Database error".to_string(),
                }),
            )
        })?
        .ok_or_else(|| {
            (
                StatusCode::FORBIDDEN,
                Json(ErrorResponse {
                    error: "Invalid token".to_string(),
                }),
            )
        })
}

async fn validate_auth_token(
    state: &AppState,
    auth_token: &str,
    token_entry: &tokens::Model,
) -> Result<(), (StatusCode, Json<ErrorResponse>)> {
    let token_without_prefix = auth_token
        .strip_prefix(&state.token_prefix)
        .unwrap_or(auth_token);

    let pepper_bytes = state.blake3_hash_token_pepper.as_bytes();
    let mut key = [0u8; 32];
    let copy_len = pepper_bytes.len().min(32);
    key[..copy_len].copy_from_slice(&pepper_bytes[..copy_len]);
    let token_hash = blake3::keyed_hash(&key, token_without_prefix.as_bytes());
    let token_hash_hex = token_hash.to_hex().to_string();

    if token_hash_hex != token_entry.token_hash {
        return Err((
            StatusCode::FORBIDDEN,
            Json(ErrorResponse {
                error: "Invalid confirmation token".to_string(),
            }),
        ));
    }

    Ok(())
}

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/api/withdraw", post(create_withdraw))
        .with_state(state)
}
