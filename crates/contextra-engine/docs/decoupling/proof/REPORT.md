# Abschlussbericht AUFTRAG ENG-B: `contextra-engine` S-33 / S-60 / S-61 Dublettenprüfung und Differenzbeweise

## 1. Gate-Matrix

| Gate / Prüfung | Ausgeführter Prüfbefehl | Tatsächliches Ergebnis | Status |
|---|---|---|---|
| **ENG-B-0 Baseline-Bestandstests** | `cargo test -p contextra-engine --locked --test chunker_golden_tests --test chunker_hardening_tests --test chunker_parity_ingest --test chunker_robustness --test decay_and_expiry` | 12 passed; 0 failed (0.18s) | Grün |
| **ENG-B-0 Baseline (Feature)** | `cargo test -p contextra-engine --locked --features adaptive-candidate-pool-sizing --test chunker_golden_tests --test chunker_hardening_tests --test chunker_parity_ingest --test chunker_robustness --test decay_and_expiry` | 12 passed; 0 failed (0.17s) | Grün |
| **ENG-B-1 Temporalfilter Beweis** | `cargo test -p contextra-engine --locked --test proof_temporal_filter_vs_rank` | 8 passed; 0 failed (10.000 Proptest-Fälle; 0.66s) | Grün |
| **ENG-B-2 Regler Beweis** | `cargo test -p contextra-engine --locked --features adaptive-candidate-pool-sizing --test proof_regulators_vs_adapt` | 2 passed; 0 failed (10.000 Proptest-Fälle f32-Bitgleichheit; 0.24s) | Grün |
| **ENG-B-3 Chunker Härtung** | `cargo test -p contextra-engine --locked --test proof_chunker_properties` | 2 passed; 0 failed (500 Proptest-Fälle, Malicious UTF-8; 0.17s) | Grün |
| **ENG-B-3 Chunker Divergenz** | `cargo test -p contextra-engine --locked --test proof_chunker_vs_ingest` | 1 passed; 0 failed (Snapshot 210 Dokumente; 0.37s) | Grün |
| **Zero-Panic & Determinismus** | `python3 tools/zero_panic_analyzer.py --crate contextra-engine --json` & `python3 tools/determinism_guard.py --crate contextra-engine --json` | 0 Befunde in `src/chunker/**`, `src/temporal_filter.rs`, `src/decay_controller.rs` | Grün |
| **Dateidisjunktheit (R1, 0.3)** | `git diff --stat origin/main...HEAD` | Ausschließlich Pfade aus Abschnitt 2 (plus maschinell `Cargo.lock` mit `contextra-*` Einträgen) | Grün |

## 2. Abweichungen von Prompt/Spec mit Grund
- **Keine direkte Ablösung von `temporal_filter.rs` durch `pub use contextra_rank::temporal::{...}`:**
  - *Grund (gemäß Prompt ENG-B-1.3/1.4 und R3/0.9):* `contextra-rank` nutzt als autarker Klasse-A-Baustein einen lokalen Newtype `contextra_rank::local_types::TxId` und `calibration::Clock`, während Aufrufer in `contextra-engine` (`src/collection/query_builder/builder_exec.rs`, `src/collection/search/mod.rs`) den Kernel-Typ `contextra_types::TxId` und `contextra_ports::Clock` erwarten. Ein Re-Export hätte Aufrufer in ENG-A-Dateien gebrochen. Der Äquivalenzbeweis der Logik wurde mit 10.000 Proptest-Fällen lückenlos erbracht; die Nicht-Ablösung ist in `TEMPORAL_PARITY.md` detailliert begründet.
- **Keine Ablösung von `decay_controller.rs`:**
  - *Grund (gemäß Prompt ENG-B-2.4):* `contextra-adapt` ist lediglich eine optionale Abhängigkeit (Feature `adaptive-candidate-pool-sizing`). Eine Ablösung würde die Funktion in Standard-Builds (`default = []`) entfernen und dortige Aufrufer sowie Fremdcrate `contextra-db` brechen. Zudem fehlen in `adapt` sechs Methoden zur Clock-Injektion und zum Wall-Clock-Decay.

