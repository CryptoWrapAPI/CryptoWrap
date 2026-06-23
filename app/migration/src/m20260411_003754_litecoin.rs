use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

// #[derive(DeriveIden)]
// enum LitecoinWallet {
//     Table,
//     // Id,
//     // AccountIndex,
//     // AddressIndex,
//     // WalletAddress,
//     // CreatedAt,
//     // LastUsedAt,
//     // BlockchainHeight,
//     // IsAvailable,
//     // IsChange,
// }

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    // .table(LitecoinWallet::Table)
                    .table("litecoin_wallet")
                    .if_not_exists()
                    .col(integer("id").primary_key().auto_increment())
                    .col(integer("account_index").not_null())
                    .col(integer("address_index").not_null())
                    .col(string("wallet_address").not_null().unique_key())
                    .col(
                        date_time("created_at")
                            .default(Expr::current_timestamp())
                            .not_null(),
                    )
                    .col(date_time("last_used_at").null())
                    .col(integer("blockchain_height").not_null())
                    .col(boolean("is_available").default(true).null())
                    .col(boolean("is_change").default(false).not_null())
                    .col(boolean("keep_track").default(false).not_null())
                    .col(string("initial_balance").null()) // is used to track before balance using as deposit - so it's possible to use /get_balance instead of requesting txs which simplifies logic a lot
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table("tokens")
                    .add_column(integer("litecoin_account_index").null())
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table("tokens")
                    .drop_column("litecoin_account_index")
                    .to_owned(),
            )
            .await?;

        manager
            // .drop_table(Table::drop().table(LitecoinWallet::Table).to_owned())
            .drop_table(Table::drop().table("litecoin_wallet").to_owned())
            .await
    }
}
