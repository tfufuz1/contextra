#!/usr/bin/env bash
# forensic_test.sh — Granularer, beweispflichtiger Cargo-Test-Runner für memfuse.
#
# ZWECK (siehe PROMPTER_OPTIMIERUNG_TAGESEINSATZ.md / PROOF_OF_WORK_BLOCK-Doktrin):
#   "Der Code sieht korrekt aus" ist kein Nachweis. Dieses Skript erzeugt für JEDEN
#   geprüften Crate eine unveränderliche, zeitgestempelte Rohbeweis-Datei (Log + Exit-Code)
#   UND eine maschinenlesbare report.json — damit weder ein Mensch noch ein Agent (Jules)
#   "Tests laufen grün" behaupten kann, ohne dass ein Log existiert, das das belegt.
#
# NUTZUNG:
#   ./scripts/forensic_test.sh                      # alle Crates, Standard-Tiefe
#   ./scripts/forensic_test.sh --changed             # nur Crates mit uncommitted/staged Diff
#   ./scripts/forensic_test.sh -p memfuse-store       # ein einzelner Crate
#   ./scripts/forensic_test.sh -p memfuse-store --triple   # 3x hintereinander (Flakiness/Race-Nachweis)
#   ./scripts/forensic_test.sh --fast                # nur check+clippy, keine Tests (Pre-Commit-Tempo)
#   ./scripts/forensic_test.sh --strict-tools        # bricht ab statt zu skippen, wenn ein Tool fehlt
#
# EXIT-CODE: 0 nur wenn ALLE geprüften Crates in ALLEN Phasen bestehen. Jeder andere Code
# bedeutet: mindestens ein Beweis in report.json/*.log zeigt einen Fehlschlag.
#
# Verwendbar identisch von Jules (im Sandbox-Shell) und lokal vom Menschen.

set -uo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

TS="$(date -u +%Y%m%dT%H%M%SZ)"
EVIDENCE_DIR="target/forensic/${TS}"
mkdir -p "$EVIDENCE_DIR"
LATEST_LINK="target/forensic/latest"
rm -f "$LATEST_LINK" 2>/dev/null || true
ln -s "$TS" "$LATEST_LINK" 2>/dev/null || true

REPORT_JSON="${EVIDENCE_DIR}/report.json"
SUMMARY_TXT="${EVIDENCE_DIR}/SUMMARY.txt"

# ── CLI-Argumente ────────────────────────────────────────────────────────────
ONLY_CHANGED=0
SINGLE_CRATE=""
TRIPLE=0
FAST=0
STRICT_TOOLS=0
INCLUDE_IGNORED=0

while [[ $# -gt 0 ]]; do
  case "$1" in
    --changed) ONLY_CHANGED=1; shift ;;
    -p|--package) SINGLE_CRATE="$2"; shift 2 ;;
    --triple) TRIPLE=1; shift ;;
    --fast) FAST=1; shift ;;
    --strict-tools) STRICT_TOOLS=1; shift ;;
    --include-ignored) INCLUDE_IGNORED=1; shift ;;
    -h|--help) grep '^#' "$0" | sed 's/^# \{0,1\}//'; exit 0 ;;
    *) echo "Unbekanntes Argument: $1" >&2; exit 2 ;;
  esac
done

# ── Runner-Präfix (nix falls verfügbar, sonst nackter Aufruf) — wie justfile ──
RUNNER=""
if command -v nix >/dev/null 2>&1 && nix develop -c true >/dev/null 2>&1; then
  RUNNER="nix develop -c"
fi

# ── Tool-Erkennung (TOOLING-HARD-FAIL-Doktrin: SKIPPED explizit machen,
#    niemals stillschweigend als "läuft schon" durchwinken) ──────────────────
HAVE_NEXTEST=0
command -v cargo-nextest >/dev/null 2>&1 && HAVE_NEXTEST=1
$RUNNER cargo nextest --version >/dev/null 2>&1 && HAVE_NEXTEST=1

TEST_RUNNER_NAME="cargo test"
if [[ $HAVE_NEXTEST -eq 1 ]]; then
  TEST_RUNNER_NAME="cargo nextest"
fi

