#[derive(Debug)]
pub struct SimClock {
    pub now_ms: u64,
    pub slot_start_ms: u64,
    pub slot_index: u64,
}
