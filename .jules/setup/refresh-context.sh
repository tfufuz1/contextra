#!/usr/bin/env bash
# =============================================================================
# Contextra — Kontext-Injektion (billig, idempotent, bei JEDEM Task-Start)
# Schreibt NUR nach .jules/context/ (gitignored) → verschmutzt keinen PR.
# Ausgaben: .jules/context/REPO_STATE.md (+ Kopie SESSION_START.md)
# =============================================================================
set -uo pipefail
. "$HOME/.cargo/env" 2>/dev/null || true
cd "$(git rev-parse --show-toplevel 2>/dev/null || echo .)" || exit 0
CTX=.jules/context; mkdir -p "$CTX"
export PATH="$HOME/.local/bin:$PATH"   # flatc-Shim (siehe environment_script.sh)
XT=(cargo run -q --manifest-path xtask/Cargo.toml --)

DO_RESET=false
IS_AGENT=false

for arg in "$@"; do
  case "$arg" in
    --reset) DO_RESET=true ;;
    --agent) IS_AGENT=true ;;
  esac
done

STATUS_BEFORE=$(git status --porcelain 2>/dev/null || true)

# A) Git-Historie sicherstellen (Jules-Checkouts können flach sein)
if [ "$(git rev-parse --is-shallow-repository 2>/dev/null)" = true ]; then
  if [ "$IS_AGENT" = true ]; then
    timeout 10 git fetch --unshallow --quiet 2>/dev/null || timeout 10 git fetch --depth=50 --quiet 2>/dev/null || echo "⚠️ Historie nicht nachladbar — Commit-Liste evtl. unvollständig"
  else
    git fetch --unshallow --quiet 2>/dev/null || git fetch --depth=50 --quiet 2>/dev/null || echo "⚠️ Historie nicht nachladbar — Commit-Liste evtl. unvollständig"
  fi
fi

# B) Repo-weiter REPO_STATE / Context-Pack (HEAD, Claims, offene Tags, Commits etc.)
"${XT[@]}" context-pack --output="$CTX/REPO_STATE.md" 2>&1 | tail -n 2 || true

# C) Pre-flight (schnell); darf scheitern, wenn Token/Netz fehlen; im --agent Modus übersprungen
if [ "$IS_AGENT" = false ]; then
  "${XT[@]}" jules-preflight --fast >"$CTX/PREFLIGHT.txt" 2>&1 || echo "ℹ️ jules-preflight meldete Befunde → $CTX/PREFLIGHT.txt"
fi

# D) SESSION_START.md ist eine exakte Kopie von REPO_STATE.md
if [ -f "$CTX/REPO_STATE.md" ]; then
  cp "$CTX/REPO_STATE.md" "$CTX/SESSION_START.md"
fi

echo "✅ Kontext geschrieben: $CTX/REPO_STATE.md ($(wc -l < "$CTX/REPO_STATE.md" 2>/dev/null || echo 0) Zeilen)"

if [ "$DO_RESET" = true ]; then
  git reset --hard HEAD -q 2>/dev/null && git clean -fdq 2>/dev/null
  git reset --hard HEAD 2>&1 | tail -1
else
  STATUS_AFTER=$(git status --porcelain 2>/dev/null || true)
  if [ "$STATUS_BEFORE" != "$STATUS_AFTER" ]; then
    echo "⚠️ Warnung: Working Tree wurde während refresh-context verändert!" >&2
  fi
fi
