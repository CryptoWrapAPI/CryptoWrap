// logic for interaction with blockchain (in this case: monero-wallet-rpc)

use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum MoneroError {
    #[error("Request failed: {0}")]
    Request(#[from] reqwest::Error),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("RPC error: {0}")]
    Rpc(String),
}

#[derive(Clone)]
pub struct MoneroRpc {
    client: Client,
    url: String,
}

impl MoneroRpc {
    pub fn new(address: &str) -> Self {
        let url = format!("http://{}/json_rpc", address);
        Self {
            client: Client::new(),
            url,
        }
    }

    // #[instrument(skip(self, params), fields(method = %method))]
    async fn rpc_request<T: for<'de> Deserialize<'de>>(
        &self,
        method: &str,
        params: serde_json::Value,
    ) -> Result<T, MoneroError> {
        let request = json!({
            "jsonrpc": "2.0",
            "id": "0",
            "method": method,
            "params": params
        });

        // debug!(request = %pretty_json(&request), "Monero RPC request");

        let response = self
            .client
            .post(&self.url)
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await?;

        let rpc_response: Value = response.json().await?;

        // debug!(response = %pretty_json(&rpc_response), "Monero RPC response");

        if let Some(error) = rpc_response.get("error") {
            let msg = error
                .get("message")
                .and_then(|m| m.as_str())
                .unwrap_or(&error.to_string())
                .to_string();
            return Err(MoneroError::Rpc(msg));
        }

        let result = rpc_response
            .get("result")
            .ok_or_else(|| MoneroError::Rpc("No result in response".to_string()))?;

        Ok(serde_json::from_value(result.clone())?)
    }
}

// fn pretty_json(value: &Value) -> String {
//     serde_json::to_string_pretty(value).unwrap_or_else(|_| value.to_string())
// }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transfer {
    pub address: String,
    pub amount: u64,
    pub amounts: Option<Vec<u64>>,
    pub confirmations: Option<i32>,
    pub double_spend_seen: Option<bool>,
    pub fee: Option<u64>,
    pub height: u64,
    pub note: Option<String>,
    pub destinations: Option<Vec<TransferDestination>>,
    pub payment_id: Option<String>,
    pub subaddr_index: Option<SubaddrIndex>,
    pub subaddr_indices: Option<Vec<SubaddrIndex>>,
    pub suggested_confirmations_threshold: Option<u64>,
    pub timestamp: u64,
    pub txid: String,
    #[serde(rename = "type")]
    pub transfer_type: String,
    pub unlock_time: u64,
    pub locked: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransferDestination {
    pub amount: u64,
    pub address: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubaddrIndex {
    pub major: u32,
    pub minor: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GetTransfersParams {
    #[serde(rename = "in")]
    pub inbound: Option<bool>,
    pub out: Option<bool>,
    pub pending: Option<bool>,
    pub failed: Option<bool>,
    pub pool: Option<bool>,
    pub filter_by_height: Option<bool>,
    pub min_height: Option<i32>,
    // pub max_height: Option<i32>,
    pub account_index: Option<u32>,
    pub subaddr_indices: Option<Vec<u32>>,
    pub all_accounts: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetTransfersResponse {
    #[serde(rename = "in")]
    pub inbound: Option<Vec<Transfer>>,
    pub out: Option<Vec<Transfer>>,
    pub pending: Option<Vec<Transfer>>,
    pub failed: Option<Vec<Transfer>>,
    pub pool: Option<Vec<Transfer>>,
}

pub async fn get_transfers(
    wallet: &MoneroRpc,
    params: GetTransfersParams,
) -> Result<GetTransfersResponse, MoneroError> {
    wallet
        .rpc_request("get_transfers", serde_json::to_value(params)?)
        .await
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateAccountParams {
    // pub label: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateAccountResponse {
    pub account_index: u32,
    pub address: String,
}

pub async fn create_account(
    wallet: &MoneroRpc,
    // label: Option<&str>,
) -> Result<CreateAccountResponse, MoneroError> {
    let params = CreateAccountParams {
        // label: label.map(|s| s.to_string()),
    };
    wallet
        .rpc_request("create_account", serde_json::to_value(params)?)
        .await
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateAddressParams {
    pub account_index: u32,
    // pub label: Option<String>,
    // pub count: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateAddressResponse {
    pub address: String,
    pub address_index: u32,
    // pub address_indices: Option<Vec<u32>>,
    // pub addresses: Option<Vec<String>>,
}

pub async fn create_address(
    wallet: &MoneroRpc,
    account_index: u32,
    // label: Option<&str>,
    // count: Option<u32>,
) -> Result<CreateAddressResponse, MoneroError> {
    let params = CreateAddressParams {
        account_index,
        // label: label.map(|s| s.to_string()),
        // count,
    };
    wallet
        .rpc_request("create_address", serde_json::to_value(params)?)
        .await
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetHeightResponse {
    // pub height: u64,
    pub height: i32,
}

pub async fn get_height(wallet: &MoneroRpc) -> Result<GetHeightResponse, MoneroError> {
    wallet.rpc_request("get_height", json!({})).await
}

pub async fn get_account_balance(
    wallet: &MoneroRpc,
    major_index: u32,
) -> Result<AccountBalanceResponse, MoneroError> {
    wallet
        .rpc_request("get_balance", json!({"account_index": major_index}))
        .await
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountBalanceResponse {
    pub unlocked_balance: u64, // atomic units (e.g. piconeros for monero, satoshi for btc, litoshi for ltc, etc...)
                               // use term `unlocked` just like in monero wallet rpc (docs), other coins implementation can handle it differently
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TransferParams {
    pub destinations: Vec<TransferDestination>,
    pub account_index: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subaddr_indices: Option<Vec<u32>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub priority: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ring_size: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unlock_time: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub get_tx_key: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub do_not_relay: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub get_tx_hex: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub get_tx_metadata: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransferResponse {
    pub amount: u64,
    pub fee: u64,
    pub multisig_txset: String,
    pub tx_blob: String,
    pub tx_hash: String,
    pub tx_key: String,
    pub tx_metadata: String,
    pub unsigned_txset: String,
}

pub async fn transfer(
    wallet: &MoneroRpc,
    params: TransferParams,
) -> Result<TransferResponse, MoneroError> {
    wallet
        .rpc_request("transfer", serde_json::to_value(params)?)
        .await
}
