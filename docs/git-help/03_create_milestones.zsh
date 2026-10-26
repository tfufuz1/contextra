#!/usr/bin/env zsh
# =============================================================================
# Contextra — Milestones erstellen
# Datei: docs/git-help/03_create_milestones.zsh
# Zweck: Erstellt Milestones für Phase 0–3 (Sprints aus PLAN.md).
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

# ── Milestone-Definitionen ────────────────────────────────────────────────────
# Format: "titel|beschreibung"
typeset -a MILESTONES=(
  "Phase 0 – Baseline|WP 0.1–0.4: Build, Gates, Governance-Skelett, Spec-Ablage"
  "Phase 1 – Stubs|WP 1.1–1.3: Engine-Stub entfernen, MCP fail-closed, Panics bereinigen"
  "Phase 2 – Archivierung|WP 2.1–2.6: Archiv-Crates trennen, engine/db entkoppeln, Unsafe-Inseln"
  "Phase 3 – Fundament|WP 3.1–3.5: Sicherheitsports, WAL, Vfs-Impl, Scanning-Infrastruktur"
)

# ── Vorhandene Milestones abrufen (inkl. geschlossene) ───────────────────────
print_step "Lade vorhandene Milestones von GitHub…"
print_step "Führe aus: gh api 'repos/${REPO}/milestones?state=all' --paginate"
if ! EXISTING_MS=$(gh api "repos/${REPO}/milestones?state=all&per_page=100" \
      --paginate --jq '.[].title' 2>&1); then
  print_error "Milestones konnten nicht geladen werden:"
  print "$EXISTING_MS"
  print_info "Prüfe die Token-Rechte: Fine-grained Token braucht 'Issues: Read and write'."
  exit 1
fi
print_ok "Milestones geladen."
print ""

# ── Milestones erstellen ──────────────────────────────────────────────────────
print_info "Erstelle Milestones (idempotent — vorhandene werden übersprungen)…"
print ""

CREATED=0
SKIPPED=0
FAILED=0

for MS_DEF in "${MILESTONES[@]}"; do
  TITLE="${MS_DEF%%|*}"
  DESC="${MS_DEF#*|}"

  if printf '%s\n' "$EXISTING_MS" | grep -qxF -- "$TITLE"; then
    print_warn "Existiert bereits: '${TITLE}' — übersprungen."
    (( SKIPPED++ )) || true
    continue
  fi

  print_info "Neuer Milestone: ${TITLE}"

  # Fälligkeitsdatum abfragen (Prompt geht direkt ans Terminal)
  print -n "  Fälligkeitsdatum (YYYY-MM-DD, oder Enter für keins): "
  read -r DATE_INPUT

  ARGS=(-f "title=${TITLE}" -f "description=${DESC}")
  if [[ -n "$DATE_INPUT" ]]; then
    ARGS+=(-f "due_on=${DATE_INPUT}T00:00:00Z")
  fi

  print_step "Führe aus: gh api -X POST repos/${REPO}/milestones -f title=… -f description=…"
  if RESULT=$(gh api -X POST "repos/${REPO}/milestones" "${ARGS[@]}" \
        --jq '.number' 2>&1); then
    print_ok "  Erstellt: ${TITLE} (Nr. ${RESULT})"
    (( CREATED++ )) || true
  else
    print_error "  Fehlgeschlagen: ${TITLE}"
    print_warn "  GitHub-Antwort: ${RESULT}"
    (( FAILED++ )) || true
  fi
  print ""
done

# ── Zusammenfassung ───────────────────────────────────────────────────────────
print -P "${C_BOLD}═══════════════════════════════════════${C_RESET}"
print_info "Milestone-Zusammenfassung:"
print "  Neu erstellt:   ${CREATED}"
print "  Übersprungen:   ${SKIPPED}"
print "  Fehlgeschlagen: ${FAILED}"
print ""
print_step "Aktuelle Milestones:"
gh api "repos/${REPO}/milestones?state=all" \
  --jq '.[] | "#\(.number) \(.title) (offen: \(.open_issues))"' \
  || print_warn "Konnte Milestones nicht auflisten."

if (( FAILED > 0 )); then
  print_error "${FAILED} Milestone(s) konnten nicht erstellt werden. Die GitHub-Antwort steht oben."
  exit 1
fi
print_ok "Fertig — alle Milestones gesetzt."
