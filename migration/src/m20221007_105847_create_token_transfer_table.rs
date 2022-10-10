use sea_orm_migration::prelude::*;

use crate::m20221006_141624_create_token_table::Token;
use crate::m20221007_095717_create_transaction_output_table::TransactionOutput;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(TokenTransfer::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(TokenTransfer::Id)
                            .big_integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(TokenTransfer::OutputId)
                            .big_integer()
                            .not_null(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-token_transfer-output_id")
                            .from(TokenTransfer::Table, TokenTransfer::OutputId)
                            .to(TransactionOutput::Table, TransactionOutput::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .col(
                        ColumnDef::new(TokenTransfer::TokenId)
                            .big_integer()
                            .not_null(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-token_transfer-token_id")
                            .from(TokenTransfer::Table, TokenTransfer::TokenId)
                            .to(Token::Table, Token::Id)
                            .on_delete(ForeignKeyAction::NoAction),
                    )
                    .col(
                        ColumnDef::new(TokenTransfer::Amount)
                            .big_unsigned()
                            .not_null(),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(TokenTransfer::Table).to_owned())
            .await
    }
}

/// Learn more at https://docs.rs/sea-query#iden
#[derive(Iden)]
pub enum TokenTransfer {
    Table,
    Id,
    OutputId,
    TokenId,
    Amount,
}
