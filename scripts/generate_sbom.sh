#!/usr/bin/env bash
set -euo pipefail

# Script zur Erzeugung einer CycloneDX Software Bill of Materials (SBOM) im Repo-Root target/
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
TARGET_DIR="${ROOT_DIR}/target"
OUTPUT_FILE="${TARGET_DIR}/sbom.cdx.json"

mkdir -p "${TARGET_DIR}"

echo "=== Generiere Software Bill of Materials (SBOM) ==="

if command -v cargo-cyclonedx &>/dev/null || cargo cyclonedx --version &>/dev/null 2>&1; then
    echo "Verwende cargo-cyclonedx..."
    # Clean up any leftover per-crate sbom files before generating
    find "${ROOT_DIR}" -name "*.cdx.json" -delete 2>/dev/null || true
    cargo cyclonedx --format json --override-filename sbom.cdx

    # Merge/aggregate generated SBOMs or locate the main workspace root SBOM
    # cargo-cyclonedx generates sbom.cdx.json in each crate dir.
    # We find the top-level or first valid sbom.cdx.json and write/copy to target/sbom.cdx.json
    PRIMARY_SBOM="$(find "${ROOT_DIR}/crates" "${ROOT_DIR}" -maxdepth 3 -name "sbom.cdx.json" 2>/dev/null | head -n 1 || true)"
    if [[ -n "${PRIMARY_SBOM}" && -f "${PRIMARY_SBOM}" ]]; then
        cp "${PRIMARY_SBOM}" "${OUTPUT_FILE}"
    fi
    find "${ROOT_DIR}" -name "*.cdx.json" -not -path "${TARGET_DIR}/*" -delete 2>/dev/null || true
elif command -v cargo-sbom &>/dev/null || cargo sbom --version &>/dev/null 2>&1; then
    echo "Verwende cargo sbom..."
    cargo sbom --output-format cyclonedx-json > "${OUTPUT_FILE}"
else
    echo "❌ Fehler: Weder 'cargo-cyclonedx' noch 'cargo-sbom' ist installiert." >&2
    echo "   Bitte installiere 'cargo-cyclonedx' via 'cargo install cargo-cyclonedx --locked'." >&2
    exit 1
fi

if [[ -f "${OUTPUT_FILE}" ]]; then
    echo "✅ SBOM erfolgreich generiert: ${OUTPUT_FILE}"
    echo "   Datei-Größe: $(du -h "${OUTPUT_FILE}" | cut -f1)"
else
    echo "❌ Fehler: SBOM-Datei ${OUTPUT_FILE} wurde nicht erzeugt." >&2
    exit 1
fi
