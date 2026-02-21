use crate::state::chain_state::ChainState;
use crate::state::bucket_ops::{any_active_bucket, any_muted_bucket, move_all_validator_tickets_to_bucket};
use crate::types::ticket::TicketState;
use crate::types::validator::ValidatorState;
use crate::state::retirement_ops::{begin_retire_for_epoch, finalize_retire_for_epoch};

pub fn process_epoch_transition(state: &mut ChainState) {
    state.epoch_index += 1;
    println!("=== EPOCH TRANSITION → {} ===", state.epoch_index);
    
    println!("Processing retire begin for epoch {}", state.epoch_index);
    begin_retire_for_epoch(state, state.epoch_index);
    finalize_retire_for_epoch(state, state.epoch_index);

    for (validator_id, val) in state.validators.iter_mut() {
        let active_ticket_count = state.tickets.values()
            .filter(|t| t.owner == *validator_id && t.state == TicketState::Active)
            .count();

        if active_ticket_count == 0 && val.state == ValidatorState::Active {
            val.state = ValidatorState::Inactive;
        }
    }

    let required_min = |vault_balance: u128, initial_bond: u128| -> bool {
        // placeholder: minimum is the initial bond
        vault_balance >= initial_bond
    };

    let active_bucket = any_active_bucket(state);
    let muted_bucket = any_muted_bucket(state);

    let validator_ids: Vec<u64> = state.validators.keys().copied().collect();

    for vid in validator_ids {
        let (st, until, _vault, _bond) = {
            let v = state.validators.get(&vid).unwrap();
            (v.state, v.cooldown_until_epoch, v.vault_balance, v.initial_bond)
        };

        if st == ValidatorState::Jailed {
            continue;
        }
        
        if st == ValidatorState::PunishedCooldown {
            if let Some(until_epoch) = until {
                if state.epoch_index >= until_epoch {
                    let v = state.validators.get_mut(&vid).unwrap();
                    v.cooldown_until_epoch = None;

                    if required_min(v.vault_balance, v.initial_bond) {
                        v.state = ValidatorState::Active;
                        move_all_validator_tickets_to_bucket(state, vid, active_bucket);
                    } else {
                        v.state = ValidatorState::PausedLowVault;
                        move_all_validator_tickets_to_bucket(state, vid, muted_bucket);
                    }
                }
            }
        }
    }
}