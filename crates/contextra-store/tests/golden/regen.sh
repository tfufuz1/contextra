#!/usr/bin/env bash
set -euo pipefail
DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$DIR/../../../" && pwd)"
cd "$REPO_ROOT"
cargo test -p contextra-store --test golden_corpus -- --ignored --nocapture
