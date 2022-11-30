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
    pub asset1: i64,
    pub asset2: i64,
    pub rate: f64,
}

#[derive(Debug, Serialize)]
pub struct ExchangeHistory {
    pub amount1: i64,
    pub amount2: i64,
    pub rate: f64,
    pub tx_id: i64,
}

#[derive(Debug, Serialize)]
pub struct Swap {
    pub first: AssetAmount,
    pub second: AssetAmount,
    pub direction: bool, // false - buy, tru - sell
}

#[derive(Debug, Serialize)]
pub struct SwapHistory {
    pub amount1: i64,
    pub amount2: i64,
    pub tx_id: i64,
    pub direction: String,
}

#[derive(Debug, Serialize)]
pub struct SwapInfo {
    pub asset1: i64,
    pub amount1: i64,
    pub asset2: i64,
    pub amount2: i64,
    pub direction: String,
}

#[derive(Debug, Serialize)]
#[serde(tag = "operation", content = "data")]
pub enum BroadcastMessage {
    MeanValue(ExchangeRate),
    Swap(SwapInfo),
}
