use crate::state::chain_state::ChainState;
use crate::state::bucket_ops::{any_muted_bucket, move_ticket};
use crate::types::ticket::TicketState;

pub fn request_ticket_retire(state: &mut ChainState, validator_id: u64, ticket_ids: Vec<u64>) {
    // Filter: must be owned by validator and currently Active
    let mut eligible: Vec<u64> = ticket_ids
        .into_iter()
        .filter(|tid| {
            state.tickets.get(tid).map(|t| t.owner == validator_id && t.state == TicketState::Active).unwrap_or(false)
        })
        .collect();

    // deterministic order
    eligible.sort_unstable();

    // schedule with per-epoch limit
    let mut epoch = state.epoch_index + 1;

    let mut idx = 0usize;
    while idx < eligible.len() {
        let entry = state.retire_schedule.entry(epoch).or_default();

        // Count how many already scheduled for this validator in this epoch
        let already_scheduled_for_validator = entry
            .iter()
            .filter(|tid| state.tickets.get(tid).unwrap().owner == validator_id)
            .count() as u64;

        let room = state.retire_per_epoch_limit.saturating_sub(already_scheduled_for_validator);
        if room == 0 {
            epoch += 1;
            continue;
        }

        let take = std::cmp::min(room as usize, eligible.len() - idx);
        for _ in 0..take {
            entry.push(eligible[idx]);
            idx += 1;
        }
    }
}

pub fn begin_retire_for_epoch(state: &mut ChainState, epoch: u64) {
    let Some(ticket_ids) = state.retire_schedule.remove(&epoch) else { return; };

    println!("Tickets scheduled this epoch: {:?}", ticket_ids);

    let muted_bucket = any_muted_bucket(state);

    for tid in ticket_ids {
        let t = state.tickets.get_mut(&tid).unwrap();

        // If ticket already dead (e.g., validator jailed), skip
        if t.state != TicketState::Active {
            continue;
        }

        // Immediate ineligibility on request start
        t.state = TicketState::Retiring;
        t.retire_requested_epoch = Some(epoch);

        // retirement delay: choose your constant (example: 2 epochs)
        let finalize_epoch = epoch + 2;
        t.retire_effective_epoch = Some(finalize_epoch);

        // move to MUTED so it can't be selected (cleaner than keeping in ACTIVE)
        let from = t.bucket;
        if from != muted_bucket {
            move_ticket(state, tid, from, muted_bucket);
        }

        // enqueue finalization
        state.retire_finalize.entry(finalize_epoch).or_default().push(tid);
    }
}

pub fn finalize_retire_for_epoch(state: &mut ChainState, epoch: u64) {
    let Some(ticket_ids) = state.retire_finalize.remove(&epoch) else { return; };

    println!("Finalizing tickets this epoch: {:?}", ticket_ids);

    let dead_bucket = state.dead_bucket_id;

    for tid in ticket_ids {
        let t = state.tickets.get_mut(&tid).unwrap();

        // If already dead, skip
        if t.state == TicketState::Dead {
            continue;
        }

        t.state = TicketState::Dead;

        // Move to DEAD bucket (unselectable forever)
        let from = t.bucket;
        if from != dead_bucket {
            move_ticket(state, tid, from, dead_bucket);
        }
    }
}