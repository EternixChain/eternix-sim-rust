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
