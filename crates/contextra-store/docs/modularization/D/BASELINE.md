# BASELINE: Komponente D (Compaction & System-Pressure)

## Status vor Beginn
- Commit: `922ee269471eb76eb4be9370c0eb0e03b0e2a3ea` (HEAD fcd387021 / 922ee26)
- Toolchain: `rustc 1.89.0`, `cargo 1.89.0`

## Baseline-Befunde (Workspace & Manifest)
1. **Manifest-Defekt in `crates/contextra-store/Cargo.toml`**:
   - `workspace = ["dep:contextra-types", "dep:contextra-ports"]` verlangt `optional = true` für `contextra-types` und `contextra-ports`.
   - In `[dependencies]` sind diese aber non-optional deklariert.
   - Cargo bricht mit Fehler ab:
     `feature "workspace" includes "dep:contextra-types", but "contextra-types" is not an optional dependency`
   - Da `Cargo.toml` außerhalb des Eigentums liegt (Agent G), wird dies unter Übergaben im Abschlussbericht festgehalten und nicht direkt comitted.

2. **Fehlende Modul-Deklarationen in `crates/contextra-store/src/lib.rs`**:
   - `src/lib.rs` enthielt weder `pub mod budget;` noch `pub mod stats;`.
   - Dies führte zu `unresolved import crate::stats` und `crate::budget`.
   - Gemäß Baseline-Vorbedingung WP-D.0a wird dies in `crates/contextra-store/src/lib.rs` unmittelbar behoben.

## Datei-Übersicht vor Modularisierung
- `src/compaction/engine.rs`: 983 Zeilen (Monolith für Auswahl, Merge, Install, Throttle)
- `src/compaction/adaptive.rs`: 362 Zeilen (Metriken und heuristischer Planer)
- `src/compaction/config.rs`: 101 Zeilen
- `src/compaction/retention.rs`: 84 Zeilen
- `src/compaction/merge_operator.rs`: 16 Zeilen
- `src/compaction.rs`: 20 Zeilen (Re-Export Modul)
- `src/system_pressure.rs`: 411 Zeilen (Signal, Monitor, Schwellenwert-Berechnung)
- Tests:
  - `src/compaction/tests/basic.rs`: 1170 Zeilen
  - `src/compaction/tests/advanced.rs`: 1045 Zeilen
