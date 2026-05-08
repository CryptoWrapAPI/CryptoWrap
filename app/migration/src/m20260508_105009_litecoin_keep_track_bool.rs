use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[derive(DeriveIden)]
enum LitecoinWallet {
    Table,
    Id,
    AccountIndex,
    AddressIndex,
    WalletAddress,
    CreatedAt,
    LastUsedAt,
    BlockchainHeight,
    IsAvailable,
    IsChange,
    InitialBalance,
    KeepTrack,
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(LitecoinWallet::Table)
                    .add_column(boolean("keep_track").default(false).not_null())
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(LitecoinWallet::Table)
                    .drop_column("keep_track")
                    .to_owned(),
            )
            .await
    }
}
