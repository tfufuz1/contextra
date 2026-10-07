#!/usr/bin/env bash
set -euo pipefail

YEAR_WEEK="${YEAR_WEEK:-${1:-$(date -u +'%Y-W%V')}}"
DATE="${DATE:-$(date -u +'%Y-%m-%d')}"
TASK_FILE="${TASK_FILE:-${2:-.jules/tasks/AUDIT-${YEAR_WEEK}.toml}}"

if [ -n "${GITHUB_OUTPUT:-}" ]; then
  echo "task_date=${YEAR_WEEK}" >> "$GITHUB_OUTPUT"
  echo "task_file=${TASK_FILE}" >> "$GITHUB_OUTPUT"
fi

if [ -f "$TASK_FILE" ]; then
  echo "::notice::Audit-Task-Datei ${TASK_FILE} existiert bereits für ISO-Woche ${YEAR_WEEK}. Überspringe Erstellung."
  if [ -n "${GITHUB_OUTPUT:-}" ]; then
    echo "task_created=false" >> "$GITHUB_OUTPUT"
  fi
  exit 0
fi

mkdir -p .jules/tasks

if [ -z "${SHA_LIST:-}" ]; then
  SHAS=$(git log --since="7 days ago" --format="%H" --no-merges)
  if [ -z "$SHAS" ]; then
    SHA_LIST='# Keine Commits in den letzten 7 Tagen'
  else
    SHA_LIST=$(echo "$SHAS" | sed 's/^/"/;s/$/",/')
  fi
fi

cat << EOF > "$TASK_FILE"
# Audit-Task-Karte
# ID Format: AUDIT-YYYY-Www
id        = "AUDIT-${YEAR_WEEK}"
title     = "Woechentlicher Proaktiv-Audit-Auftrag ${YEAR_WEEK}"
crate     = "workspace"
tier      = 1
risk      = "sec"
goal      = "Proaktive Analyse der Commits der letzten 7 Tage auf Race Conditions, unwrap-Regressions, Silent Failures und DAG-Grenzverletzungen."
non_goals = ["Refactoring ausserhalb verifizierter Befunde", "Stilles Ignorieren nicht verifizierter Findings"]
scope     = ["crates/**", "xtask/**"]
forbidden = ["capabilities.toml", "AGENTS.md", "rust-toolchain.toml", "deny.toml", ".jules/setup/**"]
invariants = ["Zero-Panic", "P28 Determinismus", "Audit Intake Protocol"]
acceptance = [
  "cargo test --workspace --locked",
  "cargo xtask check-ring-layering",
  "cargo xtask panic-inventory"
]
evidence_required = ["Verifizierte Befunde mit Test oder Entkraeftungsbegruendung im PR-Text"]

[audit_context]
created_date = "${DATE}"
instructions = "Lies .jules/AUDIT_INTAKE_PROTOCOL.md. Führe 'git show <sha>' für die aufgeführten commit_shas aus, um Diff/Änderungen zu analysieren. Jeder Fund muss gegen den AKTUELLEN Quellcode verifiziert werden."
commit_shas = [
${SHA_LIST}
]

[budget]
files = 10
lines = 300
iterations = 10
EOF

if [ -n "${GITHUB_OUTPUT:-}" ]; then
  echo "task_created=true" >> "$GITHUB_OUTPUT"
fi
