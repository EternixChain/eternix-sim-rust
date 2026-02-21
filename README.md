# Eternix Simulator

A Rust-based deterministic simulator for the Eternix consensus protocol.

This project models the core mechanics of Eternix’s Proof-of-Ash design, including:

- Deterministic 3-second slot timing
- Leader selection with bucket-based ticket partitioning
- Protocol-produced fallback blocks
- Liveness tracking and escalating slashing
- Double-sign detection with escalating punishment
- Epoch-based cooldown mechanics
- Validator states (Active, PunishedCooldown, PausedLowVault, Jailed)
- Bucket transitions (ACTIVE / MUTED / DEAD)

The simulator is designed to stress-test consensus invariants before implementation in the production chain.

---

## Current Features

### Deterministic Slot Engine
Every block occurs exactly 3 seconds apart.  
No early blocks. No delayed blocks.

If a leader fails to propose in time, the protocol produces an empty block.

---

### Ticket Buckets
Tickets are partitioned into buckets:

- **ACTIVE** — eligible for leader selection (254)
- **MUTED** — temporarily ineligible (cooldown) (1)
- **DEAD** — permanently ineligible (jailed / retired) (1)

Leader selection operates in two stages:
1. Select an ACTIVE bucket weighted by ticket count
2. Select a ticket within that bucket

This preserves probability fairness while reducing selection cost.

---

### Liveness Slashing
Missed blocks (when selected leader fails to propose):

- Slash at 5 misses
- Slash again at 105, 205, ...
- Slash applies to current vault balance
- Validator enters cooldown after slash

Cooldown lasts multiple epochs to guarantee minimum downtime.

---

### Double-Sign Punishment
If a leader produces two distinct blocks for the same slot:

- 1st offense: 50% slash + 2 epoch mute
- 2nd offense: 75% slash + 5 epoch mute
- 3rd offense: 100% slash + permanent jail

Jailed validators move to the DEAD bucket and never return.

Double-sign punishment is orthogonal to liveness tracking.

---

## Design Goals

This simulator prioritizes:

- Determinism
- Explicit state transitions
- Non-halting behavior
- Clear validator lifecycle
- Economic stress-testability

It is intentionally minimal and simulation-focused.

No networking. No cryptography. No persistence.

Only state logic.

---

## Running

```bash
cargo run
```
The main loop simulates sequential slots and prints validator state transitions.

---

## Status

Core validator lifecycle logic implemented.
Next steps include:
- Reward distribution modeling
- Inflation simulation
- Long-horizon economic testing

---

This repository models the rules before the chain exists.

The protocol must survive here first.