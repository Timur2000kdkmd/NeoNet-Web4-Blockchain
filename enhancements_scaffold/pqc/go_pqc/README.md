Go PQC interop example (updated)
--------------------------------
This program attempts to read `test_key.json` and `last_signature.json` produced by the Rust persistence/sign test, and then (optionally)
verify the PQC signature using liboqs-go.

Steps to run interoperbility test locally:
1. Build and run Rust tests in `enhancements_scaffold/pqc/rust_pqc`:
   cd enhancements_scaffold/pqc/rust_pqc
   cargo test -- --nocapture
   This writes `test_key.json` and `last_signature.json` in the crate folder.

2. Copy `test_key.json` and `last_signature.json` to `enhancements_scaffold/pqc/go_pqc/` (or run Go from the crate root if files are reachable).

3. Install liboqs C library and liboqs-go bindings (see enhancements_scaffold/ci/pqc_test.Dockerfile for automated build).

4. Update this Go program: uncomment the import of liboqs-go and verification code. Then run:
   go run main.go
Note: verification code is provided as pseudo-code and must be adapted to the liboqs-go API version in your environment.
