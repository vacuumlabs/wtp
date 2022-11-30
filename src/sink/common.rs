use crate::{
    config::PoolConfig,
    types::{AssetAmount, Swap},
};
use async_trait::async_trait;
use oura::model::{TransactionRecord, TxOutputRecord};
use sea_orm::DatabaseConnection;

#[async_trait]
pub trait Dex {
    async fn mean_value(
        &self,
        pool: &PoolConfig,
        db: &DatabaseConnection,
        transaction: &TransactionRecord,
    ) -> Option<(AssetAmount, AssetAmount)>;
    async fn swaps(
        &self,
        pool: &PoolConfig,
        db: &DatabaseConnection,
        transaction: &TransactionRecord,
    ) -> anyhow::Result<Vec<Swap>>;
}

pub fn get_amount(output: &TxOutputRecord, policy_id: &str, asset: &str) -> u64 {
    if asset.is_empty() && policy_id.is_empty() {
        return output.amount as u64;
    }
    output
        .assets
        .iter()
        .flatten()
        .filter(|a| a.asset == *asset && a.policy == *policy_id)
        .fold(0, |sum, a| sum + a.amount) as u64
}

pub fn reduce_ada_amount(policy_id: &str, asset: &str, amount: u64) -> u64 {
    if policy_id.is_empty() && asset.is_empty() {
        return amount;
    }
    0
}
