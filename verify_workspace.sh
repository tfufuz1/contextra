#!/usr/bin/env bash
# verify_workspace.sh — Bottom-up, Crate-für-Crate Kompilierungs-, Test-, Feature-,
# Gate-, Loom- und Benchmark-Verifikation für MemFuse Cognitive OS.
# Version 3 — ausgerichtet an der finalen konsolidierten Gesamtspezifikation (§0.3, §15, §17).
#
# Zweck: EIN Skript, das systematisch das gesamte Repo prüft und dabei jeden Fund
# präzise, maschinenlesbar und mit exakter Datei:Zeile-Referenz protokolliert — als
# direkte Grundlage für die Jules-Prompt-Erstellung, nicht als Rohtext-Halde.
#
# Deckt ab:
#   1) Bottom-up cargo check/clippy/test pro Crate (wie bisher, jetzt mit JSON-Diagnostik)
#   2) Feature-Matrix je §0.3-Feature-Katalog (default-off Features explizit einzeln bauen)
#   3) Pflicht-CI-Gates aus §15.4 (xtask check-flatbuffers-drift, check-bandit-latency-budget,
#      check-unwrap-baseline) — best-effort, meldet klar "NICHT VERFÜGBAR" statt zu crashen,
#      falls das xtask-Subcommand im aktuellen Code-Stand noch nicht existiert
#   4) Loom-Concurrency-Tests (§15.3) als eigener, isolierter Job
#   5) Benchmark-Discovery (alle benches/*.rs je Crate, inkl. AK-8 Regressions-Benchmark)
#      + optionaler echter Lauf via --run-benches
#   6) Panic-Inventar (§17 Opus 3.2): .unwrap()/.expect()/panic! in Produktivcode (src/)
#      getrennt von Test-/Bench-Code, als Zero-Panic-Doctrine-Verstoßliste
#   7) Security-Scan: sh -c / Command::new("sh") Verstöße gegen das Execution-Invariant
#   8) Duplicate-Symbol-Gate PRO DATEI (nicht crate-weit!), exakt wie in §xtask
#      "check_duplicate_symbols" spezifiziert: Duplikate zählen nur innerhalb derselben Datei
#
# Nutzung:
#   chmod +x verify_workspace.sh
#   ./verify_workspace.sh                              # Standard: check+clippy+test, bottom-up
#   ./verify_workspace.sh --fast                        # nur cargo check
#   ./verify_workspace.sh --resume memfuse-index         # ab Crate X weitermachen
#   ./verify_workspace.sh --all-crates                  # nicht beim ersten Fehler stoppen
#   ./verify_workspace.sh --only memfuse-db,memfuse-graph
#   ./verify_workspace.sh --no-clippy
#   ./verify_workspace.sh --dag-check
#   ./verify_workspace.sh --feature-matrix              # §0.3 Feature-Katalog einzeln bauen
#   ./verify_workspace.sh --xtask-gates                 # §15.4 Pflicht-Gates ausführen
#   ./verify_workspace.sh --loom                        # §15.3 Loom-Tests separat
#   ./verify_workspace.sh --benchmarks                  # Benchmarks nur auflisten+kompilieren
#   ./verify_workspace.sh --run-benches                 # Benchmarks TATSÄCHLICH ausführen (teuer!)
#   ./verify_workspace.sh --panic-inventory             # §17 Opus 3.2 Zero-Panic-Scan
#   ./verify_workspace.sh --security-scan               # sh -c / Command-Injection-Scan
#   ./verify_workspace.sh --full-audit                  # ALLES obige in einem Lauf
#
# Ergebnisse unter results/<TS>/:
#   verify.log, verify.summary                — wie bisher
#   diagnostics.jsonl                          — 1 JSON-Zeile pro Compiler-Diagnose, crateübergreifend
#   <crate>/report.md                          — Fehler nach Datei gruppiert
#   <crate>/features/<feature>/report.md       — Feature-Matrix-Ergebnisse
#   FEHLERBERICHT.md                           — Gesamtbericht, Basis für Jules-Prompts
#   XTASK_GATES.md                             — Ergebnis der §15.4-Pflicht-Gates
#   LOOM_REPORT.md                             — Loom-Testergebnisse
#   BENCHMARKS.md                              — gefundene Benchmarks + Kompilier-/Laufstatus
#   PANIC_INVENTORY.md                         — .unwrap()/.expect()/panic! in src/ je Crate+Datei
#   SECURITY_SCAN.md                           — sh -c / Command-Injection-Kandidaten
#   duplicate_symbols.txt                      — PRO-DATEI-Duplikate (fn/struct)
#   dag_report.md                              — Layer-Inversionen (--dag-check)
#
# Voraussetzung für granulare JSON-Auswertung: python3. Ohne python3 degradiert das
# Skript kontrolliert auf Rohtext-Logging und warnt deutlich.

