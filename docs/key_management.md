# Key generation & production readiness instructions (DO NOT commit secrets to VCS)

## Ethereum keystores (recommended, non-demo)
1. Install Node.js and run the provided script to create encrypted keystores:
   ```bash
   cd contracts/hardhat
   npm install ethers
   node ../scripts/generate_eth_keystore.js <strong-password> 3
   ```
   This writes keystore JSON files to `secrets/keystores` and a `secrets/keystore_manifest.json`

2. Secure the password in your environment or KMS. Do NOT store master passwords in plain files.

3. Use the keystore files by copying the keystore path to `relayer/.env` as `KEYSTORE_PATH` and set `KEYSTORE_PASSWORD` (or configure KMS for decryption).

## Ed25519 keys for WASM / Validators
Generate Ed25519 key pairs for your WASM runtime (CosmWasm validators or nodes):
```bash
./scripts/generate_ed25519.sh validator1
```
Store private keys on each node securely (file permissions, KMS, or HSM).

## Post-Quantum (PQC) Integration
1. Choose a PQ library that has stable bindings (e.g., liboqs + PQClean bindings, or pydilithium if available).
2. Provision dedicated key pairs for PQ algorithms using that library and manage keys via KMS/HSM.
3. Off-chain verification is the pragmatic approach: verify PQ signatures in relayer/ai-service and submit aggregated results or hashes on-chain. On-chain verification requires special runtime/precompiles.

## Hardware-backed keys and HSM
- For production, use an HSM (AWS CloudHSM, Azure Key Vault HSM) or cloud KMS.
- Configure relayer and node signing to use the HSM provider API instead of local files.

## Secrets hygiene
- Never commit private keys or keystore passwords.
- Use environment variables with restricted access or secret management systems.


## mTLS and network hardening (placeholders)

- Use mutual TLS between services: generate CA, sign server and client certs, and configure services (ai-service, relayer, geth rpc) to require client certs for sensitive endpoints.
- Example (nginx sidecar): use certs mounted from secrets and configure proxy_pass with client cert verification.
- For HSM/KMS integration, ensure service account roles are mapped to KMS keys and permissions for sign-only.
- Always rotate keys and enforce least-privilege access.

Add to k8s manifests:
- Use SecretProviderClass or CSI driver to mount secrets.
- Add NetworkPolicies to restrict egress/ingress between namespaces.
