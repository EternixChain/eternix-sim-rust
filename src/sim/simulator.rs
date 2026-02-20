use crate::state::chain_state::ChainState;
use crate::types::block::Block;
use crate::sim::clock::SimClock;
use crate::consensus::slot::process_slot;

pub struct Simulator {
    pub clock: SimClock,
    pub state: ChainState,
    pub blocks: Vec<Block>,
}

impl Simulator {
    pub fn run_one_slot(&mut self) -> Block {
        let _slot_index = self.clock.slot_index;
        let _slot_start_ms = self.clock.slot_start_ms;

        // V2 always tries proposing
        let buffered_proposal: Option<u64> = Some(2);

        let block = process_slot(
            &mut self.state,
            self.clock.slot_index,
            self.clock.slot_start_ms,
            buffered_proposal,
        );

        self.blocks.push(block.clone());

        // Advance time deterministically
        self.clock.slot_index += 1;
        self.clock.slot_start_ms += 3_000;
        self.clock.now_ms = self.clock.slot_start_ms;

        block
    }
}

