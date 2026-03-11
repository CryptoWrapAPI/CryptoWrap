use crate::AppState;
use crate::PAYMENT_TAG;
use crate::entity::{deposits, monero_wallet};
use crate::routes::auth_helper::extract_user_row;
use crate::wallet::monero_helper::{self, DepositCheckResult};
use axum::{Json, extract::Query, extract::State, http::HeaderMap, http::StatusCode};
use chrono::Utc;
use reqwest::Client;
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
        (status = 500, description = "Internal server error", body = String),
    )
)]
pub async fn create(
    state: State<AppState>,
    headers: HeaderMap,
    Json(deposit_request): Json<CreateDepositRequest>,
) -> Result<Json<CreateDepositResponse>, (StatusCode, String)> {
    let user_row = extract_user_row(&state, &headers)
        .await
        .ok_or((StatusCode::UNAUTHORIZED, "Unauthorized".to_string()))?;

    let network = if let Some(net) = deposit_request.network {
        net.to_string()
    } else {
        match deposit_request.currency {
            Currency::XMR => Network::Monero.to_string(),
        }
    };

    // before using next code, first check for what coin is selected/specified, because next code handles only monero xmr
    // we have only one option in createdepositrequest so it's fine for now

    // get monero wallet address for this payment
    // first check if user has monero wallet initialized (e.g. has major wallet index in users db)
    let major_wallet_index = monero_helper::ensure_monero_major_wallet_index_for_user(
        &user_row,
        &state.monero_wallet,
        &state.conn,
    )
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            // Json(json!({"message": format!("Failed to initialize Monero wallet: {}", e)})),
            format!("Failed to initialize Monero wallet: {}", e),
        )
    })?;

    // use this index to create new subaddress (or reuse first available subaddress under this account (major index))
    let free_subaddress = monero_helper::get_free_monero_subaddress_with_major_index(
        major_wallet_index,
        &state.monero_wallet,
        &state.conn,
    )
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            // Json(json!({"message": format!("Failed to get Monero subaddress: {}", e)})),
            format!("Failed to get Monero subaddress: {}", e),
        )
    })?;

    // let wallet_address = free_subaddress.0; // new subaddress under this major index
    let wallet_address = free_subaddress;

    let deposit = deposits::ActiveModel {
        currency: Set(deposit_request.currency.to_string()),
        network: Set(network),
        payment_status: Set(DepositStatus::Waiting.as_str().to_string()),
        wallet_address: Set(wallet_address.clone()),
        // min_blockchain_height: Set(Some(free_subaddress.1 - 1)), // why Some? Because it can return Option<>
        // ^^^  set current blockchain height to start search (transfers) from, but -1, so it can detect very fast payments
        //graceful l
        // another layer of protection can be to implement grace period of re-use (for example: 1 hour of wait at least before using is_available address)
        //  --- moved min_blockchain_height to blockchain_height from monero_wallet using another sql query
        notify_url: Set(deposit_request.notify_url.clone()),
        ..Default::default()
    };
    let deposit = deposit.insert(&state.conn).await.unwrap();
    let deposit_uuid = deposit.deposit_id;

    let checkout_page = format!("{0}/checkout?uuid={deposit_uuid}", &state.current_url);

    Ok(Json(CreateDepositResponse {
        deposit_uuid,
        wallet_address,
        currency: deposit_request.currency,
        checkout_page,
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
        (status = 500, description = "Internal server error", body = String),
    )
)]
pub async fn check(
    state: State<AppState>,
    Query(deposit_request): Query<CheckDepositRequest>,
) -> Result<Json<CheckDepositResponse>, (StatusCode, String)> {
    // get deposit entry from deposits table with given deposit_uuid (from GET arguments)
    // get actual wallet address from this row
    // check wallet address on transfers using monero::get_transfers() specfying monero_wallet, address, and min. blockchain height (starting point)
    // blockchain height is stored in the deposit entry row we got earlier ^
    //
    //
    // NOTICE:
    // current implementation detects only last transaction, both in pool (mempool) and confirmed received (in/inbound)
    //

    let deposit_uuid = deposit_request.deposit_uuid;

    // 1. Get deposit entry from database
    let deposit = deposits::Entity::find()
        .filter(deposits::Column::DepositId.eq(deposit_uuid))
        .one(&state.conn)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or((StatusCode::NOT_FOUND, "Deposit not found".to_string()))?;

    if deposit.finalized {
        // tracing::debug!(
        //     "Finalized deposit payment_status: {:?}",
        //     deposit.payment_status
        // );

        // let payment_status = match deposit.payment_status.as_str() {
        //     "detected" => DepositStatus::Detected,
        //     "confirmed" => DepositStatus::Confirmed,
        //     "waiting" => DepositStatus::Waiting,
        //     _ => DepositStatus::Error,
        // };

        return Ok(Json(CheckDepositResponse {
            deposit_uuid,
            wallet_address: deposit.wallet_address,
            amount_received: deposit.amount_received,
            payment_status: DepositStatus::from_str(&deposit.payment_status),
            confirmations: deposit.confirmations.map(|c| c as u32),
            txid: deposit.txid,
            is_finalized: deposit.finalized,
        }));
    }

    let wallet_address = deposit.wallet_address.clone();

    let address_entry = monero_wallet::Entity::find()
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
        monero_helper::check_for_first_inbound_transfer_confirmed_or_mempool_with_min_height(
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

    // tracing::debug!(
    //     "Returned deposit payment_status: {:?}",
    //     result.payment_status
    // );
    // let payment_status = match result.payment_status.as_str() {
    //     "detected" => DepositStatus::Detected,
    //     "confirmed" => DepositStatus::Confirmed,
    //     "waiting" => DepositStatus::Waiting,
    //     _ => DepositStatus::Error,
    // };

    let payment_status = DepositStatus::from_str(&result.payment_status);

    let amount_received = result.amount_received.clone();
    let should_finalize = result.confirmations.map(|c| c >= 10).unwrap_or(false);

    let is_finalized: bool;
    if should_finalize == false {
        is_finalized = false;
    } else {
        is_finalized = true;
    };

    let payment_status_before_update = deposit.payment_status.clone();
    let notify_url = deposit.notify_url.clone();

    let mut deposit_active_model: deposits::ActiveModel = deposit.clone().into();
    deposit_active_model.amount_received = Set(amount_received.clone());
    deposit_active_model.confirmations = Set(result.confirmations);
    deposit_active_model.txid = Set(result.txid.clone());
    deposit_active_model.payment_status = Set(result.payment_status.clone());
    deposit_active_model.updated_at = Set(Some(Utc::now().naive_local()));
    deposit_active_model.finalized = Set(should_finalize);
    deposit_active_model
        .update(&state.conn)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // check if deposit.payment_status is different from result.payment_status
    // if true - notify shop using notify_url, sending same response as we send on request below

    let deposit_checked = CheckDepositResponse {
        deposit_uuid,
        wallet_address,
        amount_received,
        payment_status,
        confirmations: result.confirmations.map(|c| c as u32),
        txid: result.txid,
        is_finalized,
    };

    if payment_status_before_update != result.payment_status {
        if let Some(url) = notify_url {
            if let Err(e) = notify_shop(&url, &deposit_checked).await {
                tracing::warn!("Failed to notify shop: {}", e);
            }
        }
    };

    Ok(Json(deposit_checked))
}

async fn notify_shop(
    notify_url: &str,
    deposit_checked: &CheckDepositResponse,
) -> Result<(), String> {
    // let client = Client::new();
    let client = Client::builder()
        .danger_accept_invalid_certs(true)
        .build()
        .unwrap();
    // .map_err(|e| e.to_string())?;

    let max_retries = 3;

    for attempt in 1..=max_retries {
        let response = client
            .post(notify_url)
            .timeout(tokio::time::Duration::from_secs(5))
            .json(deposit_checked)
            .send()
            .await
            .map_err(|e| e.to_string())?;
        // potential issue: can convert POST to GET if redirected (for example, from http to https)

        if response.status() == StatusCode::ACCEPTED {
            return Ok(());
        }

        if attempt < max_retries {
            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        }
    }

    Err(format!(
        "Failed to notify shop after {} attempts",
        max_retries
    ))
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
    Error,
}

impl DepositStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            DepositStatus::Waiting => "waiting",
            DepositStatus::Detected => "detected",
            DepositStatus::Confirmed => "confirmed",
            DepositStatus::Expired => "expired",
            _ => "error", // actually can cover all enum elements
        }
    }
    pub fn from_str(s: &str) -> Self {
        match s {
            "waiting" => DepositStatus::Waiting,
            "detected" => DepositStatus::Detected,
            "confirmed" => DepositStatus::Confirmed,
            "expired" => DepositStatus::Expired,
            _ => DepositStatus::Error,
        }
    }
}

#[derive(Serialize, Deserialize, ToSchema)]
#[schema(example = json!({
    "deposit_uuid":"3f270a5a-50be-4ad7-9f01-fffc2c5144b3",
    "wallet_address":"46QYvqx4Z8JKk26DVyNbFjMgFqXyrXgAb3W8kEHBiSN78XrcoPRHk4ATjoCJ9eia5MVQMxDdQ6nAaa2D9MgLgZV31V2bCRS",
    "amount_received":"0",
    "payment_status":"waiting",
    "confirmations":Option::<u32>::None,
    "txid":Option::<String>::None,
    "is_finalized":false,
}))]

pub struct CheckDepositResponse {
    #[schema(value_type = String)]
    pub deposit_uuid: Uuid,
    pub wallet_address: String,
    pub amount_received: String,
    pub payment_status: DepositStatus,
    pub confirmations: Option<u32>,
    pub txid: Option<String>,
    pub is_finalized: bool,
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
    pub checkout_page: String, // simple gateway page that show all the payment details to users by providing uuid (comes with it in query string)
                               // can be used to send to user for the payment
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
    pub notify_url: Option<String>,
}

// add redirect url for url to return back to shop
// add hmac signature for webhook verification using sk
