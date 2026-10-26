#!/usr/bin/env zsh
# =============================================================================
# Contextra — Branch-Protection einrichten
# Datei: docs/git-help/05_branch_protection.zsh
# Zweck: Richtet Branch-Protection-Regeln für `main` ein.
# =============================================================================

set -euo pipefail

REPO="tfufuz1/contextra"
BRANCH="main"

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

# ── gh-Prüfung ───────────────────────────────────────────────────────────────
if ! command -v gh &>/dev/null; then
  print_error "gh CLI nicht gefunden. Führe zuerst 01_setup_github.zsh aus."
  exit 1
fi
if ! gh auth status &>/dev/null; then
  print_error "Nicht bei GitHub authentifiziert. Führe zuerst 01_setup_github.zsh aus."
  exit 1
fi

# ── Erklärung ─────────────────────────────────────────────────────────────────
print ""
print -P "${C_BOLD}Was macht Branch-Protection?${C_RESET}"
print ""
print "Branch-Protection verhindert direkte Pushes auf 'main' und erzwingt:"
print ""
print "  1. Pull Requests (PR) vor dem Merge"
print "     → Jede Änderung muss als PR eingereicht werden"
print "     → Direktes 'git push origin main' wird ABGELEHNT"
print ""
print "  2. Pflicht-Status-Checks (CI muss grün sein)"
print "     → Ein PR kann erst gemergt werden, wenn CI erfolgreich war"
print "     → Verhindert kaputten Code auf main"
print ""
print "  3. Kein Force-Push auf main"
print "     → Schützt die Git-Geschichte vor Überschreiben"
print ""
print_warn "Voraussetzung: GitHub Actions CI muss für das Repo eingerichtet sein."
print_info "Ohne grüne CI-Checks kann kein PR gemergt werden."
print ""

# ── Bestätigung ───────────────────────────────────────────────────────────────
print -n "Branch-Protection für '${BRANCH}' einrichten? [ja/nein]: "
read -r CONFIRM
if [[ "$CONFIRM" != "ja" && "$CONFIRM" != "j" ]]; then
  print_warn "Abgebrochen."
  exit 0
fi

# ── Status-Check-Name abfragen ────────────────────────────────────────────────
print ""
print_info "Welchen CI-Status-Check soll GitHub als Pflicht-Check kennen?"
print "  (Standard: 'CI' — muss exakt dem Workflow-Job-Namen in .github/workflows/ entsprechen)"
print -n "Status-Check-Name [CI]: "
read -r CHECK_NAME
CHECK_NAME="${CHECK_NAME:-CI}"

# ── Branch-Protection-Regel setzen ───────────────────────────────────────────
print ""
print_step "Setze Branch-Protection für '${BRANCH}'…"

PAYLOAD=$(cat <<JSONEOF
{
  "required_status_checks": {
    "strict": true,
    "contexts": ["${CHECK_NAME}"]
  },
  "enforce_admins": false,
  "required_pull_request_reviews": {
    "dismiss_stale_reviews": false,
    "require_code_owner_reviews": false,
    "required_approving_review_count": 0
  },
  "restrictions": null,
  "allow_force_pushes": false,
  "allow_deletions": false,
  "required_linear_history": false,
  "required_conversation_resolution": false
}
JSONEOF
)

print_step "Führe aus: gh api repos/${REPO}/branches/${BRANCH}/protection (PUT)"
RESULT=$(echo "$PAYLOAD" | gh api \
  "repos/${REPO}/branches/${BRANCH}/protection" \
  --method PUT \
  --input - \
  2>&1) && API_OK=0 || API_OK=$?

if (( API_OK == 0 )); then
  print_ok "Branch-Protection erfolgreich gesetzt!"
else
  print_error "Branch-Protection fehlgeschlagen."
  print_warn "Fehler-Details:"
  echo "$RESULT"
  print ""
  print_info "Mögliche Ursache: Du brauchst Admin-Rechte auf dem Repository."
  print_info "Alternativ: Branch-Protection manuell einrichten unter:"
  print "  https://github.com/${REPO}/settings/branches"
  exit 1
fi

# ── Ergebnis anzeigen ─────────────────────────────────────────────────────────
print ""
print_step "Aktuelle Branch-Protection-Einstellungen:"
print_step "Führe aus: gh api repos/${REPO}/branches/${BRANCH}/protection"
gh api "repos/${REPO}/branches/${BRANCH}/protection" \
  --jq '{
    enforce_admins: .enforce_admins.enabled,
    require_pr: (.required_pull_request_reviews != null),
    status_checks: .required_status_checks.contexts,
    allow_force_push: .allow_force_pushes.enabled
  }' 2>/dev/null || print_warn "Konnte Details nicht laden."

print ""
print_ok "Branch-Protection für '${BRANCH}' ist aktiv."
print_info "Kein direktes Pushen auf main mehr möglich — nur noch per PR."
