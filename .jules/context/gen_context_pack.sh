#!/usr/bin/env bash
# Erzeugt .jules/context/CONTEXT_PACK.md — alles, was eine Session initial braucht,
# in EINER Datei, EINEM read_file-Aufruf.
set -euo pipefail

OUT=".jules/context/CONTEXT_PACK.md"
mkdir -p .jules/context

{
  echo "# Context Pack (generiert: $(date -u +%FT%TZ))"
  echo "## Aktueller Branch & Status"
  git status -s 2>/dev/null || echo "git status unkonfigurieren"
  git log -n 5 --oneline 2>/dev/null || true
  echo "## Offene Claims"
  if [ -f .jules/claims.json ]; then
    cat .jules/claims.json
  else
    echo "[]"
  fi
  echo "## Offene AI-TAGs (kritisch zuerst)"
  rg -n 'AI-TAG\[\w+\]\[(CRITICAL|MAJOR)\]' crates/ --include='*.rs' 2>/dev/null | grep -v RESOLVED || echo "  ✅ Keine"
  echo "## Governance-Kurzstatus"
  echo "unwrap-baseline: $(jq '.count' .unwrap-baseline.json 2>/dev/null || echo n/a)"
  echo "## Relevante AGENTS.md (root + touched crates)"
  if [ -f AGENTS.md ]; then
    cat AGENTS.md
  fi
} > "$OUT"

echo "Context Pack geschrieben: $OUT ($(wc -l < "$OUT") Zeilen)"
