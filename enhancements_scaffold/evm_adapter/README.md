EVM Adapter (Stage 4)
---------------------

This adapter provides tools and example code to deploy and interact with an Ethereum-compatible contract
from Go (using go-ethereum). It is intended as a scaffold — replace placeholders (private key, RPC URL,
and contract bytecode) with real values for deployment.

Files:
- SampleContract.sol             Solidity source (simple contract with uint public x and setX)
- contract.abi.json              ABI file (provided)
- contract.bin                   Bytecode placeholder (replace with real compiled bytecode)
- go/                           Go adapter with example deployment and interaction code
- README.md                      This file

Steps to compile the Solidity contract (locally):
1. Install solc (solidity compiler) or use hardhat/truffle.
2. Compile and extract ABI and BIN. Example using solc:
   solc --combined-json abi,bin SampleContract.sol > out.json
   Then split ABI and BIN to files `contract.abi.json` and `contract.bin`

How to use Go adapter (example):
1. Ensure you have a local Ethereum node or test node (geth, ganache, or anvil) running and RPC reachable at RPC_URL.
2. Place `contract.abi.json` and `contract.bin` (raw hex, starting with 0x) into this folder.
3. Edit `go/main.go` to set `rpcURL` and `privateKeyHex` variables, or pass them via env vars.
4. From `go/` run:
   go mod tidy
   go run main.go
This will deploy the contract, call setX(42), and read x via a call.

Security notes:
- Never commit private keys to source control.
- For production, use secure key management (HSM/KMS) and proper nonce/account management.
