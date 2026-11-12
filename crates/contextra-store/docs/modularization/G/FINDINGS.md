# Findings für STORE-MOD-G (contextra-store)

1. **Manifest-Defekt in Cargo.toml (origin/main):**
   - Datei: `crates/contextra-store/Cargo.toml:17`
   - Befund: Feature `workspace = ["dep:contextra-types", "dep:contextra-ports"]` erfordert `optional = true` bei den entsprechenden Abhängigkeiten unter `[dependencies]`.
   - Behebung in WP-G.0.

2. **Fehlende Moduldeklarationen in lib.rs:**
   - Datei: `crates/contextra-store/src/lib.rs`
   - Befund: `pub mod budget;` und `pub mod stats;` fehlten im Crate-Root, wurden aber an 13 Stellen im Crate referenziert.
   - Behebung in WP-G.0a.

3. **Integritätsschlüssel-Format in Golden Corpus:**
   - Datei: `crates/contextra-store/tests/golden/.wal_integrity_key`
   - Befund: Die Datei enthielt eine unvollständige oder mit Escape-Sequenzen versehene Bytefolge von 59 Bytes anstelle der erwarteten 32 Roh-Bytes für HMAC-SHA256, wodurch `Wal::open` in `tests/golden_corpus.rs` fehlschlug.

4. **I/O-Abstraktions-Divergenz:**
   - Datei: `src/traits.rs` vs. `src/io_seam.rs`
   - Befund: Beide Dateien definieren parallele Abstraktionen für VFS/Dateizugriff (`StoreFs`/`StoreFile` vs. `Vfs`/`VfsFile`/`StoreIo`). Entscheidung B08 belässt VFS in `contextra-ports`, während `traits.rs` autarke lokale Typen für den isolierten Betrieb bereitstellt.

5. **Isolation Proof & Gate Layering (Governance-Befund):**
   - Befund: `tools/isolation_proof.py` bricht für `contextra-store` ab, da `contextra-testkit` (in `[dev-dependencies]`) in der isolierten Workspace-Manifest-Generierung nicht in `workspace.dependencies` kopiert wird (Tool-Einschränkung).
   - Befund: `tools/gate_layering.py` meldet Verstöße wegen transitiver Einbindung von `contextra-ports`/`contextra-types` über `contextra-crypto` und `contextra-mvcc`.
