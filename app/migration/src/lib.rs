pub use sea_orm_migration::prelude::*;

mod m20260220_121747_tokens_table;
mod m20260226_092939_monero_wallet;
mod m20260304_200054_deposits_table;
mod m20260311_052529_fiat_prices;
mod m20260411_003754_litecoin;
mod m20260521_020827_withdrawals;
mod m20260623_163122_invoices_table;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20260220_121747_tokens_table::Migration),
            Box::new(m20260226_092939_monero_wallet::Migration),
            Box::new(m20260304_200054_deposits_table::Migration),
            Box::new(m20260311_052529_fiat_prices::Migration),
            Box::new(m20260411_003754_litecoin::Migration),
            Box::new(m20260521_020827_withdrawals::Migration),
            Box::new(m20260623_163122_invoices_table::Migration),
        ]
    }
}
