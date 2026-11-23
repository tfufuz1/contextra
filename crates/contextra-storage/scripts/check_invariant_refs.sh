#!/usr/bin/env bash
# Zitierprüfung (INDEX §4 Regel 3). Platzhalter: listet INV-Referenzen; Abgleich gegen SPEC.md §15 folgt (WP-15.x).
set -euo pipefail
cd "$(dirname "$0")/.."
grep -rhoE 'INV-[A-Z]+-[0-9]+' crates tests 2>/dev/null | sort -u || true
echo "INFO: Abgleich gegen SPEC.md §15 noch nicht implementiert"
