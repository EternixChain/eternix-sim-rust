use std::collections::{HashMap, HashSet};

use crate::types::{validator::Validator, ticket::Ticket, bucket::Bucket};

#[derive(Debug)]
pub struct ChainState {
    pub validators: HashMap<u64, Validator>,
    pub tickets: HashMap<u64, Ticket>,

    pub buckets: HashMap<u64, Bucket>,

    pub active_bucket_ids: HashSet<u64>,
    pub muted_bucket_ids: HashSet<u64>,
    pub dead_bucket_id: u64,

    pub epoch_index: u64,
    pub sub_epoch_index: u64,
    pub epoch_seed: [u8; 32],
}
