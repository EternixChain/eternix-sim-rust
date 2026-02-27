use crate::state::chain_state::ChainState;
use crate::types::block::Block;
use crate::sim::clock::SimClock;
use crate::consensus::slot::process_slot;
use crate::consensus::epoch::process_epoch_transition;
use crate::types::proposal::Proposal;

pub struct Simulator {
    pub clock: SimClock,
    pub state: ChainState,
    pub blocks: Vec<Block>,
    pub epoch_len_slots: u64,
}

impl Simulator {
    pub fn run_one_slot(&mut self) -> Block {
        let slot_index = self.clock.slot_index;
        let _slot_start_ms = self.clock.slot_start_ms;

        let proposals = vec![
            Proposal { proposer_id: 1, block_id: slot_index },
            Proposal { proposer_id: 1, block_id: slot_index + 9999 },
            Proposal { proposer_id: 2, block_id: slot_index },
        ];

        let block = process_slot(
            &mut self.state,
            self.clock.slot_index,
            self.clock.slot_start_ms,
            &proposals,
        );

        self.blocks.push(block.clone());

        // Advance time deterministically
        self.clock.slot_index += 1;
        self.clock.slot_start_ms += 3_000;
        self.clock.now_ms = self.clock.slot_start_ms;

        if self.clock.slot_index % self.epoch_len_slots == 0 {
            process_epoch_transition(&mut self.state);
        }

        block
    }
}

