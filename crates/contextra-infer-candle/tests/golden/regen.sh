#!/usr/bin/env bash
set -euo pipefail
cargo test -p contextra-infer-candle --test golden_test -- --ignored regen_golden_candle --nocapture
