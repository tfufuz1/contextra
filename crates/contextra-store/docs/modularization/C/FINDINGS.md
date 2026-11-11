# Befunde außerhalb des Eigentums (FINDINGS)

Die folgenden Beobachtungen wurden während der Arbeiten an STORE-MOD-C im Crate `contextra-store` dokumentiert:

1. **`crates/contextra-store/Cargo.toml` & `src/lib.rs` (Baseline-Defekt)**:
   - `Cargo.toml`: `contextra-types` und `contextra-ports` waren als nicht-optional deklariert, während Feature `workspace = ["dep:..."]` aktiv war.
   - `src/lib.rs`: Deklaration von `pub mod budget;` und `pub mod stats;` fehlte im Crate-Root.
   - Wurde via WP-C.0a repariert.

2. **`tests/golden_corpus.rs` (WAL Integrity Key Länge)**:
   - Datei `tests/golden/.wal_integrity_key` hat 59 Bytes anstelle der strikt von `Wal::open` geforderten 32 Bytes (enthält Textrepräsentation).
   - Fällt in den Zuständigkeitsbereich von Agent A (WAL).

3. **`tests/manifest_compaction_audit.rs` (ResourceTracker Typmismatch)**:
   - Test übergibt `contextra_types::ResourceTracker`, während `CompactionEngine::new` `contextra_store::budget::ResourceTracker` erwartet.
   - Fällt in den Zuständigkeitsbereich von Agent D / G.

4. **`tests/manifest_crash_no_resurrection.rs` (Fehlende SALT-Datei)**:
   - Schlägt fehl mit `Storage("SALT nicht lesbar: I/O error: No such file or directory")`.
   - Fällt in den Zuständigkeitsbereich von Agent G / LSM.

5. **`tests/memtable_shard_prefix_distribution.rs` (Annahme an Hash-Sharding)**:
   - Test erwartet `keyA <= keyB => shard_for(keyA) <= shard_for(keyB)`, was bei Hash-Sharding für reservierte Systempräfixe prinzipbedingt nicht gilt.
