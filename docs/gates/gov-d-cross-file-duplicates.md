# Governance GOV-D: Dateübergreifende Symbolduplikate im selben Modulverzeichnis

## Übersicht und Unterscheidung

Dieses Dokument dokumentiert den Status des Governance-Gates `GOV-D` (`xtask check-duplicate-symbols-cross-file`).

### Unterschied zu `check-module-reachability`
`check-module-reachability` prüft, ob `.rs`-Dateien im Modulbaum über `mod`-Deklarationen erreichbar sind (Erkennung von Unerreichbarkeit / verwaisten Modulen).
`check-duplicate-symbols-cross-file` prüft stattdessen auf öffentliche Namenskollisionen: Es meldet identisch benannte `pub`-Top-Level-Items (`pub trait`, `pub struct`, `pub enum`, `pub fn`, `pub type`, `pub const`, `pub static`) in verschiedenen Dateien desselben Modulverzeichnisses, unabhängig von deren Erreichbarkeit.

---

## Ergebnisse der Ersterfassung

Das Gate wurde gegen den gesamten Workspace ausgeführt (`cargo xtask check-duplicate-symbols-cross-file`). Es wurden **9 Kollisionen** identifiziert:

| Modulverzeichnis | Fundstellen | Symbol-Typ | Symbolname | Aktion / Folge-PR |
| :--- | :--- | :--- | :--- | :--- |
| `benchmarks/contextra-bench/src` | `ann_benchmarks.rs:112` / `beir_eval.rs:239` | `fn` | `recall_at_k` | TODO: In gemeinsames Bench-Utility auslagern |
| `crates/contextra-cognition/src` | `memory_consolidation.rs:379` / `synthesis_phase.rs:21` | `struct` | `SynthesisPhaseResult` | TODO: Struct-Definition in `contextra-cognition` konsolidieren |
| `xtask/src` | `check_audit_duplication.rs:141` / `check_audit_verdict_independence.rs:226` | `fn` | `find_audit_files` | TODO: Hilfsfunktion in xtask modulieren |
| `xtask/src` | `check_duplicate_symbols_cross_file.rs:210` / `check_placeholder_refs.rs:175` / `gen_prompter_data.rs:164` | `fn` | `run` | TODO: Entrypoints oder Sichtbarkeiten anpassen |
| `xtask/src` | `check_max_results_unbound.rs:6` / `check_nan_validation_in_hot_loop.rs:6` / `check_result_dropped_on_io.rs:6` / `check_toctou_trait_defaults.rs:5` | `struct` | `Violation` | TODO: `Violation`-Structs in shared xtask-Modul verschieben |
| `xtask/src` | `main.rs:1253` / `session_history.rs:93` | `fn` | `find_root_dir` | TODO: Hilfsfunktion entkoppeln |

---

## Modus & CI-Einbindung

Aufgrund der oben dokumentierten Vorbestände läuft das Gate aktuell im **Informationsmodus** (Exit-Code 0 mit detaillierter Warnung und Fundstellen-Ausgabe).
Sobald die oben genannten Blocker in Folge-PRs der jeweiligen Fach-Crates/Tools behoben sind, kann das Gate auf strikt verfehlende Exit-Codes (Exit-Code ≠ 0) umgestellt werden.
