#!/usr/bin/env zsh
# =============================================================================
# Contextra — WP-Issues erstellen
# Datei: docs/git-help/04_create_issues.zsh
# Zweck: Erstellt alle Arbeitspaket-Issues für Phase 0–2 aus PLAN.md.
#        Format exakt nach GITHUB_WORKFLOW.md Issue-Anatomie.
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

# ── Milestone-Nummer abrufen ──────────────────────────────────────────────────
get_milestone_number() {
  local TITLE="$1"
  gh api "repos/${REPO}/milestones" \
    --jq ".[] | select(.title == \"${TITLE}\") | .number" 2>/dev/null || echo ""
}

# ── Issue-Erstellung ──────────────────────────────────────────────────────────
create_issue() {
  local TITLE="$1"
  local BODY="$2"
  local MILESTONE_TITLE="$3"
  local LABELS="${4:-jules,task}"

  # Prüfen ob Issue schon existiert (by title)
  EXISTING=$(gh issue list --repo "${REPO}" --state all \
    --json title,number \
    --jq ".[] | select(.title == \"${TITLE}\") | .number" 2>/dev/null || echo "")

  if [[ -n "$EXISTING" ]]; then
    print_warn "  Existiert bereits: #${EXISTING} — '${TITLE}'"
    return 0
  fi

  MS_NUM=$(get_milestone_number "$MILESTONE_TITLE")

  print_step "  Erstelle: ${TITLE}"
  print_step "  Führe aus: gh issue create --repo ${REPO} --title '${TITLE}' --label ${LABELS}..."

  local ISSUE_URL
  if [[ -n "$MS_NUM" ]]; then
    ISSUE_URL=$(gh issue create \
      --repo "${REPO}" \
      --title "${TITLE}" \
      --body "${BODY}" \
      --label "${LABELS}" \
      --milestone "${MS_NUM}" \
      2>/dev/null || echo "")
  else
    print_warn "  Milestone '${MILESTONE_TITLE}' nicht gefunden — Issue ohne Milestone."
    ISSUE_URL=$(gh issue create \
      --repo "${REPO}" \
      --title "${TITLE}" \
      --body "${BODY}" \
      --label "${LABELS}" \
      2>/dev/null || echo "")
  fi

  if [[ -n "$ISSUE_URL" ]]; then
    print_ok "  Erstellt: ${ISSUE_URL}"
    return 0
  else
    print_error "  Fehlgeschlagen: ${TITLE}"
    return 1
  fi
}

# ── Issue-Definitionen (exaktes Format aus GITHUB_WORKFLOW.md) ────────────────

