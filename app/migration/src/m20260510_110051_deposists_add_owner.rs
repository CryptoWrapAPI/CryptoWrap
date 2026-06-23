use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    // .table(Deposits::Table)
                    .table("deposits")
                    // uuid from tokens table (not foreign key, simple uuid-string field) //
                    .add_column(uuid("owner_id").not_null())
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    // .table(Deposits::Table)
                    .table("deposits")
                    // .drop_column(Deposits::OwnerId)
                    .drop_column("owner_id")
                    .to_owned(),
            )
            .await
    }
}

// #[derive(DeriveIden)]
// enum Deposits {
//     Table,
//     OwnerId,
// }
