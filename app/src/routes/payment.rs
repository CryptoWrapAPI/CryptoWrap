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

    // rate limiting
    // no more than 1 invoice per second/minutes and no more than 1000 invoices per day (hard limit, can be unjusted in future)
    // maybe add new table for stateless rrrate limiting
    // (or don't apply rate limiting ? maybe at least for daily or hourly invoices limit, for example 1000 invoices per hour)
    // returning error if it's violated
    //
    // ^ rate limit is applied to a single token

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

#[derive(Serialize, Deserialize, ToSchema)]
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
    params(
        ("invoice_uuid" = String, Query, description = "UUID of the invoice to check")
    ),
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
    // here should be rate-limiting logic
    // don't query monero-wallet-rpc if request is received less than 2 minutes ago (block time) or 1 minute (for mempool)
    // return saved in database data, so it's just a db lookup (for spamming requests)

    Ok(Json(CheckInvoiceResponse {
        invoice_uuid: Uuid::new_v4(),
        wallet_address: "mock wallet address".to_string(),
        amount_requested: "0.30".to_string(),
        amount_received: "3.321".to_string(),
        payment_status: PaymentStatus::Waiting,
        confirmations: None,
        transactions: vec![],
    }))
}

pub fn router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new().routes(routes!(create_invoice, check_invoice))
}

// How would invoices table look like?
// invoice_id (uuid default, not null)
// currency (string, currency code: XMR, USDC, BTC)
// network (string, MONERO, SOLANA, LIGHTNING / BITCOIN)
// wallet_address (string, actual wallet address request to pay to is created, e.g. invoice payment address)
// amount_requested (string, applied only once at creation/insert/ion)
// amount_received (string, by default 0, will be updated on each check until payment is finalized)
// payment_status (string, enum from payment status at payment.rs: waiting, detected, confirmed, expired)
// confirmations (u32, updated from monero-wallet-rpc, electrum, etc, on each request)
// transactions (list, I assume we can use postgres' jsonb)

// How would MoneroWallet table look like?
// id (primary-key-auto-incremental)
// major index (account)
// minor index (subaddress)
// wallet address (actual)
// created at (default now, timestamp, won't change)
// last used at (default now, timestamp, will be updated if address is borrowed of freed, e.g. is_available or blockchain height changed)
// blockchain_height (u32)
// is_available (bool, for reuse)
