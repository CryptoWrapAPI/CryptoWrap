use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(FiatPrices::Table)
                    .if_not_exists()
                    .col(pk_auto(FiatPrices::Id))
                    .col(string_len(FiatPrices::Coin, 20).unique_key())
                    .col(decimal(FiatPrices::Usd))
                    .col(decimal(FiatPrices::Eur))
                    .col(decimal(FiatPrices::Rub))
                    .col(timestamp_with_time_zone(FiatPrices::UpdatedAt))
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(FiatPrices::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum FiatPrices {
    Table,
    Id,
    Coin,
    Usd,
    Eur,
    Rub,
    UpdatedAt,
}
