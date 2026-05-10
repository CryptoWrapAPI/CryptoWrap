use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Invoices::Table)
                    .if_not_exists()
                    .col(
                        uuid("invoice_id")
                            .primary_key()
                            .default(Expr::cust("gen_random_uuid()"))
                            .unique_key(),       
                    )
                    .col(string("currency").string_len(10).not_null())
                    .col(string("network").string_len(20).not_null())
                    .col(string("wallet_address").not_null())
                    .col(string("amount_requested").not_null())
                    .col(
                        string("amount_received")
                            .default(Expr::value("0"))
                    )
                    .col(string("payment_status").string_len(20).not_null())
                    .col(integer("confirmations").null())
                    .col(
                        ColumnDef::new(Invoices::Transactions)
                            .custom(Alias::new("jsonb"))
                            .default(Expr::cust("'[]'::jsonb"))
                            .null(),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Invoices::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum Invoices {
    Table,
    InvoiceId,
    Currency,
    Network,
    WalletAddress,
    AmountRequested,
    AmountReceived,
    PaymentStatus,
    Confirmations,
    Transactions,
}
