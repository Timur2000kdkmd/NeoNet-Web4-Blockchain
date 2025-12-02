# NeoNet - AI-Powered Web4 Blockchain

## Overview

NeoNet is an autonomous, AI-driven Web4 blockchain network designed for enhanced security, scalability, and intelligence. It integrates post-quantum cryptography, a dual EVM and WASM runtime for smart contracts, and an AI-powered Proof of Intelligence (PoI) consensus mechanism. The project aims to create a highly decentralized and intelligent blockchain ecosystem with a hybrid governance model (DualGov) combining AI analysis and DAO voting.

## Recent Changes (December 2024)

- Created full-featured dApp with React/Vite
- Dashboard with live network statistics
- AI Tasks management interface with backend integration
- Staking functionality with proper amount tracking
- DualGov governance with proposals and voting
- Configured Vite proxy for API communication
- Updated UI with modern dark theme matching NeoNet branding

## Quick Start

### Running the Full Stack

1. **Start the dApp (Frontend)** - Automatically configured:
   ```bash
   cd NeoNetPQC/dapp && npm run dev
   ```
   - Runs on port 5000
   - Proxies API requests to port 8000

2. **Start the AI Service (Backend)** - Optional for full functionality:
   ```bash
   cd NeoNetPQC/python-ai-service/app && python -m uvicorn main_simplified:app --host 0.0.0.0 --port 8000
   ```

### Verification

1. Open the webview to see the dApp
2. Navigate through Dashboard, AI Tasks, Staking, Governance tabs
3. If AI service is running, tasks will be submitted to the backend

## Architecture

### Frontend (dApp)
- **React 18 + Vite**: Modern single-page application
- **Port 5000**: Frontend dev server
- **API Proxy**: Forwards `/api/*` to `http://localhost:8000`

### Backend (AI Service)
- **FastAPI**: Python REST API
- **Port 8000**: Backend server
- **Endpoints**:
  - `GET /health` - Service health check
  - `GET /tasks` - List AI tasks
  - `POST /submit_task` - Submit new task
  - `POST /ai/validate_block` - AI block validation
  - `GET /pqc/status` - PQC status
  - `GET /governance/status` - DualGov status

### Technical Stack

- **Rust Blockchain Core** (`rust-core/`): Block structure, chain validation, WASM and EVM runtimes, PQC with Ed25519 and Dilithium/Kyber
- **Go Consensus Layer** (`go-consensus/`): P2P network using libp2p with GossipSub, PBFT consensus
- **Python AI Service** (`python-ai-service/`): Fraud detection, gas optimization, Proof of Intelligence, federated learning
- **Smart Contracts** (`contracts/`): Solidity (EVM) and CosmWasm (WASM)

## Token Economy

1. **NEO (Main)**: Native utility token for gas fees, governance, and network security
2. **WASM-X**: Reward token for developers executing high-performance WASM smart contracts
3. **CPT (Compute)**: Payout token for providers contributing GPU/CPU power to the grid

## Key Features

- **Proof of Intelligence**: AI validates blocks using fraud detection models
- **DualGov Governance**: AI (30%) + DAO voting (70%) for decisions
- **Post-Quantum Security**: Dilithium3 signatures, Kyber1024 encryption
- **Dual Runtime**: EVM + WASM smart contract execution

## Project Structure

```
NeoNetPQC/
├── dapp/                    # React dApp (frontend)
│   ├── src/App.jsx         # Main application
│   ├── vite.config.js      # Vite config with proxy
│   └── package.json
├── python-ai-service/       # Python AI services
│   └── app/
│       ├── main_simplified.py  # Standalone AI service
│       └── ai_engine.py        # PyTorch models
├── rust-core/               # Rust blockchain core
├── go-consensus/            # Go P2P and consensus
├── contracts/               # Smart contracts
└── docs/                    # Documentation
```

## User Preferences

Prefer concise explanations and iterative development. Ask for confirmation before implementing significant changes or making architectural decisions.
