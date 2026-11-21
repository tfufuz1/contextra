#!/usr/bin/env bash
# tools/quickcheck.sh — Einstiegspunkt für Sitzungen (siehe CLAUDE.md).
#
#   tools/quickcheck.sh build      cargo check --workspace (stable; 1.89-Download ist im Sandbox blockiert)
#   tools/quickcheck.sh gates      Build-Gates aus SPEC §11.6 per grep, mit Soll/Ist
#   tools/quickcheck.sh lint       Dokumentations-Lint (G-08 / doc_lint.py) und LOC je Crate
#   tools/quickcheck.sh coverage   Test-Matrix je Crate -> evidence/coverage/
#   tools/quickcheck.sh all        build + gates + lint + coverage
#
# Ausgaben landen ausschließlich in evidence/ (gitignored), nie in docs/ oder im Repo-Baum.
set -u
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT" || exit 1
EVID="$ROOT/evidence"
mkdir -p "$EVID"

# rust-toolchain.toml pinnt 1.89.0; dessen Download ist im Sandbox blockiert.
# `stable` (1.97) baut das Workspace. Überschreibbar via RC_TOOLCHAIN.
TC="${RC_TOOLCHAIN:-stable}"

cmd_build() {
  echo "== cargo check --workspace (RUSTUP_TOOLCHAIN=$TC) =="
  RUSTUP_TOOLCHAIN="$TC" cargo check --workspace --keep-going --message-format=short 2>&1 \
    | grep -E "^error|could not compile|Finished" | sed 's/^/  /' | head -n 40
  return "${PIPESTATUS[0]}"
}

cmd_gates() {
  count() { grep "$@" 2>/dev/null | wc -l | tr -d ' '; }
  echo "G-04  MAX_SEARCH_K-Definitionen   (Soll 1):  $(count -rn 'const MAX_SEARCH_K' crates --include=*.rs)"
  echo "G-09  Dateien mit ANCHOR/FILE-CONTEXT (Soll 0): $(grep -rl 'ANCHOR\[\|FILE-CONTEXT' crates --include=*.rs 2>/dev/null | wc -l | tr -d ' ')"
  echo "G-09  contract.rs                (Soll 0):  $(find crates -name contract.rs | wc -l | tr -d ' ')"
  echo "G-11  compat-Verzeichnisse       (Soll 0):  $(find crates benchmarks -type d -name compat | wc -l | tr -d ' ')"
  echo "G-12  Instant/SystemTime außerhalb env (Soll 0): $(grep -rn 'Instant::now\|SystemTime::now' crates --include=*.rs | grep -v 'crates/contextra-env/' | wc -l | tr -d ' ')"
  echo "G-13  unwrap/expect/panic in src, grob (Soll 0): $(grep -rn 'unwrap()\|expect(\|panic!(' crates/*/src --include=*.rs | wc -l | tr -d ' ')"
  echo "G-02  Dateien mit unsafe nach Crate (Soll: nur contextra-sys, contextra-simd):"
  grep -rln '^[^/]*\bunsafe\b' crates --include=*.rs | sed 's#crates/##;s#/.*##' | sort | uniq -c | sed 's/^/        /'
  echo "G-10  benchmarks/budgets.toml:                  $([ -f benchmarks/budgets.toml ] && echo vorhanden || echo FEHLT)"
  echo "G-03  crates/contextra-testkit/tests/layering.rs: $([ -f crates/contextra-testkit/tests/layering.rs ] && echo vorhanden || echo FEHLT)"
  echo "G1    capabilities.toml (Schichtprüfung):        $([ -f capabilities.toml ] && echo vorhanden || echo FEHLT)"
}

cmd_lint() {
  echo "== doc_lint (G-08) =="
  python3 tools/doc_lint.py --root .
  local rc=$?
  echo
  echo "== LOC je Crate =="
  python3 tools/loc.py
  return $rc
}

cmd_sync() {
  echo "== SYNC-1: Anhänge D/E gegen budgets.toml/capabilities.toml =="
  python3 tools/check_spec_sync.py check
}

cmd_layering() {
  echo "== G-03 / SYNC-2: Schichtregeln (Ratchet gegen layering_baseline.txt) =="
  RUSTUP_TOOLCHAIN="$TC" python3 tools/check_spec_sync.py layering --ratchet
}

cmd_coverage() {
  echo "== test_coverage_matrix -> $EVID/coverage =="
  python3 tools/test_coverage_matrix.py --workspace . --output-dir "$EVID/coverage" > "$EVID/coverage.log" 2>&1
  local rc=$?
  if [ $rc -eq 0 ]; then
    tail -n 3 "$EVID/coverage.log" | sed 's/^/  /'
  else
    echo "  FEHLER (Log: $EVID/coverage.log)"; tail -n 5 "$EVID/coverage.log" | sed 's/^/  /'
  fi
  return $rc
}

# Mehrere Befehle möglich: quickcheck.sh sync layering build
run_one() {
  case "$1" in
    build)    cmd_build ;;
    sync)     cmd_sync ;;
    layering) cmd_layering ;;
    gates)    cmd_gates ;;
    lint)     cmd_lint ;;
    coverage) cmd_coverage ;;
    all)      cmd_build; echo; cmd_sync; echo; cmd_layering; echo; cmd_gates; echo; cmd_lint; echo; cmd_coverage ;;
    *) echo "Usage: $0 {build|sync|layering|gates|lint|coverage|all} [...]"; return 2 ;;
  esac
}

if [ $# -eq 0 ]; then set -- all; fi
rc=0
for c in "$@"; do run_one "$c" || rc=$?; echo; done
exit $rc
