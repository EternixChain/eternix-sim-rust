#[derive(Debug, Clone)]
pub struct Block {
    pub slot_index: u64,
    pub timestamp_ms: u64,
    pub proposer: Option<u64>, // None = protocol block
}

#[derive(Debug, Clone)]
pub struct SubEpochBlockRecord {
    pub proposer: Option<u64>,
}