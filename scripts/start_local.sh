#!/usr/bin/env bash
set -e
echo "Starting local services via docker-compose"
pushd infra
docker-compose up -d
popd
echo "Services started. Visit http://localhost:8000 for AI service."
