use crate::AppState;
use crate::PAYMENT_TAG;
use crate::routes::auth_helper::extract_api_key;
use axum::{Json, extract::Query, extract::State, http::HeaderMap, http::StatusCode};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use utoipa_axum::{router::OpenApiRouter, routes};
use uuid::Uuid;

#[derive(Serialize, ToSchema)]
pub struct InvoiceResponse {
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
pub struct InvoiceRequest {
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
        (status = 200, description = "Invoice created successfully", body = InvoiceResponse),
        (status = 401, description = "Token is missing or invalid"),
    )
)]
pub async fn create_invoice(
    // checkout / bill
    state: State<AppState>,
    headers: HeaderMap,
    Json(invoice_request): Json<InvoiceRequest>,
) -> Result<Json<InvoiceResponse>, StatusCode> {
    let _token_id = extract_api_key(&state, &headers)
        .await
        .ok_or(StatusCode::UNAUTHORIZED)?;

    let mock_invoice = Uuid::new_v4();

    Ok(Json(InvoiceResponse {
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
    // Spendable ?
}

#[derive(Serialize, ToSchema)]
pub struct CheckInvoiceResponse {
    #[schema(value_type = String)]
    pub invoice_uuid: Uuid,
    pub wallet_address: String,
    pub amount_requested: String,
    pub payment_status: PaymentStatus,
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
    OpenApiRouter::new().routes(routes!(create_invoice))
}
