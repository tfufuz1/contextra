#!/usr/bin/env bash
set -euo pipefail
DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cargo test -p contextra-audit-export --test golden_generator -- --ignored
cd "$DIR"
sha256sum golden_register.json golden_register.md bsi_mapping.md bsi_mapping.json > MANIFEST.sha256
echo "Audit export golden files regenerated successfully."
