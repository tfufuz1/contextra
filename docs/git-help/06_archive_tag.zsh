#!/usr/bin/env zsh
# =============================================================================
# Contextra — Archiv-Tag archiv/v0 setzen (WP 2.1)
# Datei: docs/git-help/06_archive_tag.zsh
# Zweck: Markiert den Stand vor der Archivierung der Archiv-Crates unveränderlich.
# =============================================================================

set -euo pipefail

REPO="tfufuz1/contextra"
TAG="archiv/v0"

# ── Farbausgabe ───────────────────────────────────────────────────────────────
if [[ -t 1 ]]; then
  C_GREEN=$'\033[0;32m'; C_RED=$'\033[0;31m'; C_YELLOW=$'\033[0;33m'
  C_BLUE=$'\033[0;34m'; C_BOLD=$'\033[1m'; C_RESET=$'\033[0m'
else
  C_GREEN=""; C_RED=""; C_YELLOW=""; C_BLUE=""; C_BOLD=""; C_RESET=""
fi

print_ok()    { print -r -- "${C_GREEN}✅ $*${C_RESET}" }
print_error() { print -r -- "${C_RED}❌ Fehler: $*${C_RESET}" >&2 }
print_warn()  { print -r -- "${C_YELLOW}⚠️  $*${C_RESET}" }
print_step()  { print -r -- "${C_BLUE}▶  $*${C_RESET}" }
print_info()  { print -r -- "${C_BOLD}ℹ  $*${C_RESET}" }

# ── Erklärung ─────────────────────────────────────────────────────────────────
print ""
print -r -- "${C_BOLD}Was macht der Tag '${TAG}'?${C_RESET}"
print ""
print "In WP 2.1 (Archivierung) werden diese Crates aus dem aktiven Workspace entfernt:"
print ""
print "  • contextra-graph     (8.708 LOC) — Graph-Reasoning kein v1.0-Ziel"
print "  • contextra-cognition (3.191 LOC) — Konsolidierung kein v1.0-Ziel"
print "  • contextra-adapt     (2.193 LOC) — Online-Adaption kein v1.0-Ziel"
print "  • contextra-kvcache   (2.434 LOC) — KV-Cache kein v1.0-Ziel"
print "  • contextra-router    (1.465 LOC) — Router kein v1.0-Ziel"
print "  • contextra-agent     (2.413 LOC) — Agent-Workflow kein v1.0-Ziel"
print "  • contextra-infer-onnx / ollama / core"
print ""
print "Der Tag '${TAG}' markiert den Commit-Stand BEVOR diese Entfernung beginnt."
print "So kann man jederzeit zurückschauen, wie der Stand VOR der Archivierung war."
print ""
print_warn "Dieser Tag ist unveränderlich (annotierter Tag — nicht einfach zu löschen)."
print_info "PLAN.md §4 — WP 2.1 Akzeptanzkriterium: Tag '${TAG}' muss existieren und gepusht sein."
print ""

# ── Prüfen ob Tag bereits existiert ──────────────────────────────────────────
print_step "Prüfe ob Tag '${TAG}' bereits existiert…"
if git rev-parse -q --verify "refs/tags/${TAG}" &>/dev/null; then
  EXISTING_COMMIT=$(git rev-list -n 1 "${TAG}" 2>/dev/null || echo "unbekannt")
  print_warn "Tag '${TAG}' existiert bereits auf Commit: ${EXISTING_COMMIT}"
  print ""

  # Prüfen ob Tag auf Remote gepusht ist
  if git ls-remote --exit-code --tags origin "refs/tags/${TAG}" &>/dev/null; then
    print_ok "Tag ist bereits auf Remote (origin) vorhanden — nichts zu tun."
    print ""
    print_info "Nächster Schritt: WP 2.1 Issue auf GitHub kommentieren."
    print "  gh issue list --repo ${REPO} --label jules | grep 'WP 2.1'"
    exit 0
  else
    print_warn "Tag existiert lokal, aber ist noch NICHT gepusht."
    print -n "Jetzt pushen? [ja/nein]: "
    read -r PUSH_CONFIRM
    if [[ "$PUSH_CONFIRM" == "ja" || "$PUSH_CONFIRM" == "j" ]]; then
      print_step "Führe aus: git push origin ${TAG}"
      git push origin "${TAG}"
      print_ok "Tag '${TAG}' gepusht."
    else
      print_warn "Push abgebrochen."
    fi
    exit 0
  fi
fi

# ── Aktuellen Stand anzeigen ──────────────────────────────────────────────────
CURRENT_BRANCH=$(git rev-parse --abbrev-ref HEAD 2>/dev/null || echo "unbekannt")
CURRENT_COMMIT=$(git rev-parse --short HEAD 2>/dev/null || echo "unbekannt")
print ""
print_info "Aktueller Stand:"
print "  Branch: ${CURRENT_BRANCH}"
print "  Commit: ${CURRENT_COMMIT}"
print ""
print_warn "Der Tag wird auf diesem Commit gesetzt."

# ── Bestätigung ───────────────────────────────────────────────────────────────
print -n "Tag '${TAG}' auf aktuellem HEAD (${CURRENT_COMMIT}) setzen und pushen? [ja/nein]: "
read -r CONFIRM
if [[ "$CONFIRM" != "ja" && "$CONFIRM" != "j" ]]; then
  print_warn "Abgebrochen. Kein Tag gesetzt."
  exit 0
fi

# ── Tag setzen ────────────────────────────────────────────────────────────────
print ""
TAG_MESSAGE="archiv/v0 — Stand vor Phase-2-Archivierung (WP 2.1)

Dieser annotierte Tag markiert den Commit-Stand bevor die Archiv-Crates
(graph, cognition, adapt, kvcache, router, agent, infer-onnx, infer-ollama, core)
aus dem aktiven Workspace entfernt werden.

Zweck: Unveränderlicher Referenzpunkt für die Archivierungs-Migration.
PLAN.md: §4 WP 2.1"

print_step "Führe aus: git tag -a ${TAG} -m '...'"
git tag -a "${TAG}" -m "${TAG_MESSAGE}"
print_ok "Tag '${TAG}' lokal gesetzt."

print_step "Führe aus: git push origin ${TAG}"
if git push origin "${TAG}"; then
  print_ok "Tag '${TAG}' erfolgreich zu origin gepusht."
else
  print_error "Push fehlgeschlagen."
  print_info "Tag ist lokal gesetzt. Push später erneut starten (Skript erneut ausführen) — KEIN --force."
  exit 1
fi

# ── Nächste Schritte ──────────────────────────────────────────────────────────
print ""
print -r -- "${C_BOLD}═══════════════════════════════════════${C_RESET}"
print_ok "Archiv-Tag '${TAG}' ist gesetzt und gepusht."
print ""
print_info "Nächste Schritte für WP 2.1:"
print "  1. WP 2.1 Issue auf GitHub suchen:"
print "     gh issue list --repo ${REPO} --label jules | grep 'WP 2.1'"
print ""
print "  2. Issue kommentieren (ersetze <N> durch Issue-Nummer):"
print "     gh issue comment <N> --repo ${REPO} --body 'Tag archiv/v0 gesetzt: \$(git rev-parse ${TAG})'"
print ""
print "  3. MIGRATION-EDGES.md im Repo-Root erstellen (Abhängigkeits-Inventar)"
print "  4. Archiv-Crates aus Cargo.toml default-members entfernen"
