#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
echo "Regenerating golden files for contextra-ids in $SCRIPT_DIR..."
cargo test -p contextra-types --test generate_golden_ids -- --ignored --nocapture
cargo test -p contextra-types --features docid-128 --test generate_golden_ids -- --ignored --nocapture

cd "$SCRIPT_DIR"
sha256sum *.json > MANIFEST.sha256 2>/dev/null || true
echo "Manifest updated."
