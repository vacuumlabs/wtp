use pallas::{
    ledger::primitives::{alonzo, babbage},
    ledger::traverse::MultiEraOutput,
};

#[derive(Debug)]
pub struct CoinAmount {
    pub name: String,
    pub amount: u64,
}

impl CoinAmount {
    pub fn new(name: String, amount: u64) -> Self {
        Self { name, amount }
    }
}

pub fn coins_amounts(output: &MultiEraOutput) -> Vec<CoinAmount> {
    match output {
        MultiEraOutput::Byron(byron) => {
            vec![CoinAmount::new(String::from("ada"), byron.amount)]
        }
        MultiEraOutput::AlonzoCompatible(x) => match &x.amount {
            alonzo::Value::Coin(c) => vec![CoinAmount::new(String::from("ada"), u64::from(c))],
            alonzo::Value::Multiasset(c, m) => {
                let mut out = vec![CoinAmount::new(String::from("ada"), u64::from(c))];
                for (_, tokens) in m.iter() {
                    for (name, amount) in tokens.iter() {
                        out.push(CoinAmount::new(
                            String::from_utf8_lossy(name).to_string(),
                            u64::from(amount),
                        ));
                    }
                }
                out
            }
        },
        MultiEraOutput::Babbage(x) => match x.as_ref().as_ref() {
            babbage::TransactionOutput::Legacy(x) => match &x.amount {
                babbage::Value::Coin(c) => vec![CoinAmount {
                    name: String::from("ada"),
                    amount: u64::from(c),
                }],

                babbage::Value::Multiasset(c, m) => {
                    let mut out = vec![CoinAmount {
                        name: String::from("ada"),
                        amount: u64::from(c),
                    }];
                    for (_, tokens) in m.iter() {
                        for (name, amount) in tokens.iter() {
                            out.push(CoinAmount::new(
                                String::from_utf8_lossy(name).to_string(),
                                u64::from(amount),
                            ));
                        }
                    }
                    out
                }
            },
            babbage::TransactionOutput::PostAlonzo(x) => match &x.value {
                babbage::Value::Coin(c) => vec![CoinAmount {
                    name: String::from("ada"),
                    amount: u64::from(c),
                }],

                babbage::Value::Multiasset(c, m) => {
                    let mut out = vec![CoinAmount {
                        name: String::from("ada"),
                        amount: u64::from(c),
                    }];
                    for (_, tokens) in m.iter() {
                        for (name, amount) in tokens.iter() {
                            out.push(CoinAmount::new(
                                String::from_utf8_lossy(name).to_string(),
                                u64::from(amount),
                            ));
                        }
                    }
                    out
                }
            },
        },
        _ => vec![CoinAmount {
            name: String::from("ada"),
            amount: 0,
        }],
    }
}