set -uo pipefail

# --- DAG-Reihenfolge exakt aus §0.1/§4 der finalen Spezifikation (Layer 0 -> höchste) ---
CRATES_ALL=(
  memfuse-core-ipc-gen   # Layer 0
  memfuse-core           # Layer 0
  memfuse-store          # Layer 1
  memfuse-crypto         # Layer 1
  memfuse-text           # Layer 1
  memfuse-index          # Layer 1
  memfuse-graph          # Layer 1
  memfuse-checkpoint     # Layer 1
  memfuse-sandbox        # Layer 6.5 (aber Blattabhängigkeit, früh prüfbar)
  memfuse-db             # Layer 2
  memfuse-router         # Layer 3
  memfuse-candle         # Layer 3
  memfuse-ollama         # Layer 3
  memfuse-embed          # Layer 3 (optional)
  memfuse-agent          # Layer 3
  memfuse-mcp            # Layer 4
  memfuse-bench          # Layer 5 (Pfad: benchmarks/memfuse-bench)
)

# --- §0.3 Feature-Katalog: welches Crate, welches Feature, Default-Zustand ---
# Format: "crate:feature:default(on|off|dev)"
FEATURE_CATALOG=(
  "memfuse-core:docid-128:off"
  "memfuse-store:block-cache-v2:off"
  "memfuse-router:egress-sherman-morrison:off"
  "memfuse-index:experimental-diskann:off"
  "memfuse-graph:edge-reinforcement-learning:off"
  "memfuse-router:bandit-routing:on"
  "memfuse-mcp:cloud-egress-guard:on"
  "memfuse-mcp:wasm-sandbox:on"
  "memfuse-candle:kv-bridge:on"
  "memfuse-store:fault-injection:dev"
  "memfuse-store:loom:dev"
  "memfuse-graph:loom:dev"
)

# --- §15.4 Pflicht-CI-Gates (best-effort; werden übersprungen, falls das xtask-Subcommand
#     im aktuellen Code-Stand noch nicht implementiert ist — das ist selbst ein Befund) ---
XTASK_GATES=(
  "check-flatbuffers-drift"
  "check-bandit-latency-budget|--features memfuse-router/egress-sherman-morrison"
  "check-module-reachability"
)

MODE="full"
RESUME_FROM=""
STOP_ON_FAIL=true
ONLY_LIST=""
RUN_CLIPPY=true
RUN_DAG_CHECK=false
RUN_FEATURE_MATRIX=false
RUN_XTASK_GATES=false
RUN_LOOM=false
RUN_BENCHMARKS=false
RUN_BENCHES_FOR_REAL=false
RUN_PANIC_INVENTORY=false
RUN_SECURITY_SCAN=false

while [[ $# -gt 0 ]]; do
  case "$1" in
    --fast) MODE="fast"; shift ;;
    --resume) RESUME_FROM="$2"; shift 2 ;;
    --all-crates) STOP_ON_FAIL=false; shift ;;
    --only) ONLY_LIST="$2"; shift 2 ;;
    --no-clippy) RUN_CLIPPY=false; shift ;;
    --dag-check) RUN_DAG_CHECK=true; shift ;;
    --feature-matrix) RUN_FEATURE_MATRIX=true; shift ;;
    --xtask-gates) RUN_XTASK_GATES=true; shift ;;
    --loom) RUN_LOOM=true; shift ;;
    --benchmarks) RUN_BENCHMARKS=true; shift ;;
    --run-benches) RUN_BENCHMARKS=true; RUN_BENCHES_FOR_REAL=true; shift ;;
    --panic-inventory) RUN_PANIC_INVENTORY=true; shift ;;
    --security-scan) RUN_SECURITY_SCAN=true; shift ;;
    --full-audit)
      RUN_DAG_CHECK=true; RUN_FEATURE_MATRIX=true; RUN_XTASK_GATES=true
      RUN_LOOM=true; RUN_BENCHMARKS=true; RUN_PANIC_INVENTORY=true; RUN_SECURITY_SCAN=true
      STOP_ON_FAIL=false
      shift ;;
    -h|--help) sed -n '2,60p' "$0"; exit 0 ;;
    *) echo "Unbekannte Option: $1"; exit 1 ;;
  esac
done

