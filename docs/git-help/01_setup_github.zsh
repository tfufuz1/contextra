#!/usr/bin/env zsh
# =============================================================================
# Contextra — GitHub-Ersteinrichtung Wizard
# Datei: docs/git-help/01_setup_github.zsh
# Zweck: gh installieren/authentifizieren und Standard-Repo setzen.
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

# ── Schritt 1: gh-Installation prüfen ────────────────────────────────────────
print_step "Schritt 1/4: Prüfe gh CLI Installation…"
if command -v gh &>/dev/null; then
  GH_VERSION=$(gh --version | head -1)
  print_ok "gh ist installiert: ${GH_VERSION}"
else
  print_error "gh CLI ist NICHT installiert."
  print ""
  print_info "Installation (wähle dein System):"
  print "  Ubuntu/Debian:  sudo apt install gh"
  print "  Arch Linux:     sudo pacman -S github-cli"
  print "  macOS (brew):   brew install gh"
  print "  Manuell:        https://cli.github.com/manual/installation"
  print ""
  print_warn "Bitte gh installieren und dieses Skript erneut ausführen."
  exit 1
fi

# ── Schritt 2: Authentifizierung prüfen ──────────────────────────────────────
print_step "Schritt 2/4: Prüfe GitHub-Authentifizierung…"
if gh auth status &>/dev/null; then
  LOGGED_IN=$(gh auth status 2>&1 | grep "Logged in to" || echo "bereits angemeldet")
  print_ok "Bereits authentifiziert: ${LOGGED_IN}"
else
  print_warn "Nicht authentifiziert. Starte Browser-Login-Flow…"
  print_info "Ein Browser-Fenster öffnet sich. Folge den Anweisungen auf GitHub."
  print ""
  print_step "Führe aus: gh auth login"
  gh auth login --web --git-protocol ssh
  print_ok "Authentifizierung abgeschlossen."
fi

# ── Schritt 3: Standard-Repo setzen ──────────────────────────────────────────
print_step "Schritt 3/4: Setze Standard-Repository auf ${REPO}…"
print_step "Führe aus: gh repo set-default ${REPO}"
if gh repo set-default "${REPO}" 2>/dev/null; then
  print_ok "Standard-Repository gesetzt: ${REPO}"
else
  print_warn "gh repo set-default fehlgeschlagen (möglicherweise kein gh-Context nötig)."
  print_info "Alternativ: gh-Befehle mit --repo ${REPO} ausführen."
fi

# ── Schritt 4: Git-Remote prüfen ─────────────────────────────────────────────
print_step "Schritt 4/4: Prüfe Git-Remote…"
REMOTE_URL=$(git remote get-url origin 2>/dev/null || echo "")
EXPECTED="git@github.com:${REPO}.git"

if [[ "$REMOTE_URL" == "$EXPECTED" ]]; then
  print_ok "Git-Remote korrekt gesetzt: ${REMOTE_URL}"
elif [[ -z "$REMOTE_URL" ]]; then
  print_warn "Kein 'origin'-Remote gefunden."
  print_info "Setze Remote mit:"
  print "  git remote add origin ${EXPECTED}"
  print -n "Jetzt setzen? [ja/nein]: "
  read -r CONFIRM
  if [[ "$CONFIRM" == "ja" || "$CONFIRM" == "j" ]]; then
    print_step "Führe aus: git remote add origin ${EXPECTED}"
    git remote add origin "${EXPECTED}"
    print_ok "Remote 'origin' gesetzt: ${EXPECTED}"
  else
    print_warn "Remote nicht gesetzt. Bitte manuell erledigen."
  fi
else
  print_warn "Remote weicht ab:"
  print "  Ist:      ${REMOTE_URL}"
  print "  Erwartet: ${EXPECTED}"
  print_info "Falls gewünscht: git remote set-url origin ${EXPECTED}"
fi

# ── Zusammenfassung ───────────────────────────────────────────────────────────
print ""
print -P "${C_BOLD}═══════════════════════════════════════${C_RESET}"
print_ok "GitHub-Einrichtung abgeschlossen!"
print_info "Nächste Schritte:"
print "  → Skript 02: Labels erstellen"
print "  → Skript 03: Milestones erstellen"
print "  → Skript 04: WP-Issues erstellen"
