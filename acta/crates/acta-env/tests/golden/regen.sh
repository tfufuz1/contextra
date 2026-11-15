#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
echo "Regenerating golden files for acta-env in $SCRIPT_DIR..."
cargo test -p acta-ports --test generate_golden_env -- --ignored --nocapture

cd "$SCRIPT_DIR"
sha256sum *.json > MANIFEST.sha256 2>/dev/null || true
echo "Manifest updated."
