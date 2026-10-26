#!/usr/bin/env bash
# =============================================================================
# Contextra — Kontext-Injektion (billig, idempotent, bei JEDEM Task-Start)
# Schreibt NUR nach .agents/jules/context/ (gitignored) → verschmutzt keinen PR.
#
# Philosophie: GitHub IST der agile Kontext (Issues = Tasks, PRs = Reviews,
# Runs = CI-Status, Labels = Sprint-Signale). Diese Datei zieht alles live
# ab. Statische Repo-Dateien enthalten NUR unveränderliche Regeln.
#
# Ausgabe: .agents/jules/context/SESSION_START.md (eine Datei, alles drin)
# =============================================================================
set -uo pipefail
. "$HOME/.cargo/env" 2>/dev/null || true
cd "$(git rev-parse --show-toplevel 2>/dev/null || echo .)" || exit 0
CTX=.agents/jules/context; mkdir -p "$CTX"
export PATH="$HOME/.local/bin:$PATH"   # flatc-Shim (siehe environment_script.sh)

# ── A) Git-Historie sicherstellen ─────────────────────────────────────────────
if [ "$(git rev-parse --is-shallow-repository 2>/dev/null)" = true ]; then
  git fetch --unshallow --quiet 2>/dev/null || git fetch --depth=50 --quiet 2>/dev/null \
    || echo "⚠️ Historie nicht nachladbar"
fi

# ── B) GitHub-Daten via gh CLI (bevorzugt) oder API-Fallback ─────────────────
# gh CLI ist in der Jules-VM verfügbar und nutzt GITHUB_TOKEN automatisch
GH_OK=false; command -v gh >/dev/null 2>&1 && gh auth status >/dev/null 2>&1 && GH_OK=true

gh_or_api() {
  # $1 = gh-Befehl (Array-String), $2 = API-URL-Suffix, $3 = jq-Filter
  local result=""
  if $GH_OK; then
    result=$(eval "$1" 2>/dev/null) || result=""
  fi
  if [ -z "$result" ]; then
    local url="https://api.github.com/repos/tfufuz1/contextra/$2"
    local args=(--max-time 8 -H "Accept: application/vnd.github+json" -H "User-Agent: contextra-jules-env")
    [ -n "${GITHUB_TOKEN:-}" ] && args+=(-H "Authorization: Bearer $GITHUB_TOKEN")
    result=$(curl -fsS "${args[@]}" "$url" 2>/dev/null | jq -r "${3}" 2>/dev/null) || result=""
  fi
  echo "$result"
}

# Task-Issues (Label: jules, task, priority-p0, enhancement, bug)
TASK_ISSUES=$(gh_or_api \
  "gh issue list --state open --label 'jules,task,priority-p0,enhancement,bug' --limit 20 --json number,title,labels,milestone,assignees,body" \
  "issues?state=open&labels=jules,task,priority-p0&sort=updated&direction=desc&per_page=20" \
  '.[] | "- #\(.number) [\(.labels | map(.name) | join(","))] \(.title)"')

# Alle offenen Issues (agiler Backlog)
ALL_OPEN_ISSUES=$(gh_or_api \
  "gh issue list --state open --limit 15 --json number,title,labels,milestone,assignees" \
  "issues?state=open&sort=updated&direction=desc&per_page=15" \
  '.[] | select(.pull_request == null) | "- #\(.number) \(.title) [\(.labels | map(.name) | join(","))]"')

# Offene PRs (Review-Bedarf, laufende Arbeiten)
OPEN_PRS=$(gh_or_api \
  "gh pr list --state open --limit 10 --json number,title,labels,isDraft,headRefName,reviews" \
  "pulls?state=open&sort=updated&direction=desc&per_page=10" \
  '.[] | "- #\(.number) \(if .draft then "[DRAFT] " else "" end)\(.title) (\(.head.ref // .headRefName // "?"))"')

# Letzte 10 PRs (Merge-Historie = was wurde zuletzt gemacht)
RECENT_PRS=$(gh_or_api \
  "gh pr list --state all --limit 10 --json number,title,state,mergedAt,headRefName" \
  "pulls?state=all&sort=updated&direction=desc&per_page=10" \
  '.[] | "- #\(.number) [\(.state)] \(.title)"')

# CI-Status: letzte Runs (alle Workflows, nicht nur eine)
CI_RUNS=$(gh_or_api \
  "gh run list --limit 8 --json name,status,conclusion,headBranch,event,updatedAt,databaseId" \
  "actions/runs?per_page=8" \
  '.workflow_runs[] | "\(if .conclusion == "failure" then "❌" elif .conclusion == "success" then "✅" else "⏳" end) \(.name) [\(.head_branch)] \(.updated_at[:10])"')

