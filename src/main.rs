use clap::Parser;
use sea_orm::Database;
use std::fs;
use tracing_subscriber::prelude::*;

mod config;
mod entity;
mod queries;
mod server;
mod setup;
mod sink;
mod utils;

use hyper::service::{make_service_fn, service_fn};
use hyper::Server;
use std::convert::Infallible;
use std::net::SocketAddr;

#[derive(Parser, Debug)]
#[command(version)]
struct Args {
    /// Block to start from
    #[arg(long)]
    start: Option<String>,

    /// Cardano node socket
    #[arg(short, long)]
    socket: String,

    // Postgres connection string
    #[arg(short, long)]
    database: String,

    /// Config file
    #[arg(short, long, default_value_t = String::from("example.toml"))]
    config: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let config: config::Config = toml::from_str(&fs::read_to_string(&args.config)?)?;

    let fmt_layer = tracing_subscriber::fmt::layer();
    let filter = tracing_subscriber::filter::Targets::new()
        .with_target("oura", tracing::Level::WARN)
        .with_target("cardano_price_feed", tracing::Level::TRACE);

    tracing_subscriber::registry()
        .with(fmt_layer)
        .with(filter)
        .init();

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    let make_service =
        make_service_fn(|_conn| async { Ok::<_, Infallible>(service_fn(server::handle)) });
    let server = Server::bind(&addr).serve(make_service);
    tokio::spawn(async move { server.await });

    let db = Database::connect(args.database).await?;

    let (_handles, input) = setup::oura_bootstrap(args.start, args.socket)?;
    sink::start(input, db, &config.pools).await?;
    Ok(())
}
