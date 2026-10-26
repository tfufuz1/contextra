#!/usr/bin/env zsh
# =============================================================================
# Contextra — GitHub Wizard Hauptmenü
# Datei: docs/git-help/00_wizard_menu.zsh
# Zweck: Interaktiver Einstiegspunkt für alle GitHub-Wizard-Skripte.
# =============================================================================

set -euo pipefail

REPO="tfufuz1/contextra"
SCRIPT_DIR="${0:A:h}"

# ── Farbausgabe ──────────────────────────────────────────────────────────────
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
check_gh() {
  if ! command -v gh &>/dev/null; then
    print_error "gh CLI nicht gefunden."
    print_info  "Installation: https://cli.github.com/"
    print_info  "Oder: sudo apt install gh  /  brew install gh"
    exit 1
  fi
  if ! gh auth status &>/dev/null; then
    print_warn "gh ist nicht authentifiziert."
    print_info "Starte Skript 01 (GitHub-Ersteinrichtung) um dich anzumelden."
    return 1
  fi
  return 0
}

# ── Menü ─────────────────────────────────────────────────────────────────────
show_menu() {
  print ""
  print -P "${C_BOLD}╔══════════════════════════════════════════════════════════╗${C_RESET}"
  print -P "${C_BOLD}║   Contextra — GitHub Wizard  (Repo: ${REPO})  ║${C_RESET}"
  print -P "${C_BOLD}╚══════════════════════════════════════════════════════════╝${C_RESET}"
  print ""
  print_info "Wähle eine Option:"
  print ""
  print "  1) GitHub-Ersteinrichtung (gh auth + repo-default)"
  print "  2) Labels erstellen"
  print "  3) Milestones (Sprints) erstellen"
  print "  4) WP-Issues erstellen (Phase 0–2)"
  print "  5) Branch-Protection für main einrichten"
  print "  6) Archiv-Tag archiv/v0 setzen (WP 2.1)"
  print "  7) Täglichen Workflow anzeigen (Issues / CI / PRs)"
  print ""
  print "  0) Beenden"
  print ""
}

run_script() {
  local script="${SCRIPT_DIR}/$1"
  if [[ ! -f "$script" ]]; then
    print_error "Skript nicht gefunden: $script"
    return 1
  fi
  print_step "Starte: $script"
  print ""
  zsh "$script"
}

# ── Hauptschleife ─────────────────────────────────────────────────────────────
main() {
  print_step "Prüfe gh CLI…"
  check_gh || true

  while true; do
    show_menu
    print -n "Deine Wahl [0-7]: "
    read -r CHOICE

    case "$CHOICE" in
      1) run_script "01_setup_github.zsh" ;;
      2) run_script "02_create_labels.zsh" ;;
      3) run_script "03_create_milestones.zsh" ;;
      4) run_script "04_create_issues.zsh" ;;
      5) run_script "05_branch_protection.zsh" ;;
      6) run_script "06_archive_tag.zsh" ;;
      7) run_script "07_daily_workflow.zsh" ;;
      0) print_ok "Auf Wiedersehen!"; exit 0 ;;
      *) print_warn "Ungültige Eingabe: '$CHOICE'. Bitte 0–7 eingeben." ;;
    esac

    print ""
    print -n "Zurück zum Menü? [Enter] oder 'q' zum Beenden: "
    read -r BACK
    [[ "$BACK" == "q" || "$BACK" == "Q" ]] && { print_ok "Auf Wiedersehen!"; exit 0; }
  done
}

main "$@"
