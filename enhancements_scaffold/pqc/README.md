Post-Quantum Cryptography (PQC) scaffold for NeoNet
==================================================

Purpose
-------
This folder contains scaffolds, notes and example code for adding post-quantum cryptography (PQC)
and hybrid signatures to NeoNet. It is intended as a developer starting point — not a drop-in production implementation.

Key recommendations (short):
- Favor NIST-selected algorithms for standardization (CRYSTALS-Kyber for KEM; CRYSTALS-Dilithium, FALCON, SPHINCS+ for signatures) and track NIST updates.
- Use liboqs / Open Quantum Safe (OQS) tooling for prototyping and vetted implementations.
- Adopt hybrid signatures (classical scheme + PQC scheme) to provide crypto-agility and transitional protection.
- Require sandboxed, memory-safe, constant-time implementations; do not hand-roll crypto.

This folder contains:
- `notes.md` — detailed notes, migration plan and references.
- `rust_pqc/` — Rust scaffold showing how to structure hybrid signatures (uses pqcrypto crates as examples; calls are placeholders).
- `go_pqc/` — Go scaffold showing how to call liboqs-go (placeholders) and combine ECDSA + Kyber into a hybrid envelope.
- `SECURITY_PQC.md` — concrete checklist for PQ migration and testing.

IMPORTANT:
- These are scaffolds with example code and placeholders. Do full security review and use vetted libs (liboqs, PQClean-based crates, or vendor-provided FIPS-capable modules) for any real use.
