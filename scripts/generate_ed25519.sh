#!/usr/bin/env bash
# generate_ed25519.sh
# Usage: ./generate_ed25519.sh <outname>
OUT=${1:-ed25519_demo}
mkdir -p secrets/ed25519
openssl genpkey -algorithm ED25519 -out secrets/ed25519/${OUT}_priv.pem
openssl pkey -in secrets/ed25519/${OUT}_priv.pem -pubout -out secrets/ed25519/${OUT}_pub.pem
echo "Generated secrets/ed25519/${OUT}_priv.pem and ${OUT}_pub.pem"
