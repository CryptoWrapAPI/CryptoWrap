use crate::entity::litecoin_wallet::{self, ActiveModel as LitecoinWalletActiveModel, Column as LitecoinWalletColumn};
use crate::entity::tokens::{self, ActiveModel as TokensActiveModel};
use crate::wallet::litecoin::{AddressPair, LitecoinError, LitecoinWallet};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QuerySelect, Set,
};
use std::fmt::{Display, Formatter, Result as FmtResult};
use std::slice::from_ref;

/// Custom error type for Litecoin helper functions.
#[derive(Debug)]
pub enum LitecoinHelperError {
    Litecoin(LitecoinError),
    Db(sea_orm::DbErr),
    InvalidAddress(String),
}

impl Display for LitecoinHelperError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            LitecoinHelperError::Litecoin(err) => write!(f, "Litecoin error: {}", err),
            LitecoinHelperError::Db(err) => write!(f, "Database error: {}", err),
            LitecoinHelperError::InvalidAddress(addr) => {
                write!(f, "Invalid Litecoin address: {addr}")
            }
        }
    }
}

impl From<LitecoinError> for LitecoinHelperError {
    fn from(err: LitecoinError) -> Self {
        LitecoinHelperError::Litecoin(err)
    }
}

impl From<sea_orm::DbErr> for LitecoinHelperError {
    fn from(err: sea_orm::DbErr) -> Self {
        LitecoinHelperError::Db(err)
    }
}

/// Ensures an account index exists for the user.
/// If not, finds the highest used account index in the database and uses the next one.
/// Also creates a change address (is_change = true) at address_index 0.
/// Returns the account index.
pub async fn ensure_litecoin_account_index_for_user(
    user_row: &tokens::Model,
    litecoin_wallet_client: &LitecoinWallet,
    conn: &DatabaseConnection,
) -> Result<u32, LitecoinHelperError> {
    if let Some(account_index) = user_row.litecoin_account_index {
        Ok(account_index as u32)
    } else {
        // Get the highest used account index from the database
        let max_account_index = litecoin_wallet::Entity::find()
            .select_only()
            .column_as(litecoin_wallet::Column::AccountIndex.max(), "max_index")
            .into_model::<MaxIndexResult>()
            .one(conn)
            .await?
            .and_then(|r| r.max_index)
            .unwrap_or(0);

        let new_account_index = (max_account_index + 1) as u32;

        // Create change address at index 0 (is_change = true)
        let change_address = litecoin_wallet_client
            .derive_address(new_account_index, 0)
            .await?;

        let blockchain_height = litecoin_wallet_client.get_block_height().await?.height as i32;

        let new_litecoin_wallet_entry = LitecoinWalletActiveModel {
            account_index: Set(new_account_index as i32),
            address_index: Set(0),
            wallet_address: Set(change_address.address.clone()),
            is_available: Set(Some(false)), // change addresses are not available for deposits
            is_change: Set(true),
            blockchain_height: Set(blockchain_height),
            ..Default::default()
        };
        new_litecoin_wallet_entry.insert(conn).await?;

        // Update the user's token entry with the new account index
        let mut token_active_model: TokensActiveModel = user_row.clone().into();
        token_active_model.litecoin_account_index = Set(Some(new_account_index as i32));
        token_active_model.update(conn).await?;

        Ok(new_account_index)
    }
}