# Fehlgeschlagene CI-Jobs der letzten Runs (konkreter Fehlerort)
FAILED_RUN_ID=""
if $GH_OK; then
  FAILED_RUN_ID=$(gh run list --limit 5 --json databaseId,conclusion 2>/dev/null \
    | jq -r '[.[] | select(.conclusion == "failure")] | .[0].databaseId' 2>/dev/null) || FAILED_RUN_ID=""
fi

FAILED_JOBS=""
if [ -n "$FAILED_RUN_ID" ] && [ "$FAILED_RUN_ID" != "null" ]; then
  FAILED_JOBS=$(gh run view "$FAILED_RUN_ID" --json jobs 2>/dev/null \
    | jq -r '.jobs[] | select(.conclusion == "failure") | "  ❌ Job: \(.name)\n  Schritt: \(.steps | map(select(.conclusion == "failure")) | .[0].name // "?")"' 2>/dev/null | head -20) || FAILED_JOBS=""
fi

# Milestones (Sprints)
MILESTONES=$(gh_or_api \
  "gh api repos/tfufuz1/contextra/milestones --jq '.[] | \"- M\(.number) \(.title) (\(.open_issues) open, due: \(.due_on // \"∞\"))\"'" \
  "milestones?state=open" \
  '.[] | "- M\(.number) \(.title) (\(.open_issues) open)"')

# ── C) xtask context-pack (Claims, Tag-Inventar) ──────────────────────────────
XT=(cargo run -q --manifest-path xtask/Cargo.toml --)
CONTEXT_PACK=$("${XT[@]}" context-pack 2>/dev/null) || CONTEXT_PACK=""

# ── D) Preflight (Gate-Status) ────────────────────────────────────────────────
PREFLIGHT=$("${XT[@]}" jules-preflight --fast 2>/dev/null) || PREFLIGHT=""

