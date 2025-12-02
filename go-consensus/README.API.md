Go-consensus API and persistence

HTTP API:
- GET /chain -> returns JSON array of blocks
- GET /peers -> returns JSON array of peer addresses
- POST /tx with JSON {"data":"..."} -> creates a new block, broadcasts, persists, returns created block

Persistence:
- Chain is saved to chain_store.json after new blocks are added.
- Node loads chain_store.json on startup if present.

Environment:
- AI_BASE env for AI service (default http://127.0.0.1:8000)

EVM_RELAYER env var controls external EVM relayer endpoint (default http://127.0.0.1:9000)
