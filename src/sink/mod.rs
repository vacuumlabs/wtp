use crate::{
    config, queries, server,
    types::{BroadcastMessage, ExchangeRate, SwapInfo},
    utils,
};

use oura::{model::EventData, pipelining::StageReceiver};
use sea_orm::DatabaseConnection;

pub mod common;
pub mod minswap_v1;
pub mod sundaeswap_v1;
pub mod wingriders_v1;

pub async fn start(
    input: StageReceiver,
    db: DatabaseConnection,
    pools: &[config::PoolConfig],
    persistent: bool,
) -> anyhow::Result<()> {
    tracing::info!("Starting");
    let pools: Vec<&config::PoolConfig> = pools.iter().filter(|p| p.enable).collect();

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
                        (true, true) => Some(
                            queries::insert_transaction(transaction_record, block_id.unwrap(), &db)
                                .await?,
                        ),
                        _ => None,
                    };

                    for pool in pools.iter() {
                        let script_hash = hex::decode(&pool.script_hash).unwrap();
                        let dex_trait = pool.as_trait();
                        if let Some((asset1, asset2)) =
                            dex_trait.mean_value(pool, &db, transaction_record).await
                        {
                            let asset1_id = queries::get_token_id(&asset1.asset, &db).await?;
                            let asset2_id = queries::get_token_id(&asset2.asset, &db).await?;

                            let exchange_rate = ExchangeRate {
                                asset1: asset1_id,
                                asset2: asset2_id,
                                script_hash: pool.script_hash.clone(),
                                rate: asset1.amount as f64 / asset2.amount as f64,
                            };
                            server::ws_broadcast(&BroadcastMessage::MeanValue(exchange_rate));

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
                            }
                            let swaps = dex_trait.swaps(pool, &db, transaction_record).await?;
                            for swap in swaps.iter() {
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
                                if let Some(tx_id) = tx_id {
                                    queries::insert_swap(tx_id, &script_hash, &swap_info, &db)
                                        .await?;
                                }
                                server::ws_broadcast(&BroadcastMessage::Swap(swap_info));
                            }
                            tracing::info!("SWAPS[{}] {:?}", transaction_record.hash, swaps);
                        }
                    }
                }
            }
            _ => {
                tracing::info!("{:?}", event.data);
            }
        }
    }
}
