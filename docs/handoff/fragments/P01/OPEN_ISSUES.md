# Open Issues (Fragment P01: contextra-store)

1. **Fremdfehler `contextra-crypto` (`unexpected-cfgs` für Feature `serde`):**
   - **Befund:** `cargo check -p contextra-crypto` meldet `error: unexpected 'cfg' condition value: 'serde'` in `crates/contextra-crypto/src/secret.rs:94:7`.
   - **Regel-Konformität:** Gemäß Regel 2 (Besitzprinzip) und Regel 5 (Fremdfehler) wird dies nicht in fremden Crates repariert, sondern hier als Fremdfehler dokumentiert. Für interne Store-Prüfungen werden Lints über `--cap-lints allow` gesteuert.

2. **[H] Unaufgeteilte Großdateien im Store:**
   - Großdateien (> 800 Zeilen: `tenant_codec.rs`, `memtable.rs`, `wal/flusher.rs`) wurden in WP-33 mechanisch strukturiert. Verbleibende große Dateien (`lsm/recovery.rs`, `sstable/reader.rs`, `wal/io.rs`) können in Folgepaketen bei Bedarf weiter untergliedert werden.

3. **Status Start-Check (Regel 12):**
   - `docs/handoff/fragments/P00/BUILD_LADDER.md` war im Repository nicht vorhanden.
   - `contextra-store` baute vor Arbeitsbeginn sauber mit Ausnahme von kürzlich gemergten internen Seams (`pub mod kv` Reste und LsmConfig-Parameter-Drift), die synchronisiert wurden.
