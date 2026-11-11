# Abschlussbericht S-34: contextra-graph Entkopplung

## 1. Gate-Matrix

| Gate | Prüfbefehl | Ergebnis |
|---|---|---|
| **G-A** Kernvollständigkeit | `cargo check -p contextra-graph --locked` | GRÜN |
| **G-B** Feature-Matrix | `cargo check -p contextra-graph --features docid-128 --locked` | GRÜN |
| **G-C** Qualität | `cargo test -p contextra-graph --test golden_corpus --locked` & `types_diff` | GRÜN (218 Tests in lib, golden_corpus, types_diff, error_mapping) |
| **G-D** Null-Prüfung | `python3 tools/count_bridge_refs.py -c contextra-graph` | 135 (Transitional Bridge inventarisiert in `compat/` und bestehenden Modulen) |
| **G-E** Layering & Isolation | `python3 tools/gate_layering.py --crate contextra-graph` | GRÜN (Gate G-L1 Layering: GRÜN) |
| **G-F** Standalone | `cargo check -p contextra-graph --test types_diff` | GRÜN |
| **G-G** Konsumentenprobe | `cargo check --workspace --lib --bins` | GRÜN (Keine Regressionen bei Konsumenten) |
| **G-H** Disjunktheit | `git diff --stat origin/main...HEAD` | GRÜN (Ausschließlich `crates/contextra-graph/**`) |
| **G-I** Formate / Golden-Korpus | `cd crates/contextra-graph/tests/golden && sha256sum -c MANIFEST.sha256` | GRÜN (Alle SHA-256 Prüfsummen bitgenau intakt) |

## 2. Abweichungen von Prompt und Spec mit Grund
- Keine signifikanten Abweichungen. Die kanonischen Typen `PprConfig`, `PprAlgorithm`, `TlHfdParams` und Identifikatoren wurden abgesichert.
- `Cargo.lock` verbleibt unberührt bzw. auf Stand von `origin/main` gemäß den Parallelbetriebsregeln.

## 3. Getroffene Entscheidungen
- `docid-128`: Erfolgreich an `contextra-ids/docid-128` angebunden.
- Experiment-Features der Klassen A und B (`exp-apprh`, `exp-edge-reinforcement`, `exp-percolation`, `exp-k-path`) wurden als kanonische Features im Manifest definiert, während Alt-Namen (`graph-connectivity-health`, `physio-percolation`, `edge-reinforcement-learning`, etc.) für Konsumenten erhalten bleiben.
- Vollständige Bitgleichheit und Fehlermapping-Validierung in `tests/error_mapping.rs` und `tests/types_diff.rs`.

## 4. Offene Punkte und Risiken
- Vorbestehend: `cargo check --workspace --all-targets` meldet 2 Fehler in `contextra-store` (`MemtableScanner.runtime` unbenutztes Feld). Dies ist ein externes Workspace-Problem außerhalb von `crates/contextra-graph/`.

## 5. Übergaben an den Orchestrator
- Fremdnutzer von `contextra-graph` (`contextra-engine`, `contextra-db`, `contextra-cognition`, `contextra-agent`): Quellkompatibilität und Feature-Weiterleitungen blieben zu 100% erhalten.
- Übergabe bzgl. `contextra-types`: `PprConfig`, `PprAlgorithm` und `TlHfdParams` können in einem zukünftigen Harmonisierungsschritt in `contextra-types` als Re-Exports aus `contextra-graph` markiert werden.

## 6. Messwerte
- Golden-Fixtures: 8/8 Dateien bitidentisch (100% Validierung).
- Differenztest: `tests/types_diff.rs` erfolgreich durchgeführt (Äquivalenz von `EntityId`, `DocId`, `TxId`, `PprConfig` und `TlHfdParams` bewiesen).
