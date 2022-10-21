use sea_orm_migration::prelude::*;

use crate::m20221006_114228_create_transaction_table::Transaction;
use crate::m20221007_095717_create_transaction_output_table::TransactionOutput;
use crate::m20221007_105847_create_token_transfer_table::TokenTransfer;
use crate::m20221013_162928_create_price_update_table::PriceUpdate;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Replace the sample below with your own migration scripts
        // todo!();

        manager
            .create_index(
                Index::create()
                    .table(PriceUpdate::Table)
                    .name("index-price_update-token1_id")
                    .col(PriceUpdate::Token1Id)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .table(PriceUpdate::Table)
                    .name("index-price_update-token2_id")
                    .col(PriceUpdate::Token2Id)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .table(TokenTransfer::Table)
                    .name("index-token_transfer-output_id")
                    .col(TokenTransfer::OutputId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .table(TokenTransfer::Table)
                    .name("index-token_transfer-token_id")
                    .col(TokenTransfer::TokenId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .table(Transaction::Table)
                    .name("index-transaction-hash")
                    .col(Transaction::Hash)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .table(Transaction::Table)
                    .name("index-transaction-block_id")
                    .col(Transaction::BlockId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .table(TransactionOutput::Table)
                    .name("index-transaction_output-tx_id")
                    .col(TransactionOutput::TxId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .table(TransactionOutput::Table)
                    .name("index-transaction_output-address_id")
                    .col(TransactionOutput::AddressId)
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_index(
                Index::drop()
                    .table(PriceUpdate::Table)
                    .name("index-price_update-token1_id")
                    .to_owned(),
            )
            .await?;

        manager
            .drop_index(
                Index::drop()
                    .table(PriceUpdate::Table)
                    .name("index-price_update-token2_id")
                    .to_owned(),
            )
            .await?;

        manager
            .drop_index(
                Index::drop()
                    .table(TokenTransfer::Table)
                    .name("index-token_transfer-output_id")
                    .to_owned(),
            )
            .await?;

        manager
            .drop_index(
                Index::drop()
                    .table(TokenTransfer::Table)
                    .name("index-token_transfer-token_id")
                    .to_owned(),
            )
            .await?;

        manager
            .drop_index(
                Index::drop()
                    .table(Transaction::Table)
                    .name("index-transaction-hash")
                    .to_owned(),
            )
            .await?;

        manager
            .drop_index(
                Index::drop()
                    .table(Transaction::Table)
                    .name("index-transaction-block_id")
                    .to_owned(),
            )
            .await?;

        manager
            .drop_index(
                Index::drop()
                    .table(TransactionOutput::Table)
                    .name("index-transaction_output-tx_id")
                    .to_owned(),
            )
            .await?;

        manager
            .drop_index(
                Index::drop()
                    .table(TransactionOutput::Table)
                    .name("index-transaction_output-address_id")
                    .to_owned(),
            )
            .await
    }
}
