# ADR-096: Analyse & Ersatzstrategie für das gelöschte `scripts/fast_diff_lint.py` (Substitutionsprüfung & Fähigkeitslücken)

* **Status:** ✅ Accepted
* **Datum:** 2026-09-26
* **Kontext / Auslöser:** Im Zuge der Konsolidierung der Meta-Entwicklungsinfrastruktur (PR #3746) wurde das Python-Skript `scripts/fast_diff_lint.py` (261 Zeilen) ersatzlos aus dem Quellbaum entfernt. Dieses ADR analysiert maschinell und strukturell alle Prüffunktionen des gelöschten Skripts, vergleicht sie mit bestehenden `xtask`-Gates sowie `cargo clippy`-Lints und bewertet verbleibende Fähigkeitslücken.

## 1. Detaillierte Vergleichstabelle (Prüfungen vs. Gate-Abdeckung)

| # | Prüfung aus `fast_diff_lint.py` | Funktionsbeschreibung | Abgedeckt durch (Gate-Name / Lint) | Status / Befund |
|---|---|---|---|---|
| 1 | **Fast-Diff Dateispreizung** (`git diff --name-only`) | Eingrenzung der Inspektion auf in `git diff` geänderte Dateien/Zeilen für Sub-Sekunden-Feedback | `cargo xtask check-duplicate-symbols` / `check-commit-diff-integrity` (partiell via `get_changed_rs_files_from_git_diff`) | **LÜCKE** (Workspace-Gates wie `jules-preflight` prüfen stets den gesamten Workspace statt reiner Diff-Inkremente) |
| 2 | **Verbogene / unerlaubte Unwraps & Panics** (`.unwrap()`, `.expect()`, `panic!()`, `todo!()`, `unimplemented!()`) | Erkennung expliziter Panics und Unwraps in neu hinzugefügten Diff-Zeilen | `[workspace.lints.clippy]` (`unwrap_used = "deny"`, `expect_used = "deny"`, `panic = "deny"`, `todo = "deny"`, `unimplemented = "deny"` in `Cargo.toml`) | **Abgedeckt** (Vollständige statische Absicherung über Cargo Workspace Lints) |
| 3 | **Git-Merge-Konfliktmarker** (`<<<<<<<`, `=======`, `>>>>>>>`) | Scan geänderter Dateien auf verbliebene Merge-Konflikt-Artefakte | *Kein dediziertes Gate* | **LÜCKE** (Echte Fähigkeitslücke: Verbliebene Conflict Markers in Nicht-Rust-Dateien werden nicht vor dem Commit abgefangen) |
| 4 | **Verbleibende Debug-Prints & Macro-Relikte** (`dbg!`, `breakpoint()`, `console.log`) | Schutz vor versehentlich eingebrachten temporären Debug-Aufrufen in Produktions-Code-Zeilen | `clippy::dbg_macro` (bei Full-Builds), jedoch kein Lightweight-Diff-Filter für `println!`/`eprintln!` in Nicht-CLI-Crates | **LÜCKE** (Partielle Fähigkeitslücke bei schnellem Pre-Commit-Filtering) |
| 5 | **Formatierung Hygiene & Trailing Whitespace** | Prüft Whitespace-Qualität und fehlende EOF-Newlines an geänderten Stellen | `cargo fmt --check` (im `gate-check`-Workflow) | **Abgedeckt** (Rustfmt erzwingt Whitespace- und Newline-Standards workspace-weit) |
| 6 | **AI-TAG & ANCHOR Syntax-Validierung** | Prüft ISO-8601 Zeitstempel und SESSION-IDs in neuen Inline-Kommentaren | `cargo xtask validate-tags` | **Abgedeckt** (Überprüft alle Tag-Formate in `crates/`) |
| 7 | **FFI-Panic-Isolation & Bounds-Schutz** | Überprüfung von FFI-Panics an C/Python-Boundaries | `cargo xtask check-ffi-panic-boundary` & `cargo xtask lint-unsafe-slices` | **Abgedeckt** (Dedizierte Ring-0/FFI-Gates in `xtask`) |
| 8 | **Phantom-Dateien & Unerreichbare Module** | Prüft verwaiste `.rs`-Dateien oder nicht-verlinkte Modulstrukturen | `cargo xtask check-phantom-files` & `cargo xtask check-module-reachability` | **Abgedeckt** (Statische Reachability-Analyse in `xtask`) |
| 9 | **Commit Claim vs. Diff Integrity** | Gleicht Behauptungen in Commit-Messages mit tatsächlichen Diff-Statistiken ab | `cargo xtask check-commit-diff-integrity` | **Abgedeckt** (Phantom-Commit Protection Gate) |

---

## 2. Bewertung der echten Fähigkeitslücken (Gaps)

Aus der Gegenüberstellung ergeben sich **drei (3) konkrete Fähigkeitslücken**:

1. **Gap A (Inkrementeller Line-Diff-Scan vs. Full Workspace Inspection):**
   - *Problem:* `fast_diff_lint.py` arbeitete in <50ms rein auf dem Git-Diff. Das `xtask`-Ökosystem führt stattdessen vollständige Workspace-Scans durch (`jules-preflight`, `sync-docs`).
   - *Bewertung & Empfehlung:* **Bewusst akzeptierter Verlust.** Ein reiner Regex-Line-Diff-Linter täuscht Sicherheit vor, da er makrobasierte oder kontextuelle Regelverstöße übersieht. Die paar Hundert Millisekunden Mehraufwand für vollständige, typ- und AST-sichere Scans via `cargo clippy` und `cargo xtask` garantieren deterministische Freiheit von Regressionen.

2. **Gap B (Git Merge Conflict Marker Detection):**
   - *Problem:* Wenn bei Merges Konflikt-Marker (`<<<<<<<`, `=======`, `>>>>>>>`) in Markdown-, Config- oder Schemadateien verbleiben, schlägt `cargo clippy` nicht an (da sie außerhalb des Rust-AST liegen).
   - *Bewertung & Empfehlung:* **Migration als `xtask`-Subcommand.** Erweiterung von `cargo xtask check-phantom-files` oder Erstellung eines Subcommands `cargo xtask check-diff-hygiene`, das geänderte Dateien auf Konflikt-Marker prüft.

3. **Gap C (Leftover Debug Print Detection in Non-CLI Crates):**
   - *Problem:* `dbg!()` wird zwar von Clippy bemängelt, aber `println!`/`eprintln!`-Aufrufe in tiefen Ring-0/Ring-1 Kernkomponenten (z.B. `contextra-store`, `contextra-mvcc`) entgehen dem Standard-Linting.
   - *Bewertung & Empfehlung:* **Erweiterung von `xtask`.** Ergänzung eines AST/Regex-Lints in `xtask`, der `println!`-Aufrufe außerhalb von CLI/Binary-Crates unterbindet.

---

## 3. Entscheidung & Handlungsanweisungen

1. **Keine Wiederbelebung von `scripts/fast_diff_lint.py`:**
   - Gemäß ADR-N01 und den Architektur-Richtlinien werden keine unstrukturierten Python-Skripte im Repository re-instanziiert.
2. **Geführte Migration der Lücken in `xtask`:**
   - Die verbleibenden Prüf-Aufgaben für Merge-Konfliktmarker und unzulässige Debug-Logs werden in Nachfolge-Tasks als nativ kompilierte `xtask`-Rust-Module (`xtask/src/check_diff_hygiene.rs`) implementiert.
3. **Dokumentierte Akzeptanz der Geschwindigkeits-Differenz:**
   - Die geringfügige Zeitersparnis des alten Python-Diff-Linters wiegt das Risiko unvollständiger Prüfungen nicht auf.
