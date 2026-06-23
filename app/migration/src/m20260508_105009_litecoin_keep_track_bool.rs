use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table("litecoin_wallet")
                    .add_column(boolean("keep_track").default(false).not_null())
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    // .table(LitecoinWallet::Table)
                    .table("litecoin_wallet")
                    .drop_column("keep_track")
                    .to_owned(),
            )
            .await
    }
}
