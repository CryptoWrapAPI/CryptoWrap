use crate::AppState;
use crate::entity::tokens;
use axum::extract::State;
use axum::response::Json;
use axum::{Router, routing::post};
use axum_extra::extract::cookie::PrivateCookieJar;
use hyper::StatusCode;
use sea_orm::EntityTrait;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct WithdrawRequest {
    coin_id: String,
    destination_address: String,
    amount: f64,
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

    // TODO: implement real withdrawal logic
    // let mock_tx_id = Uuid::new_v4().to_string();

    let resulted_tx_id; // only broadcasted/relayed txs are going in the withdrawals db table

    // separate my coin_id ***
    match coin_id.as_str() {
        "monero" => {
            // XMR ___
            // Monero withdrawal logic

            resulted_tx_id = Uuid::new_v4().to_string();
        }
        "litecoin" => {
            // LTC ---
            // Litecoin withdrawal logic

            resulted_tx_id = Uuid::new_v4().to_string();
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

    tokens::Entity::find_by_id(token_id)
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
