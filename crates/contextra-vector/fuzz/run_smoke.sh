#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
CRATE_DIR="$(cd "$SCRIPT_DIR/../.." && pwd)"

echo "Checking fuzz target builds..."
cd "$SCRIPT_DIR"
cargo check --offline

echo "Executing fuzz targets against all corpus entries via driver test..."
cd "$CRATE_DIR"
cargo test --test proof_fuzz_smoke_driver --features experimental-diskann --offline -- --nocapture

echo "Fuzz smoke test successful across all targets and corpus entries."
