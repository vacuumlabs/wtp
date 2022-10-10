# Fast Cardano swap price feed

## Setting up

Run a Postgres instance and create an empty databas for this project:

```bash
psql -U postgres -c 'CREATE DATABASE cardano_price_feed;'
```

## Running

```bash
export DATABASE_URL='postgres://postgres:postgres@localhost:5432/cardano_price_feed'
cargo migrate up
cargo run -- -s 'relays-new.cardano-mainnet.iohk.io:3001' -d $DATABASE_URL
```

## Development

Add precommit hook:

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
