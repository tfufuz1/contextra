# OPEN ISSUES · AP P10 (engine)

## Fremdfehler (Stand vor Beginn AP P10)
1. **contextra-crypto**:
   - `crates/contextra-crypto/src/secret.rs`: `error: unexpected 'cfg' condition value: 'serde'` bei `#[cfg(feature = "serde")]`. Verursacht Abbruch beim Kompilieren von `contextra-crypto` mit `-D unexpected-cfgs`.
2. **contextra-vector**:
   - `crates/contextra-vector/src/hnsw/deletion.rs`: Duplicate definitions with name `remove`.
   - `crates/contextra-vector/src/hnsw/types.rs`: Duplicate definitions with name `rebuild_sync`.
   - `crates/contextra-vector/src/hnsw/deletion.rs`: Trait `Zeroize` not in scope for `as_mut_slice().zeroize()`.
   - `crates/contextra-vector/src/hnsw/core_rebuild.rs`: No field `kernel` on type `HnswColdCore`.
   - `crates/contextra-vector/src/hnsw/types.rs`: No function `try_new_with_rng_and_kernel` on struct `GenericHnswIndex`.
3. **contextra-graph**:
   - `crates/contextra-graph/src/csr/graph_write/mod.rs` & `graph_index/`: Fehlende Imports für `contextra_types` und `contextra_ports` (151 Fehler).

Gemäß Regel 5 (Fremdfehler) werden diese Fehler nicht in fremden Crates repariert, sondern hier protokolliert.

## Hintergrund-Worker & Komponenten-Nutzung (WP-82)
- `orchestration/`: `crates/contextra-engine/src/orchestration/mod.rs` definiert `EngineServiceContainer` und Typestate-Pipelines, wird jedoch an keiner Stelle innerhalb von `contextra-engine` oder externen Workspace-Crates referenziert/genutzt. Kandidat für Stilllegung oder Hinterlegung hinter Feature in späteren Wellen.
- `decay_controller.rs`: `AdaptiveDecayController` wird in `src/collection/maintenance.rs` (`evict_decayed_memories`) und in `expiry_workers.rs` genutzt, wenn das Feature `background-maintenance` aktiv ist.
- `extraction/`: `EntityExtractionConfig` und `extract_triples_for_mode` werden in `src/collection/crud/auto_extraction.rs` genutzt (für automatische Triple-Extraktion beim Einfügen, wenn konfiguriert).