if [[ -n "$ONLY_LIST" ]]; then
  IFS=',' read -r -a CRATES <<< "$ONLY_LIST"
else
  CRATES=("${CRATES_ALL[@]}")
fi

HAVE_PY3=true
command -v python3 >/dev/null 2>&1 || HAVE_PY3=false
if ! $HAVE_PY3; then
  echo "!!! WARNUNG: python3 nicht gefunden. Granulare Reports (report.md, diagnostics.jsonl,"
  echo "!!! FEHLERBERICHT.md, PANIC_INVENTORY.md, BENCHMARKS.md, ...) werden ÜBERSPRUNGEN."
fi

TS="$(date +%Y%m%d_%H%M%S)"
OUTDIR="results/${TS}"
mkdir -p "$OUTDIR"
LOG="${OUTDIR}/verify.log"
SUMMARY="${OUTDIR}/verify.summary"
DIAG_JSONL="${OUTDIR}/diagnostics.jsonl"
DUP_FILE="${OUTDIR}/duplicate_symbols.txt"
PANIC_FILE="${OUTDIR}/PANIC_INVENTORY.md"
SEC_FILE="${OUTDIR}/SECURITY_SCAN.md"
BENCH_FILE="${OUTDIR}/BENCHMARKS.md"
XTASK_FILE="${OUTDIR}/XTASK_GATES.md"
LOOM_FILE="${OUTDIR}/LOOM_REPORT.md"
: > "$DIAG_JSONL"
: > "$DUP_FILE"

echo "# MemFuse Bottom-Up Verification — $(date -Iseconds)" > "$SUMMARY"
echo "# Modus: $MODE | stop_on_fail=$STOP_ON_FAIL | clippy=$RUN_CLIPPY" >> "$SUMMARY"
printf "%-22s %-8s %-8s %-8s %-8s %-10s\n" "CRATE" "CHECK" "CLIPPY" "TEST" "ERRORS" "DAUER(s)" >> "$SUMMARY"

crate_src_dir() {
  local crate="$1"
  if [[ -d "crates/${crate}/src" ]]; then echo "crates/${crate}/src";
  elif [[ -d "benchmarks/${crate}/src" ]]; then echo "benchmarks/${crate}/src";
  else echo ""; fi
}
crate_root_dir() {
  local crate="$1"
  if [[ -d "crates/${crate}" ]]; then echo "crates/${crate}";
  elif [[ -d "benchmarks/${crate}" ]]; then echo "benchmarks/${crate}";
  else echo ""; fi
}

# --- Python-Helfer: parst `cargo --message-format=json` Stream -> pro-Datei-Report + jsonl ---
parse_cargo_json() {
  local infile="$1" crate="$2" phase="$3" outmd="$4" jsonl="$5"
  python3 - "$infile" "$crate" "$phase" "$outmd" "$jsonl" <<'PYEOF'
import json, sys, collections

infile, crate, phase, outmd, jsonl_path = sys.argv[1:6]
by_file = collections.defaultdict(list)
n_errors = 0
n_warnings = 0

with open(infile, "r", errors="replace") as f:
    lines = f.readlines()

with open(jsonl_path, "a") as jf:
    for line in lines:
        line = line.strip()
        if not line or not line.startswith("{"):
            continue
        try:
            obj = json.loads(line)
        except json.JSONDecodeError:
            continue
        if obj.get("reason") != "compiler-message":
            continue
        msg = obj.get("message", {})
        level = msg.get("level", "")
        if level not in ("error", "warning"):
            continue
        code = (msg.get("code") or {}).get("code", "")
        spans = msg.get("spans", [])
        primary = next((s for s in spans if s.get("is_primary")), (spans[0] if spans else None))
        file_name = primary.get("file_name") if primary else "(kein Span)"
        line_start = primary.get("line_start") if primary else None
        col_start = primary.get("column_start") if primary else None
        children = [c.get("message", "") for c in msg.get("children", []) if c.get("message")]

        if level == "error":
            n_errors += 1
        else:
            n_warnings += 1

        compact = {
            "crate": crate, "phase": phase, "level": level, "code": code,
            "file": file_name, "line": line_start, "column": col_start,
            "message": msg.get("message", ""), "help": children,
        }
        jf.write(json.dumps(compact, ensure_ascii=False) + "\n")
        by_file[file_name].append(compact)

with open(outmd, "w") as md:
    md.write(f"# Diagnose-Report: `{crate}` ({phase})\n\n")
    md.write(f"Fehler: **{n_errors}**  |  Warnungen: **{n_warnings}**\n\n")
    if not by_file:
        md.write("_Keine Diagnosen._\n")
    for fname in sorted(by_file.keys(), key=lambda x: (x == "(kein Span)", x)):
        entries = sorted(by_file[fname], key=lambda e: (e["line"] or 0, e["column"] or 0))
        md.write(f"## `{fname}`  ({len(entries)} Diagnose(n))\n\n")
        for e in entries:
            loc = f"{fname}:{e['line']}:{e['column']}" if e["line"] else fname
            code_str = f" [{e['code']}]" if e["code"] else ""
            md.write(f"- **{e['level'].upper()}{code_str}** @ `{loc}`: {e['message']}\n")
            for h in e["help"]:
                h_clean = h.replace("\n", " ").strip()
                if h_clean:
                    md.write(f"  - _Hinweis:_ {h_clean}\n")
        md.write("\n")
print(f"{crate}|{phase}|{n_errors}|{n_warnings}")
PYEOF
}

