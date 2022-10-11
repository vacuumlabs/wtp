use crate::config;
use oura::{
    model::{EventData, TransactionRecord, TxOutputRecord},
    pipelining::StageReceiver,
};

#[allow(dead_code)]
struct Asset {
    policy: String,
    name: String,
    amount: u64,
}

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
    script_hash: &String, // TODO: it is not prepared for hash, only for address
) -> Option<(Asset, Asset)> {
    // Find correct address
    if let Some(output) = transaction
        .outputs
        .iter()
        .flatten()
        .find(|&o| &o.address == script_hash)
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
                amount1 / amount2
            );

            return Some((
                Asset {
                    policy: policy1,
                    name: token1,
                    amount: amount1,
                },
                Asset {
                    policy: policy2,
                    name: token2,
                    amount: amount2,
                },
            ));
        }
    }
    None
}

pub async fn start(input: StageReceiver, pools: &[config::PoolConfig]) -> anyhow::Result<()> {
    tracing::info!("Starting");

    loop {
        let event = input.recv()?;

        match &event.data {
            EventData::Block(block) => {
                tracing::debug!("Block: {} {}", block.slot, block.hash);
                // TODO add block to db
            }
            EventData::RollBack {
                block_slot,
                block_hash,
            } => {
                tracing::debug!("Rollback, current block: {} {}", block_slot, block_hash);
                // TODO remove blocks from db
            }
            EventData::Transaction(transaction_record) => {
                //tracing::debug!(
                //    "Transaction: {} (in block {})",
                //    transaction_record.hash,
                //    event.context.block_hash.unwrap(),
                //);
                // TODO add transaction to db
                pools.iter().for_each(|p| {
                    if let Some((_first, _second)) =
                        get_wr_transaction(transaction_record, &p.address)
                    {
                        // Do something
                    }
                });
            }
            _ => {
                tracing::info!("{:?}", event.data);
            }
        }
    }
}
