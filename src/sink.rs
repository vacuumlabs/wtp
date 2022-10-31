use crate::{
    config, queries, server,
    types::{Asset, AssetAmount, BroadcastMessage, BroadcastType, ExchangeRate, PlutusData, Swap, SwapInfo},
    utils,
};
use oura::{
    model::{EventData, TransactionRecord, TxOutputRecord},
    pipelining::StageReceiver,
};
use sea_orm::DatabaseConnection;

static WR_ADA_POOL: u64 = 3_000_000;
static WR_ADA_SWAP_IN: u64 = 4_000_000;
static WR_ADA_SWAP_OUT: u64 = 2_000_000;

fn get_amount(output: &TxOutputRecord, policy_id: &str, asset: &str) -> u64 {
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

fn wr_extract_plutus_datum(datum: &serde_json::Value) -> PlutusData {
    PlutusData {
        first: AssetAmount {
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
        second: AssetAmount {
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
        direction: false,
    }
}

fn wr_transaction(policy_id: &str, asset: &str, amount: u64) -> u64 {
    if policy_id.is_empty() && asset.is_empty() {
        return amount;
    }
    0
}

fn wr_get_transaction(
    transaction: &TransactionRecord,
    script_hash: &[u8],
) -> Option<(AssetAmount, AssetAmount)> {
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
            let plutus_datum = wr_extract_plutus_datum(&datum.plutus_data["fields"][1]);
            // Get amount of tokens
            let amount1 = get_amount(
                output,
                &plutus_datum.first.asset.policy_id,
                &plutus_datum.first.asset.name,
            ) - plutus_datum.first.amount
                - wr_transaction(
                    &plutus_datum.first.asset.policy_id,
                    &plutus_datum.first.asset.name,
                    WR_ADA_POOL,
                );
            let amount2 = get_amount(
                output,
                &plutus_datum.second.asset.policy_id,
                &plutus_datum.second.asset.name,
            ) - plutus_datum.second.amount
                - wr_transaction(
                    &plutus_datum.second.asset.policy_id,
                    &plutus_datum.second.asset.name,
                    WR_ADA_POOL,
                );

            return Some((
                AssetAmount {
                    asset: Asset {
                        policy_id: plutus_datum.first.asset.policy_id,
                        name: plutus_datum.first.asset.name,
                    },
                    amount: amount1,
                },
                AssetAmount {
                    asset: Asset {
                        policy_id: plutus_datum.second.asset.policy_id,
                        name: plutus_datum.second.asset.name,
                    },
                    amount: amount2,
                },
            ));
        }
    }
    None
}

async fn wr_get_swaps(
    transaction: &TransactionRecord,
    db: &DatabaseConnection,
) -> anyhow::Result<Vec<Swap>> {
    // Map inputs
    let mut swaps: Vec<Swap> = Vec::new();

    tracing::info!("{:?}", transaction.plutus_redeemers);

    // Get pool input from redemeers
    let pool_input = transaction
        .plutus_redeemers
        .iter()
        .flatten()
        .find(|_| true)
        .unwrap()
        .plutus_data["fields"][0]["int"]
        .as_i64();

    if let Some(pool_input) = pool_input {
        // Find main redemeer
        if let Some(redeemer) = transaction
            .plutus_redeemers
            .iter()
            .flatten()
            .find(|r| (r.input_idx as usize) == pool_input as usize)
        {
            // Extract input list from redemeer
            let redeemer_map: Vec<usize> = redeemer.plutus_data["fields"][2]["list"]
                .as_array()
                .unwrap()
                .iter()
                .map(|r| r["int"].as_i64().unwrap() as usize)
                .collect();

            // Find main transaction
            let mother = redeemer.plutus_data["fields"][0]["int"].as_i64().unwrap() as usize;

            // Restore inputs
            let inputs = queries::get_utxo_input(&transaction.inputs.clone().unwrap(), db).await?;

            // Zip outputs with redemeer index
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
                // tracing::info!("{:?} -> {:?} | {:?}", redeemer, out, inp);

                // get information about swap from pool plutus data
                if let Some(datum) =
                    transaction.plutus_data.iter().flatten().find(|p| {
                        p.datum_hash == inputs[mother].clone().unwrap().datum_hash.unwrap()
                    })
                {
                    let plutus_datum = wr_extract_plutus_datum(&datum.plutus_data["fields"][1]);
                    let amount1;
                    let amount2;

                    // get actual plutus data
                    let datum = transaction
                        .plutus_data
                        .iter()
                        .flatten()
                        .find(|p| p.datum_hash == inp.datum_hash.as_ref().unwrap().clone())
                        .unwrap();

                    // identify operation 0 - swap
                    let operation = datum.plutus_data["fields"][1]["constructor"]
                        .as_i64()
                        .unwrap();

                    if operation == 0 {
                        let direction = datum.plutus_data["fields"][1]["fields"][0]["constructor"]
                            .as_i64()
                            .unwrap();

                        if direction == 0 {
                            amount1 = get_amount(
                                &inp,
                                &plutus_datum.first.asset.policy_id,
                                &plutus_datum.first.asset.name,
                            ) - wr_transaction(
                                &plutus_datum.first.asset.policy_id,
                                &plutus_datum.first.asset.name,
                                WR_ADA_SWAP_IN,
                            );
                            amount2 = get_amount(
                                out,
                                &plutus_datum.second.asset.policy_id,
                                &plutus_datum.second.asset.name,
                            ) - wr_transaction(
                                &plutus_datum.second.asset.policy_id,
                                &plutus_datum.second.asset.name,
                                WR_ADA_SWAP_OUT,
                            );
                        } else {
                            amount1 = get_amount(
                                out,
                                &plutus_datum.first.asset.policy_id,
                                &plutus_datum.first.asset.name,
                            ) - wr_transaction(
                                &plutus_datum.first.asset.policy_id,
                                &plutus_datum.first.asset.name,
                                WR_ADA_SWAP_OUT,
                            );
                            amount2 = get_amount(
                                &inp,
                                &plutus_datum.second.asset.policy_id,
                                &plutus_datum.second.asset.name,
                            ) - wr_transaction(
                                &plutus_datum.second.asset.policy_id,
                                &plutus_datum.second.asset.name,
                                WR_ADA_SWAP_IN,
                            );
                        }

                        swaps.push(Swap {
                            first: AssetAmount {
                                asset: Asset {
                                    policy_id: plutus_datum.first.asset.policy_id,
                                    name: plutus_datum.first.asset.name,
                                },
                                amount: amount1,
                            },
                            second: AssetAmount {
                                asset: Asset {
                                    policy_id: plutus_datum.second.asset.policy_id,
                                    name: plutus_datum.second.asset.name,
                                },
                                amount: amount2,
                            },
                            direction: direction == 0,
                        })
                    } else {
                        tracing::info!("Operation is not swap");
                    }
                } else {
                    tracing::info!("Datum not found");
                }
            }
        } else {
            tracing::info!("Redeemer not found");
        }
    }
    Ok(swaps)
}