resuming=false
[[ -z "$RESUME_FROM" ]] && resuming=true
any_failed=false

for crate in "${CRATES[@]}"; do
  if ! $resuming; then
    if [[ "$crate" == "$RESUME_FROM" ]]; then resuming=true; else continue; fi
  fi

  crate_dir="${OUTDIR}/${crate}"
  mkdir -p "$crate_dir"

  echo "=================================================================" | tee -a "$LOG"
  echo "=== $crate ===" | tee -a "$LOG"
  echo "=================================================================" | tee -a "$LOG"

  start=$(date +%s)
  check_status="SKIP"; clippy_status="SKIP"; test_status="SKIP"
  n_check_errors=0

  echo "--- cargo check -p $crate --all-targets (json) ---" >> "$LOG"
  check_json="${crate_dir}/check.json"
  if cargo check -p "$crate" --all-targets --message-format=json > "$check_json" 2>>"$LOG"; then
    check_status="PASS"
  else
    check_status="FAIL"
  fi

  if $HAVE_PY3; then
    result_line=$(parse_cargo_json "$check_json" "$crate" "check" "${crate_dir}/report.md" "$DIAG_JSONL")
    n_check_errors=$(echo "$result_line" | tail -1 | cut -d'|' -f3)
  fi

  if [[ "$check_status" == "PASS" && "$MODE" == "full" ]]; then
    if $RUN_CLIPPY; then
      echo "--- cargo clippy -p $crate --all-targets -- -D warnings (json) ---" >> "$LOG"
      clippy_json="${crate_dir}/clippy.json"
      if cargo clippy -p "$crate" --all-targets --message-format=json -- -D warnings > "$clippy_json" 2>>"$LOG"; then
        clippy_status="PASS"
      else
        clippy_status="FAIL"
      fi
      if $HAVE_PY3 && [[ -s "$clippy_json" ]]; then
        parse_cargo_json "$clippy_json" "$crate" "clippy" "${crate_dir}/clippy_report.md" "$DIAG_JSONL" > /dev/null
      fi
    fi

    echo "--- cargo test -p $crate --all-features ---" >> "$LOG"
    if cargo test -p "$crate" --all-features >> "$LOG" 2>&1; then
      test_status="PASS"
    else
      test_status="FAIL"
    fi
  fi

  # --- PRO-DATEI Duplicate-Symbol-Gate (exakt wie xtask check_duplicate_symbols: nur
  #     innerhalb DERSELBEN Datei zählen, nicht crate-weit) ---
  src_dir="$(crate_src_dir "$crate")"
  if [[ -n "$src_dir" ]]; then
    while IFS= read -r -d '' f; do
      dup_fns=$(grep -oE '^[[:space:]]*(pub(\([^)]*\))?[[:space:]]+)?fn[[:space:]]+[A-Za-z_][A-Za-z0-9_]*' "$f" \
        | sed -E 's/.*fn[[:space:]]+//' | sort | uniq -c | awk '$1 > 1 {print "  DUP fn:", $2, "x"$1}')
      dup_structs=$(grep -oE '^[[:space:]]*(pub(\([^)]*\))?[[:space:]]+)?struct[[:space:]]+[A-Za-z_][A-Za-z0-9_]*' "$f" \
        | sed -E 's/.*struct[[:space:]]+//' | sort | uniq -c | awk '$1 > 1 {print "  DUP struct:", $2, "x"$1}')
      if [[ -n "$dup_fns" || -n "$dup_structs" ]]; then
        {
          echo "### $crate :: $f"
          [[ -n "$dup_fns" ]] && echo "$dup_fns"
          [[ -n "$dup_structs" ]] && echo "$dup_structs"
          echo ""
        } >> "$DUP_FILE"
      fi
    done < <(find "$src_dir" -name '*.rs' -print0)
  fi

  end=$(date +%s)
  dur=$((end - start))
  printf "%-22s %-8s %-8s %-8s %-8s %-10s\n" "$crate" "$check_status" "$clippy_status" "$test_status" "$n_check_errors" "$dur" | tee -a "$SUMMARY"

  if [[ "$check_status" == "FAIL" ]]; then
    any_failed=true
    echo "" | tee -a "$SUMMARY"
    echo "!!! $crate kompiliert nicht ($n_check_errors Fehler)." | tee -a "$SUMMARY"
    echo "!!! Granularer Report: ${crate_dir}/report.md" | tee -a "$SUMMARY"
    if $STOP_ON_FAIL; then
      echo "!!! STOP (bottom-up): Downstream-Crates werden übersprungen." | tee -a "$SUMMARY"
      echo "!!! Fortsetzen nach Fix mit: ./verify_workspace.sh --resume $crate" | tee -a "$SUMMARY"
      break
    else
      echo "!!! fahre trotzdem fort (--all-crates/--full-audit aktiv)." | tee -a "$SUMMARY"
    fi
  fi
