use crate::state::bucket_ops::{any_active_bucket, move_all_validator_tickets_to_bucket};
use crate::types::validator::ValidatorState;
use crate::state::chain_state::ChainState;

pub fn on_vault_refill(state: &mut ChainState, validator_id: u64, amount: u128) {
    let v = state.validators.get_mut(&validator_id).unwrap();
    v.vault_balance += amount;

    // Instant rejoin only from PausedLowVault
    if v.state == ValidatorState::PausedLowVault && v.vault_balance >= v.initial_bond {
        v.state = ValidatorState::Active;

        let active_bucket = any_active_bucket(state);
        move_all_validator_tickets_to_bucket(state, validator_id, active_bucket);
    }
}