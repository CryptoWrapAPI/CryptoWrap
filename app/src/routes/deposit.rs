use crate::AppState;
use crate::PAYMENT_TAG;
use crate::entity::deposits;
use crate::routes::auth_helper::extract_api_key;
use axum::{Json, extract::Query, extract::State, http::HeaderMap, http::StatusCode};
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set};
use serde::{Deserialize, Serialize};
use strum_macros::Display;
use utoipa::ToSchema;
use utoipa_axum::{router::OpenApiRouter, routes};
use uuid::Uuid;

/// Create a deposit address for selected coin and (optionally) network
///
/// Returns assigned wallet address with UUID for internal tracking.
#[utoipa::path(
    post,
    path = "/create",
    tag = PAYMENT_TAG,
    security(("api_key" = [])),
    responses(
        (status = 200, description = "Deposit address generated successfully", body = CreateDepositResponse),
        (status = 401, description = "Token is missing or invalid"),
    )
)]
pub async fn create(
    state: State<AppState>,
    headers: HeaderMap,
    Json(deposit_request): Json<CreateDepositRequest>,
) -> Result<Json<CreateDepositResponse>, StatusCode> {
    let _token_id = extract_api_key(&state, &headers)
        .await
        .ok_or(StatusCode::UNAUTHORIZED)?;

    let network = if let Some(net) = deposit_request.network {
        net.to_string()
    } else {
        match deposit_request.currency {
            Currency::XMR => Network::Monero.to_string(),
        }
    };

    let deposit = deposits::ActiveModel {
        currency: Set(deposit_request.currency.to_string()),
        network: Set(network),
        payment_status: Set(DepositStatus::Waiting.to_string()),
        wallet_address: Set("someaddr".to_string()),
        ..Default::default()
    };
    let deposit = deposit.insert(&state.conn).await.unwrap();
    let deposit_uuid = deposit.deposit_id;

    let wallet_address = "...".to_string();

    Ok(Json(CreateDepositResponse {
        deposit_uuid,
        wallet_address,
        currency: deposit_request.currency,
    }))
}

/// Check deposit status
///
/// Returns deposit status.
#[utoipa::path(
    get,
    path = "/check",
    tag = PAYMENT_TAG,
    params(
        ("deposit_uuid" = String, Query, description = "UUID of the deposit to check")
    ),
    responses(
        (status = 200, description = "Deposit request information", body = CheckDepositResponse),
        (status = 404, description = "Deposit request not found"),
    )
)]
pub async fn check(
    state: State<AppState>,
    Query(deposit_request): Query<CheckDepositRequest>,
) -> Result<Json<CheckDepositResponse>, StatusCode> {
    Ok(Json(CheckDepositResponse {
        deposit_uuid: Uuid::new_v4(),
        wallet_address: "mock wallet address".to_string(),
        amount_received: "3.321".to_string(),
        payment_status: DepositStatus::Waiting,
        confirmations: None,
        txid: None,
    }))
}

pub fn router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new().routes(routes!(create, check))
}

// txid = transaction id (e.g. transaction identificator or hash from blockchain) /

#[derive(Deserialize, ToSchema)]
pub struct CheckDepositRequest {
    #[schema(value_type = String)]
    pub deposit_uuid: Uuid,
}

#[derive(Serialize, Deserialize, ToSchema, Display)]
#[serde(rename_all = "lowercase")]
pub enum DepositStatus {
    Waiting,
    Detected,
    Confirmed,
    Expired,
}

#[derive(Serialize, Deserialize, ToSchema)]
#[schema(example = json!({
    "deposit_uuid":"3f270a5a-50be-4ad7-9f01-fffc2c5144b3",
    "wallet_address":"46QYvqx4Z8JKk26DVyNbFjMgFqXyrXgAb3W8kEHBiSN78XrcoPRHk4ATjoCJ9eia5MVQMxDdQ6nAaa2D9MgLgZV31V2bCRS",
    "amount_received":"0",
    "payment_status":"waiting",
    "confirmations":Option::<u32>::None,
    "txid":Option::<String>::None,     // None String
}))]

pub struct CheckDepositResponse {
    #[schema(value_type = String)]
    pub deposit_uuid: Uuid,
    pub wallet_address: String,
    pub amount_received: String,
    pub payment_status: DepositStatus,
    pub confirmations: Option<u32>,
    pub txid: Option<String>,
}

#[derive(Serialize, ToSchema)]
#[schema(example = json!({
    "deposit_uuid":"3f270a5a-50be-4ad7-9f01-fffc2c5144b3",
    "wallet_address":"46QYvqx4Z8JKk26DVyNbFjMgFqXyrXgAb3W8kEHBiSN78XrcoPRHk4ATjoCJ9eia5MVQMxDdQ6nAaa2D9MgLgZV31V2bCRS",
    "currency":"XMR",
}))]
pub struct CreateDepositResponse {
    #[schema(value_type = String)]
    pub deposit_uuid: Uuid,
    pub wallet_address: String,
    pub currency: Currency,
}

#[derive(Serialize, Deserialize, ToSchema, Display)]
#[serde(rename_all = "UPPERCASE")]
pub enum Currency {
    XMR,
}

#[derive(Serialize, Deserialize, ToSchema, Display)]
#[serde(rename_all = "UPPERCASE")]
pub enum Network {
    Monero,
}

#[derive(Deserialize, ToSchema)]
#[schema(example = json!({
    "currency":"XMR",
}))]
pub struct CreateDepositRequest {
    pub currency: Currency,
    pub network: Option<Network>,
}
