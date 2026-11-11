#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
echo "Regenerating golden files for contextra-erasure in $SCRIPT_DIR..."
cargo test -p contextra-erasure --test golden_erasure_test -- generate_golden_files --ignored --nocapture

cd "$SCRIPT_DIR"
sha256sum *.json > MANIFEST.sha256 2>/dev/null || true
echo "Manifest updated."
