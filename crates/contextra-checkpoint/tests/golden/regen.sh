#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

echo "Regenerating golden corpus for contextra-checkpoint..."
cargo test -p contextra-checkpoint --test golden_corpus_test -- --ignored regen_golden_corpus

echo "Recomputing sha256 checksums..."
cd "${SCRIPT_DIR}"
sha256sum meta_golden.json manifest_golden.json hardlink_clone_golden.json orphan_state_golden.json > MANIFEST.sha256

echo "Golden corpus regenerated successfully."
