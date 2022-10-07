use clap::Parser;
use std::fs;
use tracing_subscriber::prelude::*;

mod config;
mod setup;
mod sink;

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
    let config: config::Config = toml::from_str(&fs::read_to_string("example.toml")?)?;

    let fmt_layer = tracing_subscriber::fmt::layer();
    let filter = tracing_subscriber::filter::Targets::new()
        .with_target("oura", tracing::Level::WARN)
        .with_target("cardano_price_feed", tracing::Level::TRACE);

    tracing_subscriber::registry()
        .with(fmt_layer)
        .with(filter)
        .init();

    let (_handles, input) = setup::oura_bootstrap(args.start, args.socket)?;
    sink::start(input, &config.pools).await?;
    Ok(())
}
