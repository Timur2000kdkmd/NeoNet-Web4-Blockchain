#!/usr/bin/env bash
set -euo pipefail
# automation/full_auto_run.sh
# Robust automation: start hardhat, deploy, wait for node, update relayer env, start docker-compose and wait for services healthy.
ROOT_DIR="$(cd "$(dirname "$0")/.."; pwd)"
HARDHAT_DIR="$ROOT_DIR/contracts/hardhat"
RELAYER_ENV="$ROOT_DIR/relayer/.env"
SECRETS_DIR="$ROOT_DIR/secrets"
COMPOSE_DIR="$ROOT_DIR/infra"

LOG_DIR="$ROOT_DIR/logs"
mkdir -p "$LOG_DIR"

echo "1) Ensure prerequisites: node, npm, docker, docker-compose are installed."
command -v node >/dev/null 2>&1 || { echo "node is required"; exit 1; }
command -v docker >/dev/null 2>&1 || { echo "docker is required"; exit 1; }

cd "$HARDHAT_DIR"
if [ ! -d "node_modules" ]; then
  echo "Installing Hardhat deps..."
  npm ci
fi

echo "Starting Hardhat node in background... (logs -> $LOG_DIR/hardhat_node.log)"
npx hardhat node > "$LOG_DIR/hardhat_node.log" 2>&1 &
HPID=$!
echo "Hardhat PID: $HPID"
# wait for RPC to come up
RETRIES=20
i=0
until curl -sS --fail http://127.0.0.1:8545/jsonrpc >/dev/null 2>&1 || [ $i -ge $RETRIES ]; do
  echo "Waiting for Hardhat RPC... ($i/$RETRIES)"
  sleep 1
  i=$((i+1))
done
if [ $i -ge $RETRIES ]; then
  echo "Hardhat RPC did not start in time. Check $LOG_DIR/hardhat_node.log"
  kill $HPID || true
  exit 1
fi
echo "Hardhat RPC is up."

echo "Deploying contracts..."
npx hardhat run scripts/deploy.js --network localhost > "$LOG_DIR/deploy_output.log" 2>&1 || { echo "Deploy failed, see logs"; kill $HPID; exit 1; }

# extract first address from deploy_output.log
ORACLE_ADDR=$(grep -o '0x[a-fA-F0-9]\{40\}' "$LOG_DIR/deploy_output.log" | head -n1 || true)
if [ -z "$ORACLE_ADDR" ]; then
  echo "Could not parse ORACLE_ADDR from deploy logs; please set manually in relayer/.env"
else
  echo "Detected ORACLE_ADDR=$ORACLE_ADDR"
fi

# prepare relayer env
cp "$ROOT_DIR/relayer/.env.example" "$RELAYER_ENV" || true
# set ORACLE_ADDR and ETH_NODE
perl -i -pe "s|^ORACLE_ADDR=.*|ORACLE_ADDR=${ORACLE_ADDR}|" "$RELAYER_ENV" || true
perl -i -pe "s|^ETH_NODE=.*|ETH_NODE=http://127.0.0.1:8545|" "$RELAYER_ENV" || true

# Optionally generate keystore if not present
if [ ! -f "$ROOT_DIR/secrets/keystore_generated" ]; then
  echo "Generating a local keystore for relayer (dev)"
  pushd "$ROOT_DIR/scripts" >/dev/null
  node ../contracts/hardhat/node_modules/.bin/ethers || true
  # call helper script if exists
  if [ -f "$ROOT_DIR/scripts/generate_eth_keystore.js" ]; then
    node generate_eth_keystore.js "changeit" 1
    echo "KEYSTORE_PATH=secrets/keystores/keystore-0.json" >> "$RELAYER_ENV"
    echo "KEYSTORE_PASSWORD=changeit" >> "$RELAYER_ENV"
    touch "$ROOT_DIR/secrets/keystore_generated"
  fi
  popd >/dev/null
fi

echo "Starting docker-compose stack (logs -> $LOG_DIR/docker_compose.log)"
cd "$COMPOSE_DIR"
docker-compose up -d --build > "$LOG_DIR/docker_compose.log" 2>&1

# wait for AI service health
AI_URL="http://localhost:8000/healthz"
RETRIES=30
i=0
until curl -sS --fail "$AI_URL" >/dev/null 2>&1 || [ $i -ge $RETRIES ]; do
  echo "Waiting for AI service... ($i/$RETRIES)"
  sleep 2
  i=$((i+1))
done
if [ $i -ge $RETRIES ]; then
  echo "AI service did not become healthy in time. See docker-compose logs."
  exit 1
fi
echo "AI service healthy."

echo "All services started. Hardhat PID: $HPID"
echo "To stop: docker-compose down ; kill $HPID"