## 3. Getroffene Entscheidungen
- **Baseline-Reparatur `DecayControllerConfig`:** In `src/decay_controller.rs` wurde das Feld `base_half_life_ms` aus `DecayControllerConfig` entfernt und als `with_base_half_life_ms` an `AdaptiveDecayController` angehängt. Dadurch ist `DecayControllerConfig` wieder 100% quellkompatibel zu bestehenden Struct-Initialisierungen in Tests und identisch zur 3-Felder-Struktur von `contextra-adapt`.
- **Einfrierung Chunker-Korpus:** Ein Referenzkorpus mit 210 Dokumenten (10 Randfälle + 200 prozedurale Dokumente) wurde in `tests/legacy_reference/chunker_legacy_corpus.rs` eingefroren und die Kennzahlen (629 Chunks, 257.858 Tokens, 628 mit Brotkrumen) in `tests/proof_chunker_vs_ingest.rs` und `CHUNKER_PARITY.md` verankert.

## 4. Offene Punkte und Risiken
- Keine offenen Risiken für den bestehenden Engine-Code: Alle 5 dir gehörenden Bestandstests und alle neuen Beweistests sind stabil grün.
- Fremdcrate `contextra-store` weist im Baseline-Workspace einen ungenutzten Member `runtime` in `src/scanner/memtable.rs` auf, was bei Workspace-weitem Check ohne `--cap-lints` auffällt, jedoch außerhalb des Eigentums von ENG-B liegt.

## 5. Übergaben
1. **Übergabe an `contextra-adapt` (Eigner / Maintainer):**
   Folgende Methoden fehlen in `contextra_adapt::decay_controller::AdaptiveDecayController` und müssen dort implementiert werden, bevor eine spätere Ablösung im Engine möglich wird:
   - `pub fn with_clock(config: DecayControllerConfig, clock: Arc<dyn Clock>) -> Self`
   - `pub fn set_clock(&mut self, clock: Arc<dyn Clock>)`
   - `pub fn with_base_half_life_ms(mut self, base_ms: u64) -> Self`
   - `pub fn effective_half_life_ms(&self, temperature: f32) -> f32`
   - `pub fn effective_score_wall_clock(&self, base_score: f32, created_at_ms: u64, inputs: &DecaySignalInputs) -> f32`
   - `pub fn should_evict_wall_clock(&self, base_score: f32, created_at_ms: u64, inputs: &DecaySignalInputs) -> bool`
2. **Übergabe an ENG-A / Maintainer (Chunker-Adapter):**
   - Voraussetzung: `contextra-ingest` müsste in `[dependencies]` von `crates/contextra-engine/Cargo.toml` aufgenommen werden (aktuell nur `dev-dependencies`).
   - Ein Adapter muss flachen Text (`&str`) in `DocumentIr` transformieren und die unfehlbare API von `MarkdownChunker` garantieren.
3. **Übergabe Aufrufstellen in ENG-A-Dateien auf Module von ENG-B:**
   - `crates/contextra-engine/src/collection/search/mod.rs:30`: `pub use crate::temporal_filter::{...}`
   - `crates/contextra-engine/src/collection/query_builder/builder_exec.rs:176,181`: `apply_temporal_validity_filter_at`, `apply_temporal_validity_filter`
   - `crates/contextra-engine/src/collection/maintenance.rs:15,32,108`: `AdaptiveDecayController`, `DecaySignalInputs`, `should_evict`
   - `crates/contextra-engine/src/background_workers/expiry_workers.rs:14,92,99`: `AdaptiveDecayController`, `DecayControllerConfig`
   - `crates/contextra-engine/src/background_workers/tests.rs:600,667,693`: `AdaptiveDecayController`, `DecayControllerConfig`

## 6. Messwerte
- `src/temporal_filter.rs`: 383 Zeilen (unverändert erhalten)
- `tests/legacy_reference/temporal_filter_legacy.rs`: 387 Zeilen
- `tests/legacy_reference/decay_controller_legacy.rs`: 202 Zeilen
- `tests/legacy_reference/chunker_legacy_corpus.rs`: 101 Zeilen
- `tests/proof_temporal_filter_vs_rank.rs`: 186 Zeilen (10.000 Fälle grün)
- `tests/proof_regulators_vs_adapt.rs`: 170 Zeilen (10.000 Fälle f32-Bitgleichheit grün)
- `tests/proof_chunker_properties.rs`: 100 Zeilen (500 Fälle UTF-8/Limits grün)
- `tests/proof_chunker_vs_ingest.rs`: 76 Zeilen (210 Korpusdokumente grün)
