#!/usr/bin/env bash
set -euo pipefail
DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cargo test -p contextra-license --test golden_generator -- --ignored
cd "$DIR"
sha256sum license_payload.bin license_signature.bin verifying_key.bin gate_decisions.csv > MANIFEST.sha256
echo "License golden files regenerated successfully."
