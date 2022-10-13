use crate::{config, queries, types::Asset, utils};
use oura::{
    model::{EventData, TransactionRecord, TxOutputRecord},
    pipelining::StageReceiver,
};
use sea_orm::DatabaseConnection;

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

fn wr_transaction(policy_id: &str, asset: &str) -> u64 {
    if policy_id.is_empty() && asset.is_empty() {
        return 3;
    }
    0
}

fn get_wr_transaction(
    transaction: &TransactionRecord,
    script_hash: Vec<u8>,
) -> Option<(Asset, Asset)> {
    // Find correct address
    if let Some(output) = transaction
        .outputs
        .iter()
        .flatten()
        .find(|&o| utils::get_payment_hash(&o.address) == Some(script_hash.clone()))
    {
        // Check plutus data
        if let Some(datum) = transaction.plutus_data.iter().flatten().find(
            |&p| matches!(&output.datum_hash, Some(datum_hash) if *datum_hash == p.datum_hash),
        ) {
            // Get treasury from plutus
            let treasury1 = datum.plutus_data["fields"][1]["fields"][2]["int"].as_i64();
            let treasury2 = datum.plutus_data["fields"][1]["fields"][3]["int"].as_i64();

            // Get first token from plutus
            let policy1 = datum.plutus_data["fields"][1]["fields"][0]["fields"][0]["fields"][0]
                ["bytes"]
                .as_str()
                .unwrap()
                .to_string();

            let token1 = datum.plutus_data["fields"][1]["fields"][0]["fields"][0]["fields"][1]
                ["bytes"]
                .as_str()
                .unwrap()
                .to_string();

            // Get second token from plutus
            let policy2 = datum.plutus_data["fields"][1]["fields"][0]["fields"][1]["fields"][0]
                ["bytes"]
                .as_str()
                .unwrap()
                .to_string();
            let token2 = datum.plutus_data["fields"][1]["fields"][0]["fields"][1]["fields"][1]
                ["bytes"]
                .as_str()
                .unwrap()
                .to_string();

            // Get amount of tokens
            let amount1 = get_amount(output, &policy1, &token1)
                - (treasury1.unwrap() as u64)
                - wr_transaction(&policy1, &token1);
            let amount2 = get_amount(output, &policy2, &token2)
                - (treasury2.unwrap() as u64)
                - wr_transaction(&policy2, &token2);

            // First asset info
            tracing::info!(
                "Token 1 {}, Policy {} Treasury {:?} amount {:?}",
                token1,
                policy1,
                treasury1,
                amount1
            );

            // Second asset info
            tracing::info!(
                "Token 2 {}, Policy {} Treasury {:?} amount {:?}",
                token2,
                policy2,
                treasury2,
                amount2
            );

            tracing::info!(
                "{}/{} = {}/{} = {}",
                token1,
                token2,
                amount1,
                amount2,
                amount1 as f64 / amount2 as f64
            );

            return Some((
                Asset {
                    policy_id: hex::decode(policy1).unwrap(),
                    name: hex::decode(token1).unwrap(),
                    amount: amount1,
                },
                Asset {
                    policy_id: hex::decode(policy2).unwrap(),
                    name: hex::decode(token2).unwrap(),
                    amount: amount2,
                },
            ));
        }
    }
    None
}

pub async fn start(
    input: StageReceiver,
    db: DatabaseConnection,
    pools: &[config::PoolConfig],
) -> anyhow::Result<()> {
    tracing::info!("Starting");

    loop {
        let event = input.recv()?;

        match &event.data {
            EventData::Block(block) => {
                tracing::debug!("Block: {} {}", block.slot, block.hash);
                queries::insert_block(block, &db).await?;
            }
            EventData::RollBack {
                block_slot,
                block_hash,
            } => {
                tracing::debug!("Rollback, current block: {} {}", block_slot, block_hash);
                queries::rollback_to_slot(block_slot, &db).await?;
            }
            EventData::Transaction(transaction_record) => {
                let block_hash = event
                    .context
                    .block_hash
                    .ok_or_else(|| anyhow::anyhow!("No block hash"))?;
                let tx_id =
                    queries::insert_transaction(transaction_record, &block_hash, &db).await?;

                for pool in pools {
                    let script_hash = hex::decode(pool.script_hash.clone()).unwrap();
                    if let Some((asset1, asset2)) =
                        get_wr_transaction(transaction_record, script_hash.clone())
                    {
                        queries::insert_price_update(tx_id, script_hash, asset1, asset2, &db)
                            .await?;
                    }
                }
            }
            _ => {
                tracing::info!("{:?}", event.data);
            }
        }
    }
}
