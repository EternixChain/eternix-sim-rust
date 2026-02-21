#[derive(Debug, Clone)]
pub struct Ticket {
    pub id: u64,
    pub owner: u64,
    pub bucket: u64,
    pub creation_epoch: u64,

    // retirement lifecycle
    pub state: TicketState,
    pub retire_requested_epoch: Option<u64>,
    pub retire_effective_epoch: Option<u64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TicketState {
    Active,
    Retiring, // requested, not yet DEAD
    Dead,     // retired or jailed owner
}