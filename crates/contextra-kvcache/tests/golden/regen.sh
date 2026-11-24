#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
echo "Regenerating golden files for contextra-kvcache in $SCRIPT_DIR..."
cargo test -p contextra-kvcache --features kivi-quantization --test generate_golden_kvcache -- --ignored --nocapture

cd "$SCRIPT_DIR"
sha256sum *.json *.bin > MANIFEST.sha256 2>/dev/null || true
echo "Manifest updated."
