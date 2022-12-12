# What the price
Tool for Cardano to get Dex SWAP operation, store it to the Database and broadcast it through the WebSocket.

## DEXes
We have implemented 3 Dexes
* WingRiders
* MinSwap
* SundaeSwap

Dex can have more versions and more addresses per version.

## Interface
* `/health` - Health check endpoint
* `/assets` - List of assets present in the database. This is a place, where pair asset_id with name and policy
* `/exchange_rates` - Calculate exchange rate. There is no information about decimal numbers
* `/mean_history/TOKEN1_ID/TOKEN2_ID?count=<number>` - Return mean swap price for tokens. Mean is not AVG, but ration on the pool address
* `/asset_swap/TOKEN1_ID/TOKEN2_ID?count=<number>` - Return last swap price for tokens.
* `/socket/` - WebSocket endpoint for Live information about the swap.


## Setting up

Run a Postgres instance and create an empty database for this project:

```bash
export DATABASE_URL='postgres://postgres:postgres@localhost:5432/wtp'
psql -U postgres -c 'CREATE DATABASE wtp;'
cargo migrate up
cargo run -- -s 'relays-new.cardano-mainnet.iohk.io:3001' -d $DATABASE_URL
```

## Development

Add pre-commit hook:

```bash
ln -s ../../pre-commit.sh .git/hooks/pre-commit
```

### How to modify the database schema

```bash
cargo install sea-orm-cli
sea-orm-cli migrate generate "your_migration_name"
# now edit the new migration file in ./migrations/src
cargo migrate up
sea-orm-cli generate entity -o src/entity
```

## Run

```bash
# Ideal run parametres for WR
cargo run -- --socket localhost:3001 --database $DATABASE_URL --persistent  --start 57270168:17a26b5607a6f61fe89bf73a7a242ff4fa6dd6c667f3b2d6fc56bbcad644e90b
```
