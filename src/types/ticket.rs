#[derive(Debug, Clone)]
pub struct Ticket {
    pub id: u64,
    pub owner: u64,
    pub bucket: u64,
    pub creation_epoch: u64,
}
