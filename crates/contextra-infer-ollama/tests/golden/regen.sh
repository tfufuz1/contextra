#!/usr/bin/env bash
set -euo pipefail
cargo test -p contextra-infer-ollama --test golden_test -- --ignored regen_golden_ollama --nocapture
