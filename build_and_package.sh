#!/usr/bin/env bash
set -e
echo "Building rust-core"
cd rust-core
cargo build --release || true
cd ..

echo "Building go-consensus"
cd go-consensus
go build -o consensus || true
cd ..

echo "Done (local builds may require dev dependencies)."