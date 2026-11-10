# Baseline contextra-vault (WP-T1)

## 1. Ausgangszustand der Quelle
- Quelle: `crates/contextra-db/src/volatile_vault.rs` (592 Zeilen)
- Eingebundene Module: `vault_sys.inc`, `vault_compat.inc`
- Abhängigkeiten der Quelle in `contextra-db`:
  - `contextra_types::{DocId, TenantId, TxId}`
  - `zeroize::{Zeroize, ZeroizeOnDrop}`
  - `tracing`
  - `thiserror`

## 2. Inventar der Tests der Quelle
In `crates/contextra-db/src/volatile_vault.rs` existieren 10 Unit-Tests:
1. `test_ingest_is_memory_only`: Verifiziert INV-VAULT-2 (reine RAM-Aufnahme ohne Disk-I/O).
2. `test_capacity_exceeded_returns_error`: Verifiziert INV-VAULT-3 (Fail-Closed bei Überschreitung des Limits).
3. `test_purge_consumes_vault`: Verifiziert INV-VAULT-1 (Verwerfen des Vaults und Zeroize der Chunks).
4. `test_implicit_drop_does_not_panic`: Verifiziert panic-freien RAII-Drop ohne explizites purge.
5. `test_ingest_after_purge_fails`: Verifiziert Konsumptionsschutz (`AlreadyConsumed`).
6. `test_preview_metadata_no_content`: Verifiziert Metadaten-Ansicht ohne Offenlegung des Nutzdateninhalts.
7. `test_is_memory_locked_disabled_config`: Verifiziert `is_memory_locked == false` bei `attempt_mlock: false`.
8. `test_is_memory_locked_status_propagation`: Verifiziert echte Status-Propagierung des `LockedRegions`-Ergebnisses.
9. `test_empty_chunk_ingest_preserves_lock_status`: Verifiziert Konsistenz des Lock-Status bei leeren Chunks.
10. `test_commit_generates_receipt_with_metadata`: Verifiziert Erzeugung des `CommitReceipt` inklusive Metadaten-Referenz.

## 3. Ziel-Architektur
- Crate: `contextra-vault` (Ebene 1, Kern-Closure `{contextra-sys}`)
- Lokaler Fehlertyp: `VaultError` (thiserror, `#[non_exhaustive]`)
- Kern ist strikt unabhängig von `contextra_types` und `contextra_ports`
- Brücke nach `contextra_types` liegt isoliert in `src/compat/` hinter dem Feature `workspace`
- Speicher-Sicherheit: `#![forbid(unsafe_code)]` im gesamten Crate; Nutzung von `contextra_sys::vault::LockedRegions`.
