#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
CRATE_DIR="$(cd "${SCRIPT_DIR}/../.." && pwd)"

echo "Regenerating golden corpus for acta-crypto..."
cargo test -p acta-crypto --test golden_corpus -- --ignored generate_golden_files

echo "Golden files regenerated."
