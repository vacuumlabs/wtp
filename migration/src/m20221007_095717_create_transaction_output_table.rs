use sea_orm_migration::prelude::*;

use crate::m20221006_114228_create_transaction_table::Transaction;
use crate::m20221006_140531_create_address_table::Address;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(TransactionOutput::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(TransactionOutput::Id)
                            .big_integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(TransactionOutput::TxId)
                            .big_integer()
                            .not_null(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-transaction_output-tx_id")
                            .from(TransactionOutput::Table, TransactionOutput::TxId)
                            .to(Transaction::Table, Transaction::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .col(
                        ColumnDef::new(TransactionOutput::Index)
                            .integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(TransactionOutput::AddressId)
                            .big_integer()
                            .not_null(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-transaction_output-address_id")
                            .from(TransactionOutput::Table, TransactionOutput::AddressId)
                            .to(Address::Table, Address::Id)
                            .on_delete(ForeignKeyAction::NoAction),
                    )
                    .col(
                        ColumnDef::new(TransactionOutput::Spent)
                            .boolean()
                            .not_null(),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(TransactionOutput::Table).to_owned())
            .await
    }
}

/// Learn more at https://docs.rs/sea-query#iden
#[derive(Iden)]
pub enum TransactionOutput {
    Table,
    Id,
    TxId,
    Index,
    AddressId,
    Spent,
}
