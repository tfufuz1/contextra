#!/usr/bin/env bash
# tools/quickcheck.sh — Einstiegspunkt für Nachfolger-Sitzungen (siehe CLAUDE.md).
#
#   tools/quickcheck.sh build        cargo check (stable, Workaround für fehlenden 1.89-Download)
#   tools/quickcheck.sh gates        Build-Gates G-02/G-04/G-09/G-11/G-12/G-13 aus SPEC §11.6 per grep
#   tools/quickcheck.sh diagnostics  Verifiziert lauffähige Analyse-Tools -> evidence/diagnostics/
#   tools/quickcheck.sh all          build + gates + diagnostics
#
# Alle Ausgaben landen in evidence/ (gitignored), nie in docs/ oder im Repo-Baum.
set -u
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT" || exit 1
OUT="$ROOT/evidence/diagnostics"
mkdir -p "$OUT"

# rust-toolchain.toml pinnt 1.89.0, dessen Download im Sandbox blockiert ist.
# `stable` (1.97) baut das Workspace nachweislich. Überschreibbar via RC_TOOLCHAIN.
TC="${RC_TOOLCHAIN:-stable}"

cmd_build() {
  echo "== cargo check --workspace (RUSTUP_TOOLCHAIN=$TC) =="
  RUSTUP_TOOLCHAIN="$TC" cargo check --workspace --all-targets 2>&1 | tail -n 15
  return "${PIPESTATUS[0]}"
}

cmd_gates() {
  local rc=0
  count() { grep "$@" 2>/dev/null | wc -l | tr -d ' '; }
  echo "G-04  MAX_SEARCH_K-Definitionen (Soll: 1):   $(count -rn 'const MAX_SEARCH_K' crates --include=*.rs)"
  echo "G-09  Selbstdeklarationen ANCHOR/FILE-CONTEXT (Soll: 0): $(grep -rl 'ANCHOR\[\|FILE-CONTEXT' crates --include=*.rs 2>/dev/null | wc -l | tr -d ' ')"
  echo "G-09  contract.rs (Soll: 0):                  $(find crates -name contract.rs | wc -l | tr -d ' ')"
  echo "G-11  compat-Verzeichnisse (Soll: 0):         $(find crates benchmarks -type d -name compat | wc -l | tr -d ' ')"
  echo "G-12  Instant/SystemTime außerhalb env (Soll: 0): $(grep -rn 'Instant::now\|SystemTime::now' crates --include=*.rs | grep -v 'crates/contextra-env/' | wc -l | tr -d ' ')"
  echo "G-13  unwrap/expect/panic in src (grob, Soll: 0): $(grep -rn 'unwrap()\|expect(\|panic!(' crates/*/src --include=*.rs | wc -l | tr -d ' ')"
  echo "G-02  Dateien mit unsafe (Soll: nur contextra-sys/simd):"
  grep -rln '^[^/]*\bunsafe\b' crates --include=*.rs | sed 's#crates/##;s#/.*##' | sort | uniq -c | sed 's/^/        /'
  echo "G-10  benchmarks/budgets.toml: $([ -f benchmarks/budgets.toml ] && echo vorhanden || echo FEHLT)"
  echo "G-03  crates/contextra-testkit/tests/layering.rs: $([ -f crates/contextra-testkit/tests/layering.rs ] && echo vorhanden || echo FEHLT)"
  echo "G1    capabilities.toml (Schichtprüfung): $([ -f capabilities.toml ] && echo vorhanden || echo FEHLT)"
  return $rc
}

cmd_diagnostics() {
  # Nur Tools, die mit diesen Argumenten verifiziert liefen (Stand 21.11.2026).
  # Bewusst ausgelassen: gate_dependencies (braucht capabilities.toml, FEHLT),
  # agent_tooling_common (Bibliothek), baseline/ (eigene Aufrufe, lange Laufzeit).
  local py=python3 fails=0
  run() {
    local name="$1"; shift
    if "$py" "tools/$name.py" "$@" > "$OUT/$name.log" 2>&1; then
      echo "  OK    $name"
    else
      echo "  FAIL  $name  (Log: $OUT/$name.log)"; fails=$((fails+1))
    fi
  }
  echo "== Diagnose-Tools -> $OUT =="
  run async_boundary_scanner      --workspace . --output-dir "$OUT/async_boundary_scanner"
  run concurrency_lock_auditor    --workspace . --output-dir "$OUT/concurrency_lock_auditor"
  run contract_diff_detector      --workspace . --output-dir "$OUT/contract_diff_detector"
  run dead_code_microprobe        --workspace . --output-dir "$OUT/dead_code_microprobe"
  run determinism_guard           --workspace . --output-dir "$OUT/determinism_guard"
  run erasure_leak_prober         --workspace . --output-dir "$OUT/erasure_leak_prober"
  run error_topology_mapper       --workspace . --output-dir "$OUT/error_topology_mapper"
  run fsync_durability_tracer     --workspace . --output-dir "$OUT/fsync_durability_tracer"
  run pattern_diagnostic          --workspace . --output-dir "$OUT/pattern_diagnostic"
  run port_contract_auditor       --workspace . --output-dir "$OUT/port_contract_auditor"
  run rust_ast_inspector          --workspace . --output-dir "$OUT/rust_ast_inspector"
  run test_coverage_matrix        --workspace . --output-dir "$OUT/test_coverage_matrix"
  run test_matrix_advisor         --workspace . --output-dir "$OUT/test_matrix_advisor"
  run type_registry_sync          --workspace . --output-dir "$OUT/type_registry_sync"
  run unsafe_boundary_tracer      --workspace . --output-dir "$OUT/unsafe_boundary_tracer"
  run zero_panic_analyzer         --workspace . --output-dir "$OUT/zero_panic_analyzer"
  run agentic_context_pack        --workspace . --output-dir "$OUT/agentic_context_pack"
  run orchestrate_diagnostics     --workspace . --output-dir "$OUT/orchestrate"
  # Eigene CLI-Form (kein --workspace/--output-dir):
  echo "  --    (Einzelaufruf, eigene CLI:)"
  echo "        python3 tools/loc.py                     # LOC je Crate"
  echo "        python3 tools/crate_inventory.py --root . overview"
  echo "        python3 tools/feature_audit.py --root ."
  echo "        python3 tools/doc_lint.py --root .        # Befunde in SPEC-Negativlisten sind teils Fehlalarme"
  echo "        python3 tools/abhaengigkeits_budget.py --json > $OUT/budget.json"
  echo "        python3 tools/isolation_proof.py --crate <name> --modes default"
  echo "        python3 tools/generate_interface_spec.py --repo-root . --out-dir $OUT --no-timestamp-only"
  return $fails
}

case "${1:-all}" in
  build)       cmd_build ;;
  gates)       cmd_gates ;;
  diagnostics) cmd_diagnostics ;;
  all)         cmd_build; cmd_gates; cmd_diagnostics ;;
  *) echo "Usage: $0 {build|gates|diagnostics|all}"; exit 2 ;;
esac