/// Retrieves a free Litecoin deposit address for a given account index.
/// It first checks for an available address in the database. If none exist,
/// it creates a new one via the Litecoin API and stores it.
pub async fn get_free_litecoin_address_with_account_index(
    account_index: u32,
    litecoin_wallet_client: &LitecoinWallet,
    conn: &DatabaseConnection,
) -> Result<String, LitecoinHelperError> {
    // Get blockchain height
    let blockchain_height = litecoin_wallet_client.get_block_height().await?.height as i32;

    // 1. Search for an existing available address in the database
    loop {
        if let Some(available_address_model) = litecoin_wallet::Entity::find()
            .filter(litecoin_wallet::Column::AccountIndex.eq(account_index as i32))
            .filter(litecoin_wallet::Column::IsAvailable.eq(true))
            .filter(litecoin_wallet::Column::IsChange.eq(false))
            .one(conn)
            .await?
        {
            // 2. If found, check current balance
            let address = &available_address_model.wallet_address;
            let balance_response = litecoin_wallet_client
                .get_balance(from_ref(address))
                .await?;

            let balance_entry = balance_response.get(address);
            let confirmed = balance_entry.map(|e| e.confirmed).unwrap_or(0);
            let unconfirmed = balance_entry.map(|e| e.unconfirmed).unwrap_or(0);

            // Skip addresses with negative balance (edge case e.g. pending outgoing tx)
            if confirmed < 0 || unconfirmed < 0 {
                tracing::warn!(
                    "Skipping Litecoin address {} with negative balance: confirmed={}, unconfirmed={}",
                    address, confirmed, unconfirmed
                );
                let mut active_model: LitecoinWalletActiveModel =
                    available_address_model.clone().into();
                active_model.is_available = Set(Some(false));
                active_model.update(conn).await?;
                continue;
            }

            let confirmed_balance = confirmed.to_string();

            let mut active_model: LitecoinWalletActiveModel =
                available_address_model.clone().into();
            active_model.is_available = Set(Some(false));
            active_model.blockchain_height = Set(blockchain_height);
            active_model.initial_balance = Set(Some(confirmed_balance));
            active_model.update(conn).await?;

            return Ok(address.clone());
        }
        break;
    }

    // 3. No available address — derive the next one
    let max_address_index = litecoin_wallet::Entity::find()
        .filter(litecoin_wallet::Column::AccountIndex.eq(account_index as i32))
        .filter(litecoin_wallet::Column::IsChange.eq(false))
        .select_only()
        .column_as(litecoin_wallet::Column::AddressIndex.max(), "max_index")
        .into_model::<MaxIndexResult>()
        .one(conn)
        .await?
        .and_then(|r| r.max_index)
        .unwrap_or(0);

    let new_address_index = (max_address_index + 1) as u32;

    // Derive the new address
    let derive_response = litecoin_wallet_client
        .derive_address(account_index, new_address_index)
        .await?;

    let new_address = derive_response.address;

    // Check balance of the newly derived address
    let balance_response = litecoin_wallet_client
        .get_balance(from_ref(&new_address))
        .await?;

    let confirmed_balance = balance_response
        .get(&new_address)
        .map(|e| e.confirmed.to_string())
        .unwrap_or("0".to_string());

    // Insert into the database with is_available = false
    let new_litecoin_wallet_entry = LitecoinWalletActiveModel {
        account_index: Set(account_index as i32),
        address_index: Set(new_address_index as i32),
        wallet_address: Set(new_address.clone()),
        is_available: Set(Some(false)),
        is_change: Set(false),
        blockchain_height: Set(blockchain_height),
        initial_balance: Set(Some(confirmed_balance)),
        ..Default::default()
    };
    new_litecoin_wallet_entry.insert(conn).await?;

    Ok(new_address)
}

/// Validate a Litecoin address format (mainnet + testnet).
fn validate_ltc_address(address: &str) -> Result<(), LitecoinHelperError> {
    if address.len() < 26 || address.len() > 62 {
        return Err(LitecoinHelperError::InvalidAddress(address.to_string()));
    }
    if !address
        .chars()
        .all(|c| c.is_ascii_alphanumeric())
    {
        return Err(LitecoinHelperError::InvalidAddress(address.to_string()));
    }
    Ok(())
}

/// Transfer Litecoin to a destination address.
///
/// Gathers all address pairs for the given account index,
/// finds the change address, and calls the build-and-send RPC.
/// After a successful transaction, marks all used addresses as keep_track = false.
pub async fn transfer_ltc(
    wallet: &LitecoinWallet,
    destination_address: &str,
    amount_litoshi: u64,
    account_index: u32,
    conn: &DatabaseConnection,
) -> Result<String, LitecoinHelperError> {
    validate_ltc_address(destination_address)?;

    let account_index_i32 = account_index as i32;

    // Get all addresses for this account
    let all_addresses = litecoin_wallet::Entity::find()
        .filter(LitecoinWalletColumn::AccountIndex.eq(account_index_i32))
        .all(conn)
        .await?;

    // Find the change address
    let change_entry = all_addresses
        .iter()
        .find(|a| a.is_change)
        .ok_or_else(|| {
            LitecoinHelperError::Litecoin(LitecoinError::Api(
                "No change address found for account".to_string(),
            ))
        })?;

    // Collect addresses that will participate in this transaction:
    // keep_track = true (addresses with UTXOs) + the change address
    let mut used_addresses: Vec<litecoin_wallet::Model> = all_addresses
        .iter()
        .filter(|a| a.keep_track)
        .cloned()
        .collect();

    // Include change address in inputs if not already present
    if !used_addresses.iter().any(|a| a.address_index == change_entry.address_index) {
        used_addresses.push(change_entry.clone());
    }

    let inputs: Vec<AddressPair> = used_addresses
        .iter()
        .map(|a| AddressPair {
            account_index,
            address_index: a.address_index as u32,
        })
        .collect();

    let response = wallet
        .build_and_send(
            &inputs,
            destination_address,
            amount_litoshi,
            &change_entry.wallet_address,
        )
        .await?;

    // Mark all used addresses as keep_track = false
    for addr in &used_addresses {
        let mut active_model: LitecoinWalletActiveModel = addr.clone().into();
        active_model.keep_track = Set(false);
        active_model.update(conn).await?;
    }

    Ok(response.tx_hash)
}

#[derive(sea_orm::FromQueryResult)]
struct MaxIndexResult {
    pub max_index: Option<i32>,
}
