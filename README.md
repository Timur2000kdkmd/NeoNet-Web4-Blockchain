# NeoNet - AI-Powered Web4 Blockchain

## Overview

NeoNet is a next-generation Web4 blockchain platform featuring:

- **AI-Powered Proof of Intelligence (PoI) Consensus** - Validators prove intelligence through ML model training
- **Unified Dual Runtime (EVM+WASM)** - Integrated fabric with cross-VM orchestration
- **Post-Quantum Cryptography** - Hybrid Ed25519 + Dilithium3 signatures on ALL transactions
- **Single Token Economy** - NEO (50,000,000 supply) - governance, staking, gas, mining rewards
- **Web4 Wallet** - Dual keys: EVM (0x...) + Quantum (neo1...)
- **DualGov Governance** - AI voting (30%) + DAO voting (70%)
- **Autonomous AI Training** - Network trains itself on transaction data every 30 seconds

## Quick Start

### Prerequisites

- Node.js 18+
- Python 3.11+
- Rust (for core blockchain)
- Go 1.21+ (for consensus layer)

### Installation

```bash
# Clone repository
git clone https://github.com/Timur2000kdkmd/NeoNet-Web4-Blockchain.git
cd NeoNet-Web4-Blockchain

# Install Python dependencies
cd python-ai-service
pip install -r requirements.txt

# Install dApp dependencies
cd ../dapp
npm install

# Start backend
cd ../python-ai-service
python -m uvicorn app.main_simplified:app --host 0.0.0.0 --port 8000 --reload

# Start frontend (in another terminal)
cd dapp
npm run dev
```

## Architecture

### Network Components

```
NeoNet Architecture
├── Frontend (React + Vite)
│   ├── Web4 Wallet with dual keys
│   ├── Dashboard with live network stats
│   └── Developer tools for contract deployment
├── Backend (Python FastAPI)
│   ├── AI Service (Proof of Intelligence)
│   ├── Federated Learning Engine
│   └── Quantum Signature Verification
├── Blockchain Core (Rust)
│   ├── Unified EVM+WASM Runtime
│   └── Post-Quantum Cryptography
└── Consensus Layer (Go)
    ├── P2P Networking (libp2p)
    └── Proof of Authority with AI validation
```

### Directory Structure

```
NeoNetPQC/
├── dapp/                    # React frontend
│   ├── src/
│   │   ├── App.jsx         # Main application
│   │   ├── components/     # React components
│   │   └── lib/           # Wallet & utilities
│   └── package.json
├── python-ai-service/       # Python backend
│   ├── app/
│   │   ├── main_simplified.py  # FastAPI server
│   │   ├── neonet_blockchain.py # Blockchain simulation
│   │   ├── federated_learning.py
│   │   └── poi_consensus.py
│   └── requirements.txt
├── rust-core/              # Rust blockchain core
│   ├── src/
│   │   └── unified_runtime.rs
│   └── Cargo.toml
├── go-consensus/           # Go consensus layer
│   └── main.go
└── contracts/
    ├── sol/               # Solidity contracts
    └── wasm/              # CosmWasm contracts
```

## Token Economy

| Feature | Description |
|---------|-------------|
| **Total Supply** | 50,000,000 NEO |
| **Mining Rewards** | Earn NEO by completing AI tasks |
| **Staking** | Stake NEO to become validator |
| **Gas** | Pay for transactions and contracts |
| **Governance** | Vote on proposals (70% weight) |

## Quantum-Safe Signatures

ALL transactions on NeoNet use hybrid quantum-safe signatures:

```
Transaction Signature = {
  evm_signature: ECDSA (classical),
  quantum_signature: Ed25519 (quantum-resistant layer),
  dilithium_signature: Dilithium3 (NIST PQC Level 3)
}
```

This protects against both classical and future quantum computer attacks.

## API Endpoints

### Health & Status
- `GET /health` - Service health check
- `GET /poi/network/stats` - Network statistics

### Miners
- `POST /blockchain/miners/register` - Register as AI miner
- `GET /blockchain/miners` - List active miners
- `POST /blockchain/miners/submit_task` - Submit AI task result

### Governance
- `GET /governance/proposals` - List proposals
- `POST /governance/proposals` - Create proposal (DualGov: 30% AI + 70% DAO)
- `POST /governance/vote` - Vote on proposal

### Contracts
- `POST /contracts/deploy` - Deploy EVM/WASM/Hybrid contract
- `GET /contracts` - List deployed contracts

### AI Service
- `GET /ai/status` - Autonomous AI training status
- `POST /ai/validate_block` - AI block validation

## Earning NEO

NEO tokens can only be earned through work:

1. **Mining** - Register GPU/CPU and complete AI tasks
2. **Staking** - Stake NEO to validate blocks
3. **AI Training** - Contribute to federated learning
4. **Governance** - Participate in DAO voting

No faucet or free tokens - all NEO is earned.

## Deploy Contracts

### EVM (Solidity)
```javascript
import { ethers } from 'ethers';

const provider = new ethers.JsonRpcProvider('https://rpc.neonet.io');
const wallet = new ethers.Wallet(privateKey, provider);

const factory = new ethers.ContractFactory(abi, bytecode, wallet);
const contract = await factory.deploy();
```

### WASM (Rust/CosmWasm)
```rust
use cosmwasm_std::{entry_point, DepsMut, Env, MessageInfo, Response};

#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    Ok(Response::new())
}
```

### Hybrid (Cross-Runtime)
```javascript
const deployHybrid = async () => {
  const response = await fetch('/api/contracts/deploy', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({
      code: contractCode,
      runtime: 'hybrid',
      deployer: walletAddress
    })
  });
  return response.json();
};
```

## How the Network Works

### Current State
- **21 Active Validators** - PoI consensus with AI validation
- **Block Time** - 3 seconds
- **AI Training** - Automatic every 30 seconds on network data
- **Fraud Detection** - Real-time AI analysis of transactions

### Validator Selection
Validators are selected based on:
1. Stake amount (weight)
2. Intelligence Score (AI performance)
3. Blocks validated history

### Miner Rewards
Miners earn NEO by:
1. Registering compute resources (CPU/GPU)
2. Completing AI tasks (fraud detection, model training)
3. Quality of work determines reward multiplier

## Security Features

- **Post-Quantum Cryptography**: Hybrid Ed25519+Dilithium3 signatures
- **Fraud Detection**: AI-powered transaction analysis
- **Rate Limiting**: DDoS protection
- **Sandboxed Execution**: Isolated contract runtime
- **Replay Protection**: Transaction nonces

## License

MIT License

## Links

- GitHub: https://github.com/Timur2000kdkmd/NeoNet-Web4-Blockchain
