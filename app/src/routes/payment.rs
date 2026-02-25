use crate::AppState;
use crate::PAYMENT_TAG;
use crate::routes::auth_helper::extract_api_key;
use axum::{Json, extract::Query, extract::State, http::HeaderMap, http::StatusCode};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use utoipa_axum::{router::OpenApiRouter, routes};
use uuid::Uuid;

#[derive(Serialize, ToSchema)]
pub struct CreateInvoiceResponse {
    #[schema(value_type = String)]
    pub invoice_uuid: Uuid,
    pub wallet_address: String,
    pub amount_requested: String,
    pub currency: Currency,
}

#[derive(Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "UPPERCASE")]
pub enum Currency {
    XMR,
    // BTC , // add later
}

#[derive(Deserialize, ToSchema)]
pub struct CreateInvoiceRequest {
    pub amount: String, // 0.15
    pub currency: Currency, // XMR / xmr // cryptocurrency / e.g. coin ,.,.,.////
                        // pub network: Option<String>, // MONERO / Monero / monero
                        // ^ let's use lowercase only
}

/// Create an invoice
///
/// Returns an invoice UUID to check the specified payment amount.
#[utoipa::path(
    post,
    path = "/create_invoice",
    tag = PAYMENT_TAG,
    security(("api_key" = [])),
    responses(
        (status = 200, description = "Invoice created successfully", body = CreateInvoiceResponse),
        (status = 401, description = "Token is missing or invalid"),
    )
)]
pub async fn create_invoice(
    // checkout / bill
    state: State<AppState>,
    headers: HeaderMap,
    Json(invoice_request): Json<CreateInvoiceRequest>,
) -> Result<Json<CreateInvoiceResponse>, StatusCode> {
    let _token_id = extract_api_key(&state, &headers)
        .await
        .ok_or(StatusCode::UNAUTHORIZED)?;

    let mock_invoice = Uuid::new_v4();

    Ok(Json(CreateInvoiceResponse {
        invoice_uuid: mock_invoice,
        wallet_address: "...".to_string(),
        amount_requested: "0.15".to_string(),
        currency: invoice_request.currency,
    }))
}

#[derive(Deserialize, ToSchema)]
pub struct CheckInvoiceRequest {
    #[schema(value_type = String)]
    pub invoice_uuid: Uuid,
}

#[derive(Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum PaymentStatus {
    Waiting,
    Detected, //Mempool tx / 0-conf
    Confirmed,
    Expired,
    // Failed ?
    // Spendable (?)
}

#[derive(Serialize, ToSchema)]
pub struct CheckInvoiceResponse {
    #[schema(value_type = String)]
    pub invoice_uuid: Uuid,
    pub wallet_address: String,
    pub amount_requested: String,
    pub amount_received: String,
    pub payment_status: PaymentStatus, // payment status changes on detected if amount needed to fulfill request is received in mempool (or at least one in mempool), and it changed to confirmed when all of transactions required to fulfill request has at least 1 confirmation
    pub confirmations: Option<u32>, // if multiple transactions received, this will show lowest confirmations count
    // ^ user can check lowest confirmation to be able to release digital goods to only when it's completely safe
    // pub recommended_confirmations: u32,
    pub transactions: Vec<String>, // list of transaction hashes (in monero), tx id (btc), etc
                                   // pub expiry_datetime: String, // 1 hour since last check
}

/// Check invoice
///
/// Returns invoice payment status.
#[utoipa::path(
    get,
    path = "/check_invoice",
    tag = PAYMENT_TAG,
    responses(
        (status = 200, description = "Invoice information", body = CheckInvoiceResponse),
        (status = 404, description = "Invoice not found"),
    )
)]
pub async fn check_invoice(
    // checkout / bill
    // receipt
    state: State<AppState>,
    Query(invoice_request): Query<CheckInvoiceRequest>,
) -> Result<Json<CheckInvoiceResponse>, StatusCode> {
    Ok(Json(CheckInvoiceResponse {
        invoice_uuid: Uuid::new_v4(),
        wallet_address: "mock wallet address".to_string(),
        amount_requested: "0.30".to_string(),
        payment_status: PaymentStatus::Waiting,
    }))
}

pub fn router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new().routes(routes!(create_invoice, check_invoice))
}
