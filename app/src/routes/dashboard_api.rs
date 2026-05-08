use crate::AppState;
use crate::entity::tokens;
use crate::wallet::litecoin_helper;
use crate::wallet::monero as monero_wallet;
// use crate::wallet::litecoin as litecoin_wallet;
use crate::wallet::monero_helper;
use axum::extract::{Query, State};
use axum::response::Json;
use axum::{Router, routing::get};
use axum_extra::extract::cookie::PrivateCookieJar;
use hyper::StatusCode;
use sea_orm::EntityTrait;
use serde::{Deserialize, Serialize};
// use std::slice::from_ref;
use uuid::Uuid;

#[derive(Debug, Deserialize)]
struct GetBalanceParams {
    asset: String,
}

#[derive(Debug, Serialize)]
struct BalanceResponse {
    asset: String,
    balance: String,
}

#[derive(Debug, Serialize)]
struct ErrorResponse {
    error: String,
}

async fn get_balance(
    Query(params): Query<GetBalanceParams>,
    State(state): State<AppState>,
    jar: PrivateCookieJar,
) -> Result<Json<BalanceResponse>, (StatusCode, Json<ErrorResponse>)> {
    // Check for auth cookie

    let user_entry;

    if let Some(user_id) = jar.get("auth") {
        let token_id_str = user_id.value();

        match token_id_str.parse::<Uuid>() {
            Ok(token_id) => {
                match tokens::Entity::find_by_id(token_id).one(&state.conn).await {
                    Ok(Some(token)) => {
                        // User identified successfully
                        // Continue to balance fetching
                        user_entry = token
                    }
                    Ok(None) => {
                        // Token not found in database
                        return Err((
                            StatusCode::FORBIDDEN,
                            Json(ErrorResponse {
                                error: "Invalid token".to_string(),
                            }),
                        ));
                    }
                    Err(_) => {
                        return Err((
                            StatusCode::INTERNAL_SERVER_ERROR,
                            Json(ErrorResponse {
                                error: "Database error".to_string(),
                            }),
                        ));
                    }
                }
            }
            Err(_) => {
                // Invalid UUID format
                return Err((
                    StatusCode::FORBIDDEN,
                    Json(ErrorResponse {
                        error: "Invalid token format".to_string(),
                    }),
                ));
            }
        }
    } else {
        // No auth cookie
        return Err((
            StatusCode::FORBIDDEN,
            Json(ErrorResponse {
                error: "Authentication required".to_string(),
            }),
        ));
    }
    // If we get here, user is authenticated

    // Mock implementation - replace with actual balance fetching logic
    // In a real implementation, you would fetch the balance for this specific user (user_token)
    // let mock_balances = std::collections::HashMap::from([
    //     ("monero".to_string(), 1.5432),
    //     ("litecoin".to_string(), 0.8765),
    // ]);

    // let balance = mock_balances
    //     .get(&params.asset.to_lowercase())
    //     .copied()
    //     .unwrap_or(0.0);

    let balance;

    // filter by asset name
    match &params.asset.to_lowercase() {
        s if s == "monero" => {
            // balance = 1.337;
            let user_major_index = monero_helper::ensure_monero_major_wallet_index_for_user(
                &user_entry,
                &state.monero_wallet,
                &state.conn,
            )
            .await
            .expect("Monero wallet error");

            let balance_in_piconero =
                monero_wallet::get_account_balance(&state.monero_wallet, user_major_index)
                    .await
                    .expect("Failed to get Monero balance for account")
                    .unlocked_balance;
            balance = monero_helper::piconero_to_xmr_string(balance_in_piconero, true);
        }
        s if s == "litecoin" => {
            balance = "420.69".to_string();
            let ltc_acc_index = litecoin_helper::ensure_litecoin_account_index_for_user(
                &user_entry,
                &state.litecoin_wallet,
                &state.conn,
            )
            .await
            .expect("Litecoin wallet error");

            // here we need to gather all uspent addresses (with UTXO/s) + change address
            //
            // sql query to get litecoin_wallet addresses with keep_track (N) - true and is_change - true (1)
            // list of addresses send to get_balance(<here>)
            //
            // but exclude is_available? (because new payments detected by confirmed address balance)
            //  --- finalizer won't re-use (set free - is_available to true) if keep_track is true
            //  > this means after spending coins (outgoing tx) -> keep_track will set to false as well as is_available
            //

            // let balance_in_litoshi = &state.litecoin_wallet.get_balance()
        }
        _ => {
            balance = "0.0".to_string();
        }
    };

    Ok(Json(BalanceResponse {
        asset: params.asset,
        balance,
    }))
}

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/api/dashboard/balance", get(get_balance))
        .with_state(state)
}
