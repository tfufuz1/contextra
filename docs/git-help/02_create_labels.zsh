#!/usr/bin/env zsh
# =============================================================================
# Contextra — Labels erstellen
# Datei: docs/git-help/02_create_labels.zsh
# Zweck: Erstellt alle GitHub-Labels aus GITHUB_WORKFLOW.md idempotent.
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

# ── gh-Prüfung ───────────────────────────────────────────────────────────────
if ! command -v gh &>/dev/null; then
  print_error "gh CLI nicht gefunden. Führe zuerst 01_setup_github.zsh aus."
  exit 1
fi
if ! gh auth status &>/dev/null; then
  print_error "Nicht bei GitHub authentifiziert. Führe zuerst 01_setup_github.zsh aus."
  exit 1
fi

# ── Label-Definitionen ────────────────────────────────────────────────────────
# Format: "name|farbe(hex ohne #)|beschreibung"
typeset -a LABELS=(
  "jules|0075ca|Task für den Jules KI-Agenten"
  "priority-p0|d73a4a|Blocker – kritischer Defekt oder Gate-Verletzung"
  "bug|d73a4a|Defekt im Code oder Verhalten"
  "enhancement|a2eeef|Neues Feature oder Verbesserung"
  "task|e4e669|Allgemeiner Arbeitsauftrag (WP)"
  "claimed|ededed|Von Agent reserviert – nicht manuell bearbeiten"
  "question|d876e3|Klärung erforderlich"
  "wontfix|ffffff|Wird nicht bearbeitet"
  "invalid|e4e4e4|Ungültig oder dupliziert"
  "protected-path|f9d0c4|PR berührt geschützten Pfad (ADR-Trailer erforderlich)"
)

# ── Vorhandene Labels abrufen ─────────────────────────────────────────────────
print_step "Lade vorhandene Labels von GitHub…"
print_step "Führe aus: gh label list --repo ${REPO} --json name"
EXISTING_LABELS=$(gh label list --repo "${REPO}" --json name --jq '.[].name' 2>/dev/null || echo "")
print_ok "Labels geladen."

# ── Labels erstellen / aktualisieren ─────────────────────────────────────────
print ""
print_info "Erstelle/aktualisiere Labels (idempotent)…"
print ""

CREATED=0
SKIPPED=0
FAILED=0

for LABEL_DEF in "${LABELS[@]}"; do
  NAME="${LABEL_DEF%%|*}"
  REST="${LABEL_DEF#*|}"
  COLOR="${REST%%|*}"
  DESC="${REST#*|}"

  if echo "$EXISTING_LABELS" | grep -qxF "$NAME"; then
    # Label existiert — aktualisieren (damit Farbe/Beschreibung stimmt)
    print_step "Aktualisiere: '${NAME}' (#${COLOR})"
    if gh label edit "${NAME}" \
        --repo "${REPO}" \
        --color "${COLOR}" \
        --description "${DESC}" 2>/dev/null; then
      print_ok "  Aktualisiert: ${NAME}"
      (( SKIPPED++ )) || true
    else
      print_warn "  Konnte '${NAME}' nicht aktualisieren (ignoriert)."
    fi
  else
    # Label fehlt — neu erstellen
    print_step "Erstelle: '${NAME}' (#${COLOR})"
    print_step "  Führe aus: gh label create '${NAME}' --color '${COLOR}' --description '${DESC}'"
    if gh label create "${NAME}" \
        --repo "${REPO}" \
        --color "${COLOR}" \
        --description "${DESC}" 2>/dev/null; then
      print_ok "  Erstellt: ${NAME}"
      (( CREATED++ )) || true
    else
      print_error "  Fehlgeschlagen: ${NAME}"
      (( FAILED++ )) || true
    fi
  fi
done

# ── Zusammenfassung ───────────────────────────────────────────────────────────
print ""
print -P "${C_BOLD}═══════════════════════════════════════${C_RESET}"
print_info "Label-Zusammenfassung:"
print "  Neu erstellt:    ${CREATED}"
print "  Aktualisiert:    ${SKIPPED}"
print "  Fehlgeschlagen:  ${FAILED}"
print ""

print_step "Alle Labels im Repository:"
print_step "Führe aus: gh label list --repo ${REPO}"
gh label list --repo "${REPO}" 2>/dev/null || print_warn "Konnte Labels nicht auflisten."

if (( FAILED > 0 )); then
  print_error "${FAILED} Label(s) konnten nicht erstellt werden."
  exit 1
fi
print_ok "Fertig — alle Labels gesetzt."
