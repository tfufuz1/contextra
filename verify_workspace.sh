#!/usr/bin/env bash
# verify_workspace.sh — Bottom-up, Crate-für-Crate Kompilierungs- & Test-Verifikation
# ERWEITERTE VERSION: feingranulare Diagnostik via `cargo ... --message-format=json`
#
# Zweck: Ersetzt Vertrauen in KI-Audit-Prosa durch echte, reproduzierbare cargo-Läufe
#        UND liefert strukturierte, maschinenlesbare Fehlerdaten (Datei, Zeile, Spalte,
#        Error-Code, Message, ggf. Suggested-Fix) statt nur Rohtext-Logs.
#
# Prinzip: Erst Layer 0 (Blätter), dann aufsteigend. Bricht beim ersten echten Fehler ab
#          (Default), kann aber mit --all-crates trotzdem bis zum Ende durchlaufen, um
#          EIN vollständiges Fehlerbild über alle Crates zu bekommen (nützlich, um danach
#          in einem Rutsch parallele Jules-Prompts pro Crate/Datei zu bauen).
#
# Nutzung:
#   chmod +x verify_workspace.sh
#   ./verify_workspace.sh                          # normaler Lauf (check + clippy + test)
#   ./verify_workspace.sh --fast                   # nur `cargo check` pro Crate
#   ./verify_workspace.sh --resume memfuse-index    # ab einem bestimmten Crate weitermachen
#   ./verify_workspace.sh --all-crates             # NICHT beim ersten Fehler stoppen
#   ./verify_workspace.sh --only memfuse-db,memfuse-graph   # nur bestimmte Crates prüfen
#   ./verify_workspace.sh --no-clippy              # clippy überspringen (nur check+test)
#   ./verify_workspace.sh --dag-check              # zusätzlich cargo-metadata DAG-Prüfung
#
# Ergebnisse (alle unter results/<TS>/):
#   verify.log                 — vollständige Roh-Ausgabe aller Kommandos
#   verify.summary             — PASS/FAIL-Tabelle pro Crate (wie bisher)
#   diagnostics.jsonl          — EINE JSON-Zeile pro Compiler-Diagnose, über alle Crates
#   <crate>/check.json         — Roh-JSON-Stream von `cargo check --message-format=json`
#   <crate>/clippy.json        — Roh-JSON-Stream von `cargo clippy --message-format=json`
#   <crate>/report.md          — pro Crate: Fehler gruppiert nach Datei, mit Code+Message+Span
#   FEHLERBERICHT.md           — Gesamtbericht über alle fehlgeschlagenen Crates, gruppiert
#                                 nach Datei — direkt verwertbar als Input für Jules-Prompts
#   duplicate_symbols.txt      — heuristischer Scan auf doppelte fn/struct-Definitionen
#
# Voraussetzung für die granulare JSON-Auswertung: python3 (Standard auf den meisten
# Dev-Boxen). Ohne python3 fällt das Skript auf reines Rohtext-Logging zurück (wie die
# Ausgangsversion), warnt aber deutlich.

set -uo pipefail

# --- DAG-Reihenfolge aus WORKING_STATE.md (Layer 0 -> Layer 9) ---
CRATES_ALL=(
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
STOP_ON_FAIL=true
ONLY_LIST=""
RUN_CLIPPY=true
RUN_DAG_CHECK=false

while [[ $# -gt 0 ]]; do
  case "$1" in
    --fast) MODE="fast"; shift ;;
    --resume) RESUME_FROM="$2"; shift 2 ;;
    --all-crates) STOP_ON_FAIL=false; shift ;;
    --only) ONLY_LIST="$2"; shift 2 ;;
    --no-clippy) RUN_CLIPPY=false; shift ;;
    --dag-check) RUN_DAG_CHECK=true; shift ;;
    -h|--help)
      sed -n '2,32p' "$0"; exit 0 ;;
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
  echo "!!! WARNUNG: python3 nicht gefunden. Granulare JSON-Diagnostik (report.md,"
  echo "!!! diagnostics.jsonl, FEHLERBERICHT.md) wird ÜBERSPRUNGEN. Nur Rohtext-Log."
fi

TS="$(date +%Y%m%d_%H%M%S)"
OUTDIR="results/${TS}"
mkdir -p "$OUTDIR"
LOG="${OUTDIR}/verify.log"
SUMMARY="${OUTDIR}/verify.summary"
DIAG_JSONL="${OUTDIR}/diagnostics.jsonl"
DUP_FILE="${OUTDIR}/duplicate_symbols.txt"
: > "$DIAG_JSONL"
: > "$DUP_FILE"

