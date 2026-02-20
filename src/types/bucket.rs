use std::collections::HashSet;

#[derive(Debug, Clone)]
pub struct Bucket {
    pub id: u64,
    pub ticket_ids: HashSet<u64>,
}
