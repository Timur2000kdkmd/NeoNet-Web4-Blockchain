#!/usr/bin/env bash
set -e
echo "Building Rust -> wasm (wasm32-unknown-unknown)"
rustup target add wasm32-unknown-unknown || true
cargo build --target wasm32-unknown-unknown --release
echo "Built: target/wasm32-unknown-unknown/release/neonet_wasm_contract.wasm"