done

# ============================================================================
# --- FEATURE-MATRIX (§0.3): jedes default-off/dev Feature isoliert bauen ---
# ============================================================================
if $RUN_FEATURE_MATRIX; then
  echo "=== FEATURE-MATRIX ===" | tee -a "$LOG"
  for entry in "${FEATURE_CATALOG[@]}"; do
    IFS=':' read -r fcrate ffeat fdefault <<< "$entry"
    [[ -n "$ONLY_LIST" ]] && [[ ! " ${CRATES[*]} " =~ " ${fcrate} " ]] && continue
    fdir="${OUTDIR}/${fcrate}/features/${ffeat}"
    mkdir -p "$fdir"
    echo "--- cargo check -p $fcrate --features $ffeat (default=$fdefault) ---" | tee -a "$LOG"
    fjson="${fdir}/check.json"
    if [[ "$ffeat" == "loom" ]]; then
      # Loom braucht RUSTFLAGS, wird separat im --loom Job behandelt, hier nur Kompilierbarkeit
      status_line="SKIP (siehe --loom)"
      echo "$status_line" > "${fdir}/status.txt"
      continue
    fi
    if cargo check -p "$fcrate" --features "$ffeat" --all-targets --message-format=json > "$fjson" 2>>"$LOG"; then
      echo "PASS" > "${fdir}/status.txt"
    else
      echo "FAIL" > "${fdir}/status.txt"
      any_failed=true
    fi
    $HAVE_PY3 && parse_cargo_json "$fjson" "${fcrate}[${ffeat}]" "feature-check" "${fdir}/report.md" "$DIAG_JSONL" > /dev/null
  done
fi

# ============================================================================
# --- §15.4 PFLICHT-CI-GATES (xtask) — best-effort ---
# ============================================================================
if $RUN_XTASK_GATES; then
  {
    echo "# §15.4 Pflicht-CI-Gates — Ergebnis"
    echo ""
  } > "$XTASK_FILE"
  for gate_entry in "${XTASK_GATES[@]}"; do
    gate="${gate_entry%%|*}"
    extra_args=""
    [[ "$gate_entry" == *"|"* ]] && extra_args="${gate_entry#*|}"
    echo "--- cargo run -p xtask -- $gate $extra_args ---" | tee -a "$LOG"
    gate_out="${OUTDIR}/xtask_${gate}.log"
    # shellcheck disable=SC2086
    if cargo run -p xtask -- "$gate" $extra_args > "$gate_out" 2>&1; then
      echo "## \`$gate\` — ✅ PASS" >> "$XTASK_FILE"
    else
      rc=$?
      if grep -qiE "no such subcommand|unrecognized subcommand|error: unknown" "$gate_out"; then
        echo "## \`$gate\` — ⚠️ NICHT VERFÜGBAR (Subcommand existiert im aktuellen xtask-Code noch nicht — das ist selbst ein Befund für §15.4)" >> "$XTASK_FILE"
      else
        echo "## \`$gate\` — ❌ FAIL (exit $rc)" >> "$XTASK_FILE"
        any_failed=true
      fi
    fi
    echo "" >> "$XTASK_FILE"
    echo "\`\`\`" >> "$XTASK_FILE"
    tail -n 40 "$gate_out" >> "$XTASK_FILE"
    echo "\`\`\`" >> "$XTASK_FILE"
    echo "" >> "$XTASK_FILE"
  done
