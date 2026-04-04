use crate::entity::monero_wallet::{self, ActiveModel as MoneroWalletActiveModel};
use crate::entity::tokens::{self, ActiveModel as TokensActiveModel};
use crate::wallet::monero::{self, MoneroError, MoneroWallet};
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set};
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter, Result as FmtResult};

fn piconero_to_xmr_string(amount: u64) -> String {
    let whole = amount / 1_000_000_000_000;
    let fraction = amount % 1_000_000_000_000;
    if fraction == 0 {
        whole.to_string()
    } else {
        let fraction_str = format!("{:012}", fraction);
        let fraction_trimmed = fraction_str.trim_end_matches('0');
        format!("{}.{}", whole, fraction_trimmed)
    }
}

/// Custom error type for Monero helper functions.
#[derive(Debug)]
pub enum MoneroHelperError {
    MoneroRpc(MoneroError),
    Db(sea_orm::DbErr),
    NotFound(String),
}

impl Display for MoneroHelperError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            MoneroHelperError::MoneroRpc(err) => write!(f, "Monero RPC error: {}", err),
            MoneroHelperError::Db(err) => write!(f, "Database error: {}", err),
            MoneroHelperError::NotFound(msg) => write!(f, "Not Found: {}", msg),
        }
    }
}

impl From<MoneroError> for MoneroHelperError {
    fn from(err: MoneroError) -> Self {
        MoneroHelperError::MoneroRpc(err)
    }
}

impl From<sea_orm::DbErr> for MoneroHelperError {
    fn from(err: sea_orm::DbErr) -> Self {
        MoneroHelperError::Db(err)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DepositCheckResult {
    pub amount_received: String,
    pub confirmations: Option<i32>,
    pub txids: Vec<String>,
    pub payment_status: String,
}

/// Ensures a major wallet index exists for the user.
/// If not, a new account is created via Monero RPC and stored in the database.
/// Returns the major wallet index.
pub async fn ensure_monero_major_wallet_index_for_user(
    user_row: &tokens::Model,
    monero_wallet_client: &MoneroWallet,
    conn: &DatabaseConnection,
) -> Result<u32, MoneroHelperError> {
    if let Some(major_index) = user_row.monero_major_index {
        Ok(major_index as u32)
    } else {
        // Create a new account in Monero wallet
        let create_account_response = monero::create_account(
            monero_wallet_client,
            // Some(&format!("user_{}", user_row.id)), // Label for the account
        )
        .await?; // Convert MoneroError to MoneroHelperError

        let new_major_index = create_account_response.account_index;
        // we can also save major index wallet address (account) which is probably the `change` address at index 0, first subaddress is index 1
        // so 0 is primary address as well/too

        // Update the user's token entry in the database with the new major index
        let mut token_active_model: TokensActiveModel = user_row.clone().into();
        token_active_model.monero_major_index = Set(Some(new_major_index as i32));
        token_active_model.update(conn).await?; // Convert DbErr to MoneroHelperError

        Ok(new_major_index)
    }
}

/// Retrieves a free Monero subaddress for a given major account index.
/// It first checks for an available address in the database. If none exist,
/// it creates a new one via Monero RPC and stores it.
pub async fn get_free_monero_subaddress_with_major_index(
    major_index: u32,
    monero_wallet_client: &MoneroWallet,
    conn: &DatabaseConnection,
    // ) -> Result<(String, i32), MoneroHelperError> {
) -> Result<String, MoneroHelperError> {
    // get height right from rpc in any case
    let blockchain_height = monero::get_height(monero_wallet_client).await?;

    let height = blockchain_height.height;

    // 1. Search for an existing available address in the database
    if let Some(available_address_model) = monero_wallet::Entity::find()
        .filter(monero_wallet::Column::MajorIndex.eq(major_index as i32))
        .filter(monero_wallet::Column::IsAvailable.eq(true))
        .one(conn)
        .await?
    {
        // 2. If found, update its status to not available and return the address
        let mut active_model: MoneroWalletActiveModel = available_address_model.clone().into();
        active_model.is_available = Set(false);
        active_model.blockchain_height = Set(height); // critical! ensure update height on reused addresses, so previous transfer won't be detected
        // because min_blockchain_height is not included in the deposits table anymore
        active_model.update(conn).await?; // No need to set last_used_at manually if default is handled by DB
        // Ok((available_address_model.wallet_address, height))
        Ok(available_address_model.wallet_address)
    } else {
        // 3. If no available address, create a new one via Monero RPC
        let create_address_response = monero::create_address(
            monero_wallet_client,
            major_index,
            // None, // No specific label for the subaddress
            // Some(1), // Create one address
        )
        .await?;

        let new_address = create_address_response.address;

        let new_minor_index = create_address_response.address_index;

        // 4. Insert the new address into the database with is_available = false
        let new_monero_wallet_entry = MoneroWalletActiveModel {
            major_index: Set(major_index as i32),
            minor_index: Set(new_minor_index as i32),
            wallet_address: Set(new_address.clone()),
            is_available: Set(false),
            blockchain_height: Set(height),
            ..Default::default() // Let the database handle created_at and last_used_at defaults
        };
        new_monero_wallet_entry.insert(conn).await?;

        // Ok((new_address, height))
        Ok(new_address)
    }
}

pub async fn check_for_inbound_transfers_confirmed_or_mempool_with_min_height(
    monero_wallet_client: &MoneroWallet,
    account_index: i32,
    subaddress_index: i32,
    min_height: i32,
) -> Result<DepositCheckResult, MoneroHelperError> {
    let transfers_response = monero::get_transfers(
        monero_wallet_client,
        monero::GetTransfersParams {
            inbound: Some(true),
            pool: Some(true),
            filter_by_height: Some(true),
            min_height: Some(min_height),
            account_index: Some(account_index as u32),
            subaddr_indices: Some(vec![subaddress_index as u32]),
            ..Default::default()
        },
    )
    .await?;

    // TODO: calculate amound from all transactions, return multiple (all) tx ids (vec)
    // is *any* amount is received - change status to detected
    // if *all* of the amount is confirmed - change status to confirmed

    let pool_transfers = transfers_response.pool.unwrap_or_default();
    if let Some(pool_transfer) = pool_transfers.iter().find(|_| true) {
        return Ok(DepositCheckResult {
            amount_received: piconero_to_xmr_string(pool_transfer.amount),
            confirmations: None,
            txids: vec![],
            payment_status: "detected".to_string(),
        });
    }

    let inbound_transfers = transfers_response.inbound.unwrap_or_default();
    if let Some(inbound_transfer) = inbound_transfers.iter().find(|_| true) {
        let confirmations = inbound_transfer.confirmations;

        let status = "confirmed";

        let mut txids = Vec::new();
        txids.push(inbound_transfer.txid.clone());

        return Ok(DepositCheckResult {
            amount_received: piconero_to_xmr_string(inbound_transfer.amount),
            confirmations: confirmations,
            txids,
            payment_status: status.to_string(),
        });
    }

    Ok(DepositCheckResult {
        amount_received: "0".to_string(),
        confirmations: None,
        txids: vec![],
        payment_status: "waiting".to_string(),
    })
}
