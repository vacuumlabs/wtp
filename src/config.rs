use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct Config {
    pub pools: Vec<PoolConfig>,
}

#[derive(Deserialize, Debug)]
pub struct PoolConfig {
    pub address: String,
    pub token1: String,
    pub token2: String,
}
