#!/usr/bin/env bash
set -euo pipefail
DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cargo test -p contextra-infer-ollama --test golden_test -- --ignored regen_golden
cd "$DIR"
sha256sum ollama_golden.json > MANIFEST.sha256
echo "Ollama golden regenerated."
