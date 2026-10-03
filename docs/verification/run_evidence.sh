#!/usr/bin/env bash
set -euo pipefail

# Contextra Evidence Runner
# Generates reproducible, unvarnished verification evidence artifacts.

REPO_ROOT="$(git rev-parse --show-toplevel)"
cd "${REPO_ROOT}"

QUICK_MODE=false
CRASH_ITERS=""

# Parse arguments
while [[ $# -gt 0 ]]; do
  case "$1" in
    --quick)
      QUICK_MODE=true
      shift
      ;;
    --crash-iters)
      CRASH_ITERS="$2"
      shift 2
      ;;
    --crash-iters=*)
      CRASH_ITERS="${1#*=}"
      shift
      ;;
    -h|--help)
      echo "Usage: $0 [--quick] [--crash-iters N]"
      echo "  --quick          Skip full workspace tests"
      echo "  --crash-iters N  Set CRASH_ITERS environment variable for tests"
      exit 0
      ;;
    *)
      echo "Unknown argument: $1" >&2
      exit 1
      ;;
  esac
done

if [[ -n "${CRASH_ITERS}" ]]; then
  export CRASH_ITERS
fi

TIMESTAMP="$(date -u +%Y%m%dT%H%M%SZ)"
SHORT_SHA="$(git rev-parse --short HEAD)"
EVIDENCE_DIR="evidence/${TIMESTAMP}-${SHORT_SHA}"

mkdir -p "${EVIDENCE_DIR}"

echo "================================================================="
echo " Contextra Evidence Generation"
echo " Output Directory : ${EVIDENCE_DIR}"
echo " Quick Mode       : ${QUICK_MODE}"
echo " Crash Iters      : ${CRASH_ITERS:-default}"
echo "================================================================="

# 1. Generate env.json
GIT_SHA="$(git rev-parse HEAD)"
GIT_STATUS="$(git status --porcelain)"
GIT_DIRTY=false
if [[ -n "${GIT_STATUS}" ]]; then
  GIT_DIRTY=true
fi

RUSTC_VV="$(rustc -vV 2>&1 || true)"
CARGO_V="$(cargo -V 2>&1 || true)"
UNAME_A="$(uname -a 2>&1 || true)"

CPU_MODEL="unknown"
if [[ -f /proc/cpuinfo ]]; then
  CPU_MODEL="$(grep -m1 "model name" /proc/cpuinfo | cut -d: -f2 | sed 's/^[ \t]*//' || echo "unknown")"
elif command -v sysctl >/dev/null 2>&1; then
  CPU_MODEL="$(sysctl -n machdep.cpu.brand_string 2>/dev/null || echo "unknown")"
fi

CPU_CORES="1"
if command -v nproc >/dev/null 2>&1; then
  CPU_CORES="$(nproc)"
elif [[ -f /proc/cpuinfo ]]; then
  CPU_CORES="$(grep -c ^processor /proc/cpuinfo || echo "1")"
fi

RAM_INFO="unknown"
if command -v free >/dev/null 2>&1; then
  RAM_INFO="$(free -h 2>&1 || true)"
elif [[ -f /proc/meminfo ]]; then
  RAM_INFO="$(grep MemTotal /proc/meminfo 2>&1 || true)"
fi

CARGO_SUBTOOLS="$(cargo --list 2>&1 || true)"

python3 -c "
import json, sys

data = {
    'git_sha': sys.argv[1],
    'git_dirty': sys.argv[2] == 'true',
    'git_status': sys.argv[3],
    'rustc_version': sys.argv[4],
    'cargo_version': sys.argv[5],
    'uname': sys.argv[6],
    'cpu_model': sys.argv[7],
    'cpu_cores': sys.argv[8],
    'ram_info': sys.argv[9],
    'cargo_subtools': sys.argv[10]
}

with open(sys.argv[11], 'w', encoding='utf-8') as f:
    json.dump(data, f, indent=2)
" "${GIT_SHA}" "${GIT_DIRTY}" "${GIT_STATUS}" "${RUSTC_VV}" "${CARGO_V}" "${UNAME_A}" "${CPU_MODEL}" "${CPU_CORES}" "${RAM_INFO}" "${CARGO_SUBTOOLS}" "${EVIDENCE_DIR}/env.json"

RESULTS_JSONL="${EVIDENCE_DIR}/results.jsonl"
rm -f "${RESULTS_JSONL}"

OVERALL_SUCCESS=true

