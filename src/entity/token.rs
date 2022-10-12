//! SeaORM Entity. Generated by sea-orm-codegen 0.9.3

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "token")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub policy_id: Vec<u8>,
    pub name: Vec<u8>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::token_transfer::Entity")]
    TokenTransfer,
}

impl Related<super::token_transfer::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::TokenTransfer.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
