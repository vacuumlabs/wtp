use serde::Serialize;

#[derive(Debug, Serialize, Clone)]
pub struct Asset {
    pub policy_id: String,
    pub name: String,
    pub amount: u64,
}

#[derive(Debug, Serialize, Clone)]
pub struct ExchangeRate {
    pub script_hash: String,
    pub asset1: Asset,
    pub asset2: Asset,
    pub rate: f64,
}