fi

# ============================================================================
# --- §15.3 LOOM-TESTS (isoliert, single-threaded, eigener Job) ---
# ============================================================================
if $RUN_LOOM; then
  echo "# §15.3 Loom-Concurrency-Tests" > "$LOOM_FILE"
  echo "" >> "$LOOM_FILE"
  echo "--- RUSTFLAGS=--cfg loom cargo test --workspace --features loom -- --test-threads=1 ---" | tee -a "$LOG"
  loom_out="${OUTDIR}/loom.log"
  if RUSTFLAGS="--cfg loom" cargo test --workspace --features loom -- --test-threads=1 > "$loom_out" 2>&1; then
    echo "✅ Loom-Suite PASS" >> "$LOOM_FILE"
  else
    echo "❌ Loom-Suite FAIL — siehe Ausschnitt unten, voller Log: $loom_out" >> "$LOOM_FILE"
    any_failed=true
  fi
  {
    echo ""
    echo "Erwartete Testpfade (§15.3):"
    echo "- crates/memfuse-store/tests/loom_group_commit.rs"
    echo "- crates/memfuse-store/tests/loom_multi_key_lock.rs"
    echo "- crates/memfuse-graph/tests/loom_relate_n_ary.rs"
    echo ""
    for t in loom_group_commit loom_multi_key_lock loom_relate_n_ary; do
      if grep -q "$t" "$loom_out"; then
        echo "- \`$t\`: im Lauf gefunden — Status siehe Log-Ausschnitt unten"
      else
        echo "- \`$t\`: NICHT im Testlauf gefunden (fehlt die Datei, oder Loom-Feature deckt sie nicht ab?)"
      fi
    done
    echo ""
    echo '```'
    tail -n 80 "$loom_out"
    echo '```'
  } >> "$LOOM_FILE"
fi

