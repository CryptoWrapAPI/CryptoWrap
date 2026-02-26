use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(MoneroWallet::Table)
                    .if_not_exists()
                    .col(integer("id").primary_key().auto_increment())
                    .col(integer("major_index").not_null())
                    .col(integer("minor_index").not_null())
                    .col(string("wallet_address").not_null())
                    .col(
                        date_time("created_at")
                            .default(Expr::current_timestamp())
                            .not_null(),
                    )
                    .col(
                        date_time("last_used_at"), // .default(Expr::current_timestamp())
                                                   // .not_null(),
                    )
                    .col(integer("blockchain_height").not_null())
                    .col(boolean("is_available").default(true).not_null())
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(MoneroWallet::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum MoneroWallet {
    Table,
    Id,
    MajorIndex,
    MinorIndex,
    WalletAddress,
    CreatedAt,
    LastUsedAt,
    BlockchainHeight,
    IsAvailable,
}