check_test_target_exists() {
  local target_name="$1"
  if [[ -n "$(find . -type f -name "${target_name}.rs" -not -path "*/target/*" 2>/dev/null)" ]]; then
    return 0
  fi
  if cargo test --test "${target_name}" --locked -- --list >/dev/null 2>&1; then
    return 0
  fi
  return 1
}

record_step() {
  local step_num="$1"
  local step_id="$2"
  local cmd="$3"
  local is_guarantee_test="${4:-false}"
  local is_workspace_test="${5:-false}"

  local log_file="step_${step_num}_${step_id}.log"
  local log_path="${EVIDENCE_DIR}/${log_file}"

  echo -n "Running Step ${step_num} [${step_id}]... "

  local exit_code=0
  local passed=false
  local skipped=false
  local missing=false
  local start_time
  local end_time
  local duration_s=0.0

  if [[ "${is_workspace_test}" == "true" && "${QUICK_MODE}" == "true" ]]; then
    skipped=true
    passed=false
    echo "SKIPPED (--quick active)" > "${log_path}"
    echo "SKIPPED"
  elif [[ "${is_guarantee_test}" == "true" ]] && ! check_test_target_exists "${step_id}"; then
    missing=true
    passed=false
    exit_code=127
    echo "MISSING: Test target '${step_id}' not found in repository." > "${log_path}"
    OVERALL_SUCCESS=false
    echo "MISSING"
  else
    start_time="$(date +%s.%N 2>/dev/null || date +%s)"
    set +e
    eval "${cmd}" > "${log_path}" 2>&1
    exit_code=$?
    set -e
    end_time="$(date +%s.%N 2>/dev/null || date +%s)"

    duration_s="$(python3 -c "print(round(float('${end_time}') - float('${start_time}'), 3))" 2>/dev/null || echo "0.0")"

    if [[ ${exit_code} -eq 0 ]]; then
      passed=true
      echo "OK (${duration_s}s)"
    else
      passed=false
      OVERALL_SUCCESS=false
      echo "FAILED (exit code ${exit_code}, ${duration_s}s)"
    fi
  fi

  # Parse test counters from log_path
  python3 -c "
import json, re, sys

log_file = sys.argv[1]
passed_tests = 0
failed_tests = 0
ignored_tests = 0
skipped_tests = 0

try:
    with open(log_file, 'r', encoding='utf-8', errors='replace') as f:
        for line in f:
            m = re.search(r'test result: \w+\.\s+(\d+)\s+passed;\s+(\d+)\s+failed;\s+(\d+)\s+ignored', line)
            if m:
                passed_tests += int(m.group(1))
                failed_tests += int(m.group(2))
                ignored_tests += int(m.group(3))
            if 'SKIPPED' in line or '... skipped' in line:
                skipped_tests += 1
            if '#[ignore]' in line:
                ignored_tests += 1
except Exception:
    pass

record = {
    'step': sys.argv[2],
    'command': sys.argv[3],
    'exit_code': int(sys.argv[4]),
    'duration_s': float(sys.argv[5]),
    'log_file': sys.argv[6],
    'passed': sys.argv[7] == 'true',
    'skipped': sys.argv[8] == 'true',
    'missing': sys.argv[9] == 'true',
    'passed_tests': passed_tests,
    'failed_tests': failed_tests,
    'ignored_tests': ignored_tests,
    'skipped_tests': skipped_tests
}

with open(sys.argv[10], 'a', encoding='utf-8') as out:
    out.write(json.dumps(record) + '\n')
" "${log_path}" "${step_id}" "${cmd}" "${exit_code}" "${duration_s}" "${log_file}" "${passed}" "${skipped}" "${missing}" "${RESULTS_JSONL}"
}

# Steps Execution
record_step "01" "cargo_fmt" "cargo fmt --check"
record_step "02" "cargo_clippy" "cargo clippy --workspace --all-targets --locked -- -D warnings"
record_step "03" "cargo_test_workspace" "cargo test --workspace --locked" false true
record_step "04" "crash_kill_durability" "cargo test --test crash_kill_durability --locked" true false
record_step "05" "crypto_vectors_extended" "cargo test --test crypto_vectors_extended --locked" true false
record_step "06" "guarantee_deletion_irrecoverable" "cargo test --test guarantee_deletion_irrecoverable --locked" true false
record_step "07" "guarantee_audit_chain_tamper" "cargo test --test guarantee_audit_chain_tamper --locked" true false
record_step "08" "e2e_compliance_scenario" "cargo test --test e2e_compliance_scenario --locked" true false
record_step "09" "check_unsafe_islands" "cargo xtask check-unsafe-islands"
record_step "10" "check_ring_layering" "cargo xtask check-ring-layering"
record_step "11" "check_dag" "cargo xtask check-dag"
record_step "12" "check_consistency" "cargo xtask check-consistency"
record_step "13" "cargo_deny" "cargo deny check"

# Generate MANIFEST.sha256
(
  cd "${EVIDENCE_DIR}"
  sha256sum env.json results.jsonl *.log > MANIFEST.sha256
)

echo "================================================================="
echo " Evidence Run Summary"
echo " Results written to : ${RESULTS_JSONL}"
echo " Manifest written to: ${EVIDENCE_DIR}/MANIFEST.sha256"
echo " Overall Status     : $( [[ "${OVERALL_SUCCESS}" == "true" ]] && echo "PASSED" || echo "FAILED / INCOMPLETE" )"
echo "================================================================="

if [[ "${OVERALL_SUCCESS}" == "true" ]]; then
  exit 0
else
  exit 1
fi
