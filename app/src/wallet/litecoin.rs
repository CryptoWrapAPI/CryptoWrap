use reqwest::Client;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum LitecoinError {
    #[error("Request failed: {0}")]
    Request(#[from] reqwest::Error),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("API error: {0}")]
    Api(String),
}

#[derive(Clone)]
pub struct LitecoinWallet {
    client: Client,
    api_url: String,
    master_public_key: String,
}

impl LitecoinWallet {
    pub fn new(api_url: &str, master_public_key: &str) -> Self {
        Self {
            client: Client::new(),
            api_url: api_url.to_string(),
            master_public_key: master_public_key.to_string(),
        }
    }

    /// Derive a Litecoin address from the master public key.
    ///
    /// Derivation path: m/84'/coin'/account_index'/CHAIN_EXT/address_index
    pub async fn derive_address(
        &self,
        account_index: u32,
        address_index: u32,
    ) -> Result<DeriveAddressResponse, LitecoinError> {
        let request = DeriveRequest {
            xpub: self.master_public_key.clone(),
            account_index,
            address_index,
        };

        let url = format!("{}/derive", self.api_url);

        let response = self
            .client
            .post(&url)
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(LitecoinError::Api(format!(
                "API request failed with status {}: {}",
                status, body
            )));
        }

        let result: DeriveAddressResponse = response.json().await?;
        Ok(result)
    }

    /// Get the current blockchain height.
    pub async fn get_block_height(&self) -> Result<BlockHeightResponse, LitecoinError> {
        let url = format!("{}/block-height", self.api_url);

        let response = self.client.get(&url).send().await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(LitecoinError::Api(format!(
                "API request failed with status {}: {}",
                status, body
            )));
        }

        let result: BlockHeightResponse = response.json().await?;
        Ok(result)
    }

    /// Get balance for a list of addresses.
    pub async fn get_balance(
        &self,
        addresses: &[String],
    ) -> Result<BalanceResponse, LitecoinError> {
        let request = BalanceRequest {
            addresses: addresses.to_vec(),
        };

        let url = format!("{}/balance", self.api_url);

        let response = self
            .client
            .post(&url)
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(LitecoinError::Api(format!(
                "API request failed with status {}: {}",
                status, body
            )));
        }

        let result: BalanceResponse = response.json().await?;
        Ok(result)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeriveRequest {
    pub xpub: String,
    #[serde(default)]
    pub account_index: u32,
    #[serde(default)]
    pub address_index: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeriveAddressResponse {
    pub address: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockHeightResponse {
    pub height: u32,
    pub last_updated: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BalanceRequest {
    pub addresses: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BalanceEntry {
    pub confirmed: u64,
    pub unconfirmed: u64,
    pub timestamp: String,
}

pub type BalanceResponse = std::collections::HashMap<String, BalanceEntry>;

/// Convert litoshis (smallest unit, 1 LTC = 100_000_000 litoshis) to LTC string.
pub fn litoshi_to_ltc(amount: u64, show_decimal_precision: bool) -> String {
    let whole = amount / 100_000_000;
    let fraction = amount % 100_000_000;
    if fraction == 0 {
        // whole.to_string()
        format!("{}.{:08}", whole, fraction)
    } else {
        let mut fraction_str = format!("{:08}", fraction);
        // let mut fraction_trimmed = fraction_str.clone();
        if !show_decimal_precision {
            // fraction_trimmed = fraction_str.trim_end_matches('0').to_string();
            fraction_str = fraction_str.trim_end_matches('0').to_string();
        }
        // format!("{}.{}", whole, fraction_trimmed)
        format!("{}.{}", whole, fraction_str)
    }
}
