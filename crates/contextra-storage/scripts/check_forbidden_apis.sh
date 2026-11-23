#!/usr/bin/env bash
# Verbotene APIs im Kern (SP-ENV Regel 1, INDEX §4). Ausgenommen: testing-Modul und contextra-storage-fs.
set -euo pipefail
cd "$(dirname "$0")/.."
PATTERN='std::fs::|SystemTime|Instant::now|rand::|\.unwrap\(\)|\.expect\(|panic!\(|unreachable!\('
hits=$(grep -rnE "$PATTERN" crates/contextra-storage/src crates/contextra-storage-ports/src --include='*.rs' | grep -v '/testing/' | grep -vE ':[0-9]+:[[:space:]]*//' || true)
if [ -n "$hits" ]; then echo "$hits"; echo "FEHLER: verbotene APIs gefunden"; exit 1; fi
echo "OK: keine verbotenen APIs"
