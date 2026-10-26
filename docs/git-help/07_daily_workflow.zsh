#!/usr/bin/env zsh
# =============================================================================
# Contextra — Täglicher Workflow-Helfer
# Datei: docs/git-help/07_daily_workflow.zsh
# Zweck: Zeigt offene Issues, CI-Status, PRs und hilft beim Branch-Erstellen.
# =============================================================================

set -euo pipefail

REPO="tfufuz1/contextra"

# ── Farbausgabe ───────────────────────────────────────────────────────────────
if [[ -t 1 ]]; then
  C_GREEN="\033[0;32m"; C_RED="\033[0;31m"; C_YELLOW="\033[0;33m"
  C_BLUE="\033[0;34m"; C_BOLD="\033[1m"; C_RESET="\033[0m"
else
  C_GREEN=""; C_RED=""; C_YELLOW=""; C_BLUE=""; C_BOLD=""; C_RESET=""
fi

print_ok()    { print -P "${C_GREEN}✅ $*${C_RESET}" }
print_error() { print -P "${C_RED}❌ Fehler: $*${C_RESET}" >&2 }
print_warn()  { print -P "${C_YELLOW}⚠️  $*${C_RESET}" }
print_step()  { print -P "${C_BLUE}▶  $*${C_RESET}" }
print_info()  { print -P "${C_BOLD}ℹ  $*${C_RESET}" }
print_sep()   { print -P "${C_BOLD}───────────────────────────────────────────${C_RESET}" }

# ── gh-Prüfung ───────────────────────────────────────────────────────────────
if ! command -v gh &>/dev/null; then
  print_error "gh CLI nicht gefunden. Führe zuerst 01_setup_github.zsh aus."
  exit 1
fi
if ! gh auth status &>/dev/null; then
  print_error "Nicht bei GitHub authentifiziert. Führe zuerst 01_setup_github.zsh aus."
  exit 1
fi

# ── Aktueller Git-Status ──────────────────────────────────────────────────────
CURRENT_BRANCH=$(git rev-parse --abbrev-ref HEAD 2>/dev/null || echo "unbekannt")
CURRENT_COMMIT=$(git rev-parse --short HEAD 2>/dev/null || echo "unbekannt")

print ""
print -P "${C_BOLD}╔═══════════════════════════════════════════╗${C_RESET}"
print -P "${C_BOLD}║   Contextra — Täglicher Workflow-Helfer   ║${C_RESET}"
print -P "${C_BOLD}╚═══════════════════════════════════════════╝${C_RESET}"
print ""
print_info "Aktueller Stand:"
print "  Branch: ${CURRENT_BRANCH}"
print "  Commit: ${CURRENT_COMMIT}"
print ""

# ── 1. Offene Jules-Issues ────────────────────────────────────────────────────
print_sep
print_step "1. Offene Jules-Issues"
print_step "Führe aus: gh issue list --repo ${REPO} --label jules --state open"
print ""
ISSUES=$(gh issue list \
  --repo "${REPO}" \
  --label "jules" \
  --state open \
  --json number,title,milestone,labels \
  --jq '.[] | "#\(.number) [\(if .milestone then .milestone.title else "kein Milestone" end)] \(.title)"' \
  2>/dev/null || echo "")

if [[ -z "$ISSUES" ]]; then
  print_warn "Keine offenen Jules-Issues gefunden."
  print_info "Führe zuerst Skript 04 aus, um Issues zu erstellen."
else
  echo "$ISSUES"
fi

# ── 2. CI-Status ─────────────────────────────────────────────────────────────
print ""
print_sep
print_step "2. Letzte CI-Läufe (max. 5)"
print_step "Führe aus: gh run list --repo ${REPO} --limit 5"
print ""
gh run list \
  --repo "${REPO}" \
  --limit 5 \
  --json status,conclusion,name,headBranch,createdAt \
  --jq '.[] | "\(if .conclusion == "success" then "✅" elif .conclusion == "failure" then "❌" elif .status == "in_progress" then "🔄" else "⏸" end) [\(.headBranch)] \(.name) — \(.createdAt[:10])"' \
  2>/dev/null || print_warn "Konnte CI-Status nicht laden."

# ── 3. Offene PRs ─────────────────────────────────────────────────────────────
print ""
print_sep
print_step "3. Offene Pull Requests"
print_step "Führe aus: gh pr list --repo ${REPO} --state open"
print ""
PRS=$(gh pr list \
  --repo "${REPO}" \
  --state open \
  --json number,title,author,headRefName \
  --jq '.[] | "PR #\(.number) [\(.headRefName)] \(.title) (@\(.author.login))"' \
  2>/dev/null || echo "")

if [[ -z "$PRS" ]]; then
  print_ok "Keine offenen PRs."
else
  echo "$PRS"
fi

# ── 4. Issue bearbeiten ───────────────────────────────────────────────────────
print ""
print_sep
print ""
print -n "Issue-Nummer anzeigen (Enter zum Überspringen): "
read -r ISSUE_NUM

if [[ -n "$ISSUE_NUM" && "$ISSUE_NUM" =~ ^[0-9]+$ ]]; then
  print ""
  print_step "Führe aus: gh issue view ${ISSUE_NUM} --repo ${REPO}"
  gh issue view "${ISSUE_NUM}" --repo "${REPO}" 2>/dev/null \
    || print_error "Issue #${ISSUE_NUM} nicht gefunden."

  print ""
  print_info "Branch für dieses Issue erstellen?"
  print_info "Empfohlenes Format: wp-X.Y-kurze-beschreibung (z.B. wp-1.1-engine-stub)"
  print -n "Branch-Name (Enter zum Überspringen): "
  read -r BRANCH_NAME

  if [[ -n "$BRANCH_NAME" ]]; then
    # Branch-Name bereinigen
    BRANCH_NAME="${BRANCH_NAME//[^a-zA-Z0-9._/-]/-}"

    print ""
    print_step "Führe aus: git fetch origin && git checkout -b ${BRANCH_NAME} origin/main"
    git fetch origin 2>/dev/null && \
    git checkout -b "${BRANCH_NAME}" origin/main 2>/dev/null && \
    print_ok "Branch '${BRANCH_NAME}' erstellt und ausgecheckt." || \
    print_error "Branch-Erstellung fehlgeschlagen. Existiert er bereits?"

    print ""
    print_info "Nächste Schritte:"
    print "  1. Code ändern"
    print "  2. just check && just test && just sync-docs"
    print "  3. git add -p && git commit -m 'wp-X.Y: Beschreibung'"
    print "  4. git fetch origin && git rebase origin/main"
    print "  5. git push origin ${BRANCH_NAME}"
    print "  6. gh pr create --repo ${REPO} --base main --head ${BRANCH_NAME} --fill"
  fi
fi

# ── 5. Abschluss ──────────────────────────────────────────────────────────────
print ""
print_sep
print_ok "Täglicher Workflow-Check abgeschlossen."
print ""
print_info "Weitere nützliche Befehle:"
print "  gh issue list --repo ${REPO} --label jules   # Alle Jules-Issues"
print "  gh run watch --repo ${REPO}                   # CI live verfolgen"
print "  gh pr status --repo ${REPO}                   # Eigene PR-Status"
print "  gh issue comment <N> --body '...'             # Issue kommentieren"
