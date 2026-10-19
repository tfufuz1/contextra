#!/usr/bin/env bash
# =============================================================================
# Contextra — Kontext-Injektion (billig, idempotent, bei JEDEM Task-Start)
# Schreibt NUR nach .jules/context/ (gitignored) → verschmutzt keinen PR.
# Ausgabe: .jules/context/SESSION_START.md (+ ENV_CONTEXT_PACK.md, GITHUB_STATE.md, RECENT_PRS.md)
# =============================================================================
set -uo pipefail
# shellcheck disable=SC1091
. "$HOME/.cargo/env" 2>/dev/null || true
cd "$(git rev-parse --show-toplevel 2>/dev/null || echo .)" || exit 0
CTX=.jules/context; mkdir -p "$CTX"
export PATH="$HOME/.local/bin:$PATH"   # flatc-Shim (siehe environment_script.sh)
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

# D) GitHub-Zustand via Git-Branch jules-state (rate-limit-frei), Fallback REST API
FETCH_STATE_OK=false
if git fetch origin jules-state --depth=1 >/dev/null 2>&1 && \
   git show origin/jules-state:STATE.md > "$CTX/GITHUB_STATE.md" 2>/dev/null; then
  FETCH_STATE_OK=true
else
  rm -f "$CTX/GITHUB_STATE.md"
fi

# Fallback auf unauthentifizierte REST-API nur wenn jules-state nicht geladen werden konnte
if [ "$FETCH_STATE_OK" = false ]; then
  RAW=$(curl -fsS --max-time 8 ${GITHUB_TOKEN:+-H "Authorization: Bearer $GITHUB_TOKEN"} \
    -H "Accept: application/vnd.github+json" -H "User-Agent: contextra-jules-env" \
    "https://api.github.com/repos/tfufuz1/contextra/pulls?state=all&sort=updated&direction=desc&per_page=15" 2>/dev/null) || RAW=""
  if echo "$RAW" | jq -e 'type=="array"' >/dev/null 2>&1; then
    echo "$RAW" | jq -r '.[] | "- #\(.number) [\(.state)] \(.title)"' > "$CTX/RECENT_PRS.md"
  else
    rm -f "$CTX/RECENT_PRS.md"
  fi
fi

# E) Ein einziges Einstiegsdokument für den Agenten (SESSION_START.md)
{
  echo "# SESSION_START — $(date -u '+%F %T UTC')"
  echo "HEAD: $(git rev-parse --short HEAD) auf $(git rev-parse --abbrev-ref HEAD)"
  echo; echo "## Pflicht vor dem ersten Edit"
  echo "AGENTS.md (§4 Gates, §6 Invarianten) · VETOES.md · .jules/SESSION_BOOTSTRAP.md"
  echo "Geschützte Pfade NICHT ändern: .github/** xtask/** justfile rust-toolchain.toml deny.toml capabilities.toml AGENTS.md .jules/setup/**"

  echo; echo "## GitHub-Zustand"
  if [ -s "$CTX/GITHUB_STATE.md" ]; then
    echo "### Milestones"
    sed -n '/## Milestones/,/## Open Issues/p' "$CTX/GITHUB_STATE.md" | grep '^- ' || echo "_Keine Milestones_"

    echo; echo "### Nächstes jules:ready-Issue"
    NEXT_ISSUE=$(sed -n '/## Open Issues (jules:ready)/,/## Open Pull Requests/p' "$CTX/GITHUB_STATE.md" | grep '^- #' | head -n 1 || true)
    if [ -n "$NEXT_ISSUE" ]; then
      echo "$NEXT_ISSUE"
    else
      echo "_Kein offenes jules:ready Issue im aktuellen Milestone._"
    fi

    echo; echo "### Offene PRs & Dateiüberschneidungen"
    # Scope/Dateien aus Task-Karten (.jules/tasks/T-*.toml) & Status ermitteln
    TASK_FILES=$(grep -rn '^scope\s*=' .jules/tasks/ 2>/dev/null | grep -o '"[^"]*"' | tr -d '"' | grep -v '\*' || true)
    MODIFIED_FILES=$(git status --porcelain 2>/dev/null | awk '{print $2}' || true)
    ALL_TARGET_FILES=$(printf "%s\n%s" "$TASK_FILES" "$MODIFIED_FILES" | sort -u | grep -v '^$' || true)

    OVERLAPS_FOUND=false
    if [ -n "$ALL_TARGET_FILES" ]; then
      while read -r target_file; do
        [ -z "$target_file" ] && continue
        matching_prs=$(grep -B 10 -F "  - $target_file" "$CTX/GITHUB_STATE.md" | grep '^### PR #' || true)
        if [ -n "$matching_prs" ]; then
          while read -r pr_line; do
            [ -z "$pr_line" ] && continue
            echo "- $pr_line (Überschneidung bei \`$target_file\`)"
            OVERLAPS_FOUND=true
          done <<< "$matching_prs"
        fi
      done <<< "$ALL_TARGET_FILES"
    fi

    if [ "$OVERLAPS_FOUND" = false ]; then
      echo "_Keine Dateiüberschneidungen mit offenen PRs ermittelt._"
    fi
  else
    echo "_GitHub-Zustand via jules-state-Branch nicht verfügbar (Fallback-Modus)._"
    [ -f "$CTX/RECENT_PRS.md" ] && { echo; echo "### Letzte PRs (via REST API)"; cat "$CTX/RECENT_PRS.md"; }
  fi

  echo; echo "## Letzte Commits"; git log -10 --format='- %h %s'
  echo; echo "## Offene CRITICAL/BLOCKER-AI-Tags"
  grep -rn 'AI-TAG\[[A-Z]*\]\[\(CRITICAL\|BLOCKER\)\]' crates/ --include='*.rs' 2>/dev/null | grep -v RESOLVED | head -20 || true
  echo; echo "## Aktive Claims / Pack"; sed -n '/## Active Claims/,/## Recent Commits/p' "$CTX/ENV_CONTEXT_PACK.md" 2>/dev/null | head -20
  echo; echo "## WORKING_STATE (Kopf)"; head -30 WORKING_STATE.md 2>/dev/null
  [ -s "$CTX/PREFLIGHT.txt" ] && { echo; echo "## Preflight (Ende)"; tail -n 15 "$CTX/PREFLIGHT.txt"; }
} > "$CTX/SESSION_START.md"
echo "✅ Kontext geschrieben: $CTX/SESSION_START.md ($(wc -l < "$CTX/SESSION_START.md") Zeilen)"

# Working Tree sauber halten (xtask/build.rs dürfen nichts Getracktes hinterlassen)
# Note: only clean untracked files that are not staged
git reset --hard HEAD -q 2>/dev/null
git reset --hard HEAD 2>&1 | tail -1
