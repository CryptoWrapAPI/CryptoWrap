use crate::AppState;
use crate::entity::tokens;
use axum::extract::{Query, State};
use axum::response::Json;
use axum::{Router, routing::get};
use axum_extra::extract::cookie::PrivateCookieJar;
use hyper::StatusCode;
use sea_orm::EntityTrait;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Deserialize)]
struct GetBalanceParams {
    asset: String,
}

#[derive(Debug, Serialize)]
struct BalanceResponse {
    asset: String,
    balance: f64,
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
    if let Some(user_id) = jar.get("auth") {
        let token_id_str = user_id.value();

        match token_id_str.parse::<Uuid>() {
            Ok(token_id) => {
                match tokens::Entity::find_by_id(token_id).one(&state.conn).await {
                    Ok(Some(_token)) => {
                        // User identified successfully
                        // Continue to balance fetching
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
    let mock_balances = std::collections::HashMap::from([
        ("monero".to_string(), 1.5432),
        ("litecoin".to_string(), 0.8765),
    ]);

    let balance = mock_balances
        .get(&params.asset.to_lowercase())
        .copied()
        .unwrap_or(0.0);

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
