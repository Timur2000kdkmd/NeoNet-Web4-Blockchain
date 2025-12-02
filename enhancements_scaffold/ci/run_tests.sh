#!/usr/bin/env bash
set -e
echo "Building PQC CI container and running tests (this will build liboqs inside container)..."
docker build -t neonet-pqc-ci -f enhancements_scaffold/ci/pqc_test.Dockerfile .
docker run --rm -v $(pwd):/workspace neonet-pqc-ci
