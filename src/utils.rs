use pallas::ledger::addresses::Address;

// We represent ADA as a token with empty policy_id and name.
pub static ADA_TOKEN: (Vec<u8>, Vec<u8>) = (Vec::new(), Vec::new());

pub fn get_payment_hash(address: &str) -> Option<Vec<u8>> {
    let parsed_address = Address::from_bech32(address).ok();
    if let Some(Address::Shelley(address)) = parsed_address {
        Some(address.payment().as_hash().to_vec())
    } else {
        None
    }
}
