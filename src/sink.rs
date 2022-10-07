use crate::config;
use oura::{model::EventData, pipelining::StageReceiver};

pub async fn start(input: StageReceiver, pools: &[config::PoolConfig]) -> anyhow::Result<()> {
    tracing::info!("Starting");
    loop {
        let event = input.recv()?;

        match &event.data {
            EventData::Transaction(transaction_record) => {
                if let Some(outputs) = &transaction_record.outputs {
                    let mut pool = None;
                    for output in outputs {
                        pool = pool.or_else(|| pools.iter().find(|p| p.address == output.address));
                    }
                    if let Some(pool) = pool {
                        tracing::info!("Found transaction for addr: {}", pool.address);
                        for output in outputs {
                            tracing::info!(
                                "Address: {} {}, {:?}",
                                output.address,
                                output.amount,
                                output.assets
                            );
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
