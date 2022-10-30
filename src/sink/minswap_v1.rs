use crate::{
    config::{MinSwapV1, PoolConfig},
    queries,
    sink::common,
    types::{Asset, AssetAmount, Swap},
    utils,
};
use async_trait::async_trait;
use oura::model::{TransactionRecord, TxOutputRecord};
use pallas::ledger::addresses::Address;
use sea_orm::DatabaseConnection;

static MS1_ADA_SWAP_IN: u64 = 4_000_000;
static MS1_ADA_SWAP_OUT: u64 = 2_000_000;

fn extract_plutus(datum: &serde_json::Value) -> (Asset, Asset) {
    (
        Asset {
            name: datum["fields"][0]["fields"][1]["bytes"]
                .as_str()
                .unwrap()
                .to_string(),
            policy_id: datum["fields"][0]["fields"][0]["bytes"]
                .as_str()
                .unwrap()
                .to_string(),
        },
        Asset {
            name: datum["fields"][1]["fields"][1]["bytes"]
                .as_str()
                .unwrap()
                .to_string(),
            policy_id: datum["fields"][1]["fields"][0]["bytes"]
                .as_str()
                .unwrap()
                .to_string(),
        },
    )
}

#[allow(dead_code)]
pub fn get_address_from_plutus(datum: &serde_json::Value) -> String {
    let first = datum["fields"][1]["fields"][0]["fields"][0]["bytes"]
        .as_str()
        .unwrap()
        .to_string();

    let second = datum["fields"][1]["fields"][1]["fields"][0]["fields"][0]["fields"][0]["bytes"]
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
impl common::Dex for MinSwapV1 {
    async fn mean_value(
        &self,
        pool: &PoolConfig,
        _db: &DatabaseConnection,
        transaction: &TransactionRecord,
    ) -> Option<(AssetAmount, AssetAmount)> {
        let script_hash = hex::decode(&pool.script_hash).unwrap();
        if let Some(output) = transaction
            .outputs
            .iter()
            .flatten()
            .find(|&o| utils::get_payment_hash(&o.address) == Some(script_hash.to_vec()))
        {
            //tracing::info!("output: {:?}, {:?}", output, output.datum_hash);
            if let Some(datum) = transaction
                .plutus_data
                .iter()
                .flatten()
                .find(|p| Some(p.datum_hash.clone()) == output.datum_hash)
            {
                let (asset1, asset2) = extract_plutus(&datum.plutus_data);

                tracing::info!(
                    "[{}] {}:{} vs {}:{}",
                    transaction.hash,
                    asset1.policy_id,
                    asset1.name,
                    asset2.policy_id,
                    asset2.name
                );
                let amount1 = common::get_amount(output, &asset1.policy_id, &asset1.name);
                let amount2 = common::get_amount(output, &asset2.policy_id, &asset2.name);
                tracing::info!("{} vs {}", amount1, amount2);
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
        let script_hash = hex::decode(&pool.script_hash).unwrap();
        let inputs = queries::get_utxo_input(transaction.inputs.as_ref().unwrap(), db).await;
        // https://cardanoscan.io/transaction/28956fc5b99977c520ce31eb49ad8fafd76fba9a9035ca5b2066a9d1741deb4d?tab=utxo
        let mut swaps: Vec<Swap> = Vec::new();

        let mut free_utxo: Vec<&TxOutputRecord> = transaction.outputs.iter().flatten().collect();

        if let Some(main_output) = transaction
            .outputs
            .iter()
            .flatten()
            .find(|o| utils::get_payment_hash(&o.address) == Some(script_hash.to_vec()))
        {
            // Extract asset information from plutus data of pool input
            let (main_asset1, main_asset2) = extract_plutus(
                &transaction
                    .plutus_data
                    .iter()
                    .flatten()
                    .find(|p| p.datum_hash == *main_output.datum_hash.as_ref().unwrap())
                    .unwrap()
                    .plutus_data,
            );

            // Get all input coresponding with correct address and plutus datum
            for input in inputs
                .iter()
                .flatten()
                .flatten()
                .filter(|i| i.address == pool.address && i.datum_hash.is_some())
            {
                if let Some(datum) = transaction
                    .plutus_data
                    .iter()
                    .flatten()
                    .find(|p| p.datum_hash == *input.datum_hash.as_ref().unwrap())
                {
                    let operation = datum.plutus_data["fields"][3]["constructor"]
                        .as_i64()
                        .unwrap();
                    // Identify SWAP operation
                    if operation == 0 {
                        // Extract amount from plutus - not sure if it is correct
                        // There is second way, how to get correct amount froum UTxO with coresponsing address
                        //let tmp_amount = datum.plutus_data["fields"][3]["fields"][1]["int"]
                        //.as_i64()
                        //.unwrap() as u64;
                        let policy_id = datum.plutus_data["fields"][3]["fields"][0]["fields"][0]
                            ["bytes"]
                            .as_str()
                            .unwrap()
                            .to_string();
                        let asset = datum.plutus_data["fields"][3]["fields"][0]["fields"][1]
                            ["bytes"]
                            .as_str()
                            .unwrap()
                            .to_string();

                        // Get transaction output - second way how to get output
                        let address = get_address_from_plutus(&datum.plutus_data);
                        // Get coresponding UTxO with result
                        let utxo_pos = free_utxo.iter().position(|o| o.address == address).unwrap();
                        let utxo = free_utxo[utxo_pos];
                        // Remove this UTxO as used
                        free_utxo.remove(utxo_pos);
                        // Get amount and direction
                        // Very ugly match case....
                        let (amount1, amount2, direction) = match main_asset2.name == asset
                            && main_asset2.policy_id == policy_id
                        {
                            true => (
                                common::get_amount(
                                    input,
                                    &main_asset1.policy_id,
                                    &main_asset1.name,
                                ) - common::reduce_amount(
                                    &main_asset1.policy_id,
                                    &main_asset1.name,
                                    MS1_ADA_SWAP_IN,
                                ),
                                common::get_amount(utxo, &main_asset2.policy_id, &main_asset2.name)
                                    - common::reduce_amount(
                                        &main_asset2.policy_id,
                                        &main_asset2.name,
                                        MS1_ADA_SWAP_OUT,
                                    ),
                                false,
                            ),
                            false => (
                                common::get_amount(utxo, &main_asset1.policy_id, &main_asset1.name)
                                    - common::reduce_amount(
                                        &main_asset1.policy_id,
                                        &main_asset1.name,
                                        MS1_ADA_SWAP_OUT,
                                    ),
                                common::get_amount(
                                    input,
                                    &main_asset2.policy_id,
                                    &main_asset2.name,
                                ) - common::reduce_amount(
                                    &main_asset2.policy_id,
                                    &main_asset2.name,
                                    MS1_ADA_SWAP_IN,
                                ),
                                true,
                            ),
                        };

                        // Add swap to the result
                        swaps.push(Swap {
                            first: AssetAmount {
                                asset: Asset {
                                    policy_id: main_asset1.policy_id.clone(),
                                    name: main_asset1.name.clone(),
                                },
                                amount: amount1 as u64,
                            },
                            second: AssetAmount {
                                asset: Asset {
                                    policy_id: main_asset2.policy_id.clone(),
                                    name: main_asset2.name.clone(),
                                },
                                amount: amount2 as u64,
                            },
                            direction,
                        })
                    }
                }
            }
        }
        Ok(swaps)
    }
}
