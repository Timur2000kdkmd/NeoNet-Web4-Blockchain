Notes, references, and migration plan
------------------------------------

1) Background & standards
   - NIST has selected and standardized a set of PQC algorithms; these are the authoritative starting point for migration. (e.g., CRYSTALS-Kyber for KEM; Dilithium/FALCON/SPHINCS+ for signatures).

2) Recommended libraries & tooling (examples)
   - Open Quantum Safe (liboqs) project provides C implementations and tooling for experimentation and integration (liboqs, liboqs-go, liboqs-python).
   - Rust PQC crates: PQClean/pqcrypto family provide Rust bindings or pure-Rust implementations for many candidates.
   - Go bindings: liboqs-go provides Go wrappers around liboqs for prototyping.

3) Hybrid signatures approach
   - For transitional security, sign messages with both a classical signature (e.g., ECDSA or Ed25519) and a PQC signature (Dilithium or Falcon), then publish both signatures with the message. Verification requires checking both signatures. Optionally, create a single "hybrid" signature structure combining both and include metadata about algorithms and versions.
   - Store algorithm identifiers and key versioning data with each signature to allow future migration.

4) Migration plan (high-level)
   - Inventory cryptographic usage and key lifetimes.
   - Add crypto-agility layers: abstraction that allows swapping primitives via configuration.
   - Introduce hybrid signatures on sensitive messages (keys, transactions) first.
   - Test performance and storage costs; PQC signatures/keys are typically larger.
   - Deploy to testnet, run interoperability tests, then gradually migrate mainnet.

References:
- NIST PQC project and selected algorithms (refer to NIST publications and updates).
- Open Quantum Safe (liboqs) and bindings for prototyping.
