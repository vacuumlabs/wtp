use crate::{
    config::{PoolConfig, WingRidersV1},
    queries,
    sink::common,
    types::{Asset, AssetAmount, Swap},
    utils,
};
use async_trait::async_trait;
use oura::model::TransactionRecord;
use sea_orm::DatabaseConnection;

static WR_ADA_POOL: u64 = 3_000_000;
static WR_ADA_SWAP_IN: u64 = 4_000_000;
static WR_ADA_SWAP_OUT: u64 = 2_000_000;

fn wr_extract_plutus_assets(datum: &serde_json::Value) -> (AssetAmount, AssetAmount) {
    (
        AssetAmount {
            asset: Asset {
                policy_id: datum["fields"][0]["fields"][0]["fields"][0]["bytes"]
                    .as_str()
                    .unwrap()
                    .to_string(),
                name: datum["fields"][0]["fields"][0]["fields"][1]["bytes"]
                    .as_str()
                    .unwrap()
                    .to_string(),
            },
            amount: datum["fields"][2]["int"].as_i64().unwrap() as u64,
        },
        AssetAmount {
            asset: Asset {
                policy_id: datum["fields"][0]["fields"][1]["fields"][0]["bytes"]
                    .as_str()
                    .unwrap()
                    .to_string(),
                name: datum["fields"][0]["fields"][1]["fields"][1]["bytes"]
                    .as_str()
                    .unwrap()
                    .to_string(),
            },
            amount: datum["fields"][3]["int"].as_i64().unwrap() as u64,
        },
    )
}