create_phase_0() {
  print_info "Erstelle Phase-0-Issues (WP 0.1–0.4)…"
  print ""

  create_issue \
    "WP 0.1 — Build-/Test-/Clippy-Baseline" \
'## Ziel
Eine reproduzierbare, dokumentierte Ausgangsbasis für alle folgenden Arbeitspakete schaffen.
Nach diesem WP sind Build, Tests und Clippy auf dem aktuellen Commit (a547a42) vollständig dokumentiert.

## Akzeptanzkriterien
- [ ] `evidence/baseline-a547a42/build.txt` — vollständige `cargo build --workspace` Ausgabe
- [ ] `evidence/baseline-a547a42/test.txt` — vollständige `cargo test --workspace` Ausgabe
- [ ] `evidence/baseline-a547a42/clippy.txt` — vollständige `cargo clippy --workspace` Ausgabe
- [ ] Build ist grün (kein Kompilierfehler)
- [ ] Anzahl Clippy-Warnungen in `ratchet.toml` als Baseline eingetragen

## Scope
Workspace-Root, alle Crates (nur Messen, kein Fixen)

## Veto-Check
keine

## Out-of-scope
Kein Fixen von Warnungen oder Fehlern in diesem WP.
Kein Ändern von Gates oder Lints.

## WP-Referenz
PLAN.md §2 — WP 0.1' \
    "Phase 0 – Baseline" \
    "jules,task"

  create_issue \
    "WP 0.2 — Gate-Läufe --full neu messen" \
'## Ziel
Alle xtask-Gates mit `--full` ausführen und Ergebnisse als Baseline archivieren.
Grundlage für alle Gate-Ratchets in Phase 1+.

## Akzeptanzkriterien
- [ ] `evidence/gates/a547a42/` Verzeichnis mit Einzelergebnissen je Gate angelegt
- [ ] `docs/spec/IST-BEFUNDE.md` aus Gate-Ergebnissen generiert
- [ ] Jedes Gate eindeutig als 🔴/🟡/🟢 klassifiziert
- [ ] `governance/ratchet.toml` mit aktuellen Baseline-Werten befüllt

## Scope
xtask/, governance/ratchet.toml, docs/spec/

## Veto-Check
keine

## Out-of-scope
Kein Beheben von Gate-Fehlern in diesem WP.
Kein Ändern von Gate-Definitionen.

## WP-Referenz
PLAN.md §2 — WP 0.2

## Abhängigkeit
WP 0.1 muss abgeschlossen sein.' \
    "Phase 0 – Baseline" \
    "jules,task"

  create_issue \
    "WP 0.3 — Spec ablegen, Dokumente konsolidieren, ADRs" \
'## Ziel
`docs/SPEC.md` ist die einzige normative Quelle der Wahrheit.
Alle anderen Dokumente sind entweder normativ referenziert oder als nicht-normativ markiert.
ADR-Index ist vollständig.

## Akzeptanzkriterien
- [ ] `docs/SPEC.md` vorhanden mit korrektem doc-stamp
- [ ] `doc-stamp`-Gate grün: alle normativen Dokumente haben gültigen Stamp
- [ ] `docs/decisions/` Index vollständig (alle ADRs aufgeführt)
- [ ] `docs/ARCHITECTURE.md` als nicht-normativ markiert (verweist auf SPEC §4)
- [ ] Keine doppelten normativen Quellen

## Scope
docs/, governance/ (nur doc-stamp)

## Veto-Check
keine

## Out-of-scope
Kein inhaltliches Ändern der Spec (nur Ablegen/Konsolidieren).

## WP-Referenz
PLAN.md §2 — WP 0.3' \
    "Phase 0 – Baseline" \
    "jules,task"

  create_issue \
    "WP 0.4 — Governance-Skelett anlegen" \
'## Ziel
Alle Governance-Dateien aus SPEC §17 existieren und sind inhaltlich korrekt befüllt.
Gates können zuverlässig gegen diese Dateien prüfen.

## Akzeptanzkriterien
- [ ] `governance/security_ports.toml` angelegt (INV-WIRING-2 vollständig)
- [ ] `governance/key_purposes.toml` angelegt (INV-KEYPURPOSE-1 vollständig)
- [ ] `governance/clippy.toml` angelegt (P28-Verbotsliste vollständig)
- [ ] `governance/gates.toml` — alle Gates aus SPEC §17.3 eingetragen
- [ ] `governance/ratchet.toml` — Baselines aus WP 0.2 eingetragen
- [ ] `governance/protected-paths.toml` — alle geschützten Pfade eingetragen
- [ ] `governance/determinism-allow.toml` — ADR-098 Ausnahmen eingetragen
- [ ] `xtask/` lädt alle neuen Governance-Dateien korrekt

## Scope
governance/, xtask/

## Veto-Check
keine

## Out-of-scope
Kein Ändern von Produktionscode in diesem WP.

## WP-Referenz
PLAN.md §2 — WP 0.4

## Abhängigkeit
WP 0.1 muss abgeschlossen sein.' \
    "Phase 0 – Baseline" \
    "jules,task"
}

