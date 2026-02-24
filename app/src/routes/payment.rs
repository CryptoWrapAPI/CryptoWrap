use crate::AppState;
use crate::PAYMENT_TAG;
use axum::{Json, extract::State};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use utoipa_axum::{router::OpenApiRouter, routes};
use uuid::Uuid;

#[derive(Serialize, ToSchema)]
pub struct InvoiceResponse {
    #[schema(value_type = String)]
    pub invoice_uuid: Uuid,
    pub wallet_address: String,
    pub payment_amount: String,
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
/// Returns an invoice UUID to check the specified payment amount
#[utoipa::path(
    post,
    path = "/create_invoice",
    tag = PAYMENT_TAG,
    responses(
        (status = 200, description = "Invoice created successfully", body = InvoiceResponse)
    )
)]
pub async fn create_invoice(
    // checkout / bill
    state: State<AppState>,
    Json(invoice_request): Json<InvoiceRequest>,
) -> Json<InvoiceResponse> {
    let mock_invoice = Uuid::new_v4();

    Json(InvoiceResponse {
        invoice_uuid: mock_invoice,
        wallet_address: "...".to_string(),
        payment_amount: "0.15".to_string(),
        currency: invoice_request.currency,
    })
}

pub fn router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new().routes(routes!(create_invoice))
}
