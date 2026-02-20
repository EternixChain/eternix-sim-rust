use crate::state::chain_state::ChainState;
use crate::types::block::Block;
use crate::consensus::leader_selection::select_leader;
use crate::state::bucket_ops::{any_muted_bucket, move_all_validator_tickets_to_bucket};
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

    // 5% slash
    let slash_amount = val.vault_balance / 20;
    val.vault_balance -= slash_amount;

    println!(
        "!!!  LIVENESS SLASH: validator {} slashed by 5%, new vault = {} !!!",
        validator_id, val.vault_balance
    );

    // Enter cooldown for 1 epoch
    val.state = ValidatorState::PunishedCooldown;
    val.cooldown_until_epoch = Some(state.epoch_index + 2);

    // Move tickets to MUTED immediately
    let muted_bucket = any_muted_bucket(state);
    move_all_validator_tickets_to_bucket(state, validator_id, muted_bucket);
}
