PQC Security Checklist
----------------------
- Use vetted libraries (liboqs, PQClean, PQCrypto crates). Prefer recommended implementations from NIST/OSS projects.
- Ensure constant-time implementations and resist side-channels (use masking where needed).
- Enforce strict memory handling and zeroing of secret material.
- Implement key rotation and versioning; track epoch and algorithm IDs in signatures.
- Provide backward compatibility: verify hybrid signatures (classical + PQC) during transition.
- Performance testing: measure signing/verification throughput and signature sizes; adapt networking payloads accordingly.
- CI: add fuzzing, unit tests for signature edge cases, and cross-language interoperability tests.
- Third-party audit: mandatory before production.
