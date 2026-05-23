use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Withdrawals::Table) // only broadcasted/relayed txs are going in the withdrawals db table
                    .if_not_exists()
                    .col(
                        uuid("transaction_id")
                            .primary_key()
                            .default(Expr::cust("gen_random_uuid()"))
                            .unique_key(),
                    )
                    .col(uuid("user_uuid").not_null())
                    .col(string("amount").not_null())
                    .col(string("coin_id").not_null())
                    .col(string("destination_address").not_null())
                    .col(
                        date_time("created_at")
                            .default(Expr::current_timestamp())
                            .not_null(),
                    )
                    // .col(date_time("updated_at").null())
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Withdrawals::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum Withdrawals {
    Table,
}
