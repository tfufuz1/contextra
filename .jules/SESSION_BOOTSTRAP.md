# Contextra — Jules Session Bootstrap
> Maschinenausführbare Checkliste. Jede Session MUSS mit dieser
> Sequenz beginnen, bevor Code geschrieben oder Dateien geändert werden.

- **VETOES.md** (Root): Permanent abgelehnte oder eingeschränkt akzeptierte Features. Vor Arbeitsbeginn prüfen.

## Phasentabelle (Jules Fassade)

Die Phasen-Runner-Konfiguration (`.jules/harness-phases.toml`) ist die kanonische Quelle aller Gate-Ausführungen.

| Phase | Agent-Gate | Befehl | Zweck |
|---|---|---|---|
| `start` | `guard` / `integrity` | `cargo xtask jules start --card <karte>` | Initialisiert Session, lintet Karte, setzt Claim, erzeugt Kontext |
| `check` | `guard` / `symbols` | `cargo xtask jules check` | Prüft Syntax, Lints, Scope, geschützte Pfade & Symbol-Existenz |
| `verify` | `arch` | `cargo xtask jules verify` | Führt Akzeptanz-Tests, DAG-Checks & Architektur-Gates aus |
| `submit` | `integrity` / `report` | `cargo xtask jules submit` | Verifiziert PR-Integrität, Diff-Budget & Session-Report |
| `stop` | `guard` | `cargo xtask jules stop --reason <grund>` | Stoppt die Session und den Loop Guard bei Eskalation |

```bash
cargo xtask session-init --crate <CRATE_NAME> --task "<KURZE_BESCHREIBUNG>" --output-env <!-- doc-ref-ignore -->
source .jules/session.env <!-- doc-ref-ignore -->
cargo xtask context-pack --crate <CRATE_NAME> <!-- doc-ref-ignore -->
```

Der Befehl `context-pack` erzeugt `.jules/context/CONTEXT_PACK.md` <!-- doc-ref-ignore -->.

## Werkzeug-Prüfung für Einzel-Crate

Für Einzel-Crate-Aufgaben benutze gezielte Checks statt Workspace-weiter Builds:

```bash
cargo check -p <CRATE_NAME>
```

## Aufgabenspezifischer Kontext

| Aufgabe-Typ | Zu lesende Dateien |
|-------------|-------------------|
| Code in `contextra-store/*` <!-- doc-ref-ignore --> | `crates/contextra-store/AGENTS.md`, `rules/wal_crypto.md`, `rules/async-io.md` |
| Code in `contextra-vector/*` <!-- doc-ref-ignore --> | `crates/contextra-vector/AGENTS.md`, `rules/simd_safety.md` |
| Code in `contextra-engine/*` <!-- doc-ref-ignore --> | `crates/contextra-engine/AGENTS.md` |
| Neue Dependency | `rules/dependencies.md` → Cargo.lock prüfen → crates.io verifizieren |
| Neue API-Oberfläche | `CONSTITUTION.md`, `docs/TYPE_REGISTRY.md` |
| ADR schreiben | `docs/decisions/` (letzte 5 ADRs lesen), `CONSTITUTION.md §Governance` |
| Tests schreiben | `rules/testing.md`, `rules/test_quality.md` |
| unsafe Code | `rules/simd_safety.md` — NUR in approved files (AGENTS.md §4) |
| Crypto/WAL | `rules/wal_crypto.md` → WAL-First-Regel verifizieren |

## Notfall-Eskalation (Prompt-Thrashing)

Wenn derselbe Compiler- oder Gate-Fehler nach 2 Iterationen nicht behoben ist:
1. **STOPP** — keinen weiteren Code schreiben.
2. Fehler auf minimales Beispiel reduzieren und in Session-Log dokumentieren.
3. Session abbrechen mit `cargo xtask jules stop --reason thrash`.