#[async_trait]
impl common::Dex for WingRidersV1 {
    async fn mean_value(
        &self,
        pool: &PoolConfig,
        _db: &DatabaseConnection,
        transaction: &TransactionRecord,
    ) -> Option<(AssetAmount, AssetAmount)> {
        let script_hash = hex::decode(&pool.script_hash).unwrap();
        // Find correct address
        if let Some(output) = transaction
            .outputs
            .iter()
            .flatten()
            .find(|&o| utils::get_payment_hash(&o.address) == Some(script_hash.to_vec()))
        {
            // Check plutus data
            if let Some(datum) = transaction.plutus_data.iter().flatten().find(
                |&p| matches!(&output.datum_hash, Some(datum_hash) if *datum_hash == p.datum_hash),
            ) {
                // Get treasury from plutus
                let (asset1, asset2) = wr_extract_plutus_assets(&datum.plutus_data["fields"][1]);
                // Get amount of tokens
                let amount1 =
                    common::get_amount(output, &asset1.asset.policy_id, &asset1.asset.name)
                        - asset1.amount
                        - common::reduce_ada_amount(
                            &asset1.asset.policy_id,
                            &asset1.asset.name,
                            WR_ADA_POOL,
                        );
                let amount2 =
                    common::get_amount(output, &asset2.asset.policy_id, &asset2.asset.name)
                        - asset2.amount
                        - common::reduce_ada_amount(
                            &asset2.asset.policy_id,
                            &asset2.asset.name,
                            WR_ADA_POOL,
                        );

                return Some((
                    AssetAmount {
                        asset: Asset {
                            policy_id: asset1.asset.policy_id,
                            name: asset1.asset.name,
                        },
                        amount: amount1,
                    },
                    AssetAmount {
                        asset: Asset {
                            policy_id: asset2.asset.policy_id,
                            name: asset2.asset.name,
                        },
                        amount: amount2,
                    },
                ));
            }
        }
        None
    }
    async fn swaps(
        &self,
        _pool: &PoolConfig,
        db: &DatabaseConnection,
        transaction: &TransactionRecord,
    ) -> anyhow::Result<Vec<Swap>> {
        // Map inputs
        let mut swaps: Vec<Swap> = Vec::new();

        // Get pool input from redemeers
        let pool_input = transaction.plutus_redeemers.as_ref().unwrap()[0].plutus_data["fields"][0]
            ["int"]
            .as_i64();

        if let Some(pool_input) = pool_input {
            let pool_input = pool_input as usize;
            // Find main redemeer
            if let Some(redeemer) = transaction
                .plutus_redeemers
                .iter()
                .flatten()
                .find(|r| (r.input_idx as usize) == pool_input)
            {
                // Extract input list from redemeer
                let redeemer_map: Vec<usize> = redeemer.plutus_data["fields"][2]["list"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|r| r["int"].as_i64().unwrap() as usize)
                    .collect();

                // Restore inputs
                let inputs =
                    queries::get_utxo_input(&transaction.inputs.clone().unwrap(), db).await?;
                // Zip outputs with redemeer index

                // get information about swap from pool plutus data
                if let Some(pool_datum) = transaction.plutus_data.iter().flatten().find(|p| {
                    p.datum_hash == inputs[pool_input].clone().unwrap().datum_hash.unwrap()
                }) {
                    let (asset1, asset2) =
                        wr_extract_plutus_assets(&pool_datum.plutus_data["fields"][1]);

                    for (out, redeemer) in transaction
                        .outputs
                        .iter()
                        .flatten()
                        .skip(1)
                        .zip(redeemer_map)
                    {
                        if inputs[redeemer].is_none() {
                            tracing::info!("Missing UTxO on {}", transaction.hash);
                            continue;
                        }
                        // pair input with output
                        let inp = inputs[redeemer].clone().unwrap();
                        let amount1;
                        let amount2;
                        // get actual plutus data
                        if let Some(datum) = transaction
                            .plutus_data
                            .iter()
                            .flatten()
                            .find(|p| p.datum_hash == inp.datum_hash.as_ref().unwrap().clone())
                        {
                            // identify operation 0 - swap
                            let operation = datum.plutus_data["fields"][1]["constructor"]
                                .as_i64()
                                .unwrap();
                            if operation == 0 {
                                let direction = datum.plutus_data["fields"][1]["fields"][0]
                                    ["constructor"]
                                    .as_i64()
                                    .unwrap();
                                if direction == 0 {
                                    amount1 = common::get_amount(
                                        &inp,
                                        &asset1.asset.policy_id,
                                        &asset1.asset.name,
                                    ) - common::reduce_ada_amount(
                                        &asset1.asset.policy_id,
                                        &asset1.asset.name,
                                        WR_ADA_SWAP_IN,
                                    );
                                    amount2 = common::get_amount(
                                        out,
                                        &asset2.asset.policy_id,
                                        &asset2.asset.name,
                                    ) - common::reduce_ada_amount(
                                        &asset2.asset.policy_id,
                                        &asset2.asset.name,
                                        WR_ADA_SWAP_OUT,
                                    );
                                } else {
                                    amount1 = common::get_amount(
                                        out,
                                        &asset1.asset.policy_id,
                                        &asset1.asset.name,
                                    ) - common::reduce_ada_amount(
                                        &asset1.asset.policy_id,
                                        &asset1.asset.name,
                                        WR_ADA_SWAP_OUT,
                                    );
                                    amount2 = common::get_amount(
                                        &inp,
                                        &asset2.asset.policy_id,
                                        &asset2.asset.name,
                                    ) - common::reduce_ada_amount(
                                        &asset2.asset.policy_id,
                                        &asset2.asset.name,
                                        WR_ADA_SWAP_IN,
                                    );
                                }
                                swaps.push(Swap {
                                    first: AssetAmount {
                                        asset: Asset {
                                            policy_id: asset1.asset.policy_id.clone(),
                                            name: asset1.asset.name.clone(),
                                        },
                                        amount: amount1,
                                    },
                                    second: AssetAmount {
                                        asset: Asset {
                                            policy_id: asset2.asset.policy_id.clone(),
                                            name: asset2.asset.name.clone(),
                                        },
                                        amount: amount2,
                                    },
                                    direction: direction == 0,
                                })
                            } else {
                                tracing::info!("Operation is not swap");
                            }
                        }
                    }
                }
            } else {
                tracing::info!("Redeemer not found");
            }
        }
        Ok(swaps)
    }
}