echo "# MemFuse Bottom-Up Verification — $(date -Iseconds)" > "$SUMMARY"
echo "# Modus: $MODE | stop_on_fail=$STOP_ON_FAIL | clippy=$RUN_CLIPPY" >> "$SUMMARY"
printf "%-22s %-8s %-8s %-8s %-8s %-10s\n" "CRATE" "CHECK" "CLIPPY" "TEST" "ERRORS" "DAUER(s)" >> "$SUMMARY"

# --- Python-Helfer: parst einen `cargo --message-format=json` Stream und schreibt
#     (a) eine kompakte diagnostics.jsonl-Zeile pro echtem Fehler/Warning mit primärem Span
#     (b) ein pro-Crate Markdown, gruppiert nach Datei, sortiert nach Zeile.
# Nimmt: <input.json> <crate_name> <phase: check|clippy> <output_md> <jsonl_append_target>
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
        rendered = msg.get("rendered", "") or msg.get("message", "")
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
            "crate": crate,
            "phase": phase,
            "level": level,
            "code": code,
            "file": file_name,
            "line": line_start,
            "column": col_start,
            "message": msg.get("message", ""),
            "help": children,
        }
        jf.write(json.dumps(compact, ensure_ascii=False) + "\n")

        by_file[file_name].append({
            "level": level,
            "code": code,
            "line": line_start,
            "column": col_start,
            "message": msg.get("message", ""),
            "help": children,
            "rendered": rendered,
        })

with open(outmd, "w") as md:
    md.write(f"# Diagnose-Report: `{crate}` ({phase})\n\n")
    md.write(f"Fehler: **{n_errors}**  |  Warnungen: **{n_warnings}**\n\n")
    if not by_file:
        md.write("_Keine Diagnosen (bereits sauber oder Parser fand nichts)._\n")
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
first_failed_crate=""

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

  # 1) cargo check — schnellster, härtester Filter. JSON-Ausgabe für Diagnostik,
  #    plus menschenlesbare Rendered-Ausgabe fürs Rohlog.
  echo "--- cargo check -p $crate --all-targets (json) ---" >> "$LOG"
  check_json="${crate_dir}/check.json"
  if cargo check -p "$crate" --all-targets --message-format=json > "$check_json" 2>>"$LOG"; then
    check_status="PASS"
  else
    check_status="FAIL"
  fi
  # Menschenlesbare Kurzfassung zusätzlich ins Hauptlog spiegeln
  if $HAVE_PY3 && [[ -s "$check_json" ]]; then
    python3 -c '
import json,sys
for line in open(sys.argv[1], errors="replace"):
    line=line.strip()
    if not line.startswith("{"): continue
    try: obj=json.loads(line)
    except Exception: continue
    if obj.get("reason")=="compiler-message":
        r=obj.get("message",{}).get("rendered")
        if r: print(r)
' "$check_json" >> "$LOG" 2>>"$LOG"
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

  # 2) Heuristischer Duplicate-Symbol-Scan (unabhängig vom Compiler-Ergebnis) —
  #    findet Kandidaten für "gleiche fn/struct in zwei Modulen" BEVOR es zum
  #    E0592-Fehler kommt (z.B. wenn check schon fehlschlägt und man vorbeugen will).
  crate_src="crates/${crate}/src"
  [[ -d "$crate_src" ]] || crate_src="benchmarks/${crate}/src"
  if [[ -d "$crate_src" ]]; then
    {
      echo "### $crate ($crate_src)"
      grep -rhoE '^[[:space:]]*(pub(\([^)]*\))?[[:space:]]+)?fn[[:space:]]+[A-Za-z_][A-Za-z0-9_]*' "$crate_src" \
        | sed -E 's/.*fn[[:space:]]+//' | sort | uniq -c | sort -rn | awk '$1 > 1 {print "  DUP fn:", $2, "x"$1}'
      grep -rhoE '^[[:space:]]*(pub(\([^)]*\))?[[:space:]]+)?struct[[:space:]]+[A-Za-z_][A-Za-z0-9_]*' "$crate_src" \
        | sed -E 's/.*struct[[:space:]]+//' | sort | uniq -c | sort -rn | awk '$1 > 1 {print "  DUP struct:", $2, "x"$1}'
      echo ""
    } >> "$DUP_FILE"
  fi

  end=$(date +%s)
  dur=$((end - start))
  printf "%-22s %-8s %-8s %-8s %-8s %-10s\n" "$crate" "$check_status" "$clippy_status" "$test_status" "$n_check_errors" "$dur" | tee -a "$SUMMARY"

  if [[ "$check_status" == "FAIL" ]]; then
    any_failed=true
    [[ -z "$first_failed_crate" ]] && first_failed_crate="$crate"
    echo "" | tee -a "$SUMMARY"
    echo "!!! $crate kompiliert nicht ($n_check_errors Fehler)." | tee -a "$SUMMARY"
    echo "!!! Granularer Report: ${crate_dir}/report.md" | tee -a "$SUMMARY"
    echo "!!! Rohlog-Abschnitt: $LOG (Abschnitt '=== $crate ===')" | tee -a "$SUMMARY"
    if $STOP_ON_FAIL; then
      echo "!!! STOP (bottom-up): Downstream-Crates werden übersprungen." | tee -a "$SUMMARY"
      echo "!!! Fortsetzen nach Fix mit: ./verify_workspace.sh --resume $crate" | tee -a "$SUMMARY"
      break
    else
      echo "!!! --all-crates aktiv: fahre trotzdem fort, um Gesamtbild zu bekommen." | tee -a "$SUMMARY"
    fi
  fi
