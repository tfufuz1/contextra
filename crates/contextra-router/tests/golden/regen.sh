#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/../../.."
cargo test -p contextra-router --test golden_corpus_test -- --ignored test_generate_golden_corpus --nocapture
cd crates/contextra-router/tests/golden
sha256sum arm_registry.json routing_decisions.json profiles_and_outcomes.json dispatch_messages.json > MANIFEST.sha256
echo "Golden corpus regenerated and MANIFEST.sha256 updated."
