use oura::{
    filters::selection::{self, Predicate},
    mapper,
    model::{BlockRecord, EventData},
    pipelining::{FilterProvider, SourceProvider, StageReceiver},
    sources::{n2c, AddressArg, BearerKind, IntersectArg, MagicArg, PointArg},
    utils::{ChainWellKnownInfo, Utils, WithUtils},
};
use std::{str::FromStr, sync::Arc, thread::JoinHandle};
use anyhow::anyhow;
use tracing_subscriber::prelude::*;


#[tokio::main]
async fn main() -> anyhow::Result<()> {
   
    let fmt_layer = tracing_subscriber::fmt::layer().with_test_writer();
    let sqlx_filter = tracing_subscriber::filter::Targets::new()
        .with_target("sqlx", tracing::Level::WARN)
        .with_target("oura", tracing::Level::WARN)
        .with_target("carp", tracing::Level::TRACE)
        .with_default(tracing_subscriber::fmt::Subscriber::DEFAULT_MAX_LEVEL);

    tracing_subscriber::registry()
        .with(fmt_layer)
        .with(sqlx_filter)
        .init();

    let (handles, input) = oura_bootstrap("mainnet", "/mnt/nvme/carp/node-ipc/node.socket".to_string())?;
    start(input).await?;
    Ok(())
}


pub async fn start(input: StageReceiver) -> anyhow::Result<()> {
    loop {
        tracing::info!("Starting");
        let event_fetch_start = std::time::Instant::now();
        let event = input.recv()?;

        match &event.data {
            EventData::Block(block_record) => {
                tracing::info!("Reading block {:?} epoch {:?}", block_record, block_record.epoch);
            },
            _ => {
                tracing::info!("{:?}", event.data);
            }
        }
    }
}

pub fn oura_bootstrap(
    network: &str,
    socket: String,
) -> anyhow::Result<(Vec<JoinHandle<()>>, StageReceiver)> {
    let magic = MagicArg::from_str(network).map_err(|_| anyhow!("magic arg failed"))?;

    let well_known = ChainWellKnownInfo::try_from_magic(*magic)
        .map_err(|_| anyhow!("chain well known info failed"))?;

    let utils = Arc::new(Utils::new(well_known));

    let mapper = mapper::Config {
        include_transaction_details: true,
        include_block_cbor: true,
        ..Default::default()
    };

    #[allow(deprecated)]
    let source_config = n2c::Config {
        address: AddressArg(BearerKind::Unix, socket),
        // address: AddressArg(BearerKind::Tcp, socket),
        magic: Some(magic),
        well_known: None,
        mapper,
        since: None,
        min_depth: 0,
        intersect: None,
        retry_policy: None,
        finalize: None, // TODO: configurable
    };

    let source_setup = WithUtils::new(source_config, utils);

    let check = Predicate::VariantIn(vec![String::from("Block"), String::from("Rollback")]);

    let filter_setup = selection::Config { check };

    let mut handles = Vec::new();

    tracing::info!("{}", "Attempting to connect to node...");

    let (source_handle, source_rx) = source_setup.bootstrap().map_err(|e| {
        tracing::error!("{}", e);
        anyhow!("failed to bootstrap source. Are you sure cardano-node is running?")
    })?;

    tracing::info!("{}", "Connection to node established");

    handles.push(source_handle);

    let (filter_handle, filter_rx) = filter_setup
        .bootstrap(source_rx)
        .map_err(|_| anyhow!("failed to bootstrap filter"))?;

    handles.push(filter_handle);

    Ok((handles, filter_rx))
}