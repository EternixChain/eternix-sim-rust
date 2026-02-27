use crate::state::chain_state::ChainState;
use crate::types::block::Block;
use crate::consensus::leader_selection::select_leader;
use crate::state::bucket_ops::{any_muted_bucket, move_all_validator_tickets_to_bucket};
use crate::types::validator::ValidatorState;
use crate::types::proposal::Proposal;
use crate::state::validator_ops::jail_validator;

pub fn process_slot(
    state: &mut ChainState,
    slot_index: u64,
    slot_start_ms: u64,
    proposals: &[Proposal],
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

    // Select leader (pure)
    let leader = select_leader(state, slot_index);

    // Collect all proposlas from the selected leader
    let leader_proposals: Vec<&Proposal> = proposals
        .iter()
        .filter(|p| p.proposer_id == leader)
        .collect();

    let mut unique_block_ids = std::collections::HashSet::new();
    for p in &leader_proposals {
        unique_block_ids.insert(p.block_id);
    }

    let leader_double_signed = unique_block_ids.len() >= 2;

    if leader_double_signed {
        apply_double_sign_punishment(state, leader);

        // Protocol-produced block
        return Block {
            slot_index,
            timestamp_ms: slot_start_ms + 3_000,
            proposer: None,
        };
    }

    let proposer: Option<u64>;

    if leader_proposals.len() == 1 {
        // Validator successfully proposed
        proposer = Some(leader);

        let val = state.validators.get_mut(&leader).unwrap();
        val.miss_counter = val.miss_counter.saturating_sub(1);
    } else {
        //Protocol-produced block (miss)
        proposer = None;

        let val = state.validators.get_mut(&leader).unwrap();
        let prev = val.miss_counter;
        val.miss_counter += 1;

        if should_liveness_slash(prev, val.miss_counter) {
            apply_liveness_slash(state, leader);
        }
    }

    // Publish block at slot end
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

fn apply_double_sign_punishment(state: &mut ChainState, validator_id: u64) {
    let val = state.validators.get_mut(&validator_id).unwrap();

    val.double_sign_offenses += 1;
    let offense = val.double_sign_offenses;

    match offense {
        1 => {
            // 50% slash
            val.vault_balance /= 2;
            println!(
                "!!! DOUBLE-SIGN: validator {} offense #1 => 50% slash, 2 epoch mute. New vault={} !!!",
                validator_id, val.vault_balance
            );

            val.state = ValidatorState::PunishedCooldown;
            val.cooldown_until_epoch = Some(state.epoch_index + 2 + 1);

            let muted = any_muted_bucket(state);
            move_all_validator_tickets_to_bucket(state, validator_id, muted);
        }
        2 => {
            // 75% slash
            val.vault_balance /= 4;
            println!(
                "!!! DOUBLE-SIGN: validator {} offense #2 => 75% slash, 5 epoch mute. New vault={} !!!",
                validator_id, val.vault_balance
            );

            val.state = ValidatorState::PunishedCooldown;
            val.cooldown_until_epoch = Some(state.epoch_index + 5 + 1);

            let muted = any_muted_bucket(state);
            move_all_validator_tickets_to_bucket(state, validator_id, muted);
        }
        _ => {
            // 100% slash + jail
            val.vault_balance = 0;
            jail_validator(state, validator_id);

            println!(
                "!!!!! DOUBLE-SIGN: validator {} offense #{} => 100% slash + JAILED. New vault=0 !!!!!",
                validator_id, offense
            );
        }
    }
}