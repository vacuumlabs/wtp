pub use sea_orm_migration::prelude::*;

mod m20221006_114228_create_transaction_table;
mod m20221006_140531_create_address_table;
mod m20221006_141624_create_token_table;
mod m20221007_095717_create_transaction_output_table;
mod m20221007_105847_create_token_transfer_table;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20221006_114228_create_transaction_table::Migration),
            Box::new(m20221006_140531_create_address_table::Migration),
            Box::new(m20221006_141624_create_token_table::Migration),
            Box::new(m20221007_095717_create_transaction_output_table::Migration),
            Box::new(m20221007_105847_create_token_transfer_table::Migration),
        ]
    }
}
