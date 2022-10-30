use crate::sink::common::Dex;
use serde::Deserialize;

use crate::types::{AssetAmount, Swap};
use async_trait::async_trait;
use oura::model::TransactionRecord;
use sea_orm::DatabaseConnection;

fn default_as_true() -> bool {
    true
}

#[derive(Deserialize, Debug)]
pub struct Config {
    pub pools: Vec<PoolConfig>,
}

#[derive(Deserialize, Debug, PartialEq, Eq)]
pub struct WingRiders;
#[derive(Deserialize, Debug, PartialEq, Eq)]
pub struct MinSwapV1;
#[derive(Deserialize, Debug, PartialEq, Eq)]
pub struct MinSwapV2;
#[derive(Deserialize, Debug, PartialEq, Eq)]
pub struct SundaeSwap;
#[derive(Deserialize, Debug, PartialEq, Eq)]
pub struct Empty;

#[derive(Debug, Deserialize, PartialEq, Eq)]
pub enum PoolType {
    WingRiders,
    SundaeSwap,
    MinSwapV1,
    MinSwapV2,
}

#[async_trait]
impl Dex for Empty {
    async fn mean_value(
        &self,
        _pool: &PoolConfig,
        _db: &DatabaseConnection,
        _transaction: &TransactionRecord,
    ) -> Option<(AssetAmount, AssetAmount)> {
        unimplemented!();
    }
    async fn swaps(
        &self,
        _pool: &PoolConfig,
        _db: &DatabaseConnection,
        _transaction: &TransactionRecord,
    ) -> anyhow::Result<Vec<Swap>> {
        unimplemented!();
    }
}

#[derive(Deserialize, Debug)]
pub struct PoolConfig {
    #[serde(default = "default_as_true")]
    pub enable: bool,
    pub script_hash: String,
    pub request_hash: String,
    pub vesting_hash: String,
    pub address: String,
    #[serde(rename = "type")]
    pub pool_type: PoolType,
}

impl PoolConfig {
    pub fn as_trait(&self) -> &dyn Dex {
        match &self.pool_type {
            PoolType::WingRiders => &WingRiders {},
            PoolType::MinSwapV1 => &MinSwapV1 {},
            PoolType::SundaeSwap => &SundaeSwap {},
            _ => &Empty {},
        }
    }
}
