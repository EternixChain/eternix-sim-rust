use crate::state::chain_state::ChainState;
use crate::types::block::Block;
use crate::consensus::leader_selection::select_leader;
use crate::state::bucket_ops::move_ticket;
use crate::types::validator::ValidatorState;

pub fn process_slot(
    state: &mut ChainState,
    slot_index: u64,
    slot_start_ms: u64,
    buffered_proposal: Option<u64>,
) -> Block {
    
    // If no ACTIVE buckets exist, protocol produces block immediately
    let has_eligible_tickets = state
        .active_bucket_ids
        .iter()
        .any(|bucket_id| {
            state
                .buckets
                .get(bucket_id)
                .map(|b| !b.ticket_ids.is_empty())
                .unwrap_or(false)
        });

    if !has_eligible_tickets {
        return Block {
            slot_index,
            timestamp_ms: slot_start_ms + 3_000,
            proposer: None,
        };
    }

    // 1. Select leader (pure)
    let leader = select_leader(state, slot_index);

    let proposer: Option<u64>;

    match buffered_proposal {
        Some(v) if v == leader => {
            // Validator successfully proposed
            proposer = Some(leader);

            let val = state.validators.get_mut(&leader).unwrap();
            val.miss_counter = val.miss_counter.saturating_sub(1);
        }
        _ => {
            // Protocol-produced block
            proposer = None;

            let val = state.validators.get_mut(&leader).unwrap();
            let prev = val.miss_counter;
            val.miss_counter += 1;

            if should_liveness_slash(prev, val.miss_counter) {
                apply_liveness_slash(state, leader);
            }
        }
    }

    // 2. Publish block at slot end
    Block {
        slot_index,
        timestamp_ms: slot_start_ms + 3_000,
        proposer,
    }
}

fn should_liveness_slash(prev: u32, now: u32) -> bool {
    if now <= prev {
        return false;
    }

    if now == 5 {
        return true;
    }

    now > 5 && (now - 5) % 100 == 0
}

fn apply_liveness_slash(state: &mut ChainState, validator_id: u64) {
    let val = state.validators.get_mut(&validator_id).unwrap();

    let slash_amount = val.vault_balance / 20;
    val.vault_balance -= slash_amount;

    println!(
        "!!! LIVENESS SLASH: validator {} slashed by 5%, new vault = {} !!!",
        validator_id, val.vault_balance
    );

    // Enforce vault invariant
    let required = val.initial_bond; // placeholder rule for now
    if val.vault_balance < required && val.state == ValidatorState::Active {
        println!(
            "!!! Validator {} entering PAUSED_LOW_VAULT (vault below minimum) !!!",
            validator_id
        );

        val.state = ValidatorState::PausedLowVault;

        // Move all owned tickets to MUTED buckets
        let muted_bucket = *state
            .muted_bucket_ids
            .iter()
            .next()
            .expect("No MUTED bucket defined");

        let ticket_ids: Vec<u64> = state
            .tickets
            .values()
            .filter(|t| t.owner == validator_id)
            .map(|t| t.id)
            .collect();

        for tid in ticket_ids {
            let from = state.tickets.get(&tid).unwrap().bucket;
            move_ticket(state, tid, from, muted_bucket);
        }
    }
}
