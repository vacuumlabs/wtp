//! SeaORM Entity. Generated by sea-orm-codegen 0.9.3

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "swap")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub tx_id: i64,
    pub script_hash: Vec<u8>,
    pub token1_id: i64,
    pub token2_id: i64,
    pub amount1: i64,
    pub amount2: i64,
    pub direction: bool,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::token::Entity",
        from = "Column::Token1Id",
        to = "super::token::Column::Id",
        on_update = "NoAction",
        on_delete = "Cascade"
    )]
    Token2,
    #[sea_orm(
        belongs_to = "super::token::Entity",
        from = "Column::Token2Id",
        to = "super::token::Column::Id",
        on_update = "NoAction",
        on_delete = "Cascade"
    )]
    Token1,
    #[sea_orm(
        belongs_to = "super::transaction::Entity",
        from = "Column::TxId",
        to = "super::transaction::Column::Id",
        on_update = "NoAction",
        on_delete = "Cascade"
    )]
    Transaction,
}

impl Related<super::transaction::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Transaction.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
