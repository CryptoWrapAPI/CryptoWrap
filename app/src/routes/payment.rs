use crate::AppState;
use crate::PAYMENT_TAG;
use crate::entity::invoices;
use crate::entity::prelude::*;
use crate::entity::{litecoin_wallet, monero_wallet};
use crate::routes::auth_helper::extract_user_row;
use crate::routes::deposit::{Currency, FiatCurrency, Network, convert_to_fiat};
use crate::routes::notify_helper::notify_shop;
use crate::wallet::litecoin::litoshi_to_ltc;
use crate::wallet::litecoin_helper;
use crate::wallet::monero_helper::{self, DepositCheckResult};
use axum::{Json, extract::Query, extract::State, http::HeaderMap, http::StatusCode};
use chrono::Utc;
use rust_decimal::Decimal;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set};
use serde::{Deserialize, Serialize};
use std::slice::from_ref;
use strum_macros::Display;
use utoipa::ToSchema;
use utoipa_axum::{router::OpenApiRouter, routes};
use uuid::Uuid;

#[derive(Serialize, ToSchema)]
#[schema(example = json!({
    "invoice_uuid":"3f270a5a-50be-4ad7-9f01-fffc2c5144b3",
    "wallet_address":"46QYvqx4Z8JKk26DVyNbFjMgFqXyrXgAb3W8kEHBiSN78XrcoPRHk4ATjoCJ9eia5MVQMxDdQ6nAaa2D9MgLgZV31V2bCRS",
    "amount_requested":"1.5",
    "currency":"XMR",
}))]
pub struct CreateInvoiceResponse {
    #[schema(value_type = String)]
    pub invoice_uuid: Uuid,
    pub wallet_address: String,
    pub amount_requested: String,
    pub currency: Currency,
}

#[derive(Deserialize, ToSchema)]
#[schema(example = json!({
    "amount":"1.5",
    "currency":"XMR",
    "wallet_address":"46QYvqx4Z8JKk26DVyNbFjMgFqXyrXgAb3W8kEHBiSN78XrcoPRHk4ATjoCJ9eia5MVQMxDdQ6nAaa2D9MgLgZV31V2bCRS",
}))]
pub struct CreateInvoiceRequest {
    pub amount: String,
    pub currency: Currency,
    pub network: Option<Network>,
    pub notify_url: Option<String>,
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
        (status = 500, description = "Internal server error", body = String),
    )
)]
pub async fn create_invoice(
    state: State<AppState>,
    headers: HeaderMap,
    Json(invoice_request): Json<CreateInvoiceRequest>,
) -> Result<Json<CreateInvoiceResponse>, (StatusCode, String)> {
    let user_row = extract_user_row(&state, &headers)
        .await
        .ok_or((StatusCode::UNAUTHORIZED, "Unauthorized".to_string()))?;

    // parse the requested amount
    let amount_requested: f64 = invoice_request
        .amount
        .parse()
        .map_err(|_| (StatusCode::BAD_REQUEST, "Invalid amount".to_string()))?;

    if amount_requested <= 0.0 {
        return Err((
            StatusCode::BAD_REQUEST,
            "Amount must be positive".to_string(),
        ));
    }

    // determine network
    let network = if let Some(net) = invoice_request.network {
        net.to_string()
    } else {
        match invoice_request.currency {
            Currency::Xmr => Network::Monero.to_string(),
            Currency::Ltc => Network::Litecoin.to_string(),
        }
    };

    // generate wallet address
    let wallet_address = if invoice_request.currency == Currency::Xmr {
        let major_wallet_index = monero_helper::ensure_monero_major_wallet_index_for_user(
            &user_row,
            &state.monero_wallet,
            &state.conn,
        )
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to initialize Monero wallet: {}", e),
            )
        })?;

        monero_helper::get_free_monero_subaddress_with_major_index(
            major_wallet_index,
            &state.monero_wallet,
            &state.conn,
        )
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to get Monero subaddress: {}", e),
            )
        })?
    } else if invoice_request.currency == Currency::Ltc {
        let account_index = litecoin_helper::ensure_litecoin_account_index_for_user(
            &user_row,
            &state.litecoin_wallet,
            &state.conn,
        )
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to initialize Litecoin wallet: {}", e),
            )
        })?;

        litecoin_helper::get_free_litecoin_address_with_account_index(
            account_index,
            &state.litecoin_wallet,
            &state.conn,
        )
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to get Litecoin address: {}", e),
            )
        })?
    } else {
        return Err((
            StatusCode::BAD_REQUEST,
            format!("Unsupported currency: {:?}", invoice_request.currency),
        ));
    };

    let invoice = invoices::ActiveModel {
        currency: Set(invoice_request.currency.to_string()),
        network: Set(network),
        wallet_address: Set(wallet_address.clone()),
        owner_id: Set(user_row.id),
        amount_requested: Set(amount_requested.to_string()),
        amount_received: Set("0".to_string()),
        payment_status: Set(PaymentStatus::Waiting.to_string()),
        confirmations: Set(None),
        transactions: Set(None),
        finalized: Set(false),
        notify_url: Set(invoice_request.notify_url.clone()),
        created_at: Set(Utc::now().naive_utc()),
        ..Default::default()
    };
    let invoice = invoice.insert(&state.conn).await.unwrap();
    let invoice_uuid = invoice.invoice_id;

    Ok(Json(CreateInvoiceResponse {
        invoice_uuid,
        wallet_address,
        amount_requested: amount_requested.to_string(),
        currency: invoice_request.currency,
    }))
}

