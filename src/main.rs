use std::collections::{HashMap, HashSet, BTreeMap};

use eternix_sim::sim::clock::SimClock;
use eternix_sim::sim::simulator::Simulator;
use eternix_sim::state::chain_state::ChainState;
use eternix_sim::types::validator::{Validator, ValidatorState};
use eternix_sim::types::ticket::Ticket;
use eternix_sim::types::bucket::Bucket;
use eternix_sim::types::ticket::TicketState;
// use eternix_sim::state::retirement_ops::request_ticket_retire;
// use eternix_sim::state::validator_ops::{on_vault_refill};

const INITIAL_SUPPLY: u128 = 100_000_000;
const K_NUMERATOR: u128 = 1;
const K_DENOMINATOR: u128 = 2;

fn main() {
    // --- Genesis validator set ---
    let validator1_id = 1u64;
    let validator2_id = 2u64;
    let validator3_id = 3u64;

    let mut validators = HashMap::new();
    validators.insert(
        validator1_id,
        Validator {
            id: validator1_id,
            state: ValidatorState::Active,
            vault_balance: 10_000_000,
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
    validators.insert(
        validator3_id,
        Validator{
            id: validator3_id,
            state: ValidatorState::Active,
            vault_balance: 1_000_000,
            initial_bond: 1_000_000,
            miss_counter: 0,
            double_sign_offenses: 0,
            cooldown_until_epoch: None,
        },
    );

    // --- Bucket IDs ---
    let active_bucket_id = 2u64;
    let muted_bucket_id = 1u64;
    let dead_bucket_id = 0u64;

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
    // --- Tickets ---
    let ticket1_id = 1u64;
    let ticket2_id = 2u64;
    let ticket3_id = 3u64;
    let ticket4_id = 4u64;

    let mut tickets = HashMap::new();
    tickets.insert(
        ticket1_id,
        Ticket {
            id: ticket1_id,
            owner: validator1_id,
            bucket: active_bucket_id,
            creation_epoch: 0,
            state: TicketState::Active,
            retire_requested_epoch: None,
            retire_effective_epoch: None,
        },
    );
    tickets.insert(
        ticket2_id,
        Ticket{
            id: ticket2_id,
            owner: validator2_id,
            bucket: active_bucket_id,
            creation_epoch: 0,
            state: TicketState::Active,
            retire_requested_epoch: None,
            retire_effective_epoch: None,
        },
    );
        tickets.insert(
        ticket3_id,
        Ticket{
            id: ticket3_id,
            owner: validator3_id,
            bucket: active_bucket_id,
            creation_epoch: 0,
            state: TicketState::Active,
            retire_requested_epoch: None,
            retire_effective_epoch: None,
        },
    );
        tickets.insert(
        ticket4_id,
        Ticket{
            id: ticket4_id,
            owner: validator3_id,
            bucket: active_bucket_id,
            creation_epoch: 0,
            state: TicketState::Active,
            retire_requested_epoch: None,
            retire_effective_epoch: None,
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

    buckets
    .get_mut(&active_bucket_id)
    .unwrap()
    .ticket_ids
    .insert(ticket3_id);

    buckets
    .get_mut(&active_bucket_id)
    .unwrap()
    .ticket_ids
    .insert(ticket4_id);

    // --- Bucket category sets ---
    let mut active_bucket_ids = HashSet::new();
    active_bucket_ids.insert(active_bucket_id);

    let mut muted_bucket_ids = HashSet::new();
    muted_bucket_ids.insert(muted_bucket_id);

    let state = ChainState {
        total_supply: INITIAL_SUPPLY,
        block_reward: 1000, //TEMPORARY

        burn_this_sub_epoch: 0,
        blocks_this_sub_epoch: Vec::new(),
        k_numerator: K_NUMERATOR,
        k_denominator: K_DENOMINATOR,

        validators,
        tickets,

        buckets,

        active_bucket_ids,
        muted_bucket_ids,
        dead_bucket_id,

        epoch_index: 0,
        sub_epoch_index: 0,
        epoch_seed: [7u8; 32],

        retire_per_epoch_limit: 2,
        retire_schedule: BTreeMap::new(),
        retire_finalize: BTreeMap::new(),
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
        epoch_len_slots: 100,
        sub_epoch_len_slots: 10,
    };

    // request_ticket_retire(&mut sim.state, 1, vec![1]);
    // println!("{:?}", sim.state.retire_schedule);

    // --- Run a few slots ---
    for _ in 0..250 {
//        let current_slot = sim.clock.slot_index;

        // Inject at specific block
//        if current_slot == 25 {
//            on_vault_refill(&mut sim.state, 1, 50_000u128);
//            println!(50,000 refilled to Validator 1)
//        }

        

        let block = sim.run_one_slot();

        let v1 = sim.state.validators.get(&1).unwrap();
        let v2 = sim.state.validators.get(&2).unwrap();
        let v3 = sim.state.validators.get(&3).unwrap();

        println!(
            "Block {} | proposer: {:?} | v1: {:?} miss={} vault={} | v2: {:?} miss={} vault={} | v3: {:?} miss={} vault={} | total_supply={}",
            block.slot_index,
            block.proposer,
            v1.state,
            v1.miss_counter,
            v1.vault_balance,
            v2.state,
            v2.miss_counter,
            v2.vault_balance,
            v3.state,
            v3.miss_counter,
            v3.vault_balance,
            sim.state.total_supply,
        );
    }

}
