pub use sea_orm_migration::prelude::*;

mod m20260220_121747_tokens_table;
mod m20260226_092935_invoices_table;
mod m20260226_092939_monero_wallet;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20260220_121747_tokens_table::Migration),
            Box::new(m20260226_092935_invoices_table::Migration),
            Box::new(m20260226_092939_monero_wallet::Migration),
        ]
    }
}
