#!/usr/bin/env bash
# refresh-context.sh — Generiert SESSION_START.md für Jules
# doc-stamp: 2026-10-26 | Normativ: AGENTS.md §2a
# Wird von .github/workflows/refresh-jules-context.yml ausgeführt.
set -euo pipefail

OUTPUT="agents/jules/context/SESSION_START.md"
COMMIT=$(git rev-parse --short HEAD 2>/dev/null || echo "unknown")
TIMESTAMP=$(date -u +"%Y-%m-%dT%H:%M:%SZ")

mkdir -p "$(dirname "$OUTPUT")"

cat > "$OUTPUT" <<EOF
# SESSION_START — ${TIMESTAMP} — ${COMMIT}
<!-- Automatisch generiert von agents/jules/setup/refresh-context.sh -->
<!-- NICHT manuell bearbeiten — wird bei jedem CI-Run überschrieben -->

## Gate-Status (cargo xtask)
\`\`\`
$(cargo run --manifest-path xtask/Cargo.toml -- check-ring-layering --strict 2>&1 | tail -5 || echo "⚠️ ring-layering: Fehler beim Ausführen")
\`\`\`

## Offene Jules-Issues
$(gh issue list --state open --label "jules" --json number,title,milestone 2>/dev/null \
  | jq -r '.[] | "- #\(.number) \(.title)"' 2>/dev/null \
  || echo "_(gh CLI nicht verfügbar oder keine offenen Issues)_")

## Offene PRs
$(gh pr list --state open --json number,title,author 2>/dev/null \
  | jq -r '.[] | "- PR #\(.number): \(.title) (@\(.author.login))"' 2>/dev/null \
  || echo "_(keine offenen PRs)_")

## Letzte Commits
$(git log --oneline -5 2>/dev/null || echo "_(kein git log verfügbar)_")

## Ratchet-Baselines (governance/ratchet.toml)
- unwrap_exact: $(grep unwrap_exact governance/ratchet.toml 2>/dev/null | cut -d= -f2 | tr -d ' ' || echo "?")
- expect_exact: $(grep expect_exact governance/ratchet.toml 2>/dev/null | cut -d= -f2 | tr -d ' ' || echo "?")

## CRITICAL-Tags im Code
$(grep -rn "AI-TAG\[.*\]\[BLOCKER\]\|AI-TAG\[.*\]\[CRITICAL\]" crates/ --include='*.rs' 2>/dev/null | grep -v RESOLVED | head -10 || echo "  (keine)")
EOF

echo "✅ SESSION_START.md generiert: ${OUTPUT}"
