#[derive(Debug)]
pub struct Asset {
    pub policy_id: Vec<u8>,
    pub name: Vec<u8>,
    pub amount: u64,
}