#[derive(Deserialize, ToSchema)]
pub struct CheckInvoiceRequest {
    #[schema(value_type = String)]
    pub invoice_uuid: Uuid,
    #[serde(default)]
    pub price_to: Option<FiatCurrency>,
}

#[derive(Serialize, Deserialize, ToSchema, Display, Clone, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum PaymentStatus {
    Waiting,
    Detected,
    Confirmed,
    Expired,
}

impl PaymentStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            PaymentStatus::Waiting => "waiting",
            PaymentStatus::Detected => "detected",
            PaymentStatus::Confirmed => "confirmed",
            PaymentStatus::Expired => "expired",
        }
    }
    pub fn from_str(s: &str) -> Self {
        match s {
            "waiting" => PaymentStatus::Waiting,
            "detected" => PaymentStatus::Detected,
            "confirmed" => PaymentStatus::Confirmed,
            "expired" => PaymentStatus::Expired,
            _ => PaymentStatus::Waiting,
        }
    }
}

#[derive(Serialize, ToSchema)]
#[schema(example = json!({
    "invoice_uuid":"3f270a5a-50be-4ad7-9f01-fffc2c5144b3",
    "wallet_address":"46QYvqx4Z8JKk26DVyNbFjMgFqXyrXgAb3W8kEHBiSN78XrcoPRHk4ATjoCJ9eia5MVQMxDdQ6nAaa2D9MgLgZV31V2bCRS",
    "amount_requested":"1.5",
    "amount_received":"0",
    "payment_status":"waiting",
    "confirmations":null,
    "transactions":[],
    "is_finalized":false,
}))]
pub struct CheckInvoiceResponse {
    #[schema(value_type = String)]
    pub invoice_uuid: Uuid,
    pub wallet_address: String,
    pub amount_requested: String,
    pub amount_received: String,
    pub payment_status: PaymentStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub confirmations: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transactions: Option<Vec<String>>,
    pub is_finalized: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fiat_amount: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fiat_currency: Option<FiatCurrency>,
}

