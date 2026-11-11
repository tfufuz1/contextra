#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
CRATE_DIR="$(cd "$SCRIPT_DIR/../.." && pwd)"

echo "Regenerating golden files for contextra-vector..."
cd "$CRATE_DIR"

cargo test --test golden_test --features experimental-diskann -- --ignored generate_golden_files --exact --nocapture

cd "$SCRIPT_DIR"
sha256sum *.bin *.json *.dann *.wal > MANIFEST.sha256
echo "Regeneration complete. MANIFEST.sha256 updated."
