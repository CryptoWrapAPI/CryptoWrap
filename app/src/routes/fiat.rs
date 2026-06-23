use crate::entity::fiat_prices;
use crate::entity::prelude::*;
use rust_decimal::Decimal;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use serde::{Deserialize, Serialize};
use strum_macros::Display;
use utoipa::ToSchema;

#[derive(Serialize, Deserialize, ToSchema, Clone, Copy, Display)]
#[serde(rename_all = "lowercase")]
pub enum FiatCurrency {
    Usd,
    Eur,
    Rub,
}

fn currency_to_coin_id(currency: &str) -> String {
    match currency.to_uppercase().as_str() {
        "XMR" => "monero".to_string(),
        "LTC" => "litecoin".to_string(),
        _ => currency.to_lowercase(),
    }
}

/// Returns the number of decimal places for a coin's human-readable amount.
fn coin_precision(coin: &str) -> u32 {
    match coin.to_uppercase().as_str() {
        "XMR" => 12,
        "LTC" => 8,
        _ => 8, // safe default
    }
}

pub struct FiatConversion {
    pub amount: String,
    pub currency: FiatCurrency,
}

pub async fn convert_to_fiat(
    conn: &sea_orm::DatabaseConnection,
    crypto_amount: &str,
    coin: &str,
    fiat_currency: FiatCurrency,
) -> Option<FiatConversion> {
    let crypto_amount: Decimal = crypto_amount.parse().ok()?;
    let coin_id = currency_to_coin_id(coin);

    let price = FiatPrices::find()
        .filter(fiat_prices::Column::Coin.eq(&coin_id))
        .one(conn)
        .await
        .ok()??;

    let fiat_price = match fiat_currency {
        FiatCurrency::Usd => price.usd,
        FiatCurrency::Eur => price.eur,
        FiatCurrency::Rub => price.rub,
    };

    let fiat_amount = crypto_amount * fiat_price;

    Some(FiatConversion {
        amount: fiat_amount.to_string(),
        currency: fiat_currency,
    })
}

pub async fn convert_from_fiat(
    conn: &sea_orm::DatabaseConnection,
    fiat_amount: Decimal,
    coin: &str,
    fiat_currency: FiatCurrency,
) -> Result<Decimal, String> {
    let coin_id = currency_to_coin_id(coin);

    let price = FiatPrices::find()
        .filter(fiat_prices::Column::Coin.eq(&coin_id))
        .one(conn)
        .await
        .map_err(|e| format!("Database error: {}", e))?
        .ok_or_else(|| format!("No price data found for coin: {}", coin_id))?;

    let fiat_price = match fiat_currency {
        FiatCurrency::Usd => price.usd,
        FiatCurrency::Eur => price.eur,
        FiatCurrency::Rub => price.rub,
    };

    if fiat_price <= Decimal::ZERO {
        return Err("Fiat price is zero or negative".to_string());
    }

    let precision = coin_precision(coin);
    let raw = fiat_amount / fiat_price;
    Ok(raw.round_dp(precision))
}
