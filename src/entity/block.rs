//! SeaORM Entity. Generated by sea-orm-codegen 0.9.3

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "block")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    #[sea_orm(unique)]
    pub hash: Vec<u8>,
    pub height: i32,
    pub epoch: i32,
    pub slot: i32,
    pub previous_block_id: i64,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "Entity",
        from = "Column::PreviousBlockId",
        to = "Column::Id",
        on_update = "NoAction",
        on_delete = "Cascade"
    )]
    SelfRef,
    #[sea_orm(has_many = "super::transaction::Entity")]
    Transaction,
}

impl Related<super::transaction::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Transaction.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}