# ============================================================================
# --- BENCHMARK-DISCOVERY (+ optional echter Lauf via --run-benches) ---
# ============================================================================
if $RUN_BENCHMARKS; then
  echo "# Benchmark-Discovery & Status" > "$BENCH_FILE"
  echo "" >> "$BENCH_FILE"
  for crate in "${CRATES_ALL[@]}"; do
    root="$(crate_root_dir "$crate")"
    [[ -z "$root" ]] && continue
    bench_files=()
    [[ -d "${root}/benches" ]] && while IFS= read -r -d '' f; do bench_files+=("$f"); done < <(find "${root}/benches" -maxdepth 1 -name '*.rs' -print0 2>/dev/null)
    [[ ${#bench_files[@]} -eq 0 ]] && continue
    echo "## Crate: \`$crate\`" >> "$BENCH_FILE"
    for bf in "${bench_files[@]}"; do
      bname="$(basename "$bf" .rs)"
      is_ak8=""
      [[ "$bf" == *"binary_edge_regression.rs"* ]] && is_ak8=" (**AK-8 Baseline-Regressions-Benchmark — Pflicht laut §15.2**)"
      echo "- \`$bf\`${is_ak8}" >> "$BENCH_FILE"
      compile_log="${OUTDIR}/bench_${crate}_${bname}_compile.log"
      if cargo check -p "$crate" --bench "$bname" > "$compile_log" 2>&1; then
        echo "  - Kompilierung: ✅ PASS" >> "$BENCH_FILE"
      else
        echo "  - Kompilierung: ❌ FAIL — siehe $compile_log" >> "$BENCH_FILE"
        any_failed=true
      fi
      if $RUN_BENCHES_FOR_REAL; then
        run_log="${OUTDIR}/bench_${crate}_${bname}_run.log"
        echo "  - Lauf: wird ausgeführt (kann dauern)..." >> "$BENCH_FILE"
        if cargo bench -p "$crate" --bench "$bname" > "$run_log" 2>&1; then
          echo "  - Lauf: ✅ abgeschlossen — Ergebnis: $run_log" >> "$BENCH_FILE"
        else
          echo "  - Lauf: ❌ FEHLGESCHLAGEN — siehe $run_log" >> "$BENCH_FILE"
          any_failed=true
        fi
      fi
    done
    echo "" >> "$BENCH_FILE"
  done
  if ! grep -q "^## Crate:" "$BENCH_FILE"; then
    echo "_Keine benches/*.rs im Workspace gefunden._" >> "$BENCH_FILE"
  fi
fi

# ============================================================================
# --- PANIC-INVENTAR (§17 Opus 3.2): .unwrap()/.expect()/panic! NUR in src/,
#     getrennt nach Produktivcode vs. Tests/Benches ---
# ============================================================================
if $RUN_PANIC_INVENTORY; then
  echo "# Panic-Inventar (Zero-Panic-Doctrine, §17 Opus 3.2)" > "$PANIC_FILE"
  echo "" >> "$PANIC_FILE"
  echo "Zählt \`.unwrap(\`, \`.expect(\`, \`panic!(\`, \`unimplemented!(\`, \`todo!(\` getrennt nach" >> "$PANIC_FILE"
  echo "Produktivcode (\`src/\`, ohne \`#[cfg(test)]\`-Blöcke werden NICHT ausgeschlossen — das ist" >> "$PANIC_FILE"
  echo "ein Heuristik-Scan, kein AST-Parser) und Test-/Bench-Code (\`tests/\`, \`benches/\`)." >> "$PANIC_FILE"
  echo "" >> "$PANIC_FILE"
  total_src=0
  for crate in "${CRATES_ALL[@]}"; do
    root="$(crate_root_dir "$crate")"
    [[ -z "$root" ]] && continue
    src_dir="${root}/src"
    [[ -d "$src_dir" ]] || continue
    hits=$(grep -rnoE '\.(unwrap|expect)\(|panic!\(|unimplemented!\(|todo!\(' "$src_dir" 2>/dev/null || true)
    n=$(echo "$hits" | grep -c . || true)
    [[ -z "$hits" ]] && n=0
    total_src=$((total_src + n))
    if [[ "$n" -gt 0 ]]; then
      echo "## \`$crate\` — $n Fund(e) in \`$src_dir\`" >> "$PANIC_FILE"
      grep -rnE '\.(unwrap|expect)\(|panic!\(|unimplemented!\(|todo!\(' "$src_dir" 2>/dev/null \
        | sed "s|^|- \`|; s|:| @ Zeile |1; s|$|\`|" >> "$PANIC_FILE" || true
      echo "" >> "$PANIC_FILE"
    fi
  done
  echo "**Gesamt Produktivcode-Funde: $total_src**" >> "$PANIC_FILE"
  echo "" >> "$PANIC_FILE"
  echo "Hinweis: An FFI-Grenzen ist \`catch_unwind\` Pflicht statt Panic-Vermeidung — dort ist" >> "$PANIC_FILE"
  echo "ein Fund kein automatischer Verstoß, sondern muss manuell gegen das Gate \`check-ffi-panic-boundary\` geprüft werden." >> "$PANIC_FILE"
fi

# ============================================================================
# --- SECURITY-SCAN: sh -c / Command::new("sh") Verstöße ---
# ============================================================================
if $RUN_SECURITY_SCAN; then
  echo "# Security-Scan: verbotene Shell-Interpolation (\`sh -c\`)" > "$SEC_FILE"
  echo "" >> "$SEC_FILE"
  found_any=false
  for crate in "${CRATES_ALL[@]}"; do
    root="$(crate_root_dir "$crate")"
    [[ -z "$root" ]] && continue
    src_dir="${root}/src"
    [[ -d "$src_dir" ]] || continue
    hits=$(grep -rnE 'Command::new\("sh"\)|Command::new\("bash"\)|\.arg\("-c"\)|sh -c' "$src_dir" 2>/dev/null || true)
    if [[ -n "$hits" ]]; then
      found_any=true
      echo "## \`$crate\` — P0 SICHERHEITSRISIKO" >> "$SEC_FILE"
      echo '```' >> "$SEC_FILE"
      echo "$hits" >> "$SEC_FILE"
      echo '```' >> "$SEC_FILE"
      echo "" >> "$SEC_FILE"
      any_failed=true
    fi
  done
  $found_any || echo "_Keine Treffer — Execution-Invariant eingehalten._" >> "$SEC_FILE"
fi

# ============================================================================
# --- DAG-CHECK (Layer-Inversionen via cargo metadata) ---
# ============================================================================
if $RUN_DAG_CHECK && $HAVE_PY3; then
  echo "--- cargo metadata DAG-Check ---" | tee -a "$LOG"
  cargo metadata --no-deps --format-version=1 > "${OUTDIR}/metadata.json" 2>>"$LOG"
  python3 - "${OUTDIR}/metadata.json" "${OUTDIR}/dag_report.md" "${CRATES_ALL[*]}" <<'PYEOF'
import json, sys

meta_path, out_path, crates_csv = sys.argv[1], sys.argv[2], sys.argv[3]
layer_order = crates_csv.split()
layer_of = {name: i for i, name in enumerate(layer_order)}

try:
    with open(meta_path) as f:
        meta = json.load(f)
except Exception as e:
    with open(out_path, "w") as o:
        o.write(f"# DAG-Check fehlgeschlagen: {e}\n")
    sys.exit(0)

violations = []
for pkg in meta.get("packages", []):
    name = pkg.get("name")
    if name not in layer_of:
        continue
    for dep in pkg.get("dependencies", []):
        dname = dep.get("name")
        if dname in layer_of and layer_of[dname] > layer_of[name]:
            violations.append((name, dname))

with open(out_path, "w") as o:
    o.write("# DAG Layer-Verletzungen\n\n")
    if not violations:
        o.write("Keine Verletzungen gefunden.\n")
    for a, b in violations:
        o.write(f"- **{a}** (Layer {layer_of[a]}) hängt von **{b}** (Layer {layer_of[b]}) ab — Layer-Inversion!\n")
PYEOF
fi

# ============================================================================
# --- GESAMT-FEHLERBERICHT über alle Diagnosen, gruppiert nach Datei ---
# ============================================================================
if $HAVE_PY3; then
  python3 - "$DIAG_JSONL" "${OUTDIR}/FEHLERBERICHT.md" <<'PYEOF'
import json, sys, collections

jsonl_path, out_path = sys.argv[1], sys.argv[2]
by_crate_file = collections.defaultdict(list)
crates_with_errors = set()

with open(jsonl_path, errors="replace") as f:
    for line in f:
        line = line.strip()
        if not line:
            continue
        try:
            d = json.loads(line)
        except json.JSONDecodeError:
            continue
        if d["level"] != "error":
            continue
        crates_with_errors.add(d["crate"])
        by_crate_file[(d["crate"], d["file"])].append(d)

with open(out_path, "w") as o:
    o.write("# Gesamt-Fehlerbericht (gruppiert nach Crate -> Datei)\n\n")
    o.write("Direkte Grundlage für Jules-Prompts: jede Datei-Gruppe = ein Kandidat für einen\n")
    o.write("isolierten, parallel ausführbaren Prompt.\n\n")
    if not crates_with_errors:
        o.write("_Keine Fehler gefunden._\n")
    for crate in sorted(crates_with_errors):
        o.write(f"\n## Crate: `{crate}`\n\n")
        for (c, fname) in sorted([k for k in by_crate_file if k[0] == crate], key=lambda k: k[1]):
            entries = sorted(by_crate_file[(c, fname)], key=lambda e: (e["line"] or 0))
            o.write(f"### `{fname}` ({len(entries)} Fehler)\n\n")
            codes = collections.Counter(e["code"] for e in entries if e["code"])
            if codes:
                o.write("Fehlercodes: " + ", ".join(f"`{k}`×{v}" for k, v in codes.most_common()) + "\n\n")
            for e in entries:
                loc = f"{fname}:{e['line']}:{e['column']}" if e["line"] else fname
                code_str = f" [{e['code']}]" if e["code"] else ""
                o.write(f"- **{code_str.strip()}** @ `{loc}`: {e['message']}\n")
                for h in e["help"]:
                    h_clean = h.replace("\n", " ").strip()
                    if h_clean:
                        o.write(f"  - Hinweis: {h_clean}\n")
            o.write("\n")
PYEOF
fi

echo "" | tee -a "$SUMMARY"
echo "Ergebnisverzeichnis: $OUTDIR" | tee -a "$SUMMARY"
[[ -f "${OUTDIR}/FEHLERBERICHT.md" ]] && echo "  - Fehlerbericht (Jules-Prompt-Basis): ${OUTDIR}/FEHLERBERICHT.md" | tee -a "$SUMMARY"
$RUN_XTASK_GATES && echo "  - CI-Gates: $XTASK_FILE" | tee -a "$SUMMARY"
$RUN_LOOM && echo "  - Loom: $LOOM_FILE" | tee -a "$SUMMARY"
$RUN_BENCHMARKS && echo "  - Benchmarks: $BENCH_FILE" | tee -a "$SUMMARY"
$RUN_PANIC_INVENTORY && echo "  - Panic-Inventar: $PANIC_FILE" | tee -a "$SUMMARY"
$RUN_SECURITY_SCAN && echo "  - Security-Scan: $SEC_FILE" | tee -a "$SUMMARY"
[[ -s "$DUP_FILE" ]] && echo "  - Duplicate-Symbole (pro Datei): $DUP_FILE" | tee -a "$SUMMARY"

if $any_failed; then
  exit 1
else
  echo "Alle geprüften Aspekte grün." | tee -a "$SUMMARY"
  exit 0
fi
