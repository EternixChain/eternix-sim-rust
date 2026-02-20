use crate::state::chain_state::ChainState;

/// Move a ticket between buckets.
/// This is the ONLY place tickets are allowed to change buckets.
pub fn move_ticket(
    state: &mut ChainState,
    ticket_id: u64,
    from_bucket: u64,
    to_bucket: u64,
) {
    {
        let from = state.buckets.get_mut(&from_bucket).unwrap();
        let removed = from.ticket_ids.remove(&ticket_id);
        assert!(removed, "Ticket {} not found in source bucket {}", ticket_id, from_bucket);
    }

    {
        let to = state.buckets.get_mut(&to_bucket).unwrap();
        let inserted = to.ticket_ids.insert(ticket_id);
        assert!(inserted, "Ticket {} already in target bucket {}", ticket_id, to_bucket);
    }

    state.tickets.get_mut(&ticket_id).unwrap().bucket = to_bucket;
}

pub fn any_muted_bucket(state: &ChainState) -> u64 {
    *state
        .muted_bucket_ids
        .iter()
        .next()
        .expect("No MUTED bucket defined")
}

pub fn any_active_bucket(state: &ChainState) -> u64 {
    *state
        .active_bucket_ids
        .iter()
        .next()
        .expect("No ACTIVE bucket defined")
}

pub fn move_all_validator_tickets_to_bucket(
    state: &mut ChainState,
    validator_id: u64,
    to_bucket: u64,
) {
    let ticket_ids: Vec<u64> = state
        .tickets
        .values()
        .filter(|t| t.owner == validator_id)
        .map(|t| t.id)
        .collect();

    for tid in ticket_ids {
        let from = state.tickets.get(&tid).unwrap().bucket;
        if from != to_bucket {
            move_ticket(state, tid, from, to_bucket);
        }
    }
}