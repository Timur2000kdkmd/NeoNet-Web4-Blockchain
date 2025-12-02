AI Module scaffold (Stage 5)
===========================

This scaffold provides a minimal in-network AI coordinator with:
- `agent.py` — AINode class: local training stub, fetch chain state, propose votes, submit votes as transactions.
- `ai_api.py` — Small Flask API to call train_step and propose/vote endpoints.
- `requirements.txt` — Flask + requests for running the API locally.

Integration with NeoNet components:
- Blockchain: uses the Rust blockchain HTTP API at http://127.0.0.1:3030 (endpoints /tx, /mine, /chain).
- P2P: The Go P2P node can relay transactions from the network into the blockchain; the AI node listens to on-chain state to form proposals.
- WASM/EVM: AI may produce payloads that reference WASM contracts or EVM adapter transactions (store policy off-chain, include references in tx payloads).

Energy accounting & participation model (prototype):
- The scaffold expects an off-chain 'energy' accounting mechanism. Users deposit 'energy' (tokens) to be allowed to initiate training or voting. This should be enforced by signed transactions and smart contracts.
- The current scaffold only shows how to package votes into tx payloads; do NOT rely on it for real accounting without cryptographic attestations.

Security checklist (required before any deployment):
1. Sandbox training runtime (e.g., run model code in a signed WASM runtime or Docker container with strict limits).
2. Differential privacy / secure aggregation for any model updates derived from user data.
3. Attestation & remote attestation: nodes must produce cryptographic attestation that their training runtime is genuine.
4. Key management: never store raw private keys in code; use HSM/KMS or secure enclaves.
5. Rate limiting, authentication, and signed messages for all network endpoints.
6. Third-party security audit of ML code and integration paths.
