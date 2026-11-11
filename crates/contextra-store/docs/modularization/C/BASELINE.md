# Baseline-Bericht: Auftrag STORE-MOD-C (Manifest & Memtable)

- **Datum:** 2026-11-11
- **Toolchain:** `rustc 1.89.0`, `cargo 1.89.0`
- **Ausgangs-Commit:** `origin/main` (HEAD `922ee2694`)

## 1. Ausgangszustand vor Beginn
1. **Workspace & Cargo.toml**:
   - `crates/contextra-store/Cargo.toml` hatte `contextra-types` und `contextra-ports` unter `[dependencies]` ohne `optional = true`, während `workspace = ["dep:contextra-types", "dep:contextra-ports"]` deklariert war.
   - `crates/contextra-store/src/lib.rs` fehlte die Deklaration von `pub mod budget;` und `pub mod stats;`.
   - Beide Punkte wurden via `WP-C.0a` behoben (identisch in allen Agentenaufträgen).
2. **Kompilierung**:
   - `cargo check -p contextra-store --locked --features workspace`: **GRÜN** (0 Fehler).
   - `cargo check -p contextra-store --locked --no-default-features`: 6 Compilerfehler durch unbenutzte private Methoden bei deaktiviertem `workspace`-Feature (bekannter Baseline-Zustand).
3. **Teststatus**:
   - `tests/manifest_corruption.rs`: **GRÜN** (5 passed, 0 failed).
   - `tests/campaign_fuzzregress_memtable_concurrent.rs`: **GRÜN** (2 passed, 0 failed).
   - `tests/memtable_shard_distribution.rs`: **GRÜN** (3 passed, 0 failed).
   - Vorbestehend rot / flaky in Baseline außerhalb des Eigentums:
     - `tests/golden_corpus.rs`: Scheitert an WAL-Integritätsschlüssel-Länge (59 Bytes statt 32 Bytes in golden-Fixture, Agent A/WAL zugeordnet).
     - `tests/manifest_compaction_audit.rs`: Scheitert an `ResourceTracker`-Typmismatch (`contextra_types` vs `contextra_store::budget`, Agent G/D zugeordnet).
     - `tests/manifest_crash_no_resurrection.rs`: Scheitert an fehlendem SALT (`I/O error: No such file or directory`, Agent G/LSM zugeordnet).
     - `tests/memtable_shard_prefix_distribution.rs`: Test setzt Monotonie-Regel `keyA <= keyB => shard_for(keyA) <= shard_for(keyB)` voraus, welche Hash-Sharding (`ahash`) prinzipbedingt nicht erfüllt.
