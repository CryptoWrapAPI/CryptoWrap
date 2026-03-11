use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Deposits::Table)
                    .if_not_exists()
                    .col(
                        uuid("deposit_id")
                            .primary_key()
                            .default(Expr::cust("gen_random_uuid()"))
                            .unique_key(),
                    )
                    .col(string("currency").string_len(10).not_null())
                    .col(string("network").string_len(20).not_null())
                    .col(string("wallet_address").not_null())
                    .col(string("amount_received").default(Expr::value("0")))
                    .col(string("payment_status").string_len(20).not_null())
                    // .col(integer("min_blockchain_height").null()) // will be suitable for monero, litecoin, etc (address re-use, to scan for new transfers from current height)
                    .col(integer("confirmations").null())
                    .col(string("txid").null()) // transaction hash
                    // theoretically major/minor indexes or uuid of row in (coin) table wallet can be added
                    // (monero accounts / subaddresses)
                    //
                    // yes, we need major and minor indicies for get_transfers monero-wallet-rpc method/endpoint/function/call (remote procedure)
                    // we will save it here to avoid querying monero_wallet table (or we can call it and also remove min_blockchiain_height, because we can get it there ... ?)
                    // actually let's get blockchain height from wallet specific table, it will be more sane, look better, and this one query isn't really a heavy.
                    //
                    .col(
                        date_time("created_at")
                            .default(Expr::current_timestamp())
                            .not_null(),
                    )
                    .col(date_time("updated_at").null())
                    // add finalized bool field, if deposit is finilized - it means associated wallet address is freed and can be reused
                    // it also means no wallet rpc check will be performed anymore - tx information will be just returned from db
                    // and updated_at won't be updated
                    //
                    // background finalizer will do the job for payments that were satisfied just by detected or a few confirmations
                    // (while to finalize funds must be unlocked, 20 confs for monero, maybe in other coins finalization will be faster)
                    .col(boolean("finalized").default(false))
                    .col(string("notify_url").null())
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Deposits::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum Deposits {
    Table,
    DepositId,
    Currency,
    Network,
    WalletAddress,
    AmountReceived,
    PaymentStatus,
    Confirmations,
    CreatedAt,
    UpdatedAt,
    Txid,
    Finalized,
}
