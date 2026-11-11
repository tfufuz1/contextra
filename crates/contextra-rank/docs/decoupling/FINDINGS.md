# Entkopplungsbefunde contextra-rank (FINDINGS.md)

Dieses Dokument erfasst alle während der Entkopplung von `contextra-rank` (Auftrag P7) festgestellten Auffälligkeiten, Altlasten und Beobachtungen, die gemäß Regel H2 nicht stillschweigend modifiziert wurden.

---

### 1. `ResonanceConfig` Struct-Definition unter verschiedenen Features
- **Befund:** In `crates/contextra-rank/src/fusion/ext.rs` existiert ein Fallback-Stub für `ResonanceConfig` unter `#[cfg(not(feature = "exp-resonance"))]`, welcher nur das Feld `pub enabled: bool` besitzt. Unter `#[cfg(feature = "exp-resonance")]` (definiert in `crates/contextra-rank/src/fusion/resonance.rs`) besitzt `ResonanceConfig` die Felder `pub beta: f32` und `pub gamma: f32`.
- **Auswirkung:** Tests wie `tests/nan_inf_safety_tests.rs` und `tests/fusion_deep_property_tests.rs` instanziieren `ResonanceConfig { beta: ..., gamma: ... }` direkt und setzen voraus, dass das Feature `exp-resonance` (bzw. das Default-Feature `experimental-rank`) aktiv ist. Beim Bauen der Integrations-Test-Targets mit `--no-default-features` ohne `exp-resonance` schlagen diese beiden spezifischen Integrationstests fehl. Bei Default-Features sowie Einzelfeature-Test-Läufen des Crates (`cargo test -p contextra-rank` oder `--lib`) sind alle Tests vollständig grün.
- **Entscheidung / Empfehlung:** Da `experimental-rank` (und damit `exp-resonance`) Teil der Default-Features ist, bleibt die Abwärtskompatibilität vollständig gewahrt. Zukünftig sollten die beiden Integrationstests mit `#[cfg(feature = "exp-resonance")]` annotiert werden.

---

### 2. Clippy-Warnungen in Tests (`rank_invariants.rs`)
- **Befund:** In `tests/rank_invariants.rs` schlägt `cargo clippy -p contextra-rank --all-targets -- -D warnings` wegen `clippy::unwrap_used` und `clippy::indexing_slicing` in Testfunktionen an.
- **Auswirkung:** In der Baseline V0 war dies bereits vorhanden. Die Crate-Bibliothek selbst (`cargo clippy -p contextra-rank --lib -- -D warnings`) ist absolut sauber und warnungsfrei.
- **Entscheidung:** Gemäß Regel H2 und H4 werden bestehende Tests nicht umformatiert.

---

### 3. Externe Workspace-Befunde (Workspace-Probe V7)
- **Befund:** Bei Ausführung von `cargo check --workspace --all-targets` existieren pre-existing Syntax-/Kompilierfehler in fremden Crates (`crates/contextra-vector/src/diskann/build.rs`, `crates/contextra-store/src/sstable/builder.rs`, `crates/contextra-checkpoint/src/store.rs`), die auf parallele Arbeiten anderer Agenten auf `origin/main` zurückgehen.
- **Isolations-Nachweis:** Alle direkten Konsumenten von `contextra-rank` (`contextra-engine`, `contextra-mcp`, `contextra-router`, `contextra-infer-candle`) kompilieren mit `--all-targets` fehlerfrei gegen den neuen `contextra-rank`-Stand.
