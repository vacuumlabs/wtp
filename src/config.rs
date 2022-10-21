use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct Config {
    pub pools: Vec<PoolConfig>,
}

#[derive(Deserialize, Debug)]
pub struct PoolConfig {
    pub script_hash: String,
    pub request_hash: String,
    pub vesting_hash: String,
    pub address: String,
}
