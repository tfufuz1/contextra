#!/usr/bin/env bash
set -euo pipefail
cd "."
sha256sum *.json > MANIFEST.sha256
