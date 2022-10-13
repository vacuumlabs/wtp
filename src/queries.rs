use std::collections::{HashMap, HashSet};

use oura::model::{BlockRecord, TransactionRecord, TxOutputRecord};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, Condition, DatabaseConnection, DbBackend, EntityTrait,
    FromQueryResult, QueryFilter, Set, Statement,
};

use crate::{
    entity::{
        address, block, price_update, token, token_transfer, transaction, transaction_output,
    },
    types::{Asset, ExchangeRate},
    utils::ADA_TOKEN,
};

pub async fn insert_block(block: &BlockRecord, db: &DatabaseConnection) -> anyhow::Result<()> {
    let previous_hash = hex::decode(block.previous_hash.clone())?;
    let previous_block_model = block::Entity::find()
        .filter(block::Column::Hash.eq(previous_hash))
        .one(db)
        .await?;
    let block_model = block::ActiveModel {
        hash: Set(hex::decode(block.hash.clone())?),
        height: Set(block.number as i64),
        epoch: Set(block
            .epoch
            .ok_or_else(|| anyhow::anyhow!("No block epoch"))? as i64),
        slot: Set(block.slot as i64),
        previous_block_id: Set(previous_block_model.map(|b| b.id)),
        ..Default::default()
    };
    block_model.insert(db).await?;
    Ok(())
}

pub async fn rollback_to_slot(slot: &u64, db: &DatabaseConnection) -> anyhow::Result<()> {
    // We remove all blocks that are after the given slot. Removing based on the rollback event's
    // block_hash might not work because it's affected by the --start option and thus the
    // corresponding block might not even be present in the db.
    block::Entity::delete_many()
        .filter(block::Column::Slot.gt(*slot))
        .exec(db)
        .await?;
    Ok(())
}

pub async fn insert_transaction(
    transaction: &TransactionRecord,
    block_hash: &String,
    db: &DatabaseConnection,
) -> anyhow::Result<i64> {
    let block_hash = hex::decode(block_hash)?;
    let block_model = block::Entity::find()
        .filter(block::Column::Hash.eq(block_hash))
        .one(db)
        .await?
        .ok_or_else(|| anyhow::anyhow!("Transaction block not found"))?;

    let transaction_model = transaction::ActiveModel {
        hash: Set(hex::decode(transaction.hash.clone())?),
        block_id: Set(block_model.id),
        ..Default::default()
    };
    let transaction_model = transaction_model.insert(db).await?;

    let mut addresses = HashSet::new();
    let mut tokens = HashSet::from([ADA_TOKEN.clone()]);
    for output in transaction.outputs.iter().flatten() {
        addresses.insert(output.address.clone());

        for token in output.assets.iter().flatten() {
            let policy_id = hex::decode(token.policy.clone())?;
            let name = hex::decode(token.asset.clone())?;
            tokens.insert((policy_id, name));
        }
    }
    let address_models = insert_missing_addresses(addresses, db).await?;
    let token_models = insert_missing_tokens(tokens, db).await?;

    let address_models =
        HashMap::from_iter(address_models.into_iter().map(|a| (a.payload.clone(), a)));
    let token_models = HashMap::from_iter(
        token_models
            .into_iter()
            .map(|t| ((t.policy_id.clone(), t.name.clone()), t)),
    );

    for (index, output) in transaction.outputs.iter().flatten().enumerate() {
        insert_output(
            output,
            &transaction_model,
            index as i32,
            &address_models,
            &token_models,
            db,
        )
        .await?;
    }

    Ok(transaction_model.id)
}

