Rust PQC implementation notes (updated)
--------------------------------------
- This crate now supports saving and loading hybrid key material to `key.json` (hex encoded).
- Use `generate_hybrid_keypair_bytes`, `save_key_json`, and `sign_with_persisted_keys` to produce signatures reproducibly.
- Run `cargo test` in this folder to execute the persistence/sign/verify roundtrip test which writes `test_key.json` and `last_signature.json` during the test and then cleans up.
- Note: building requires the pqcrypto crates which may need system prerequisites; run in the CI container if needed.