create_phase_1() {
  print_info "Erstelle Phase-1-Issues (WP 1.1–1.3)…"
  print ""

  create_issue \
    "WP 1.1 — Engine-Stub und encryption-at-rest Feature löschen (B-02, B-15)" \
'## Ziel
Der Sicherheits-Stub `DeletionProof::verify → Ok(true)` in `contextra-engine` wird entfernt.
Das optionale Feature `encryption-at-rest` wird gelöscht — Verschlüsselung ist immer aktiv (A1).

## Akzeptanzkriterien
- [ ] `rg "DeletionProof::verify.*Ok\(true\)" crates/` → 0 Treffer
- [ ] `rg "no_crypto_stubs|encryption-at-rest" crates/` → 0 Treffer
- [ ] Gate `no-security-stub` grün
- [ ] `cargo test --workspace` grün
- [ ] Kein neuer `cfg(not(feature = "..."))` um Sicherheitscode

## Scope
crates/contextra-engine/, crates/contextra-store/

## Veto-Check
keine (Befund B-02, B-15 aus PLAN.md §1)

## Out-of-scope
Kein Implementieren des echten Löschbeweises in diesem WP.
Kein Anfassen anderer Crates.

## WP-Referenz
PLAN.md §3 — WP 1.1 (Befunde B-02, B-15)

## Abhängigkeit
WP 0.1 muss abgeschlossen sein.' \
    "Phase 1 – Stubs" \
    "jules,task,priority-p0"

  create_issue \
    "WP 1.2 — MCP: Literal-Schlüssel und Mandanten-Fallback fail-closed (B-04)" \
'## Ziel
In `contextra-mcp` gibt es keine Literal-Schlüssel und keinen `SYSTEM`-Mandanten-Fallback mehr.
Fehlende Konfiguration führt zu einem benannten Fehler, nicht zu einem Dummy-Beweis.

## Akzeptanzkriterien
- [ ] `rg "default_test_deletion_proof_key" crates/contextra-mcp/` → 0 Treffer
- [ ] `rg "\"SYSTEM\"" crates/contextra-mcp/` → 0 Treffer (als Fallback-Literal)
- [ ] Gate `no-literal-secrets` grün
- [ ] Fehlender KMS-Port → `ContextraError::PortNotConfigured` (fail-closed)
- [ ] AC-26 teilweise erfüllt; U-22, U-23, U-37 geschlossen
- [ ] `cargo test --workspace` grün

## Scope
crates/contextra-mcp/

## Veto-Check
VETO-F10 geprüft: Mandant kommt aus RequestContext, niemals aus Literalen (INV-TENANT-1)

## Out-of-scope
Kein Implementieren des echten KMS-Providers in diesem WP.
Kein Anfassen anderer Crates.

## WP-Referenz
PLAN.md §3 — WP 1.2 (Befund B-04)

## Abhängigkeit
WP 0.1 muss abgeschlossen sein.' \
    "Phase 1 – Stubs" \
    "jules,task,priority-p0"

  create_issue \
    "WP 1.3 — Panics und Test-Hooks aus der Produktions-API entfernen" \
'## Ziel
Kein `unwrap`, `expect`, `panic!`, `todo!`, `unimplemented!` und kein `#[cfg(test)]`-Hook
in der Produktions-API-Schicht (MCP, Store, Text, Python-Bindings).

## Akzeptanzkriterien
- [ ] Gate `test-surface` grün: keine `#[cfg(test)]`-Importe in Produktions-Modulen
- [ ] Gate `panic-surface` grün (oder Ratchet unterschritten)
- [ ] `cargo clippy -D clippy::unwrap_used` in betroffenen Crates grün
- [ ] `cargo test --workspace` grün

## Scope
crates/contextra-store/, crates/contextra-text/, crates/contextra-mcp/, contextra-py/

## Veto-Check
keine

## Out-of-scope
Kein Anfassen von `crates/contextra-engine/` oder anderen Crates.
Kein Ändern von Gate-Definitionen.

## WP-Referenz
PLAN.md §3 — WP 1.3 (Befund D-12)

## Abhängigkeit
WP 0.1 muss abgeschlossen sein.' \
    "Phase 1 – Stubs" \
    "jules,task"
}

