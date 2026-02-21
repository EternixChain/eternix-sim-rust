#[derive(Debug, Clone)]
pub struct Proposal {
    pub proposer_id: u64,
    pub block_id: u64, // just an arbitrary “payload id” for simulation
}