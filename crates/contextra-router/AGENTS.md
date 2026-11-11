# AGENTS.md — Entwicklungsrichtlinien für `contextra-router`

<!--
Hinweis für Agenten:
- Lies diese Datei vor jeder Code-Änderung in diesem Crate.
- Alle Prüfungen müssen nach jeder Änderung ausgeführt werden.
-->

## 1. Crate-Übersicht & Baukasten-Profil

- **Name:** `contextra-router`
- **Ring:** Ebene 2 (Integration/Applikation)
- **Zweck:** Intelligentes Routing von Anfragen an geeignete SLM-Profile basierend auf Relevanz, Budgets und statistischer Konfidenz.
- **Kern-Closure:** `contextra-types`, `contextra-ports`, `contextra-adapt`.
- **Optionale Closure:** `contextra-wire`, `tokio`, `shlex` (nur hinter `subprocess-dispatch`).
- **Dateisystem-Zugriff:** Über Naht `CalibrationSource` (`src/seam.rs`).

## 2. Standard-Befehle

```bash
# Basis-Check ohne Default-Features
cargo check -p contextra-router --no-default-features --locked

# Check aller Targets mit allen Features
cargo check -p contextra-router --all-targets --all-features --locked

# Tests ausführen
cargo test -p contextra-router --locked
cargo test -p contextra-router --all-features --locked

# Clippy-Prüfung
cargo clippy -p contextra-router --all-targets --all-features --no-deps --locked -- -D warnings
```

## 3. Invarianten & Regeln

- **Kein Panik-Code:** Kein `unwrap()`, `expect()` oder `panic!()` in Nicht-Test-Code.
- **Unsafe-Code:** `#![forbid(unsafe_code)]` strikt beibehalten.
- **Determinismus:** Routing-Entscheidungen und Arm-Mappings sind deterministisch. Tie-Breaks erfolgen aufsteigend nach Entity-/Profil-ID.
- **Ports & Naht:** Dateisystem-Zugriffe für Kalibrierungen erfolgen ausschließlich über die Trait-Naht [`CalibrationSource`].

## 4. Anchor-Inventar

Höchste vergebene Anker-IDs nach Bereich:
- `BLD`: `ANCHOR[contextra-router:BLD-003]`
- `SEM`: `ANCHOR[contextra-router:SEM-004]`, Tests: `SEM-T03`
- `LCF`: `ANCHOR[contextra-router:LCF-011]`
- `RTR`: `ANCHOR[contextra-router:RTR-006]`
- `OUT`: `ANCHOR[contextra-router:OUT-009]`, Tests: `OUT-T02`
- `PRF`: `ANCHOR[contextra-router:PRF-008]`, Tests: `PRF-T08`
- `ARM`: `ANCHOR[contextra-router:ARM-004]`, Tests: `ARM-T03`
- `TRP`: `ANCHOR[contextra-router:TRP-003]`, Tests: `TRP-T02`
- `FCTS`: `ANCHOR[contextra-router:FCTS-003]`
- `DSP`: `ANCHOR[contextra-router:DSP-002]`
- `RTS`: `ANCHOR[contextra-router:RTS-001]`, Tests: `RTS-T03`
- `TST-BLD`: `ANCHOR[contextra-router:TST-BLD-001]`
- `TST-CNS`: `ANCHOR[contextra-router:TST-CNS-001]`
