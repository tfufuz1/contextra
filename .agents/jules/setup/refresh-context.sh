#!/usr/bin/env bash
# =============================================================================
# Contextra – Kontext-Injektion (billig, idempotent, bei JEDEM Sitzungsstart)
#
# Schreibt NUR nach .agents/jules/context/SESSION_START.md (gitignored, nicht versioniert).
# Verändert den Arbeitsbaum NICHT: kein reset, kein clean, kein checkout.
#
# Quellen: Repository (Arbeitsliste, Git-Historie, Rust-Pin), GitHub (optional, via gh
# oder öffentlicher API). Fehlt GitHub, bleibt der Abschnitt leer; die Sitzung läuft weiter.
# =============================================================================
set -uo pipefail
. "$HOME/.cargo/env" 2>/dev/null || true
REPO_ROOT="$(git rev-parse --show-toplevel 2>/dev/null || echo .)"
cd "$REPO_ROOT" || exit 0
CTX=.agents/jules/context
OUT="$CTX/SESSION_START.md"
mkdir -p "$CTX"

# ── Repository-Slug aus origin (nicht fest verdrahtet) ───────────────────────
ORIGIN_URL="$(git remote get-url origin 2>/dev/null || true)"
SLUG="$(printf '%s' "$ORIGIN_URL" | sed -E 's#^(git@github.com:|https://github.com/)##; s#\.git$##')"

# ── A) Historie: nur begrenzt nachladen, kein --unshallow (Policy-Hooks können es sperren)
if [ "$(git rev-parse --is-shallow-repository 2>/dev/null)" = true ]; then
  git fetch --depth=500 --quiet origin 2>/dev/null || echo "⚠️ Historie nicht nachladbar (flacher Stand)"
fi

# ── B) GitHub-Daten: gh bevorzugt, öffentliche API als Fallback, sonst leer ─────
GH_OK=false
command -v gh >/dev/null 2>&1 && gh auth status >/dev/null 2>&1 && GH_OK=true

api() {   # $1 = API-Pfad unterhalb /repos/<slug>/, $2 = jq-Filter
  local url="https://api.github.com/repos/${SLUG}/$1"
  local args=(--max-time 8 -H "Accept: application/vnd.github+json" -H "User-Agent: contextra-jules")
  [ -n "${GITHUB_TOKEN:-}" ] && args+=(-H "Authorization: Bearer $GITHUB_TOKEN")
  curl -fsS "${args[@]}" "$url" 2>/dev/null | jq -r "$2" 2>/dev/null || true
}

if $GH_OK; then
  OPEN_PRS=$(gh pr list --state open --limit 10 --json number,title,isDraft,headRefName \
    --jq '.[] | "- #\(.number) \(if .isDraft then "[DRAFT] " else "" end)\(.title) (\(.headRefName))"' 2>/dev/null || true)
  CI_RUNS=$(gh run list --limit 8 --json name,conclusion,headBranch,updatedAt \
    --jq '.[] | "\(if .conclusion == "failure" then "❌" elif .conclusion == "success" then "✅" else "⏳" end) \(.name) [\(.headBranch)] \(.updatedAt[:10])"' 2>/dev/null || true)
else
  OPEN_PRS=$(api "pulls?state=open&per_page=10" '.[] | "- #\(.number) \(.title) (\(.head.ref))"')
  CI_RUNS=$(api "actions/runs?per_page=8" '.workflow_runs[] | "\(if .conclusion == "failure" then "❌" elif .conclusion == "success" then "✅" else "⏳" end) \(.name) [\(.head_branch)] \(.updated_at[:10])"')
fi


# ── D) Rust-Pin und Werkzeuge ───────────────────────────────────────────────
PIN="$(sed -n 's/^channel *= *"\(.*\)"/\1/p' rust-toolchain.toml 2>/dev/null | head -1)"

# ── E) Einstiegsdokument ─────────────────────────────────────────────────────
{
  echo "# SESSION_START — $(date -u '+%F %T UTC')"
  echo "HEAD: $(git rev-parse --short HEAD 2>/dev/null || echo n/a) auf $(git rev-parse --abbrev-ref HEAD 2>/dev/null || echo n/a)"
  echo "Rust: $(rustc --version 2>/dev/null || echo 'nicht installiert – setup ausführen') · Pin: ${PIN:-n/a}"
  echo "Arbeitsbaum: $(git status --porcelain 2>/dev/null | wc -l | tr -d ' ') geänderte Pfade (wird NICHT verändert)"
  echo

  echo "## ⚠️ Unveränderliche Regeln (Details: AGENTS.md, docs/VETOES.md)"
  echo "- Rangfolge: docs/SPEC.md > docs/VETOES.md > docs/CONSTITUTION.md > AGENTS.md > .agents/"
  echo ""
  echo


  echo "## 🔀 Offene PRs"
  if [ -n "$OPEN_PRS" ]; then echo "$OPEN_PRS"; else echo "(keine oder GitHub nicht erreichbar)"; fi
  echo

  echo "## 📜 Letzte 15 Commits"
  git log -5 --format='- %h %as %s (%an)' 2>/dev/null || echo "(keine Historie)"
  echo

} > "$OUT"

echo "✅ Kontext geschrieben: $OUT ($(wc -l < "$OUT") Zeilen)"
exit 0
