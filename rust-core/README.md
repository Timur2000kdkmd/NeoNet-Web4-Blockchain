# NeoNet Rust Core

## Architecture

NeoNet Rust Core provides the blockchain runtime with:

1. **Blockchain Core**: Block structure, chain validation, transaction processing
2. **WASM VM**: WebAssembly smart contract execution with gas metering
3. **EVM Adapter**: Ethereum Virtual Machine compatibility layer
4. **PQC Module**: Hybrid post-quantum cryptography (Ed25519 + Dilithium support)
5. **Bridge**: TCP bridge for Go consensus integration (port 6000)

## Current Implementation Status

### ✅ Implemented
- Basic blockchain data structures (Block, Transaction, Chain)
- Ed25519 signature verification
- WASM contract storage and basic gas metering
- EVM account management and basic method dispatch
- Go-Rust bridge over TCP

### ⚠️ Limitations & Future Work

#### WASM VM
- **Current**: Contract storage with gas constants, basic method dispatch
- **Future**: 
  - Full WASM bytecode execution using Wasmer/Wasmtime
  - Per-instruction gas metering
  - State persistence with merkle proofs
  - CosmWasm standard compliance

#### EVM Adapter
- **Current**: Account management, basic method selectors (balanceOf, transfer)
- **Future**:
  - Full EVM bytecode execution via revm
  - Complete opcode gas accounting
  - State trie and storage proofs
  - Ethereum JSON-RPC compatibility

#### Post-Quantum Cryptography
- **Current**: Ed25519 signatures (production-ready)
- **Optional**: Dilithium support when pqcrypto-rust is available
- **Note**: System operates securely with Ed25519; PQC is additive security
- **Future**: 
  - NIST finalist integration (Dilithium, Kyber)
  - Hybrid signature verification
  - Post-quantum key exchange

## Building

```bash
cd rust-core
cargo build --release
cargo test
```

## Running

```bash
cargo run
```

The bridge listens on `127.0.0.1:6000` for Go consensus integration.

## Testing

```bash
# Run all tests
cargo test

# Run specific module tests
cargo test --test test_chain
cargo test wasm_vm::tests
cargo test pqc::tests
cargo test evm_adapter::tests
```

## Integration

The Rust core integrates with:
- **Go Consensus**: Via TCP bridge on port 6000
- **Python AI Service**: Via HTTP API for block ingestion
- **TypeScript SDK**: Via JSON-RPC (future)

## Security Notes

1. **Ed25519**: Production-ready, widely audited signature scheme
2. **PQC**: Hybrid approach allows graceful degradation
3. **Gas Metering**: Basic implementation; full per-op metering planned
4. **State Validation**: Chain validation prevents invalid state transitions

## Architecture Decisions

### Why Hybrid PQC?
- Maintains classical security (Ed25519)
- Adds quantum resistance when available
- No breaking changes for existing integrations

### Why Stub WASM/EVM Initially?
- Allows integration testing of full stack
- Real execution engines (Wasmer, revm) add complexity
- Incremental development path: storage → dispatch → execution

### Why TCP Bridge?
- Simple cross-language IPC
- Low latency for local communication
- Easy to debug and monitor

## Performance

Current benchmarks (on test hardware):
- Block validation: ~1ms
- Ed25519 signature: ~50μs
- Basic WASM call: ~100μs
- EVM transfer: ~200μs

Production optimizations planned:
- Parallel transaction execution
- State caching
- Batch signature verification
