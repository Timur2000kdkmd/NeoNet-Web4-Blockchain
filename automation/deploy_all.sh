#!/usr/bin/env bash
set -euo pipefail
# automation/deploy_all.sh
# 1) Start Hardhat node in a new background process
# 2) Deploy contracts to local Hardhat node
# 3) Extract deployed Oracle address and write relayer/.env
# 4) Generate local keystore if requested and update relayer env
# 5) Start docker-compose stack

ROOT_DIR="$(cd "$(dirname "$0")/.."; pwd)"
HARDHAT_DIR="$ROOT_DIR/contracts/hardhat"
RELAYER_ENV="$ROOT_DIR/relayer/.env"
SECRETS_DIR="$ROOT_DIR/secrets"

echo "Step 1: Start Hardhat node"
pushd "$HARDHAT_DIR" > /dev/null
if [ ! -d "node_modules" ]; then
  echo "Installing hardhat dependencies..."
  npm install
fi
# run hardhat node in background and log to file
npx hardhat node > hardhat_node.log 2>&1 &
HPID=$!
echo "Hardhat node started (pid $HPID). Waiting 4s for startup..."
sleep 4

echo "Step 2: Deploy contracts"
npx hardhat run scripts/deploy.js --network localhost > deploy_output.json || { echo "Deploy failed"; kill $HPID; exit 1; }
echo "Deploy output saved."

# Try to parse addresses from deploy output (last JSON printed)
ORACLE_ADDR=$(tail -n 20 deploy_output.json | tr -d '\r' | sed -n '1,200p' | tr -d '\n' | sed -n 's/.*{\(.*\)}.*/{\1}/p' || true)
# fallback: simple grep for addresses
ORACLE_ADDR=$(grep -o '0x[a-fA-F0-9]\{40\}' deploy_output.json | head -n1 || true)

if [ -z "$ORACLE_ADDR" ]; then
  echo "Could not detect ORACLE address from deploy output. Please set manually in relayer/.env"
else
  echo "Detected ORACLE_ADDR: $ORACLE_ADDR"
fi

echo "Step 3: Prepare relayer env"
cp relayer/.env.example relayer/.env || true
# Write oracle and eth node into relayer/.env
sed -i.bak '/^ORACLE_ADDR=/c\ORACLE_ADDR='${ORACLE_ADDR} relayer/.env || true

# Step 4: Optionally generate keystore
if [ -n "${1-}" ] && [ "$1" == "--gen-keystore" ]; then
  echo "Generating keystore files..."
  node ../scripts/generate_eth_keystore.js "changeit" 1
  KEYSTORE_PATH_REL="secrets/keystores/keystore-0.json"
  echo "KEYSTORE_PATH=$KEYSTORE_PATH_REL" >> relayer/.env
  echo "KEYSTORE_PASSWORD=changeit" >> relayer/.env
fi

echo "Step 5: Start docker-compose stack"
pushd "$ROOT_DIR/infra" > /dev/null
docker-compose up -d --build
popd > /dev/null

echo "All started. To stop hardhat node: kill $HPID"
echo "Relayer env is at relayer/.env (update RELAYER_KEY or KEYSTORE_PATH as needed)."
popd > /dev/null
