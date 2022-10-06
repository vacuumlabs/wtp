use anyhow::anyhow;
use clap::Parser;
use oura::{
    filters::selection::{self, Predicate},
    mapper,
    model::EventData,
    pipelining::{FilterProvider, SourceProvider, StageReceiver},
    sources::{n2n, AddressArg, BearerKind, IntersectArg, MagicArg, PointArg},
    utils::{ChainWellKnownInfo, Utils, WithUtils},
};

use std::{str::FromStr, sync::Arc, thread::JoinHandle};
use tracing_subscriber::prelude::*;

pub mod utils;

#[derive(Parser, Debug)]
#[command(version)]
struct Args {
    // Block to start from
    #[arg(long)]
    start: Option<String>,

    // Cardano node socket
    #[arg(short, long)]
    socket: String,

    // WingRiders pool adress
    #[arg(short, long, default_value_t = String::from("11e6c90a5923713af5786963dee0fdffd830ca7e0c86a041d9e5833e916cc2342da98d86b6229a37893bf06e69555c7d6de59d5e08ad0034b7"))]
    address: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    let fmt_layer = tracing_subscriber::fmt::layer();
    let filter = tracing_subscriber::filter::Targets::new()
        .with_target("oura", tracing::Level::WARN)
        .with_target("cardano_price_feed", tracing::Level::TRACE);

    tracing_subscriber::registry()
        .with(fmt_layer)
        .with(filter)
        .init();

    let (_handles, input) = oura_bootstrap(args.start, args.socket)?;
    start(input).await?;
    Ok(())
}

pub async fn start(input: StageReceiver) -> anyhow::Result<()> {
    loop {
        tracing::info!("Starting");
        let event = input.recv()?;

        match &event.data {
            EventData::Block(block_record) => {
                tracing::info!(
                    "Reading block {:?} epoch {:?}",
                    block_record,
                    block_record.epoch
                );

                let block_payload = hex::decode(block_record.cbor_hex.as_ref().unwrap()).unwrap();
                let multi_block =
                    pallas::ledger::traverse::MultiEraBlock::decode(&block_payload).unwrap();
                for transaction in multi_block.txs() {
                    for output in transaction.outputs() {
                        let coin_amount = utils::coins_amounts(&output);
                        tracing::info!("[{}] {:?}", output.address().unwrap().to_hex(), coin_amount)
                    }
                }
            }
            EventData::Transaction(transaction_record) => {
                tracing::info!("Reading transaction");
                for output in transaction_record.outputs.as_ref().unwrap().iter() {
                    tracing::info!(
                        "Address: {} {}, {:?}",
                        output.address,
                        output.amount,
                        output.assets
                    );
                }
            }
            _ => {
                tracing::info!("other!!! {:?}", event.data);
            }
        }
    }
}

pub fn oura_bootstrap(
    start_block: Option<String>,
    socket: String,
) -> anyhow::Result<(Vec<JoinHandle<()>>, StageReceiver)> {
    let magic = MagicArg::from_str("mainnet").unwrap();

    let well_known = ChainWellKnownInfo::try_from_magic(*magic)
        .map_err(|_| anyhow!("chain well known info failed"))?;

    let utils = Arc::new(Utils::new(well_known));

    let mapper = mapper::Config {
        include_transaction_details: true,
        include_block_cbor: true,
        ..Default::default()
    };

    let intersect = match start_block {
        Some(s) => {
            let (slot, hash) = match s.split_once(':') {
                Some((s, h)) => (s.parse::<u64>()?, h),
                None => return Err(anyhow!("invalid start")),
            };
            tracing::info!("{} {}", slot, hash);
            Some(IntersectArg::Point(PointArg(slot, hash.to_string())))
        }
        None => None,
    };

    #[allow(deprecated)]
    let source_config = n2n::Config {
        address: if socket.contains(':') {
            AddressArg(BearerKind::Tcp, socket)
        } else {
            AddressArg(BearerKind::Unix, socket)
        },
        magic: Some(magic),
        well_known: None,
        mapper,
        since: None,
        min_depth: 0,
        intersect,
        retry_policy: None,
        finalize: None, // TODO: configurable
    };

    let source_setup = WithUtils::new(source_config, utils);

    let check = Predicate::VariantIn(vec![String::from("Block"), String::from("Transaction")]);

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
