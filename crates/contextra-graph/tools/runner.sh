#!/usr/bin/env bash
# Granular Test-, Mess- und Audit-Runner für Crate: contextra-graph
# zsh- und bash-kompatibel.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
CRATE_DIR="$(cd "${SCRIPT_DIR}/.." && pwd)"
WORKSPACE_ROOT="$(cd "${CRATE_DIR}/../.." && pwd)"
CRATE_NAME="contextra-graph"

usage() {
    echo "Benutzung: $0 <Kommando> [Optionen]"
    echo ""
    echo "Kommandos:"
    echo "  check                 - Schneller Type- & Syntax-Check (cargo check -p ${CRATE_NAME})"
    echo "  check-all-features    - Check mit allen Features"
    echo "  check-no-default      - Check ohne Standard-Features"
    echo "  test [FILTER]         - Tests ausführen (optional gefiltert nach Modul/Funktion)"
    echo "  test-unit [FILTER]    - Nur Unit-Tests (--lib)"
    echo "  test-doc              - Dokumentationstests (--doc)"
    echo "  clippy                - Clippy Linter (-D warnings)"
    echo "  bench [BENCH_NAME]    - Benchmarks ausführen (falls vorhanden)"
    echo "  audit                 - Crate-spezifisches Audit mit Workspace-Tools (JSON)"
    echo "  loc                   - Zählt Zeilen Code (LOC)"
    echo "  summary               - Kompakte Zusammenfassung (Token-optimiert für LLM/Jules)"
    echo ""
    echo "Beispiele:"
    echo "  $0 test"
    echo "  $0 test aes"
    echo "  $0 test-unit kdf"
    echo "  $0 audit"
    echo "  $0 summary"
}

cmd="${1:-summary}"
shift || true

case "${cmd}" in
    check)
        cargo check -p "${CRATE_NAME}" "$@"
        ;;
    check-all-features)
        cargo check -p "${CRATE_NAME}" --all-features "$@"
        ;;
    check-no-default)
        cargo check -p "${CRATE_NAME}" --no-default-features "$@"
        ;;
    test)
        cargo test -p "${CRATE_NAME}" "$@"
        ;;
    test-unit)
        cargo test -p "${CRATE_NAME}" --lib "$@"
        ;;
    test-doc)
        cargo test -p "${CRATE_NAME}" --doc "$@"
        ;;
    clippy)
        cargo clippy -p "${CRATE_NAME}" -- -D warnings "$@"
        ;;
    bench)
        cargo bench -p "${CRATE_NAME}" "$@"
        ;;
    audit)
        echo "=== Crate Audit für ${CRATE_NAME} ==="
        python3 "${WORKSPACE_ROOT}/tools/loc.py" --crate "${CRATE_NAME}"
        python3 "${WORKSPACE_ROOT}/tools/zero_panic_analyzer.py" --crate "${CRATE_NAME}" --json
        python3 "${WORKSPACE_ROOT}/tools/unsafe_boundary_auditor.py" --crate "${CRATE_NAME}" --json
        python3 "${WORKSPACE_ROOT}/tools/determinism_guard.py" --crate "${CRATE_NAME}" --json
        python3 "${WORKSPACE_ROOT}/tools/decouple_inspector.py" --crate "${CRATE_NAME}" --json
        ;;
    loc)
        python3 "${WORKSPACE_ROOT}/tools/loc.py" --crate "${CRATE_NAME}" "$@"
        ;;
    summary)
        echo "=== [CRATE SUMMARY: ${CRATE_NAME}] ==="
        python3 -c "
import subprocess, json

crate = '${CRATE_NAME}'
ws = '${WORKSPACE_ROOT}'

try:
    loc_out = subprocess.check_output(['python3', f'{ws}/tools/loc.py', '--crate', crate, '--json'], stderr=subprocess.DEVNULL)
    loc_data = json.loads(loc_out)
    c_info = loc_data.get('crates', {}).get(crate, {})
    print(f'LOC: {c_info.get(\"loc\", 0)} across {c_info.get(\"files_count\", 0)} files')
except Exception:
    print('LOC: N/A')

try:
    zp_out = subprocess.check_output(['python3', f'{ws}/tools/zero_panic_analyzer.py', '--crate', crate, '--json'], stderr=subprocess.DEVNULL)
    zp_data = json.loads(zp_out)
    sum_zp = zp_data[0].get('summary', {}) if zp_data else {}
    print(f'Panic: prod_unwrap={sum_zp.get(\"prod_unwrap\", 0)}, prod_expect={sum_zp.get(\"prod_expect\", 0)}, prod_assert={sum_zp.get(\"prod_assert\", 0)}')
except Exception:
    print('Panic: N/A')

try:
    un_out = subprocess.check_output(['python3', f'{ws}/tools/unsafe_boundary_auditor.py', '--crate', crate, '--json'], stderr=subprocess.DEVNULL)
    un_data = json.loads(un_out)
    un_info = un_data[0] if un_data else {}
    print(f'Unsafe: occurrences={un_info.get(\"total_unsafe_occurrences\", 0)}, forbid_unsafe={un_info.get(\"root_forbid_unsafe\", False)}')
except Exception:
    print('Unsafe: N/A')

try:
    det_out = subprocess.check_output(['python3', f'{ws}/tools/determinism_guard.py', '--crate', crate, '--json'], stderr=subprocess.DEVNULL)
    det_data = json.loads(det_out)
    det_info = det_data[0] if det_data else {}
    print(f'Determinism: total_prod_issues={det_info.get(\"total_prod_issues\", 0)}')
except Exception:
    print('Determinism: N/A')
print('=====================================')
"
        ;;
    help|--help|-h)
        usage
        ;;
    *)
        echo "Unbekanntes Kommando: ${cmd}"
        usage
        return 1 2>/dev/null || (return 1 2>/dev/null || false)
        ;;
esac
