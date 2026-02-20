use std::collections::{HashMap, HashSet};

use eternix_sim::sim::clock::SimClock;
use eternix_sim::sim::simulator::Simulator;
use eternix_sim::state::chain_state::ChainState;
use eternix_sim::types::validator::{Validator, ValidatorState};
use eternix_sim::types::ticket::Ticket;
use eternix_sim::types::bucket::Bucket;

fn main() {
    // --- Genesis validator ---
    let validator1_id = 1u64;
    let validator2_id = 2u64;

    let mut validators = HashMap::new();
    validators.insert(
        validator1_id,
        Validator {
            id: validator1_id,
            state: ValidatorState::Active,
            vault_balance: 1_000_000,
            initial_bond: 1_000_000,
            miss_counter: 0,
            double_sign_offenses: 0,
            cooldown_until_epoch: None,
        },
    );
    validators.insert(
        validator2_id,
        Validator{
            id: validator2_id,
            state: ValidatorState::Active,
            vault_balance: 1_000_000,
            initial_bond: 1_000_000,
            miss_counter: 0,
            double_sign_offenses: 0,
            cooldown_until_epoch: None,
        },
    );

    // --- Bucket IDs ---
    let active_bucket_id = 0u64;
    let muted_bucket_id = 1u64;
    let dead_bucket_id = 2u64;

    // --- Buckets ---
    let mut buckets = HashMap::new();

    buckets.insert(active_bucket_id, Bucket {
        id: active_bucket_id,
        ticket_ids: HashSet::new(),
    });

    buckets.insert(muted_bucket_id, Bucket {
        id: muted_bucket_id,
        ticket_ids: HashSet::new(),
    });

    buckets.insert(dead_bucket_id, Bucket {
        id: dead_bucket_id,
        ticket_ids: HashSet::new(),
    });
    // --- Two tickets (start ACTIVE) ---
    let ticket1_id = 1u64;
    let ticket2_id = 2u64;

    let mut tickets = HashMap::new();
    tickets.insert(
        ticket1_id,
        Ticket {
            id: ticket1_id,
            owner: validator1_id,
            bucket: active_bucket_id,
            creation_epoch: 0,
        },
    );
    tickets.insert(
        ticket2_id,
        Ticket{
            id: ticket2_id,
            owner: validator2_id,
            bucket: active_bucket_id,
            creation_epoch: 0,
        },
    );

    // Put tickets into ACTIVE bucket
    buckets
    .get_mut(&active_bucket_id)
    .unwrap()
    .ticket_ids
    .insert(ticket1_id);

    buckets
    .get_mut(&active_bucket_id)
    .unwrap()
    .ticket_ids
    .insert(ticket2_id);

    // --- Bucket category sets ---
    let mut active_bucket_ids = HashSet::new();
    active_bucket_ids.insert(active_bucket_id);

    let mut muted_bucket_ids = HashSet::new();
    muted_bucket_ids.insert(muted_bucket_id);

    let state = ChainState {
        validators,
        tickets,
        buckets,

        active_bucket_ids,
        muted_bucket_ids,
        dead_bucket_id,

        epoch_index: 0,
        sub_epoch_index: 0,
        epoch_seed: [7u8; 32],
    };

    let clock = SimClock {
        now_ms: 0,
        slot_start_ms: 0,
        slot_index: 0,
    };

    let mut sim = Simulator {
        clock,
        state,
        blocks: Vec::new(),
    };

    // --- Run a few slots ---
    for _ in 0..50 {
        let block = sim.run_one_slot();

        let v1 = sim.state.validators.get(&1).unwrap();
        let v2 = sim.state.validators.get(&2).unwrap();

        println!(
            "Block {} | proposer: {:?} | v1: {:?} miss={} vault={} | v2: {:?} miss={} vault={}",
            block.slot_index,
            block.proposer,
            v1.state,
            v1.miss_counter,
            v1.vault_balance,
            v2.state,
            v2.miss_counter,
            v2.vault_balance,
        );
    }

}
