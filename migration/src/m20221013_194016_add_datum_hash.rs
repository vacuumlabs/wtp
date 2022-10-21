use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(TransactionOutput::Table)
                    .add_column(
                        ColumnDef::new(TransactionOutput::DatumHash)
                            .string_len(64)
                            .null(),
                    )
                    .drop_column(TransactionOutput::Spent)
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(TransactionOutput::Table)
                    .drop_column(TransactionOutput::DatumHash)
                    .add_column(
                        ColumnDef::new(TransactionOutput::Spent)
                            .boolean()
                            .not_null()
                            .default(Value::Bool(Some(false))),
                    )
                    .to_owned(),
            )
            .await
    }
}

/// Learn more at https://docs.rs/sea-query#iden
#[derive(Iden)]
pub enum TransactionOutput {
    Table,
    DatumHash,
    Spent,
}
