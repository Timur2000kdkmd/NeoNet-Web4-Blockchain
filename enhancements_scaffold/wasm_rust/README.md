WASM (Rust) scaffold
--------------------
This folder contains a Rust crate configured to build a cdylib wasm using wasm-bindgen.

Build steps (suggested):
1. Install Rust toolchain and wasm-bindgen-cli:
   - rustup target add wasm32-unknown-unknown
   - cargo install wasm-bindgen-cli
2. Build:
   - cargo build --target wasm32-unknown-unknown --release
3. Run wasm-bindgen to generate JS/WASM glue (optional for JS usage):
   - wasm-bindgen --out-dir pkg --target no-modules target/wasm32-unknown-unknown/release/neonet_wasm_contract.wasm

Note: For use with Go runtimes like wazero, you may skip wasm-bindgen and interact via raw wasm exports (with simpler functions).
