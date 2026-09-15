#!/usr/bin/env bash
set -euo pipefail

# ANN-Benchmarks SIFT1M Dataset Downloader
# APM-3 Compliance: Supports --ci-only to skip download in CI environments

if [[ "${1:-}" == "--ci-only" ]]; then
    echo "⚡ CI-Mode detected (--ci-only): Using synthetic ANN benchmark mode."
    exit 0
fi

DATA_DIR="benchmarks/data"
mkdir -p "$DATA_DIR"

URL="http://ann-benchmarks.com/sift-128-euclidean.hdf5"
TARGET_FILE="${DATA_DIR}/sift-128-euclidean.hdf5"

if [ ! -f "$TARGET_FILE" ]; then
    echo "📥 Downloading SIFT1M dataset from ${URL}..."
    wget -nc "$URL" -O "$TARGET_FILE" || curl -L "$URL" -o "$TARGET_FILE"
    echo "✅ SIFT1M dataset ready at ${TARGET_FILE}"
else
    echo "✅ SIFT1M dataset already exists at ${TARGET_FILE}"
fi
