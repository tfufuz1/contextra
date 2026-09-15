#!/usr/bin/env bash
set -euo pipefail

# BEIR Dataset Downloader for MemFuse Benchmarks
# APM-3 Compliance: Supports --ci-only to skip download in CI environments

if [[ "${1:-}" == "--ci-only" ]]; then
    echo "⚡ CI-Mode detected (--ci-only): Skipping BEIR dataset download."
    exit 0
fi

DATASET="nfcorpus"
DATA_DIR="benchmarks/data"
mkdir -p "$DATA_DIR"

URL="https://public.ukp.informatik.tu-darmstadt.de/thakur/BEIR/datasets/${DATASET}.zip"
TARGET_ZIP="${DATA_DIR}/${DATASET}.zip"

if [ ! -d "${DATA_DIR}/${DATASET}" ]; then
    echo "📥 Downloading BEIR dataset ${DATASET} from ${URL}..."
    wget -nc "$URL" -O "$TARGET_ZIP" || curl -L "$URL" -o "$TARGET_ZIP"
    unzip -n "$TARGET_ZIP" -d "$DATA_DIR/"
    echo "✅ BEIR ${DATASET} ready in ${DATA_DIR}/${DATASET}/"
else
    echo "✅ BEIR ${DATASET} already exists in ${DATA_DIR}/${DATASET}/"
fi
