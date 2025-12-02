Go WASM loader (wazero)
-----------------------
This small program demonstrates loading a wasm file and calling an exported function using wazero.

Build and run:
1. Put the compiled `neonet_wasm_contract.wasm` (from enhancements_scaffold/wasm_rust/target/...) next to this program.
2. From this folder run:
   go mod tidy
   go run main.go
