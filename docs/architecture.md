# Architecture

NeoNet is designed as a hybrid blockchain + AI compute network.

Components:
- Rust core: deterministic block building, tx pool.
- Go consensus: P2P network and proposer/validator logic.
- Solidity contracts: EVM-exposed token and governance.
- WASM contracts: model registry and on-chain metadata.
- AI service: off-chain compute orchestration; miners register and receive tasks.
- Bridges/Oracles: to move decisions from AI service to chain (not implemented in full here).

Determinism:
- All on-chain state is deterministic. AI results are produced off-chain, then submitted via signed reports and aggregated via threshold signatures or multi-sig oracles.

Security:
- Placeholder hooks for attestation (mTLS/HW enclave).
- Access control via ownership and timelocks for governance.



## Deterministic Aggregation and Oracle Workflow

- Miners (compute providers) register and receive tasks. Each miner signs its result using ECDSA and returns `result` and `sig`.
- The AI service aggregates results deterministically by sorting miner IDs and concatenating result blobs, then computing SHA-256 hash.
- An aggregated `report_id` and `result_hash` are produced and stored in `/completed_reports` for relayers.
- Relayers fetch completed reports, verify signatures off-chain (or let the Oracle contract verify on-chain), and submit `submitReport` transaction to the Oracle contract.
- The Oracle enforces that at least `threshold` registered signers endorsed the result.

## Quantum-resistant (PQC) notes

- Production should replace or augment ECDSA with post-quantum signature schemes (e.g., Dilithium for signatures, Kyber for KEM). Currently code contains placeholders and comments where PQ primitives should be integrated.
- Off-chain components (AI service, relayers) should support PQ key management and rotation; on-chain verification requires compatible smart contract primitives or preimage-challenge designs.


# Improvements in V3

- Persistence: AI service now stores tasks and miners in a relational DB (Postgres), and uses Redis as a work queue. This enables reliable scheduling and horizontal scaling of workers.
- Worker autoscaling: the AI service exposes `/queue_length` and `/scale_signal` endpoints that can be used by K8s HPA or external autoscaler to scale worker replicas based on queue backlog.
- EVM<->WASM bridge demo: relayer submits aggregated Oracle reports to the EVM Oracle contract and then calls a CosmWasm RPC endpoint to demonstrate cross-runtime execution.
- PQ-signature demo: `pq_demo.py` provides a placeholder signing API and instructions for integrating a real PQ library (Dilithium). Off-chain verification is demonstrated by the relayer and AI service.
- Security notes: add mTLS for miner/worker endpoints, use hardware attestation for trusted compute (SGX/SEV) when available, and store keys in secure KMS.
