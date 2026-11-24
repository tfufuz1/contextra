#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/../.."
cargo test -p contextra-graph --test golden_generator -- --ignored --nocapture

cd tests/golden
sha256sum entity_id.json doc_id.json tx_id.json entity_id.bin doc_id.bin tx_id.bin ppr_results.json traversal_results.json > MANIFEST.sha256
echo "Updated MANIFEST.sha256"
