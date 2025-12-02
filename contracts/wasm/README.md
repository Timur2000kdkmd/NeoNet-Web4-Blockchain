# NeoNet WASM Smart Contracts

## Overview

NeoNet supports WebAssembly smart contracts using the CosmWasm standard. This enables:
- AI model registry and versioning
- On-chain staking and rewards
- DualGov governance (AI + DAO hybrid)

## Contracts

### 1. AI Model Registry (`lib.rs`)
Registers and tracks AI models used in Proof of Intelligence consensus.

**Messages:**
- `RegisterModel`: Register a new AI model with IPFS hash
- `UpdateModel`: Update an existing model
- `GetModel`: Query model information
- `ListModels`: List all registered models

### 2. Staking Contract (`stake_contract.rs`)
Full CosmWasm staking with rewards.

**Messages:**
- `Stake {}`: Stake NEO tokens (send funds with transaction)
- `Unstake {amount}`: Unstake tokens
- `ClaimRewards {}`: Claim staking rewards

**Queries:**
- `GetStake {address}`: Get stake info for address
- `GetTotalStaked {}`: Get total staked amount
- `GetRewards {address}`: Calculate pending rewards

### 3. DualGov Governance (`governance_contract.rs`)
Hybrid AI + DAO governance system.

**Features:**
- Proposal creation with IPFS metadata
- Weighted voting (DAO 70% + AI 30%)
- AI score integration from Proof of Intelligence
- Quorum and execution logic

**Messages:**
- `CreateProposal`: Create governance proposal
- `Vote`: Vote on proposal with staked weight
- `SetAIScore`: Set AI confidence score (called by oracle)
- `Execute`: Execute proposal after voting period

## Current Status

### ✅ Implemented
- `lib.rs`: AI Model Registry with CosmWasm storage
- `stake_contract.rs`: Full CosmWasm staking contract with entry points
- `governance.rs` & `stake.rs`: Pure Rust logic modules (helpers)

### ⚠️ In Progress
- Governance contract CosmWasm entry points (see `governance_contract.rs` - TODO)
- Integration with Rust core WASM VM
- Oracle integration for AI scores

### 🔧 Future Work
- Slashing conditions for validators
- Time-locked staking for boosted rewards
- Multi-signature governance execution
- Cross-contract calls between WASM contracts

## Building

```bash
cd contracts/wasm
cargo build --release --target wasm32-unknown-unknown
```

This produces `.wasm` files in `target/wasm32-unknown-unknown/release/`.

## Testing

```bash
cargo test
```

## Deployment

Once NeoNet Rust Core supports full WASM execution:

```bash
# Deploy model registry
neonet deploy model_registry.wasm

# Deploy staking contract with reward rate
neonet deploy stake_contract.wasm --init '{"reward_rate":"100"}'

# Deploy governance contract
neonet deploy governance_contract.wasm --init '{"quorum":10,"ai_weight":30,"dao_weight":70}'
```

## Architecture

### Storage Layout

Each contract uses CosmWasm's `cw_storage_plus`:
- `Item<T>`: Single value storage
- `Map<K, V>`: Key-value map storage

### Gas Metering

CosmWasm automatically meters:
- Storage reads/writes
- Computation (WASM instructions)
- Memory allocation

### Integration with AI

The governance contract accepts AI scores from the Python AI service via:
1. Python AI analyzes proposal
2. Scores sent to Oracle contract
3. Oracle calls `SetAIScore` on governance contract
4. DualGov calculation combines AI score (30%) + DAO votes (70%)

## Helper Modules

`stake.rs` and `governance.rs` are pure Rust modules providing:
- Business logic
- Data structures
- Helper functions

These are **not** CosmWasm contracts but can be used:
- For unit testing
- As reference implementations
- In off-chain tools

The actual on-chain contracts are:
- `lib.rs` (AI Model Registry) - ✅ Full CosmWasm
- `stake_contract.rs` (Staking) - ✅ Full CosmWasm
- Future: `governance_contract.rs` (DualGov) - 🔧 In Progress

## Security

- All entry points validate inputs
- Storage access is type-safe
- Gas limits prevent DoS
- No direct token transfers (use CosmWasm bank module)

## Examples

### Stake Tokens
```rust
// Execute on chain
let msg = ExecuteMsg::Stake {};
let funds = vec![Coin::new(1000000, "uneo")];
execute(deps, env, info, msg);
```

### Vote on Proposal
```rust
let msg = ExecuteMsg::Vote {
    proposal_id: 1,
    support: true,
};
execute(deps, env, info, msg);
```

### Query Rewards
```rust
let msg = QueryMsg::GetRewards {
    address: "neonet1...".to_string(),
};
let rewards: RewardsResponse = query(deps, env, msg)?;
```
