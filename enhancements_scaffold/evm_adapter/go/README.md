Go EVM Adapter
--------------
This example shows how to deploy and interact with a simple Solidity contract from Go (using go-ethereum).
It expects `contract.abi.json` and `contract.bin` to be available in the parent folder.

Usage:
  export RPC_URL=http://127.0.0.1:8545
  export PRIVATE_KEY_HEX=<hex private key without 0x>
  cd go
  go mod tidy
  go run main.go
