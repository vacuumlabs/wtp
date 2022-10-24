use sea_orm_migration::prelude::*;

use crate::{
    m20221006_114228_create_transaction_table::Transaction,
    m20221006_141624_create_token_table::Token,
};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Swap::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Swap::Id)
                            .big_integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Swap::TxId).big_integer().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-swap-tx_id")
                            .from(Swap::Table, Swap::TxId)
                            .to(Transaction::Table, Transaction::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .col(ColumnDef::new(Swap::ScriptHash).binary().not_null())
                    .col(
                        ColumnDef::new(Swap::Token1Id)
                            .big_integer()
                            .not_null(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-swap-token1_id")
                            .from(Swap::Table, Swap::Token1Id)
                            .to(Token::Table, Token::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .col(
                        ColumnDef::new(Swap::Token2Id)
                            .big_integer()
                            .not_null(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-swap-token2_id")
                            .from(Swap::Table, Swap::Token2Id)
                            .to(Token::Table, Token::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .col(
                        ColumnDef::new(Swap::Amount1)
                            .big_unsigned()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Swap::Amount2)
                            .big_unsigned()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Swap::Timestamp)
                            .timestamp()
                            .not_null()
                            .extra("DEFAULT CURRENT_TIMESTAMP".to_owned()),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Swap::Table).to_owned())
            .await
    }
}

/// Learn more at https://docs.rs/sea-query#iden
#[derive(Iden)]
pub enum Swap {
    Table,
    Id,
    TxId,
    ScriptHash,
    Token1Id,
    Token2Id,
    Amount1,
    Amount2,
    Timestamp,
}
