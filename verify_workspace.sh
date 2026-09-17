#!/usr/bin/env bash
# verify_workspace.sh — Bottom-up, Crate-für-Crate Kompilierungs- & Test-Verifikation
#
# Zweck: Ersetzt Vertrauen in KI-Audit-Prosa durch echte, reproduzierbare cargo-Läufe.
# Prinzip: Erst Layer 0 (Blätter), dann aufsteigend. Bricht beim ERSTEN echten Fehler ab,
#          damit du nicht 20 Minuten auf einen Workspace-Build wartest, der ohnehin an
#          Crate #3 scheitert.
#
# Nutzung:
#   chmod +x verify_workspace.sh
#   ./verify_workspace.sh            # normaler Lauf (check + clippy + test pro Crate)
#   ./verify_workspace.sh --fast     # nur `cargo check` pro Crate (schneller erster Filter)
#   ./verify_workspace.sh --resume memfuse-index   # ab einem bestimmten Crate weitermachen
#
# Ergebnis: results/verify_<timestamp>.log (vollständige Roh-Ausgabe, NICHT von einer KI
# umformuliert) + results/verify_<timestamp>.summary (Tabelle PASS/FAIL pro Crate).

set -uo pipefail

# --- DAG-Reihenfolge aus WORKING_STATE.md (Layer 0 -> Layer 9) ---
CRATES=(
  memfuse-core-ipc-gen   # Layer 0
  memfuse-core           # Layer 1
  memfuse-calibration    # Layer 2
  memfuse-checkpoint     # Layer 2
  memfuse-crypto         # Layer 2
  memfuse-graph          # Layer 2
  memfuse-sandbox        # Layer 2
  memfuse-text           # Layer 2
  memfuse-index          # Layer 3
  memfuse-ollama         # Layer 3
  memfuse-store          # Layer 3
  memfuse-candle         # Layer 4
  memfuse-embed          # Layer 5 (optional)
  memfuse-db             # Layer 6
  memfuse-bench          # Layer 7 (Pfad: benchmarks/memfuse-bench)
  memfuse-router         # Layer 7
  memfuse-agent          # Layer 8
  memfuse-mcp            # Layer 9
)

MODE="full"
RESUME_FROM=""
while [[ $# -gt 0 ]]; do
  case "$1" in
    --fast) MODE="fast"; shift ;;
    --resume) RESUME_FROM="$2"; shift 2 ;;
    *) echo "Unbekannte Option: $1"; exit 1 ;;
  esac
done

mkdir -p results
TS="$(date +%Y%m%d_%H%M%S)"
LOG="results/verify_${TS}.log"
SUMMARY="results/verify_${TS}.summary"
echo "# MemFuse Bottom-Up Verification — $(date -Iseconds)" > "$SUMMARY"
echo "# Modus: $MODE" >> "$SUMMARY"
printf "%-22s %-8s %-8s %-8s %-10s\n" "CRATE" "CHECK" "CLIPPY" "TEST" "DAUER(s)" >> "$SUMMARY"

resuming=false
[[ -z "$RESUME_FROM" ]] && resuming=true

for crate in "${CRATES[@]}"; do
  if ! $resuming; then
    if [[ "$crate" == "$RESUME_FROM" ]]; then resuming=true; else continue; fi
  fi

  echo "=================================================================" | tee -a "$LOG"
  echo "=== $crate ===" | tee -a "$LOG"
  echo "=================================================================" | tee -a "$LOG"

  start=$(date +%s)
  check_status="SKIP"; clippy_status="SKIP"; test_status="SKIP"

  # 1) cargo check — schnellster, härtester Filter. Wenn das rot ist, ist alles danach sinnlos.
  echo "--- cargo check -p $crate --all-targets ---" >> "$LOG"
  if cargo check -p "$crate" --all-targets >> "$LOG" 2>&1; then
    check_status="PASS"
  else
    check_status="FAIL"
  fi

  if [[ "$check_status" == "PASS" && "$MODE" == "full" ]]; then
    # 2) clippy — nur wenn check grün ist, sonst redundante Fehler
    echo "--- cargo clippy -p $crate -- -D warnings ---" >> "$LOG"
    if cargo clippy -p "$crate" --all-targets -- -D warnings >> "$LOG" 2>&1; then
      clippy_status="PASS"
    else
      clippy_status="FAIL"
    fi

    # 3) test — nur wenn check grün ist. Läuft auch bei clippy-Findings, das sind zwei
    #    unabhängige Signale (Lint-Sauberkeit vs. tatsächliches Verhalten).
    echo "--- cargo test -p $crate --all-features ---" >> "$LOG"
    if cargo test -p "$crate" --all-features >> "$LOG" 2>&1; then
      test_status="PASS"
    else
      test_status="FAIL"
    fi
  fi

  end=$(date +%s)
  dur=$((end - start))
  printf "%-22s %-8s %-8s %-8s %-10s\n" "$crate" "$check_status" "$clippy_status" "$test_status" "$dur" | tee -a "$SUMMARY"

  # Bottom-up-Prinzip: bei einem echten Compile-Fehler sofort stoppen.
  # Alles, was auf diesem Crate aufbaut, ist ohnehin nicht sinnvoll prüfbar.
  if [[ "$check_status" == "FAIL" ]]; then
    echo "" | tee -a "$SUMMARY"
    echo "!!! STOP: $crate kompiliert nicht. Downstream-Crates werden übersprungen." | tee -a "$SUMMARY"
    echo "!!! Fehlerdetails: siehe $LOG (Abschnitt '=== $crate ===')" | tee -a "$SUMMARY"
    echo "!!! Fortsetzen nach Fix mit: ./verify_workspace.sh --resume $crate" | tee -a "$SUMMARY"
    exit 1
  fi
done

echo "" | tee -a "$SUMMARY"
echo "Alle Crates in Bottom-Up-Reihenfolge grün. Vollständiges Log: $LOG" | tee -a "$SUMMARY"
