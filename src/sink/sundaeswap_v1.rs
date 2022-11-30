use crate::{
    config::{PoolConfig, SundaeSwapV1},
    queries,
    sink::common,
    types::{Asset, AssetAmount, Swap},
    utils,
};
use async_trait::async_trait;
use oura::model::{TransactionRecord, TxOutputRecord};
use pallas::ledger::addresses::Address;
use sea_orm::DatabaseConnection;

static SS1_ADA_SWAP_IN: u64 = 4_500_000;
static SS1_ADA_SWAP_OUT: u64 = 2_000_000;

fn extract_plutus(datum: &serde_json::Value) -> (Asset, Asset) {
    (
        Asset {
            name: datum["fields"][0]["fields"][0]["fields"][1]["bytes"]
                .as_str()
                .unwrap()
                .to_string(),
            policy_id: datum["fields"][0]["fields"][0]["fields"][0]["bytes"]
                .as_str()
                .unwrap()
                .to_string(),
        },
        Asset {
            name: datum["fields"][0]["fields"][1]["fields"][1]["bytes"]
                .as_str()
                .unwrap()
                .to_string(),
            policy_id: datum["fields"][0]["fields"][1]["fields"][0]["bytes"]
                .as_str()
                .unwrap()
                .to_string(),
        },
    )
}

#[allow(dead_code)]
pub fn get_address_from_plutus(datum: &serde_json::Value) -> String {
    let first = datum["fields"][1]["fields"][0]["fields"][0]["fields"][0]["fields"][0]["bytes"]
        .as_str()
        .unwrap()
        .to_string();

    let second = datum["fields"][1]["fields"][0]["fields"][0]["fields"][1]["fields"][0]["fields"]
        [0]["fields"][0]["bytes"]
        .as_str()
        .unwrap()
        .to_string();

    let string_list = vec![String::from("01"), first, second];
    Address::from_hex(&string_list.join(""))
        .unwrap()
        .to_bech32()
        .unwrap()
}

#[async_trait]
impl common::Dex for SundaeSwapV1 {
    async fn mean_value(
        &self,
        pool: &PoolConfig,
        _db: &DatabaseConnection,
        transaction: &TransactionRecord,
    ) -> Option<(AssetAmount, AssetAmount)> {
        if let Some(output) = transaction
            .outputs
            .iter()
            .flatten()
            .find(|&o| o.address == pool.address)
        {
            if let Some(datum) = transaction
                .plutus_data
                .iter()
                .flatten()
                .find(|p| Some(p.datum_hash.clone()) == output.datum_hash)
            {
                let (asset1, asset2) = extract_plutus(&datum.plutus_data);
                let amount1 = common::get_amount(output, &asset1.policy_id, &asset1.name);
                let amount2 = common::get_amount(output, &asset2.policy_id, &asset2.name);
                return Some((
                    AssetAmount {
                        asset: asset1,
                        amount: amount1,
                    },
                    AssetAmount {
                        asset: asset2,
                        amount: amount2,
                    },
                ));
            }
        }
        None
    }

    async fn swaps(
        &self,
        pool: &PoolConfig,
        db: &DatabaseConnection,
        transaction: &TransactionRecord,
    ) -> anyhow::Result<Vec<Swap>> {
        let mut swaps: Vec<Swap> = Vec::new();
        if let Some(output) = transaction
            .outputs
            .iter()
            .flatten()
            .find(|&o| o.address == pool.address)
        {
            if let Some(datum) = transaction
                .plutus_data
                .iter()
                .flatten()
                .find(|p| Some(p.datum_hash.clone()) == output.datum_hash)
            {
                let (asset1, asset2) = extract_plutus(&datum.plutus_data);
                let order_hash = hex::decode(&pool.request_hash).unwrap();
                let inputs =
                    queries::get_utxo_input(transaction.inputs.as_ref().unwrap(), db).await;
                let mut free_utxo: Vec<&TxOutputRecord> =
                    transaction.outputs.iter().flatten().collect();

                for input in
                    inputs.iter().flatten().flatten().filter(|i| {
                        utils::get_payment_hash(&i.address) == Some(order_hash.to_vec())
                    })
                {
                    let plutus = transaction
                        .plutus_data
                        .iter()
                        .flatten()
                        .find(|p| Some(p.datum_hash.clone()) == input.datum_hash)
                        .unwrap();

                    if plutus.plutus_data["fields"][3]["constructor"]
                        .as_i64()
                        .unwrap()
                        == 0
                    {
                        let address = get_address_from_plutus(&plutus.plutus_data);
                        let utxo_pos = free_utxo.iter().position(|o| o.address == address).unwrap();
                        let utxo = free_utxo[utxo_pos];
                        // Remove this UTxO as used
                        free_utxo.remove(utxo_pos);

                        let (amount1, amount2, direction) = match plutus.plutus_data["fields"][3]
                            ["fields"][0]["constructor"]
                            .as_i64()
                            .unwrap()
                            == 0
                        {
                            true => (
                                common::get_amount(input, &asset1.policy_id, &asset1.name)
                                    - common::reduce_ada_amount(
                                        &asset1.policy_id,
                                        &asset1.name,
                                        SS1_ADA_SWAP_IN,
                                    ),
                                common::get_amount(utxo, &asset2.policy_id, &asset2.name)
                                    - common::reduce_ada_amount(
                                        &asset2.policy_id,
                                        &asset2.name,
                                        SS1_ADA_SWAP_OUT,
                                    ),
                                false,
                            ),
                            false => (
                                common::get_amount(utxo, &asset1.policy_id, &asset1.name)
                                    - common::reduce_ada_amount(
                                        &asset1.policy_id,
                                        &asset1.name,
                                        SS1_ADA_SWAP_OUT,
                                    ),
                                common::get_amount(input, &asset2.policy_id, &asset2.name)
                                    - common::reduce_ada_amount(
                                        &asset2.policy_id,
                                        &asset2.name,
                                        SS1_ADA_SWAP_IN,
                                    ),
                                true,
                            ),
                        };

                        // Add swap to the result
                        swaps.push(Swap {
                            first: AssetAmount {
                                asset: Asset {
                                    policy_id: asset1.policy_id.clone(),
                                    name: asset1.name.clone(),
                                },
                                amount: amount1 as u64,
                            },
                            second: AssetAmount {
                                asset: Asset {
                                    policy_id: asset2.policy_id.clone(),
                                    name: asset2.name.clone(),
                                },
                                amount: amount2 as u64,
                            },
                            direction,
                        });
                    }
                }
            }
        }
        Ok(swaps)
    }
}