done

# --- Optionaler DAG-Check über cargo-metadata (Layer-Verletzungen: Import aus höherem
#     Layer in niedrigeren Layer) ---
if $RUN_DAG_CHECK && $HAVE_PY3; then
  echo "--- cargo metadata DAG-Check ---" | tee -a "$LOG"
  cargo metadata --no-deps --format-version=1 > "${OUTDIR}/metadata.json" 2>>"$LOG"
  python3 - "${OUTDIR}/metadata.json" "${OUTDIR}/dag_report.md" <<'PYEOF'
import json, sys

layer_order = [
  "memfuse-core-ipc-gen","memfuse-core","memfuse-calibration","memfuse-checkpoint",
  "memfuse-crypto","memfuse-graph","memfuse-sandbox","memfuse-text","memfuse-index",
  "memfuse-ollama","memfuse-store","memfuse-candle","memfuse-embed","memfuse-db",
  "memfuse-bench","memfuse-router","memfuse-agent","memfuse-mcp",
]
layer_of = {name: i for i, name in enumerate(layer_order)}

meta_path, out_path = sys.argv[1], sys.argv[2]
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
        o.write("Keine Verletzungen gefunden — Abhängigkeiten laufen konsistent low->high.\n")
    for a, b in violations:
        o.write(f"- **{a}** (Layer {layer_of[a]}) hängt von **{b}** (Layer {layer_of[b]}) ab — Layer-Inversion!\n")
PYEOF
  echo "DAG-Report: ${OUTDIR}/dag_report.md" | tee -a "$SUMMARY"
fi

# --- Gesamt-Fehlerbericht über alle fehlgeschlagenen Crates, gruppiert nach Datei ---
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
        key = (d["crate"], d["file"])
        by_crate_file[key].append(d)

with open(out_path, "w") as o:
    o.write("# Gesamt-Fehlerbericht (alle Crates, gruppiert nach Datei)\n\n")
    o.write("Dieser Bericht ist die direkte Grundlage für die Jules-Prompt-Erstellung:\n")
    o.write("jede unten stehende Datei-Gruppe ist ein Kandidat für EINEN isolierten Jules-Prompt.\n")
    o.write("Dateien im selben Crate, die thematisch zusammenhängen (z.B. Typen aus derselben\n")
    o.write("Modul-Familie), sollten in EINEM Prompt gebündelt werden, um Merge-Konflikte zu\n")
    o.write("vermeiden — siehe Design-Prinzip 'Datei- & Crate-Isolierung'.\n\n")
    if not crates_with_errors:
        o.write("_Keine Fehler gefunden — Workspace ist grün._\n")
    for crate in sorted(crates_with_errors):
        o.write(f"\n## Crate: `{crate}`\n\n")
        keys = [k for k in by_crate_file if k[0] == crate]
        for (c, fname) in sorted(keys, key=lambda k: k[1]):
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
print("FEHLERBERICHT geschrieben:", out_path)
PYEOF
fi

echo "" | tee -a "$SUMMARY"
if $any_failed; then
  echo "Ergebnisverzeichnis: $OUTDIR" | tee -a "$SUMMARY"
  echo "  - Kompletter Fehlerbericht (Basis für Jules-Prompts): ${OUTDIR}/FEHLERBERICHT.md" | tee -a "$SUMMARY"
  echo "  - Rohes JSONL aller Diagnosen: $DIAG_JSONL" | tee -a "$SUMMARY"
  echo "  - Duplicate-Symbol-Scan: $DUP_FILE" | tee -a "$SUMMARY"
  exit 1
else
  echo "Alle geprüften Crates grün. Ergebnisverzeichnis: $OUTDIR" | tee -a "$SUMMARY"
  exit 0
fi