/// Check invoice
///
/// Returns invoice payment status.
#[utoipa::path(
    get,
    path = "/check_invoice",
    tag = PAYMENT_TAG,
    params(
        ("invoice_uuid" = String, Query, description = "UUID of the invoice to check"),
        ("price_to" = Option<String>, Query, description = "Optional fiat currency for conversion (usd, eur, rub)")
    ),
    responses(
        (status = 200, description = "Invoice information", body = CheckInvoiceResponse),
        (status = 404, description = "Invoice not found"),
        (status = 500, description = "Internal server error", body = String),
    )
)]
pub async fn check_invoice(
    state: State<AppState>,
    Query(invoice_request): Query<CheckInvoiceRequest>,
) -> Result<Json<CheckInvoiceResponse>, (StatusCode, String)> {
    let invoice_uuid = invoice_request.invoice_uuid;

    let invoice = Invoices::find()
        .filter(invoices::Column::InvoiceId.eq(invoice_uuid))
        .one(&state.conn)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or((StatusCode::NOT_FOUND, "Invoice not found".to_string()))?;

    if invoice.finalized {
        let fiat_conversion = if let Some(fiat_curr) = invoice_request.price_to {
            let coin = invoice.currency.to_lowercase();
            convert_to_fiat(&state.conn, &invoice.amount_received, &coin, fiat_curr).await
        } else {
            None
        };

        let transactions: Option<Vec<String>> = invoice.transactions.as_ref().map(|t| {
            t.as_array()
                .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
                .unwrap_or_default()
        });

        return Ok(Json(CheckInvoiceResponse {
            invoice_uuid,
            wallet_address: invoice.wallet_address,
            amount_requested: invoice.amount_requested,
            amount_received: invoice.amount_received,
            payment_status: PaymentStatus::from_str(&invoice.payment_status),
            confirmations: invoice.confirmations.map(|c| c as u32),
            transactions,
            is_finalized: invoice.finalized,
            fiat_amount: fiat_conversion.as_ref().map(|f| f.amount.clone()),
            fiat_currency: fiat_conversion.map(|f| f.currency),
        }));
    }

    let wallet_address = invoice.wallet_address.clone();
    let amount_requested = &invoice.amount_requested;

    let (amount_received, payment_status, confirmations, txids, should_finalize) =
        if invoice.currency.to_uppercase() == "XMR" {
            // === MONERO ===
            let address_entry = MoneroWallet::find()
                .filter(monero_wallet::Column::WalletAddress.eq(&wallet_address))
                .one(&state.conn)
                .await
                .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
                .ok_or((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Wallet address not found in database".to_string(),
                ))?;

            let min_height = address_entry.blockchain_height - 1;

            let result: DepositCheckResult =
                monero_helper::check_for_inbound_transfers_confirmed_or_mempool_with_min_height(
                    &state.monero_wallet,
                    address_entry.major_index,
                    address_entry.minor_index,
                    min_height,
                )
                .await
                .map_err(|e| {
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        format!("Error executing get_transfers for this address: {}", e),
                    )
                })?;

            // For invoices, check if amount_received meets or exceeds amount_requested
            let meets_request = meets_or_exceeds(&result.amount_received, amount_requested);
            let invoice_paid = meets_request && result.confirmations.map(|c| c >= 10).unwrap_or(false);

            let status = if invoice_paid {
                "confirmed"
            } else if result.amount_received != "0" && result.amount_received != "0.0" {
                "detected"
            } else {
                "waiting"
            };

            let should_finalize = invoice_paid;

            (
                result.amount_received,
                status.to_string(),
                result.confirmations,
                Some(result.txids),
                should_finalize,
            )
        } else if invoice.currency.to_uppercase() == "LTC" {
            // === LITECOIN ===
            let ltc_entry = LitecoinWallet::find()
                .filter(litecoin_wallet::Column::WalletAddress.eq(&wallet_address))
                .one(&state.conn)
                .await
                .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
                .ok_or((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Litecoin wallet address not found in database".to_string(),
                ))?;

            let balance_response = state
                .litecoin_wallet
                .get_balance(from_ref(&wallet_address))
                .await
                .map_err(|e| {
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        format!("Failed to check Litecoin balance: {}", e),
                    )
                })?;

            let balance_entry = balance_response.get(&wallet_address);

            let confirmed: u64 = balance_entry.map(|e| e.confirmed as u64).unwrap_or(0);
            let unconfirmed: u64 = balance_entry.map(|e| e.unconfirmed as u64).unwrap_or(0);

            let initial_balance: u64 = ltc_entry
                .initial_balance
                .as_deref()
                .unwrap_or("0")
                .parse()
                .unwrap_or(0);

            let total_balance = confirmed + unconfirmed;
            let amount_received_litoshi = total_balance.saturating_sub(initial_balance);
            let amount_received = litoshi_to_ltc(amount_received_litoshi, false);

            let meets_request = meets_or_exceeds(&amount_received, amount_requested);

            let (payment_status, should_finalize) = if total_balance > initial_balance {
                if unconfirmed > 0 {
                    if meets_request {
                        ("detected".to_string(), false)
                    } else {
                        ("detected".to_string(), false)
                    }
                } else {
                    if meets_request {
                        let mut ltc_active_model: litecoin_wallet::ActiveModel =
                            ltc_entry.clone().into();
                        ltc_active_model.keep_track = Set(true);
                        ltc_active_model
                            .update(&state.conn)
                            .await
                            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

                        ("confirmed".to_string(), true)
                    } else {
                        ("detected".to_string(), false)
                    }
                }
            } else {
                ("waiting".to_string(), false)
            };

            let confirmations: Option<i32> = None;

            (
                amount_received,
                payment_status,
                confirmations,
                None,
                should_finalize,
            )
        } else {
            return Err((
                StatusCode::BAD_REQUEST,
                format!("Unsupported currency: {}", invoice.currency),
            ));
        };

    let payment_status = PaymentStatus::from_str(&payment_status);

    let status_before_update = invoice.payment_status.clone();
    let notify_url = invoice.notify_url.clone();

    let mut invoice_active_model: invoices::ActiveModel = invoice.clone().into();
    invoice_active_model.amount_received = Set(amount_received.clone());
    invoice_active_model.confirmations = Set(confirmations);
    invoice_active_model.payment_status = Set(payment_status.as_str().to_string());
    invoice_active_model.updated_at = Set(Some(Utc::now().naive_utc()));
    invoice_active_model.finalized = Set(should_finalize);

    // set transactions if we have them
    if let Some(txids_list) = &txids {
        let txids_json: serde_json::Value =
            serde_json::Value::Array(txids_list.iter().map(|t| serde_json::Value::String(t.clone())).collect());
        invoice_active_model.transactions = Set(Some(txids_json));
    }

    invoice_active_model
        .update(&state.conn)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let fiat_conversion = if let Some(fiat_curr) = invoice_request.price_to {
        let coin = invoice.currency.to_lowercase();
        convert_to_fiat(&state.conn, &amount_received, &coin, fiat_curr).await
    } else {
        None
    };

    let invoice_checked = CheckInvoiceResponse {
        invoice_uuid,
        wallet_address,
        amount_requested: invoice.amount_requested,
        amount_received,
        payment_status: payment_status.clone(),
        confirmations: confirmations.map(|c| c as u32),
        transactions: txids,
        is_finalized: should_finalize,
        fiat_amount: fiat_conversion.as_ref().map(|f| f.amount.clone()),
        fiat_currency: fiat_conversion.map(|f| f.currency),
    };

    if status_before_update != payment_status.as_str() {
        if let Some(url) = notify_url
            && let Err(e) = notify_shop(&url, &invoice_checked).await
        {
            tracing::warn!("Failed to notify shop: {}", e);
        }
    }

    Ok(Json(invoice_checked))
}

/// Compare two decimal string amounts: a >= b
fn meets_or_exceeds(received: &str, requested: &str) -> bool {
    let received: Decimal = received.parse().unwrap_or(Decimal::ZERO);
    let requested: Decimal = requested.parse().unwrap_or(Decimal::ZERO);
    received >= requested
}

pub fn router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new().routes(routes!(create_invoice, check_invoice))
}
