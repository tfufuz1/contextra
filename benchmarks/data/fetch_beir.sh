#!/usr/bin/env bash
# ==============================================================================
# Script: fetch_beir.sh
# Purpose: Herunterladen und Extrahieren öffentlicher BEIR-Benchmark-Datensätze
# Lizenz des Skripts: MIT / CC0 (Eigenerstellung Contextra-Projekt)
# Lizenz der Datensätze:
#   - scifact: CC-BY-4.0 (Wadden et al., 2020)
#   - nfcorpus: CC-BY-SA-4.0 (Boteva et al., 2016)
# ==============================================================================

set -euo pipefail

DATASET="${1:-scifact}"
TARGET_DIR="${2:-benchmarks/data/beir}"

mkdir -p "${TARGET_DIR}"

BASE_URL="https://public.ukp.informatik.tu-darmstadt.de/thakur/BEIR/datasets"
ZIP_FILE="${TARGET_DIR}/${DATASET}.zip"
DEST_DIR="${TARGET_DIR}/${DATASET}"

if [ -d "${DEST_DIR}" ]; then
    echo "Datensatz '${DATASET}' existiert bereits unter '${DEST_DIR}'."
    return 0 2>/dev/null || exit 0
fi

echo "Lade BEIR-Datensatz '${DATASET}' herunter von ${BASE_URL}/${DATASET}.zip ..."
curl -fSL -o "${ZIP_FILE}" "${BASE_URL}/${DATASET}.zip"

echo "Extrahiere ${ZIP_FILE} nach ${TARGET_DIR} ..."
unzip -q -o "${ZIP_FILE}" -d "${TARGET_DIR}"

echo "Bereinige temporäres ZIP-Archiv ..."
rm -f "${ZIP_FILE}"

echo "Erfolgreich bereitgestellt: ${DEST_DIR}"
echo "Dateien im Korpus:"
ls -lh "${DEST_DIR}"
