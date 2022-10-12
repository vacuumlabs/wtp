//! SeaORM Entity. Generated by sea-orm-codegen 0.9.3

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "mean_exchange_ratio")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub token1: i64,
    pub token2: i64,
    pub ratio: f32,
    pub date: DateTime,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::token::Entity",
        from = "Column::Token1",
        to = "super::token::Column::Id",
        on_update = "NoAction",
        on_delete = "Cascade"
    )]
    Token2,
    #[sea_orm(
        belongs_to = "super::token::Entity",
        from = "Column::Token2",
        to = "super::token::Column::Id",
        on_update = "NoAction",
        on_delete = "Cascade"
    )]
    Token1,
}

impl ActiveModelBehavior for ActiveModel {}
