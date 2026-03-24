use crate::state::chain_state::ChainState;

pub fn process_sub_epoch_transition(state: &mut ChainState) {

    let offset_pool =
        state.burn_this_sub_epoch * state.k_numerator / state.k_denominator;
    
    let mut validator_blocks = 0u64;

    for block in &state.blocks_this_sub_epoch {
        if block.proposer.is_some() {
            validator_blocks += 1;
        }
    }

    println!(
        "SUB-EPOCH END | blocks: {} | burn:{}",
        validator_blocks,
        state.burn_this_sub_epoch
    );

    println!("--- SUB EPOCH BLOCKS ---");
    for (i, b) in state.blocks_this_sub_epoch.iter().enumerate() {
        println!("{}: {:?}", i, b.proposer);
    }

    if validator_blocks > 0 {
        let bonus_per_block = offset_pool / validator_blocks as u128;

        for block in &state.blocks_this_sub_epoch {
            if let Some(v_id) = block.proposer {
                let v = state.validators.get_mut(&v_id).unwrap();
                v.vault_balance += bonus_per_block;
                state.total_supply += bonus_per_block;
            }
        }
    }

    state.burn_this_sub_epoch = 0;
    state.blocks_this_sub_epoch.clear();
}