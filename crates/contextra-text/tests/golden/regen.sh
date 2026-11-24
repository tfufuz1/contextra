#!/usr/bin/env bash
set -euo pipefail
cargo test -p contextra-text --test golden_oracle_tests -- --ignored generate_golden_corpus
cd "$(dirname "$0")"
sha256sum *.* | grep -v "MANIFEST.sha256" | grep -v "regen.sh" > MANIFEST.sha256
