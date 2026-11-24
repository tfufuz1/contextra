#!/usr/bin/env bash
# Canary-Suche über Dateien/Checkpoints (INV-KEY-1).
set -euo pipefail
cd "$(dirname "$0")/.."

CANARY_PATTERN="SECRET_CANARY_[0-9A-Za-z_]+"
SEARCH_DIR="${1:-.}"

# Exclude git, target, and source files containing definitions or tests of canaries
MATCHES=$(grep -rnE "$CANARY_PATTERN" "$SEARCH_DIR" \
    --exclude-dir=".git" \
    --exclude-dir="target" \
    --exclude-dir="scratch" \
    --exclude="*.rs" \
    --exclude="*.md" \
    --exclude="*.sh" \
    --exclude="*.txt" || true)

if [ -n "$MATCHES" ]; then
    echo "FEHLER: Canary-Schlüssel in Klartext-Artefakten gefunden:" >&2
    echo "$MATCHES" >&2
    exit 1
fi

echo "OK: kein Canary-Schlüssel in persistierten Dateien gefunden"
