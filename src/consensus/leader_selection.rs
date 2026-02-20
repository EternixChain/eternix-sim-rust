use std::collections::HashMap;
use crate::state::chain_state::ChainState;
use sha2::{Digest, Sha256};

fn hash_bytes(data: &[u8]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hasher.finalize().into()
}


pub fn slot_seed(epoch_seed: [u8; 32], slot_index: u64) -> [u8; 32] {
    let mut data = Vec::new();
    data.extend_from_slice(&epoch_seed);
    data.extend_from_slice(&slot_index.to_be_bytes());
    hash_bytes(&data)
}

pub fn select_bucket(
    slot_seed: [u8; 32],
    buckets: &HashMap<u64, usize>, // bucket_id -> ticket_count
) -> u64 {
    let mut best_bucket: Option<u64> = None;
    let mut best_score: Option<u128> = None;

    for (&bucket_id, &ticket_count) in buckets {
        if ticket_count == 0 {
            continue;
        }

        // hash(slot_seed || bucket_id)
        let mut data = Vec::new();
        data.extend_from_slice(&slot_seed);
        data.extend_from_slice(&bucket_id.to_be_bytes());

        let hash = hash_bytes(&data);

        // Convert first 16 bytes to u128 (big-endian)
        let raw = u128::from_be_bytes(hash[0..16].try_into().unwrap());

        let score = raw / ticket_count as u128;

        match best_score {
            None => {
                best_bucket = Some(bucket_id);
                best_score = Some(score);
            }
            Some(current_best) => {
                if (score, bucket_id) < (current_best, best_bucket.unwrap()) {
                    best_bucket = Some(bucket_id);
                    best_score = Some(score);
                }
            }
        }
    }

    best_bucket.expect("No active buckets available")
}

pub fn select_ticket(
    slot_seed: [u8; 32],
    ticket_ids: &[u64],
) -> u64 {
    let mut best_ticket: Option<u64> = None;
    let mut best_score: Option<[u8; 32]> = None;

    for &ticket_id in ticket_ids {
        let mut data = Vec::new();
        data.extend_from_slice(&slot_seed);
        data.extend_from_slice(&ticket_id.to_be_bytes());

        let hash = hash_bytes(&data);

        match &best_score {
            None => {
                best_ticket = Some(ticket_id);
                best_score = Some(hash);
            }
            Some(current_best) => {
                if (hash, ticket_id) < (*current_best, best_ticket.unwrap()) {
                    best_ticket = Some(ticket_id);
                    best_score = Some(hash);
                }
            }
        }
    }

    best_ticket.expect("Bucket contained no tickets")
}

pub fn select_leader(
    state: &ChainState,
    slot_index: u64,
) -> u64 {
    // panic!("select_leader CALLLED");
    let seed = slot_seed(state.epoch_seed, slot_index);

    // Build bucket_id -> ticket_count map
    let mut bucket_counts: HashMap<u64, usize> = HashMap::new();

    for &bucket_id in &state.active_bucket_ids {
        if let Some(bucket) = state.buckets.get(&bucket_id) {
            let count = bucket.ticket_ids.len();
            if count > 0 {
                bucket_counts.insert(bucket_id, count);
            }
        }
    }

    let bucket_id = select_bucket(seed, &bucket_counts);

    let bucket = state
        .buckets
        .get(&bucket_id)
        .expect("Selected bucket missing");

    let ticket_id = select_ticket(seed, &bucket.ticket_ids.iter().copied().collect::<Vec<_>>());

    let ticket = state
        .tickets
        .get(&ticket_id)
        .expect("Selected ticket missing");

    ticket.owner
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::{HashMap, HashSet};

    #[test]
    fn slot_seed_is_deterministic() {
        let seed = [42u8; 32];
        let a = slot_seed(seed, 10);
        let b = slot_seed(seed, 10);
        assert_eq!(a, b);
    }

    #[test]
    fn bucket_selection_is_deterministic() {
        let seed = [1u8; 32];

        let mut buckets = HashMap::new();
        buckets.insert(1, 10);
        buckets.insert(2, 20);
        buckets.insert(3, 30);

        let a = select_bucket(seed, &buckets);
        let b = select_bucket(seed, &buckets);

        assert_eq!(a, b);
    }
}

