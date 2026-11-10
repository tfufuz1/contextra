# Baseline-Bericht contextra-text (TX6.0)

## Messungen Vorher
- **LOC:**
  - `crates/contextra-text/src`: 6.920 LOC in 31 Dateien
  - Gesamtes Crate inkl. Tests/Benches: 9.370 LOC
- **Modularitäts-Score (`decouple_inspector.py`):** 52/100 (Stark verdrahtet)
- **Zero-Panic (`zero_panic_analyzer.py --fail-on-prod`):** 0 Prod-Verstöße
- **Pattern Diagnostic (`pattern_diagnostic.py`):** 49 Funde (überwiegend in internen Tests & unwrap/expect in Tests, 5 Wording-Befunde u. a. 'garantiert')
- **Concurrency Locks (`concurrency_lock_auditor.py`):** 38 Sperren-Instanzen (parking_lot), keine ungesicherten Sperren über `.await`

## Nullprüfzahlen Vorher
- Produktionscode `crates/contextra-text/src` (ohne Test-Module): 62 Treffer auf `contextra_(types|ports|testkit)`
- Test-Module in `crates/contextra-text/src`: 20 Treffer
- Externe Tests in `crates/contextra-text/tests/`: 53 Treffer

## Feature-Befund
- `default`: `["std", "types-compat"]`
- Deklarierte Features: `std`, `serde`, `tracing`, `inspect`, `experimental`, `workspace`, `types-compat`, `ports-compat`, `bm25f`, `docid-128`.

## StorageEngine Aufrufinventar (11 Kern-Methoden)
1. `get(&self, key: &[u8]) -> BoxFuture<'_, Result<Option<Bytes>>>`
2. `get_at_seq(&self, key: &[u8], seq: u64) -> BoxFuture<'_, Result<Option<Bytes>>>`
3. `put(&self, tx: TxId, key: &[u8], value: &[u8]) -> BoxFuture<'_, Result<()>>`
4. `delete(&self, tx: TxId, key: &[u8]) -> BoxFuture<'_, Result<()>>`
5. `commit(&self, tx: TxId) -> BoxFuture<'_, Result<()>>`
6. `rollback(&self, tx: TxId) -> BoxFuture<'_, Result<()>>`
7. `rollback_to_tx(&self, tx: TxId) -> BoxFuture<'_, Result<()>>`
8. `last_tx_id(&self) -> BoxFuture<'_, Result<TxId>>`
9. `last_seq_no(&self) -> BoxFuture<'_, Result<u64>>`
10. `scan_prefix(&self, prefix: &[u8]) -> BoxFuture<'_, Result<Vec<(Vec<u8>, Vec<u8>)>>>`
11. `scan_prefix_at(&self, prefix: &[u8], seq: u64) -> BoxFuture<'_, Result<Vec<(Vec<u8>, Vec<u8>)>>>`