pub async fn start(
    input: StageReceiver,
    db: DatabaseConnection,
    pools: &[config::PoolConfig],
    persistent: bool,
) -> anyhow::Result<()> {
    tracing::info!("Starting");

    loop {
        let event = input.recv()?;

        match &event.data {
            EventData::RollBack {
                block_slot,
                block_hash,
            } => {
                tracing::debug!("Rollback, current block: {} {}", block_slot, block_hash);
                if persistent {
                    queries::rollback_to_slot(block_slot, &db).await?;
                }
            }

            EventData::Block(block) => {
                tracing::debug!("Block: {} {}", block.slot, block.hash);

                let block_id = match persistent {
                    true => Some(queries::insert_block(block, &db).await?),
                    _ => None,
                };

                for transaction_record in block.transactions.iter().flatten() {
                    let watched = pools.iter().any(|p| {
                        let pool_hash = hex::decode(&p.script_hash).unwrap();
                        let request_hash = hex::decode(&p.request_hash).unwrap();
                        let vesting_hash = hex::decode(&p.vesting_hash).unwrap();

                        transaction_record.outputs.iter().flatten().any(|o| {
                            let hash = utils::get_payment_hash(&o.address).unwrap_or_default();

                            pool_hash == hash
                                || request_hash == hash
                                || vesting_hash == hash
                                || o.address == p.address
                        })
                    });

                    let tx_id = match (persistent, watched) {
                        (true, true) => {
                            tracing::info!("tx_id {:?}", transaction_record.inputs);
                            Some(
                                queries::insert_transaction(
                                    transaction_record,
                                    block_id.unwrap(),
                                    &db,
                                )
                                .await?,
                            )
                        }
                        _ => None,
                    };

                    for pool in pools {
                        let script_hash = hex::decode(&pool.script_hash).unwrap();
                        if let Some((asset1, asset2)) =
                            wr_get_transaction(transaction_record, &script_hash)
                        {
                            let asset1_id = queries::get_token_id(&asset1.asset, &db).await?;
                            let asset2_id = queries::get_token_id(&asset2.asset, &db).await?;

                            let exchange_rate = ExchangeRate {
                                asset1: asset1_id,
                                asset2: asset2_id,
                                script_hash: pool.script_hash.clone(),
                                rate: asset1.amount as f64 / asset2.amount as f64,
                            };
                            server::ws_broadcast(
                                serde_json::to_string(&BroadcastMessage {
                                    operation: BroadcastType::MeanValue,
                                    data: serde_json::to_string(&exchange_rate).unwrap(),
                                })
                                .unwrap(),
                            );

                            if let Some(tx_id) = tx_id {
                                queries::insert_price_update(
                                    tx_id,
                                    &script_hash,
                                    asset1_id,
                                    asset1.amount as i64,
                                    asset2_id,
                                    asset2.amount as i64,
                                    &db,
                                )
                                .await?;

                                tracing::info!(
                                    "price update: {} {:?} {:?}",
                                    transaction_record.hash,
                                    asset1,
                                    asset2
                                );
                            }

                            let swaps = wr_get_swaps(transaction_record, &db).await;
                            if let Some(tx_id) = tx_id {
                                for swap in swaps.iter().flatten() {
                                    let swap_info = SwapInfo {
                                        asset1: asset1_id,
                                        amount1: swap.first.amount as i64,
                                        asset2: asset2_id,
                                        amount2: swap.second.amount as i64,
                                        direction: match swap.direction {
                                            true => "Buy".to_string(),
                                            false => "Sell".to_string(),
                                        },
                                    };
                                    queries::insert_swap(tx_id, &script_hash, &swap_info, &db)
                                        .await?;
                                    server::ws_broadcast(
                                        serde_json::to_string(&BroadcastMessage {
                                            operation: BroadcastType::Swap,
                                            data: serde_json::to_string(&swap_info).unwrap(),
                                        })
                                        .unwrap(),
                                    );
                                }
                            }
                            tracing::info!("SWAPS[{}] {:?}", transaction_record.hash, swaps);
                        }
                    }
                }
                tracing::debug!("Block ends");
            }
            _ => {
                tracing::info!("{:?}", event.data);
            }
        }
    }
}