async fn insert_missing_addresses(
    addresses: HashSet<String>,
    db: &DatabaseConnection,
) -> anyhow::Result<Vec<address::Model>> {
    let found_address_models = address::Entity::find()
        .filter(address::Column::Payload.is_in(addresses.clone()))
        .all(db)
        .await?;
    let missing_addresses: HashSet<String> = addresses
        .difference(
            &found_address_models
                .iter()
                .map(|a| a.payload.clone())
                .collect(),
        )
        .cloned()
        .collect();
    let missing_address_models: Vec<address::ActiveModel> = missing_addresses
        .iter()
        .map(|a| address::ActiveModel {
            payload: Set(a.clone()),
            ..Default::default()
        })
        .collect();
    if missing_address_models.is_empty() {
        return Ok(found_address_models);
    }
    let added_address_models = address::Entity::insert_many(missing_address_models)
        .exec_many_with_returning(db)
        .await?;
    Ok(found_address_models
        .into_iter()
        .chain(added_address_models.into_iter())
        .collect())
}

async fn insert_missing_tokens(
    tokens: HashSet<(Vec<u8>, Vec<u8>)>,
    db: &DatabaseConnection,
) -> anyhow::Result<Vec<token::Model>> {
    // TODO is there a better way how to match tuples in sea-orm? I tried filtering based on this,
    // but it seems that is_in() doesn't support tuples (even though SQL does):
    //     Expr::tuple([
    //         Expr::col(token::Column::PolicyId).into_simple_expr(),
    //         Expr::col(token::Column::Name).into_simple_expr(),
    //     ])
    //     .is_in(tokens.clone()) // -> error
    let mut condition = Condition::any();
    for (policy_id, name) in tokens.iter() {
        condition = condition.add(
            token::Column::PolicyId
                .eq(policy_id.clone())
                .and(token::Column::Name.eq(name.clone())),
        );
    }
    let found_token_models = token::Entity::find().filter(condition).all(db).await?;
    let missing_tokens: HashSet<(Vec<u8>, Vec<u8>)> = tokens
        .difference(
            &found_token_models
                .iter()
                .map(|t| (t.policy_id.clone(), t.name.clone()))
                .collect(),
        )
        .cloned()
        .collect();
    let missing_token_models: Vec<token::ActiveModel> = missing_tokens
        .iter()
        .map(|(p, n)| token::ActiveModel {
            policy_id: Set(p.clone()),
            name: Set(n.clone()),
            ..Default::default()
        })
        .collect();
    if missing_token_models.is_empty() {
        return Ok(found_token_models);
    }
    let added_token_models = token::Entity::insert_many(missing_token_models)
        .exec_many_with_returning(db)
        .await?;
    Ok(found_token_models
        .into_iter()
        .chain(added_token_models.into_iter())
        .collect())
}

async fn insert_output(
    output: &TxOutputRecord,
    transaction_model: &transaction::Model,
    index: i32,
    address_models: &HashMap<String, address::Model>,
    token_models: &HashMap<(Vec<u8>, Vec<u8>), token::Model>,
    db: &DatabaseConnection,
) -> anyhow::Result<()> {
    let address_model = address_models
        .get(&output.address)
        .ok_or_else(|| anyhow::anyhow!("Address not found"))?;
    let output_model = transaction_output::ActiveModel {
        tx_id: Set(transaction_model.id),
        index: Set(index),
        address_id: Set(address_model.id),
        spent: Set(false), // TODO we should set this to true if we observe the UTXO being spent
        ..Default::default()
    };
    let output_model = output_model.insert(db).await?;

    // ADA transfer
    let token_model = token_models
        .get(&ADA_TOKEN)
        .ok_or_else(|| anyhow::anyhow!("Token not found"))?;
    let token_transfer_model = token_transfer::ActiveModel {
        output_id: Set(output_model.id),
        token_id: Set(token_model.id),
        amount: Set(output.amount as i64),
        ..Default::default()
    };
    token_transfer_model.insert(db).await?;

    // other token transfers
    for token_transfer in output.assets.iter().flatten() {
        let policy_id = hex::decode(token_transfer.policy.clone())?;
        let name = hex::decode(token_transfer.asset.clone())?;
        let token_model = token_models
            .get(&(policy_id, name))
            .ok_or_else(|| anyhow::anyhow!("Token not found"))?;
        let token_transfer_model = token_transfer::ActiveModel {
            output_id: Set(output_model.id),
            token_id: Set(token_model.id),
            amount: Set(token_transfer.amount as i64),
            ..Default::default()
        };
        token_transfer_model.insert(db).await?;
    }

    Ok(())
}

