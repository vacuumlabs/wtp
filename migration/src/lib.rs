pub use sea_orm_migration::prelude::*;

mod m20221006_114228_create_transaction_table;
mod m20221006_140531_create_address_table;
mod m20221006_141624_create_token_table;
mod m20221007_095717_create_transaction_output_table;
mod m20221007_105847_create_token_transfer_table;
mod m20221010_123828_create_block_table;
mod m20221013_162928_create_price_update_table;
mod m20221013_194016_add_datum_hash;
mod m20221014_125218_create_indices;
mod m20221021_115605_add_indices_to_foreign_keys;
mod m20221024_135934_create_swap_table;

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
            Box::new(m20221010_123828_create_block_table::Migration),
            Box::new(m20221013_162928_create_price_update_table::Migration),
            Box::new(m20221013_194016_add_datum_hash::Migration),
            Box::new(m20221014_125218_create_indices::Migration),
            Box::new(m20221024_135934_create_swap_table::Migration),
        ]
    }
}
