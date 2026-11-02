# Änderungsprotokoll: Optimierung der Jules-Kontextkette (2026-11-02)

Nicht normativ. Bezug: SPEC 4.3.0, geprüft am Repository-HEAD `909ff27`.

## Geändert (zum Überschreiben)

| Datei | Grund |
|---|---|
| `AGENTS.md` | Jules-Teil auf eigenständiges Arbeiten mit Stoppregeln umgestellt. Ist-Zustand auf geprüfte Fakten gebracht (Testmakro fehlt, `contextra-erasure` ist Skelett, Lockdatei veraltet). Verweis auf `xtask` entfernt, da das Ziel nicht existiert. |
| `.agents/README.md` | Versionsangabe 4.3.0. Ehrliche Angabe, welche Dateien geprüft sind. |
| `.agents/hooks/session_start.sh` | Veralteter Verweis „Ebene A/B“ ersetzt (SPEC 4.x hat keine Ebene B). |
| `.agents/jules/PREAMBLE.md` | Entscheidungsregeln, Stopp-Katalog (S1 bis S4), Übergangsregel für das fehlende Testmakro, Prüfbefehle, Commit- und PR-Regeln. |
| `.agents/jules/SESSION_BOOTSTRAP.md` | Ablauf ohne Rückfragen. Arbeitsliste als Quelle statt GitHub-Issues. Keine destruktiven Git-Befehle mehr. |
| `.agents/jules/JULES_CONTEXT.md` | Fakten aktualisiert. Stolperstellen benannt (kaputtes `just gates`, `cargo xtask`, `docs/PLAN.md`). |
| `.agents/jules/tasks/README.md` | Beschreibt das tatsächlich verwendete Kartenschema. |
| `.agents/jules/tasks/_TEMPLATE.toml` | Felder an SPEC angepasst. Verweise auf nicht vorhandene Dateien (`prompter-tiers.toml`, `contextra-core`, `.jules/`) entfernt. |
| `.agents/jules/tasks/BACKLOG.md` | Neu. Abarbeitungsreihenfolge aus SPEC 15.4 mit Status. |
| `.agents/jules/tasks/WP-0.3.toml`, `WP-1.1.toml` | Neu. Karten für die ersten Pakete, mit geprüften Ausgangsdaten. |
| `.agents/jules/setup/refresh-context.sh` | `git reset --hard` und `git clean` entfernt, weil sie nicht committete Arbeit der Sitzung verwarfen. Repository-Slug aus `origin` statt fest verdrahtet. Liest die Arbeitsliste ein. |
| `.agents/jules/setup/environment_script.sh` | Prüfung auf nicht vorhandenes `contextra-mcp` ersetzt. Kommentar zu `contextra-wire` korrigiert. |
| `.github/pull_request_template.md` | Verweis auf `docs/PLAN.md` entfernt. Pflichtabschnitte aus PREAMBLE ergänzt. |
| `.github/workflows/refresh-jules-context.yml` | Pfad `agents/` → `.agents/`. Commit-Schritt entfernt, weil die Datei nicht versioniert wird. Berechtigung auf `read` gesenkt. |

## Nur durch einen Menschen zu übernehmen (geschützte Pfade)

Laut `.agents/governance/protected-paths.toml` gelten `AGENTS.md`, `.github/**`, `.agents/jules/setup/**`, `justfile`, `rust-toolchain.toml`, `deny.toml`, `capabilities.toml`, `CODEOWNERS` und `governance/**` als geschützt. `.agents/hooks/` ist nicht geschützt. Jules darf die geschützten Pfade nicht ändern.

## Offen, nicht erledigt

1. **Build nicht verifiziert.** Die Prüfumgebung dieses Pakets hatte keine Rust-Toolchain (Download blockiert). Alle Build-Aussagen stammen aus SPEC F-32 und aus statischer Prüfung. Die erste Jules-Sitzung muss `cargo check --workspace --all-targets --keep-going` ausführen und die Fehlerliste zur Grundlage von WP-0.3 machen.
2. **Testmakro `ctx_test` fehlt.** Übergangsregel in PREAMBLE D. Ablösung durch WP-0.1.
3. **`just gates` ist defekt.** Das Ziel ruft `tests/layering.rs` auf, das nicht existiert (Arch-Test liegt in `tests/arch`). Korrektur durch einen Menschen am `justfile` (geschützt) oder durch WP-0.1.
4. **`.gitignore`, `CODEOWNERS`, `scheduled-audit.yml` und weitere Workflows** verweisen noch auf `.jules/`. Die aktive Ablage ist `.agents/jules/`.
5. **Nicht überarbeitet:** `.agents/jules/routines/*`, `COMMON_LLM_ERRORS.md`, `HARNESS_QUICKSTART.md`, `ISSUE_AUTOMATION_POLICY.md`, `AUDIT_INTAKE_PROTOCOL.md`, `SCHEDULED_AUDIT.md`, `.agents/rules/*`, `.agents/skills/*`, `.agents/claude/CLAUDE.md`, `docs/Jules-Prompts/*` (letztere enthalten 56 Verweise auf `cargo xtask`). Diese Dateien sind vor der nächsten Routine zu prüfen.
6. **Arbeitsliste lückenhaft.** Karten für WP-0.1, WP-0.2 und ab WP-1.2 entstehen bei Aufnahme. Das ist Absicht, damit Jules die Karte aus dem aktuellen Stand ableitet.
7. **Geschützte Dateien mit Befunden:** `deny.toml` (WP-1.3), `rust-toolchain.toml` (WP-0.2), `.github/workflows/*` (Graph-Verweise in WP-1.1).