pub async fn insert_price_update(
    tx_id: i64,
    script_hash: Vec<u8>,
    asset1: Asset,
    asset2: Asset,
    db: &DatabaseConnection,
) -> anyhow::Result<()> {
    let token1_model = token::Entity::find()
        .filter(
            token::Column::PolicyId
                .eq(hex::decode(asset1.policy_id)?)
                .and(token::Column::Name.eq(hex::decode(asset1.name)?)),
        )
        .one(db)
        .await?
        .ok_or_else(|| anyhow::anyhow!("Token1 not found"))?;
    let token2_model = token::Entity::find()
        .filter(
            token::Column::PolicyId
                .eq(hex::decode(asset2.policy_id)?)
                .and(token::Column::Name.eq(hex::decode(asset2.name)?)),
        )
        .one(db)
        .await?
        .ok_or_else(|| anyhow::anyhow!("Token2 not found"))?;

    let price_update_model = price_update::ActiveModel {
        tx_id: Set(tx_id),
        script_hash: Set(script_hash),
        token1_id: Set(token1_model.id),
        token2_id: Set(token2_model.id),
        amount1: Set(asset1.amount as i64),
        amount2: Set(asset2.amount as i64),
        ..Default::default()
    };
    price_update_model.insert(db).await?;
    Ok(())
}

pub async fn get_latest_prices(db: &DatabaseConnection) -> anyhow::Result<Vec<ExchangeRate>> {
    // The raw SQL query here is rather unlucky, but we need to join the token table twice,
    // and the sea-orm version usde by us (dcSpark's fork which implements
    // exec_many_with_returning) doesn't seem to support join aliases.
    // TODO figure out how to do both multi-joining here and exec_many_with_returning above.

    #[derive(Debug, FromQueryResult)]
    struct RawExchangeRate {
        script_hash: Vec<u8>,
        policy_id1: Vec<u8>,
        name1: Vec<u8>,
        policy_id2: Vec<u8>,
        name2: Vec<u8>,
        amount1: i64,
        amount2: i64,
    }

    let raw_exchange_rates: Vec<RawExchangeRate> =
        RawExchangeRate::find_by_statement(Statement::from_sql_and_values(
            DbBackend::Postgres,
            r#"
            SELECT
                script_hash,
                t1.policy_id AS policy_id1,
                t1.name AS name1,
                t2.policy_id AS policy_id2,
                t2.name AS name2,
                amount1,
                amount2
            FROM price_update
            JOIN token AS t1 ON t1.id = price_update.token1_id
            JOIN token AS t2 ON t2.id = price_update.token2_id
            WHERE (script_hash, token1_id, token2_id, timestamp) IN (
                SELECT script_hash, token1_id, token2_id, MAX(timestamp)
                FROM price_update
                GROUP BY script_hash, token1_id, token2_id
            )
            "#,
            vec![],
        ))
        .all(db)
        .await
        .unwrap();

    Ok(raw_exchange_rates
        .iter()
        .map(|r| ExchangeRate {
            script_hash: hex::encode(r.script_hash.clone()),
            asset1: Asset {
                policy_id: hex::encode(r.policy_id1.clone()),
                name: hex::encode(r.name1.clone()),
                amount: r.amount1 as u64,
            },
            asset2: Asset {
                policy_id: hex::encode(r.policy_id2.clone()),
                name: hex::encode(r.name2.clone()),
                amount: r.amount2 as u64,
            },
            rate: r.amount1 as f64 / r.amount2 as f64,
        })
        .collect())
}
