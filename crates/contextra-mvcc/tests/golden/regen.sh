#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
echo "Regenerating golden files for contextra-mvcc in $SCRIPT_DIR..."
cargo test -p contextra-mvcc --test golden_corpus generate_golden_mvcc -- --ignored --nocapture

cd "$SCRIPT_DIR"
sha256sum mvcc_golden.json mvcc_golden.bin > MANIFEST.sha256
echo "Manifest updated:"
cat MANIFEST.sha256
