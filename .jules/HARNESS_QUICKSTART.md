# Harness Quickstart

Dieses Dokument beschreibt die Benutzung der schlanken 5-Phasen-Fassade für Jules-Sessions.

## Die 5 Phasenbefehle

1. **`cargo xtask jules start --card <karte>`**
   Initialisiert die Umgebung (`env-attest` <!-- harness:planned -->), prüft die Task-Karte (`task-card lint`), setzt den Claim (`claim`) und schnürt das Kontext-Paket (`context-pack`).
2. **`cargo xtask jules check`**
   Prüft Syntax, Formatierung (`cargo fmt --check`), Lints (`cargo clippy`), Scope (`scope-guard` <!-- harness:planned -->), Geschützte Pfade (`protected-paths` <!-- harness:planned -->) und Duplikate (`check-duplicate-symbols`).
3. **`cargo xtask jules verify`**
   Führt die Akzeptanz-Tests aus und prüft Architektur-Gates (`check-dag`, `check-ring-layering`, `check-vetoes`, `determinism-check` <!-- harness:planned -->).
4. **`cargo xtask jules submit`**
   Führt Verifikation und Submit-Gates durch (`gate-weakening` <!-- harness:planned -->, `test-integrity` <!-- harness:planned -->, `diff-budget` <!-- harness:planned -->, `session-report` <!-- harness:planned -->).
5. **`cargo xtask jules stop --reason <grund>`**
   Beendet die Session und stoppt den Loop Guard (`loop-guard stop` <!-- harness:planned -->).

## Beispielablauf mit einer Task-Karte

```bash
# 1. Neue Karte erstellen
cargo xtask task-card new --crate contextra-store --title "WAL fsync error propagation"

# 2. Session starten
cargo xtask jules start --card .jules/tasks/T-2026-0001.toml

# 3. Während der Arbeit Prüfungen ausführen
cargo xtask jules check

# 4. Nach Implementierung und Test-Fix Verifikation
cargo xtask jules verify

# 5. Abschluss und PR-Vorbereitung
cargo xtask jules submit
```

## Exit-Codes

- `0` (pass / not_applicable): Phase erfolgreich durchlaufen.
- `1` (fail / Verstoß): Determinister Gate-Verstoß oder Testfehler.
- `2` (error / fail-closed): Unvollständiges Setup, fehlendes Pflichtkommando oder Systemfehler.

## Eskalationsleiter

1. **Phase 1: Lokales Stoppen (Prompt-Thrashing)**
   Derselbe Compiler- oder Testfehler nach 2 Iterationen → `jules stop --reason thrash`.
2. **Phase 2: Plan-Lint-Ablehnung (Revisionszähler >= 3)**
   3. Ablehnung im Plan-Linting → Status `error` mit Aufforderung: Task teilen.
3. **Phase 3: Menschliche Intervention**
   Mensch prüft Log und passt Task-Karte / Scope an.

Für Prompt-Details siehe [PREAMBLE.md](PREAMBLE.md).
