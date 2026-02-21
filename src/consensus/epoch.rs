use crate::state::chain_state::ChainState;
use crate::state::bucket_ops::{any_active_bucket, any_muted_bucket, move_all_validator_tickets_to_bucket};
use crate::types::validator::ValidatorState;

pub fn process_epoch_transition(state: &mut ChainState) {
    state.epoch_index += 1;

    let required_min = |vault_balance: u128, initial_bond: u128| -> bool {
        // placeholder: minimum is the initial bond (your current rule)
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