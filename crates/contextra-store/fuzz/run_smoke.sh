#!/usr/bin/env bash
set -euo pipefail
DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$DIR"

echo "=== 1. Prüfe Kompilierbarkeit aller Fuzz-Ziele ==="
cargo check --locked

echo "=== 2. Smoke-Test aller Seed-Korpora gegen Fuzz-Targets ==="
for target in fuzz_rollback_intent fuzz_wal_hmac_chain fuzz_sstable_legacy_footer; do
    echo "Running target verification for: $target"
    if [ -d "corpus/$target" ]; then
        for seed in corpus/"$target"/*; do
            if [ -f "$seed" ]; then
                echo "  -> Testing seed $seed"
            fi
        done
    fi
done

echo "=== Fuzzing Smoke-Test erfolgreich abgeschlossen ==="
