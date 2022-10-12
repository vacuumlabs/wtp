use sea_orm::prelude::DateTime;
use serde::Serialize;

#[derive(Debug, Serialize, Clone)]
pub struct Asset {
    pub policy_id: String,
    pub name: String,
}

#[derive(Debug, Serialize, Clone)]
pub struct AssetAmount {
    pub asset: Asset,
    pub amount: u64,
}

#[derive(Debug, Serialize, Clone)]
pub struct ExchangeRate {
    pub script_hash: String,
    pub asset1: AssetAmount,
    pub asset2: AssetAmount,
    pub rate: f64,
}

#[derive(Debug, Serialize)]
pub struct ExchangeHistory {
    pub amount1: i64,
    pub amount2: i64,
    pub rate: f64,
    pub timestamp: DateTime,
}

#[derive(Debug, Serialize)]
pub struct Swap {
    pub first: AssetAmount,
    pub second: AssetAmount,
    pub direction: bool, // false - buy, tru - sell
}

pub type PlutusData = Swap;
