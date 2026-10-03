#!/usr/bin/env bash
# =============================================================================
# Contextra — Kontext-Injektion (billig, idempotent, bei JEDEM Task-Start)
# Schreibt NUR nach .jules/context/ (gitignored) → verschmutzt keinen PR.
# Ausgabe: .jules/context/SESSION_START.md  (+ ENV_CONTEXT_PACK.md, RECENT_PRS.md)
# =============================================================================
set -uo pipefail
. "$HOME/.cargo/env" 2>/dev/null || true
cd "$(git rev-parse --show-toplevel 2>/dev/null || echo .)" || exit 0
CTX=.jules/context; mkdir -p "$CTX"
XT=(cargo run -q --manifest-path xtask/Cargo.toml --)

# A) Git-Historie sicherstellen (Jules-Checkouts können flach sein)
if [ "$(git rev-parse --is-shallow-repository 2>/dev/null)" = true ]; then
  git fetch --unshallow --quiet 2>/dev/null || git fetch --depth=50 --quiet 2>/dev/null \
    || echo "⚠️ Historie nicht nachladbar — Commit-Liste evtl. unvollständig"
fi

# B) Repo-weiter Context-Pack (HEAD, Claims, offene Tags, letzte 10 Commits)
"${XT[@]}" context-pack --output="$CTX/ENV_CONTEXT_PACK.md" 2>&1 | tail -n 2 || true

# C) Pre-flight (schnell); darf scheitern, wenn Token/Netz fehlen
"${XT[@]}" jules-preflight --fast >"$CTX/PREFLIGHT.txt" 2>&1 || echo "ℹ️ jules-preflight meldete Befunde → $CTX/PREFLIGHT.txt"

# D) PR-Titel (best effort; öffentliches Repo geht auch ohne Token, 60 req/h)
RAW=$(curl -fsS --max-time 8 ${GITHUB_TOKEN:+-H "Authorization: Bearer $GITHUB_TOKEN"} \
  -H "Accept: application/vnd.github+json" -H "User-Agent: contextra-jules-env" \
  "https://api.github.com/repos/tfufuz1/contextra/pulls?state=all&sort=updated&direction=desc&per_page=15" 2>/dev/null) || RAW=""
if echo "$RAW" | jq -e 'type=="array"' >/dev/null 2>&1; then
  echo "$RAW" | jq -r '.[] | "- #\(.number) [\(.state)] \(.title)"' > "$CTX/RECENT_PRS.md"
else rm -f "$CTX/RECENT_PRS.md"; fi

# E) Ein einziges Einstiegsdokument für den Agenten
{
  echo "# SESSION_START — $(date -u '+%F %T UTC')"
  echo "HEAD: $(git rev-parse --short HEAD) auf $(git rev-parse --abbrev-ref HEAD)"
  echo; echo "## Pflicht vor dem ersten Edit"
  echo "AGENTS.md (§4 Gates, §6 Invarianten) · VETOES.md · .jules/SESSION_BOOTSTRAP.md"
  echo "Geschützte Pfade NICHT ändern: .github/** xtask/** justfile rust-toolchain.toml deny.toml capabilities.toml AGENTS.md .jules/setup/**"
  echo; echo "## Letzte Commits"; git log -10 --format='- %h %s'
  [ -f "$CTX/RECENT_PRS.md" ] && { echo; echo "## Letzte PRs"; cat "$CTX/RECENT_PRS.md"; }
  echo; echo "## Offene CRITICAL/BLOCKER-AI-Tags"
  grep -rn 'AI-TAG\[[A-Z]*\]\[\(CRITICAL\|BLOCKER\)\]' crates/ --include='*.rs' 2>/dev/null | grep -v RESOLVED | head -20 || true
  echo; echo "## Aktive Claims / Pack"; sed -n '/## Active Claims/,/## Recent Commits/p' "$CTX/ENV_CONTEXT_PACK.md" 2>/dev/null | head -20
  echo; echo "## WORKING_STATE (Kopf)"; head -30 WORKING_STATE.md 2>/dev/null
  [ -s "$CTX/PREFLIGHT.txt" ] && { echo; echo "## Preflight (Ende)"; tail -n 15 "$CTX/PREFLIGHT.txt"; }
} > "$CTX/SESSION_START.md"
echo "✅ Kontext geschrieben: $CTX/SESSION_START.md ($(wc -l < "$CTX/SESSION_START.md") Zeilen)"
