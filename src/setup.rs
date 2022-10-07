use anyhow::anyhow;
use oura::{
    filters::selection::{self, Predicate},
    mapper,
    pipelining::{FilterProvider, SourceProvider, StageReceiver},
    sources::{n2n, AddressArg, BearerKind, IntersectArg, MagicArg, PointArg},
    utils::{ChainWellKnownInfo, Utils, WithUtils},
};
use std::{str::FromStr, sync::Arc, thread::JoinHandle};

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
            println!("{} {}", slot, hash);
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

    let check = Predicate::VariantIn(vec![String::from("Transaction")]);

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
