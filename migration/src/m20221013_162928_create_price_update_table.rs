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
                    .table(PriceUpdate::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(PriceUpdate::Id)
                            .big_integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(PriceUpdate::TxId).big_integer().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-price_update-tx_id")
                            .from(PriceUpdate::Table, PriceUpdate::TxId)
                            .to(Transaction::Table, Transaction::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .col(ColumnDef::new(PriceUpdate::ScriptHash).binary().not_null())
                    .col(
                        ColumnDef::new(PriceUpdate::Token1Id)
                            .big_integer()
                            .not_null(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-price_update-token1_id")
                            .from(PriceUpdate::Table, PriceUpdate::Token1Id)
                            .to(Token::Table, Token::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .col(
                        ColumnDef::new(PriceUpdate::Token2Id)
                            .big_integer()
                            .not_null(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-price_update-token2_id")
                            .from(PriceUpdate::Table, PriceUpdate::Token2Id)
                            .to(Token::Table, Token::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .col(
                        ColumnDef::new(PriceUpdate::Amount1)
                            .big_unsigned()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(PriceUpdate::Amount2)
                            .big_unsigned()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(PriceUpdate::Timestamp)
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
            .drop_table(Table::drop().table(PriceUpdate::Table).to_owned())
            .await
    }
}

/// Learn more at https://docs.rs/sea-query#iden
#[derive(Iden)]
pub enum PriceUpdate {
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