# ── E) Ein einziges, maximales Einstiegsdokument ──────────────────────────────
{
  echo "# SESSION_START — $(date -u '+%F %T UTC')"
  echo "HEAD: $(git rev-parse --short HEAD) auf $(git rev-parse --abbrev-ref HEAD)"
  echo "Rust: $(rustc --version 2>/dev/null || echo 'n/a')"
  echo "gh CLI: $(gh --version 2>/dev/null | head -1 || echo 'n/a')"
  echo

  # ── Pflicht-Kurzregeln (Regeln stehen in AGENTS.md — hier nur Stop-Signale) ─
  echo "## ⚠️ Unveränderliche Regeln (Details: AGENTS.md, VETOES.md, capabilities.toml)"
  echo "- Rangfolge: docs/SPEC.md > VETOES.md > capabilities.toml > AGENTS.md"
  echo "- Geschützte Pfade (NIE ändern): .github/** xtask/** justfile rust-toolchain.toml deny.toml capabilities.toml AGENTS.md .agents/jules/setup/**"
  echo "- Zero-Panic: kein unwrap()/expect()/panic!() in Produktionspfaden (§3)"
  echo "- Unsafe nur in: contextra-simd, contextra-sys, contextra-wire"
  echo "- Vetos prüfen: VETOES.md F-NN vor jeder Feature-Arbeit"
  echo

  # ── STOP-Bedingungen (sofort stoppen) ─────────────────────────────────────
  echo "## 🚨 STOP-Bedingungen (vor allem anderen prüfen)"
  BLOCKERS=$(grep -rn 'AI-TAG\[[A-Z]*\]\[\(CRITICAL\|BLOCKER\)\]' crates/ --include='*.rs' 2>/dev/null | grep -v RESOLVED | head -10)
  if [ -n "$BLOCKERS" ]; then
    echo "❌ CRITICAL/BLOCKER-Tags gefunden → STOP, erst beheben:"
    echo "$BLOCKERS"
  else
    echo "✅ Keine CRITICAL/BLOCKER-Tags"
  fi
  # CI-Hauptzweig rot?
  MAIN_RED=$(echo "$CI_RUNS" | grep -c "❌" 2>/dev/null || echo 0)
  [ "$MAIN_RED" -gt 0 ] && echo "⚠️  $MAIN_RED CI-Workflow(e) auf main fehlgeschlagen (Details: CI-Status-Abschnitt unten)"
  echo

  # ── GITHUB: Aktiver Sprint / Milestones ───────────────────────────────────
  echo "## 📅 Aktive Milestones (Sprints)"
  [ -n "$MILESTONES" ] && echo "$MILESTONES" || echo "(keine Milestones)"
  echo

  # ── GITHUB: Task-Issues (was Jules jetzt tun soll) ────────────────────────
  echo "## 🎯 Task-Issues (Label: jules / task / priority-p0)"
  [ -n "$TASK_ISSUES" ] && echo "$TASK_ISSUES" || echo "(keine offenen Task-Issues)"
  echo "Volltext abrufen: gh issue view <N>"
  echo

  # ── GITHUB: Gesamter offener Backlog ──────────────────────────────────────
  echo "## 📋 Offener Backlog (alle Issues)"
  [ -n "$ALL_OPEN_ISSUES" ] && echo "$ALL_OPEN_ISSUES" || echo "(kein offener Backlog)"
  echo

  # ── GITHUB: Offene PRs ────────────────────────────────────────────────────
  echo "## 🔀 Offene PRs (Review / laufende Arbeit)"
  [ -n "$OPEN_PRS" ] && echo "$OPEN_PRS" || echo "(keine offenen PRs)"
  echo

  # ── GITHUB: CI-Status ─────────────────────────────────────────────────────
  echo "## 🔬 CI-Status (letzte 8 Runs)"
  [ -n "$CI_RUNS" ] && echo "$CI_RUNS" || echo "(nicht erreichbar)"
  if [ -n "$FAILED_JOBS" ]; then
    echo
    echo "### Fehlgeschlagene Jobs (Run #$FAILED_RUN_ID)"
    echo "$FAILED_JOBS"
    echo "Vollständiges Log: gh run view $FAILED_RUN_ID --log-failed"
  fi
  echo

  # ── Git: Letzte Commits + Diff ────────────────────────────────────────────
  echo "## 📜 Letzte 15 Commits"
  git log -15 --format='- %h %as %s (%an)'
  echo
  echo "## 📁 Geänderte Dateien vs origin/main"
  git diff --name-status origin/main...HEAD 2>/dev/null | head -30 || echo "(kein Diff)"
  echo

  # ── GitHub: Merge-Historie ────────────────────────────────────────────────
  echo "## 🔁 Letzte PRs (was wurde zuletzt gebaut)"
  [ -n "$RECENT_PRS" ] && echo "$RECENT_PRS" || echo "(nicht erreichbar)"
  echo

  # ── In-Progress ANKERs ────────────────────────────────────────────────────
  ANCHORS=$(grep -rn 'ANCHOR\[.*\] STATUS:IN-PROGRESS' crates/ --include='*.rs' 2>/dev/null | head -10)
  if [ -n "$ANCHORS" ]; then
    echo "## ⚓ In-Progress ANKERs"
    echo "$ANCHORS"
    echo
  fi

  # ── Claims / Context-Pack ─────────────────────────────────────────────────
  if [ -n "$CONTEXT_PACK" ]; then
    echo "## 🏷️ Aktive Claims"
    echo "$CONTEXT_PACK" | sed -n '/## Active Claims/,/##/p' | head -20
    echo
  fi

  # ── WORKING_STATE ─────────────────────────────────────────────────────────
  if [ -f WORKING_STATE.md ]; then
    echo "## 🗂️ WORKING_STATE (Kopf)"
    head -40 WORKING_STATE.md
    echo
  fi

  # ── Preflight ─────────────────────────────────────────────────────────────
  if [ -n "$PREFLIGHT" ]; then
    echo "## 🚦 Preflight Gate-Status"
    echo "$PREFLIGHT" | tail -25
    echo
  fi

  # ── GitHub-Kommunikations-Protokoll ───────────────────────────────────────
  echo "## 📡 GitHub-Kommunikation (Protokollpflicht)"
  echo "Task-Start:   gh issue comment <N> --body '🤖 Starte Arbeit an diesem Task: <kurze Beschreibung>'"
  echo "Task-Ende:    gh issue close <N> --comment '✅ Erledigt: <PR-Link>'"
  echo "Blocker:      gh issue create --title '🚫 Blocker: <Titel>' --label 'bug,priority-p0'"
  echo "Frage:        gh issue comment <N> --body '❓ <Frage an Maintainer>'"
  echo "Neuer Fund:   gh issue create --title '<Titel>' --label 'bug' (nur echte Fehler, kein Routine-Spam)"
  echo

  # ── Crate-AGENTS.md Lookup ────────────────────────────────────────────────
  echo "## 📦 Crate-AGENTS.md (vor Code-Edit PFLICHT lesen)"
  find crates/ -name "AGENTS.md" -printf "  %h → %p\n" 2>/dev/null | sort

  # ── Session-Ende ──────────────────────────────────────────────────────────
  echo
  echo "## ✅ Session-Ende (VOR Submit)"
  echo "cargo fmt --all && just check && just dag-check && just test && just sync-docs"
  echo "cargo xtask jules-submit-gate --crate <CRATE>"

} > "$CTX/SESSION_START.md"

echo "✅ Kontext geschrieben: $CTX/SESSION_START.md ($(wc -l < "$CTX/SESSION_START.md") Zeilen)"

# Working Tree sauber halten
git reset --hard HEAD -q 2>/dev/null && git clean -fdq 2>/dev/null
git reset --hard HEAD 2>&1 | tail -1
