use sea_orm_migration::prelude::*;

use crate::m20221006_140531_create_address_table::Address;
use crate::m20221006_141624_create_token_table::Token;
use crate::m20221010_123828_create_block_table::Block;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_index(
                Index::create()
                    .table(Address::Table)
                    .name("index-address-payload")
                    .col(Address::Payload)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .table(Token::Table)
                    .name("index-token-policy_id-name")
                    .col(Token::PolicyId)
                    .col(Token::Name)
                    .unique()
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .table(Block::Table)
                    .name("index-block-hash")
                    .col(Block::Hash)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .table(Block::Table)
                    .name("index-block-slot")
                    .col(Block::Slot)
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_index(
                Index::drop()
                    .table(Address::Table)
                    .name("index-address-payload")
                    .to_owned(),
            )
            .await?;

        manager
            .drop_index(
                Index::drop()
                    .table(Token::Table)
                    .name("index-token-policy_id-name")
                    .to_owned(),
            )
            .await?;

        manager
            .drop_index(
                Index::drop()
                    .table(Block::Table)
                    .name("index-block-hash")
                    .to_owned(),
            )
            .await?;

        manager
            .drop_index(
                Index::drop()
                    .table(Block::Table)
                    .name("index-block-slot")
                    .to_owned(),
            )
            .await
    }
}