create_phase_2() {
  print_info "Erstelle Phase-2-Issues (WP 2.1–2.6)…"
  print ""

  create_issue \
    "WP 2.1 — Archiv-Tag archiv/v0 setzen und Kanten-Inventar erstellen" \
'## Ziel
Den Stand vor der Archivierung der Archiv-Crates unveränderlich markieren.
Ein vollständiges Inventar aller Abhängigkeitskanten zu Archiv-Crates erstellen.

## Akzeptanzkriterien
- [ ] Git-Tag `archiv/v0` auf aktuellem HEAD gesetzt und gepusht
- [ ] `MIGRATION-EDGES.md` im Repo-Root: listet alle `engine→graph`, `engine→adapt`,
      `engine→kvcache`, `engine→infer-candle`, `db→cognition`, `db→graph`,
      `db→adapt`, `mcp→*`, `contextra→*` Kanten
- [ ] Archiv-Crates aus `default-members` in `Cargo.toml` entfernt
- [ ] `cargo build --workspace` weiterhin grün
- [ ] `cargo test --workspace` weiterhin grün

## Scope
Workspace-Root (Cargo.toml default-members), Dokumentation

## Veto-Check
keine

## Out-of-scope
Kein Entfernen von Code in diesem WP — nur Tag und Inventar.
Kein Entkoppeln von Crates in diesem WP (→ WP 2.2+).

## WP-Referenz
PLAN.md §4 — WP 2.1

## Abhängigkeit
WP 1.x (Phase 1) abgeschlossen.' \
    "Phase 2 – Archivierung" \
    "jules,task"

  create_issue \
    "WP 2.2 — contextra-engine von Archiv-Crates entkoppeln" \
'## Ziel
`contextra-engine` hängt nicht mehr von `graph`, `adapt`, `kvcache`, `infer-candle`, `sandbox` ab.
Gate `ring-layering` grün für `engine`.

## Akzeptanzkriterien
- [ ] `Cargo.toml` von `contextra-engine`: keine Abhängigkeit auf Archiv-Crates
- [ ] Gate `ring-layering` für `contextra-engine` grün
- [ ] `cargo build -p contextra-engine` grün
- [ ] `cargo test -p contextra-engine` grün
- [ ] Alle betroffenen Codepfade ersetzt durch Feature-Flags oder Ports

## Scope
crates/contextra-engine/

## Veto-Check
keine

## Out-of-scope
Kein Entkoppeln von `db`, `mcp`, `contextra` in diesem WP (→ WP 2.3).
Kein physisches Entfernen der Archiv-Crates (→ WP 2.1b).

## WP-Referenz
PLAN.md §4 — WP 2.2

## Abhängigkeit
WP 2.1 muss abgeschlossen sein.' \
    "Phase 2 – Archivierung" \
    "jules,task"

  create_issue \
    "WP 2.3 — db/mcp/contextra entkoppeln; contextra-core entfernen" \
'## Ziel
`contextra-db`, `contextra-mcp` und `contextra` (Composition Root) hängen nicht mehr von
Archiv-Crates ab. `contextra-core` ist vollständig entfernt (Migrationsfassade ersetzt durch
`contextra-types` / `contextra-ports`, D-05).

## Akzeptanzkriterien
- [ ] `contextra-db`, `contextra-mcp`, `contextra` — keine Archiv-Crate-Abhängigkeiten
- [ ] `contextra-core` aus Workspace entfernt (Cargo.toml)
- [ ] `rg "contextra-core" Cargo.toml` → 0 Treffer in allen Crate-Manifesten
- [ ] Gate `ring-layering` grün für alle betroffenen Crates
- [ ] `cargo build --workspace` grün

## Scope
crates/contextra-db/, crates/contextra-mcp/, crates/contextra/

## Veto-Check
VETO-F10 geprüft: Mandanten-Isolation bleibt vollständig erhalten.

## Out-of-scope
Kein physisches Entfernen der anderen Archiv-Crates (→ WP 2.1b nach 2.2+2.3).

## WP-Referenz
PLAN.md §4 — WP 2.3 (D-05)

## Abhängigkeit
WP 2.2 muss abgeschlossen sein.' \
    "Phase 2 – Archivierung" \
    "jules,task"

  create_issue \
    "WP 2.4 — Rand-Crates als optional markieren (optional = true + Features)" \
'## Ziel
Rand-Crates (`infer-candle`, `sandbox`, `privacy`, `py`) sind über Cargo-Features opt-in.
Default-Build enthält keine Rand-Crates.

## Akzeptanzkriterien
- [ ] Alle Rand-Crate-Abhängigkeiten in `crates/contextra/Cargo.toml` als `optional = true`
- [ ] Feature-Flags korrekt benannt und dokumentiert
- [ ] `cargo build -p contextra` (ohne Features) grün
- [ ] `cargo build -p contextra --all-features` grün
- [ ] Gate `feature-consistency` grün

## Scope
crates/contextra/ (Composition Root), capabilities.toml

## Veto-Check
A1: Sicherheit ist kein Feature-Schalter — Verschlüsselung darf NICHT optional sein.

## Out-of-scope
Kein Ändern der Rand-Crate-Implementierungen selbst.

## WP-Referenz
PLAN.md §4 — WP 2.4

## Abhängigkeit
WP 2.3 muss abgeschlossen sein.' \
    "Phase 2 – Archivierung" \
    "jules,task"

  create_issue \
    "WP 2.5 — capabilities.toml auf Klassen und strikte may_depend_on erweitern" \
'## Ziel
`capabilities.toml` ist vollständig: jedes Crate hat Ring, Klasse [K/R/E/A] und
`may_depend_on`-Allowlist. Gate-Checks prüfen tatsächlich gegen diese Allowlist.

## Akzeptanzkriterien
- [ ] Alle Crates in `capabilities.toml` eingetragen mit `ring`, `class`, `may_depend_on`
- [ ] Gate `ring-layering` nutzt `may_depend_on` als einzige Erlaubnisliste
- [ ] Gate `ring-layering` vollständig grün
- [ ] `cargo xtask check-ring-layering --strict` grün
- [ ] Keine Archiv-Crates mehr in `may_depend_on` aktiver Crates

## Scope
capabilities.toml, governance/, xtask/

## Veto-Check
keine

## Out-of-scope
Kein Ändern von Produktionscode.

## WP-Referenz
PLAN.md §4 — WP 2.5 (INV-DOC-SINGLE-SOURCE-1)

## Abhängigkeit
WP 2.3 muss abgeschlossen sein.' \
    "Phase 2 – Archivierung" \
    "jules,task"

  create_issue \
    "WP 2.6 — Unsafe-Inseln bereinigen (simd, sys, wire)" \
'## Ziel
Alle `unsafe`-Blöcke in `contextra-simd`, `contextra-sys`, `contextra-wire` haben
ein vollständiges `// SAFETY:`-Kommentar. Kernel-Funktion in `simd` ist `pub(crate)`.
`core`-Abhängigkeit in `simd` ist entfernt (P2, P6).

## Akzeptanzkriterien
- [ ] Gate `unsafe-audit` grün: 0 SAFETY-Lücken
- [ ] `rg "unsafe \{" crates/contextra-simd/ | grep -v "// SAFETY:"` → 0 Treffer
- [ ] `rg "unsafe \{" crates/contextra-sys/ | grep -v "// SAFETY:"` → 0 Treffer
- [ ] `rg "unsafe \{" crates/contextra-wire/ | grep -v "// SAFETY:"` → 0 Treffer
- [ ] `allow(unwrap)` in `wire` nur auf generierte Dateien eingeschränkt
- [ ] `cargo test --workspace` grün

## Scope
crates/contextra-simd/, crates/contextra-sys/, crates/contextra-wire/

## Veto-Check
P2: Unsafe nur in drei Inseln (simd, sys, wire) — keine neuen Inseln anlegen.

## Out-of-scope
Kein Anfassen anderer Crates.
Kein Entfernen von `unsafe`-Code wenn er korrekt begründet ist.

## WP-Referenz
PLAN.md §4 — WP 2.6 (Befunde: 5 SAFETY-Lücken in unsafe-audit)

## Abhängigkeit
WP 0.4 muss abgeschlossen sein.' \
    "Phase 2 – Archivierung" \
    "jules,task"
}

