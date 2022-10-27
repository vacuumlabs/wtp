use sea_orm_migration::prelude::*;

use crate::m20221013_162928_create_price_update_table::PriceUpdate;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(PriceUpdate::Table)
                    .drop_column(PriceUpdate::Timestamp)
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(PriceUpdate::Table)
                    .add_column(
                        ColumnDef::new(PriceUpdate::Timestamp)
                            .timestamp()
                            .not_null()
                            .extra("DEFAULT CURRENT_TIMESTAMP".to_owned()),
                    )
                    .to_owned(),
            )
            .await
    }
}
