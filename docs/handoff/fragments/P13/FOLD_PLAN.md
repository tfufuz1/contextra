# Falt-Plan: Einfalten von `contextra-db` in `contextra` (Vorbereitung für WP-100 / P42)

Stand: 2026-11-09
Autor: Agent P13
Status: Konzeptioneller Plan (keine direkte Umsetzung in P13)

---

## 1. Ziel & Kontext

`contextra-db` dient gegenwärtig als High-Level Orchestrator und Integrationsfassade. Die oberste Endanwender-Fassade ist das Crate `contextra`. Gemäß der System-Spezifikation (Entscheidung E6 / WP-100 / P42) soll `contextra-db` vollständig in `contextra` aufgehen, sodass es nur noch eine einzige, unifizierte Produktfassade gibt.

---

## 2. Analyse der genutzten `pub`-Symbole außerhalb von `contextra-db`

Ein systemweites Audit (`grep -rn "contextra_db::" crates benchmarks examples`) identifiziert die folgenden externen Konsumenten und genutzten Symbole:

### 2.1 Crate `contextra` (direkter Aufnehmer)
- **Symbole re-exportiert in `crates/contextra/src/lib.rs`:**
  - `Collection`, `CollectionConfig` (eigentlich aus `contextra-engine::collection`)
  - `Contextra`, `ContextraConfig`, `ContextraStats`, `Document`, `SearchResult`, `EmbeddingBackend` (aus `contextra-engine`)
  - `chunker` (aus `contextra-engine::chunker`)
  - `DriftStatusProvider`, `TextEmbeddingEngine` (aus `contextra-ports`)
- **Symbole in `crates/contextra/src/builder.rs` & `agent_memory.rs`:**
  - `Contextra`, `ContextraConfig`, `EmbeddingBackend`, `TextEmbeddingEngine`, `ProvenanceRecord`, `SearchResult`
- **Symbole in `crates/contextra/src/query_rewriter.rs`:**
  - `contextra_db::QueryRewriter` (definiert in `contextra-db/src/multistep.rs`)
- **Symbole in `crates/contextra/src/performance_profile.rs`:**
  - `contextra_db::DurabilityMode` (aus `contextra_engine::DurabilityMode`)

### 2.2 Crate `contextra-agent`
- `contextra_db::Contextra`, `contextra_db::ContextraConfig`, `contextra_db::Collection`
- `contextra_db::DistanceMetric` (aus `contextra-types`)
- `contextra_db::volatile_vault::{SignalModality, VaultChunk, VaultConfig, VolatileContextVault}`

### 2.3 Benchmarks & Examples
- `benchmarks/rrf_scale_bench.rs`: `contextra_db::fusion::weighted_reciprocal_rank_fusion`, `SearchResult`
- `benchmarks/competitive_bench.rs`, `relate_bench.rs`, `scale_bench.rs`, `migration_benchmarks.rs`, `contextra-bench`: `Contextra`, `ContextraConfig`, `SearchStrategy`
- `examples/{quickstart, hybrid_search, collections}.rs`: `Contextra`, `ContextraConfig`

---

## 3. Namenskollisionen mit `contextra`

Bei der Zusammenführung mit `contextra` sind folgende Module und Typen auf Namensgleichheit geprüft worden:

1. **`Contextra` / `ContextraConfig`:**
   - In `contextra-db`: Re-Export von `contextra_engine::Contextra`.
   - In `contextra`: Re-Export von `contextra_db::Contextra` (über `ContextraBuilder::build()`).
   - *Kollision:* Keine semantische Kollision. In `contextra` wird `Contextra` künftig direkt aus `contextra-engine` importiert und re-exportiert.
2. **`volatile_vault`:**
   - Existiert nur in `contextra-db`. Kann 1:1 als `contextra::volatile_vault` (unter feature `volatile-vault`) übernommen werden.
3. **`multistep`:**
   - `contextra-db` enthält `multistep.rs` mit `MultiStepEngine`, `MultiStepConfig`, `MultiStepResult` und `QueryRewriter`.
   - `contextra` enthält `query_rewriter.rs` mit `LlmQueryRewriter` (der `contextra_db::QueryRewriter` implementiert).
   - *Harmonisierung:* `multistep.rs` zieht nach `contextra::multistep` um; `LlmQueryRewriter` implementiert dann direkt das lokale `crate::multistep::QueryRewriter`.
4. **`homeostat` und `pid_latency_controller`:**
   - Existieren in `contextra-db/src/homeostat.rs` und `src/pid_latency_controller.rs`.
   - Können als interne Module nach `contextra::latency` oder `contextra::pid` überführt werden.
5. **`DurabilityMode`:**
   - Sowohl in `contextra` als auch in `contextra-engine` definiert. Die Konvertierung `From<contextra_db::DurabilityMode>` in `performance_profile.rs` entfällt bzw. bindet direkt an `contextra_engine::DurabilityMode` an.

---

## 4. Empfohlene Reihenfolge der Dateiverschiebung (WP-100)

1. **Schritt 1: Quellcode überführen (`src/`)**
   - `crates/contextra-db/src/volatile_vault.rs` (+ `.inc`-Dateien) -> `crates/contextra/src/volatile_vault.rs`
   - `crates/contextra-db/src/multistep.rs` (+ `.inc`-Dateien) -> `crates/contextra/src/multistep.rs`
   - `crates/contextra-db/src/homeostat.rs` -> `crates/contextra/src/homeostat.rs`
   - `crates/contextra-db/src/pid_latency_controller.rs` -> `crates/contextra/src/pid_latency_controller.rs`
   - `crates/contextra/src/lib.rs` erweitern um die Re-Exports aus `contextra-engine` (`Collection`, `Contextra`, etc.) und die lokalen Module.

2. **Schritt 2: Fuzzing-Target umhängen**
   - `crates/contextra-db/fuzz/fuzz_targets/rrf_fusion.rs` -> Verschieben zu `contextra-rank/fuzz` (da es reine RRF-Fusion testet) oder nach `contextra/fuzz`.

3. **Schritt 3: Tests überführen (`tests/`)**
   - Die 50 verbleibenden Testdateien aus `crates/contextra-db/tests/` nach `crates/contextra/tests/` verschieben.
   - Such-/Ersetzungsregel in den verschobenen Tests:
     - `use contextra_db::` -> `use contextra::`
   - Ergänzung der benötigten Dev-Dependencies im Manifest `crates/contextra/Cargo.toml`.

4. **Schritt 4: Konsumenten anpassen (Zwei-Phasen-Regel)**
   - `crates/contextra-agent`: Abhängigkeit von `contextra-db` umstellen auf `contextra`.
   - `benchmarks/` und `examples/`: Imports von `contextra_db` auf `contextra` umstellen.

5. **Schritt 5: `contextra-db` stilllegen / depublizieren**
   - `crates/contextra-db` aus dem Workspace entfernen oder als deprecated Dummy-Crate belassen, bis alle Zweige gemergt sind.