# ── Haupt-Ablauf ──────────────────────────────────────────────────────────────
main() {
  print ""
  print_info "Welche Phasen sollen erstellt werden?"
  print "  0) Nur Phase 0 (WP 0.1–0.4)"
  print "  1) Nur Phase 1 (WP 1.1–1.3)"
  print "  2) Nur Phase 2 (WP 2.1–2.6)"
  print "  a) Alle Phasen (0+1+2)"
  print ""
  print -n "Deine Wahl [0/1/2/a]: "
  read -r CHOICE

  TOTAL_CREATED=0
  TOTAL_FAILED=0

  case "$CHOICE" in
    0) create_phase_0 ;;
    1) create_phase_1 ;;
    2) create_phase_2 ;;
    a|A) create_phase_0; create_phase_1; create_phase_2 ;;
    *) print_error "Ungültige Auswahl: '$CHOICE'"; exit 1 ;;
  esac

  print ""
  print -P "${C_BOLD}═══════════════════════════════════════${C_RESET}"
  print_ok "Issue-Erstellung abgeschlossen."
  print ""
  print_step "Offene Jules-Issues:"
  print_step "Führe aus: gh issue list --repo ${REPO} --label jules --state open"
  gh issue list --repo "${REPO}" --label "jules" --state open 2>/dev/null \
    || print_warn "Konnte Issues nicht auflisten."
}

main "$@"