MISSING_TOOLS=()
for tool_check in "cargo fmt --version" "cargo clippy --version"; do
  if ! $RUNNER cargo ${tool_check#cargo } >/dev/null 2>&1; then
    MISSING_TOOLS+=("${tool_check}")
  fi
done
if [[ ${#MISSING_TOOLS[@]} -gt 0 ]]; then
  echo "⚠️  Fehlende Basis-Tools: ${MISSING_TOOLS[*]}" | tee -a "$SUMMARY_TXT"
  if [[ $STRICT_TOOLS -eq 1 ]]; then
    echo "❌ --strict-tools aktiv: breche ab statt zu skippen." | tee -a "$SUMMARY_TXT"
    exit 3
  fi
fi

# ── Crate-Liste aus Cargo.toml workspace.members extrahieren ─────────────────
mapfile -t ALL_CRATE_PATHS < <(python3 - <<'PY'
import re
with open("Cargo.toml") as f:
    txt = f.read()
m = re.search(r"members\s*=\s*\[(.*?)\]", txt, re.S)
members = re.findall(r'"([^"]+)"', m.group(1)) if m else []
for mem in members:
    if mem.startswith("crates/") or mem.startswith("benchmarks/"):
        print(mem)
PY
)

if [[ -n "$SINGLE_CRATE" ]]; then
  TARGET_PATHS=()
  for p in "${ALL_CRATE_PATHS[@]}"; do
    [[ "$(basename "$p")" == "$SINGLE_CRATE" ]] && TARGET_PATHS+=("$p")
  done
  if [[ ${#TARGET_PATHS[@]} -eq 0 ]]; then
    echo "❌ Crate '$SINGLE_CRATE' nicht in workspace.members gefunden." >&2
    exit 2
  fi
elif [[ $ONLY_CHANGED -eq 1 ]]; then
  mapfile -t CHANGED_FILES < <(git diff --name-only HEAD; git diff --cached --name-only; git status --porcelain | awk '{print $2}')
  TARGET_PATHS=()
  for p in "${ALL_CRATE_PATHS[@]}"; do
    for f in "${CHANGED_FILES[@]}"; do
      if [[ "$f" == "$p"/* ]]; then
        TARGET_PATHS+=("$p")
        break
      fi
    done
  done
  # Dedupe
  mapfile -t TARGET_PATHS < <(printf '%s\n' "${TARGET_PATHS[@]}" | sort -u)
  if [[ ${#TARGET_PATHS[@]} -eq 0 ]]; then
    echo "ℹ️  --changed: keine Crate-Dateien im Diff gefunden. Nichts zu tun." | tee -a "$SUMMARY_TXT"
    echo '{"generated_at":"'"$TS"'","mode":"changed","crates":[],"all_passed":true}' > "$REPORT_JSON"
    exit 0
  fi
else
  TARGET_PATHS=("${ALL_CRATE_PATHS[@]}")
fi

echo "=== forensic_test.sh — ${TS} ===" | tee "$SUMMARY_TXT"
echo "Runner: ${RUNNER:-<nativ>} | Test-Backend: ${TEST_RUNNER_NAME} | Crates: ${#TARGET_PATHS[@]} | Modus: $( [[ $FAST -eq 1 ]] && echo fast || echo full )$( [[ $TRIPLE -eq 1 ]] && echo ' + triple' )" | tee -a "$SUMMARY_TXT"
echo "Evidenz-Verzeichnis: ${EVIDENCE_DIR}" | tee -a "$SUMMARY_TXT"
echo "" | tee -a "$SUMMARY_TXT"

# git-Zustand mit protokollieren — jede Behauptung "das war der geprüfte Stand"
# muss gegen HEAD + Diff verifizierbar sein (Anti-Hallucination-Anker).
{
  echo "--- GIT-ZUSTAND ZUM ZEITPUNKT DIESES LAUFS ---"
  echo "HEAD: $(git rev-parse HEAD 2>/dev/null || echo unbekannt)"
  echo "Branch: $(git rev-parse --abbrev-ref HEAD 2>/dev/null || echo unbekannt)"
  echo "Dirty-Diff (uncommitted):"
  git status --porcelain 2>/dev/null || echo "  (kein Git-Repo erkennbar)"
} > "${EVIDENCE_DIR}/GIT_STATE.txt"

declare -A RESULTS   # crate -> "pass"|"fail"|"skip"
declare -A DETAILS   # crate -> Kurzbeschreibung
ALL_PASSED=1

json_crates_entries=()

for crate_path in "${TARGET_PATHS[@]}"; do
  crate_name="$(basename "$crate_path")"
  clog_dir="${EVIDENCE_DIR}/${crate_name}"
  mkdir -p "$clog_dir"
  echo "─────────────────────────────────────────────" | tee -a "$SUMMARY_TXT"
  echo "▶ ${crate_name}  (${crate_path})" | tee -a "$SUMMARY_TXT"

  crate_pass=1
  phase_results=()

  # Phase A: cargo check (schnell, fängt Kompilationsfehler isoliert vom Rest)
  echo "  [A] cargo check -p ${crate_name}" | tee -a "$SUMMARY_TXT"
  if $RUNNER cargo check -p "$crate_name" --all-targets > "${clog_dir}/01_check.log" 2>&1; then
    echo "      ✅ check OK" | tee -a "$SUMMARY_TXT"
    phase_results+=('"check":"pass"')
  else
    echo "      ❌ check FAILED — Details: ${clog_dir}/01_check.log" | tee -a "$SUMMARY_TXT"
    tail -n 20 "${clog_dir}/01_check.log" | sed 's/^/        | /' | tee -a "$SUMMARY_TXT"
    phase_results+=('"check":"fail"')
    crate_pass=0
  fi

  # Phase B: cargo clippy -D warnings (nur wenn check bestanden hat — sonst redundantes Rauschen)
  if [[ $crate_pass -eq 1 ]]; then
    echo "  [B] cargo clippy -p ${crate_name} -- -D warnings" | tee -a "$SUMMARY_TXT"
    if $RUNNER cargo clippy -p "$crate_name" --all-targets -- -D warnings > "${clog_dir}/02_clippy.log" 2>&1; then
      echo "      ✅ clippy OK" | tee -a "$SUMMARY_TXT"
      phase_results+=('"clippy":"pass"')
    else
      echo "      ❌ clippy FAILED — Details: ${clog_dir}/02_clippy.log" | tee -a "$SUMMARY_TXT"
      tail -n 20 "${clog_dir}/02_clippy.log" | sed 's/^/        | /' | tee -a "$SUMMARY_TXT"
      phase_results+=('"clippy":"fail"')
      crate_pass=0
    fi
  else
    phase_results+=('"clippy":"skip"')
  fi

  if [[ $FAST -eq 1 ]]; then
    RESULTS[$crate_name]=$([[ $crate_pass -eq 1 ]] && echo pass || echo fail)
    [[ $crate_pass -eq 0 ]] && ALL_PASSED=0
    json_crates_entries+=("{\"name\":\"${crate_name}\",\"mode\":\"fast\",${phase_results[0]},${phase_results[1]},\"result\":\"${RESULTS[$crate_name]}\"}")
    continue
  fi

  # Phase C: Testlauf (nextest bevorzugt — striktere Isolation je Testbinary,
  # sauberer strukturierter Output). Fällt zurück auf `cargo test`.
  runs=1
  [[ $TRIPLE -eq 1 ]] && runs=3
  test_ok_all_runs=1
  test_summaries=()

  if [[ $crate_pass -eq 1 ]]; then
    for run_i in $(seq 1 "$runs"); do
      tlog="${clog_dir}/03_test_run${run_i}.log"
      echo "  [C] Testlauf ${run_i}/${runs} (${TEST_RUNNER_NAME}) -p ${crate_name}" | tee -a "$SUMMARY_TXT"
      ignored_flag=""
      [[ $INCLUDE_IGNORED -eq 1 ]] && ignored_flag="--run-ignored=all"

      if [[ $HAVE_NEXTEST -eq 1 ]]; then
        if [[ $INCLUDE_IGNORED -eq 1 ]]; then
          $RUNNER cargo nextest run -p "$crate_name" --run-ignored all > "$tlog" 2>&1
        else
          $RUNNER cargo nextest run -p "$crate_name" > "$tlog" 2>&1
        fi
      else
        if [[ $INCLUDE_IGNORED -eq 1 ]]; then
          $RUNNER cargo test -p "$crate_name" -- --include-ignored > "$tlog" 2>&1
        else
          $RUNNER cargo test -p "$crate_name" > "$tlog" 2>&1
        fi
      fi
      run_status=$?

      # Rohe Zahlen aus dem Log extrahieren statt zu behaupten — funktioniert
      # für sowohl `cargo test`- als auch `cargo nextest`-Ausgabeformate.
      passed_n=$(grep -oE '[0-9]+ passed' "$tlog" | tail -1 | grep -oE '[0-9]+' || echo "?")
      failed_n=$(grep -oE '[0-9]+ failed' "$tlog" | tail -1 | grep -oE '[0-9]+' || echo "0")
      summary_line="Run ${run_i}: exit=${run_status} passed=${passed_n} failed=${failed_n}"
      test_summaries+=("$summary_line")

      if [[ $run_status -eq 0 ]]; then
        echo "      ✅ ${summary_line} — Log: ${tlog}" | tee -a "$SUMMARY_TXT"
      else
        echo "      ❌ ${summary_line} — Log: ${tlog}" | tee -a "$SUMMARY_TXT"
        grep -B2 -A15 -iE 'FAILED|panicked at|thread .* panicked' "$tlog" | head -60 | sed 's/^/        | /' | tee -a "$SUMMARY_TXT"
        test_ok_all_runs=0
      fi
    done
  else
    test_summaries+=("Tests übersprungen — check/clippy zuvor fehlgeschlagen")
    test_ok_all_runs=0
  fi

  printf '%s\n' "${test_summaries[@]}" > "${clog_dir}/TEST_SUMMARY.txt"

  if [[ $crate_pass -eq 1 && $test_ok_all_runs -eq 1 ]]; then
    RESULTS[$crate_name]="pass"
    DETAILS[$crate_name]="${runs}/${runs} Testläufe grün"
  else
    RESULTS[$crate_name]="fail"
    ALL_PASSED=0
    DETAILS[$crate_name]="siehe ${clog_dir}"
  fi

  test_phase_str="\"fail\""
  [[ $crate_pass -eq 1 && $test_ok_all_runs -eq 1 ]] && test_phase_str="\"pass\""
  json_crates_entries+=("{\"name\":\"${crate_name}\",\"mode\":\"full\",${phase_results[0]},${phase_results[1]},\"test\":${test_phase_str},\"test_runs\":${runs},\"result\":\"${RESULTS[$crate_name]}\"}")
done

# ── report.json schreiben (maschinenlesbar, für xtask/CI/Prompter-Import) ────
{
  printf '{\n  "generated_at": "%s",\n  "head": "%s",\n  "mode": "%s",\n  "test_backend": "%s",\n  "all_passed": %s,\n  "crates": [\n' \
    "$TS" "$(git rev-parse --short HEAD 2>/dev/null || echo unknown)" \
    "$( [[ $FAST -eq 1 ]] && echo fast || echo full )" "$TEST_RUNNER_NAME" \
    "$( [[ $ALL_PASSED -eq 1 ]] && echo true || echo false )"
  n=${#json_crates_entries[@]}
  for i in "${!json_crates_entries[@]}"; do
    printf '    %s' "${json_crates_entries[$i]}"
    [[ $i -lt $((n-1)) ]] && printf ','
    printf '\n'
  done
  printf '  ]\n}\n'
} > "$REPORT_JSON"

echo "" | tee -a "$SUMMARY_TXT"
echo "═══════════════════════════════════════════════" | tee -a "$SUMMARY_TXT"
if [[ $ALL_PASSED -eq 1 ]]; then
  echo "✅ FORENSIC-TEST-GATE BESTANDEN (${#TARGET_PATHS[@]} Crate(s))" | tee -a "$SUMMARY_TXT"
else
  echo "❌ FORENSIC-TEST-GATE FEHLGESCHLAGEN — betroffene Crates:" | tee -a "$SUMMARY_TXT"
  for c in "${!RESULTS[@]}"; do
    [[ "${RESULTS[$c]}" == "fail" ]] && echo "   - ${c} (${DETAILS[$c]:-siehe Log})" | tee -a "$SUMMARY_TXT"
  done
fi
echo "Report: ${REPORT_JSON}" | tee -a "$SUMMARY_TXT"
echo "Diesen Report NIEMALS aus dem Gedächtnis zusammenfassen — bei jeder Zusammenfassung" | tee -a "$SUMMARY_TXT"
echo "gegen die Datei zurückverweisen, nicht gegen die eigene Erinnerung an den Lauf." | tee -a "$SUMMARY_TXT"

[[ $ALL_PASSED -eq 1 ]] && exit 0 || exit 1
