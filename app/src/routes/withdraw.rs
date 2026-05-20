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
    // coin_symbol: String,
    destination_address: String,
    amount: f64,
    // auth_token: String,
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
    if let Some(user_id) = jar.get("auth") {
        let token_id_str = user_id.value();
        match token_id_str.parse::<Uuid>() {
            Ok(token_id) => {
                match tokens::Entity::find_by_id(token_id).one(&state.conn).await {
                    Ok(Some(_token)) => {}
                    Ok(None) => {
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
                return Err((
                    StatusCode::FORBIDDEN,
                    Json(ErrorResponse {
                        error: "Invalid token format".to_string(),
                    }),
                ));
            }
        }
    } else {
        return Err((
            StatusCode::FORBIDDEN,
            Json(ErrorResponse {
                error: "Authentication required".to_string(),
            }),
        ));
    }

    let _coin_id = req.coin_id;
    let _destination_address = req.destination_address;
    let _amount = req.amount;

    // TODO: implement real withdrawal logic
    // For now, mock success
    let mock_tx_id = Uuid::new_v4().to_string();

    Ok(Json(WithdrawResponse {
        success: true,
        transaction_id: Some(mock_tx_id),
        error: None,
    }))
}

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/api/withdraw", post(create_withdraw))
        .with_state(state)
}
