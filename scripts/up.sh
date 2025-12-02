#!/usr/bin/env bash
set -e
pushd infra
docker-compose up -d --build
popd
echo "Stack starting. Allow containers a few seconds to boot. AI service: http://localhost:8000"
