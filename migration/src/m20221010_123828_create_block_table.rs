use sea_orm_migration::prelude::*;

use crate::m20221006_114228_create_transaction_table::Transaction;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Block::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Block::Id)
                            .big_integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Block::Hash).binary().not_null().unique_key())
                    .col(ColumnDef::new(Block::Height).integer().not_null())
                    .col(ColumnDef::new(Block::Epoch).integer().not_null())
                    .col(ColumnDef::new(Block::Slot).integer().not_null())
                    .col(
                        ColumnDef::new(Block::PreviousBlockId)
                            .big_integer()
                            .not_null(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-block-previous_block_id")
                            .from(Block::Table, Block::PreviousBlockId)
                            .to(Block::Table, Block::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(Transaction::Table)
                    .add_column_if_not_exists(
                        ColumnDef::new(Transaction::BlockId)
                            .big_integer()
                            .not_null(),
                    )
                    .add_foreign_key(
                        ForeignKey::create()
                            .name("fk-transaction-block_id")
                            .from(Transaction::Table, Transaction::BlockId)
                            .to(Block::Table, Block::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .get_foreign_key(),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Transaction::Table)
                    .drop_column(Transaction::BlockId)
                    .to_owned(),
            )
            .await?;

        manager
            .drop_table(Table::drop().table(Block::Table).to_owned())
            .await
    }
}

/// Learn more at https://docs.rs/sea-query#iden
#[derive(Iden)]
pub enum Block {
    Table,
    Id,
    Hash,
    Height,
    Epoch,
    Slot,
    PreviousBlockId,
}
