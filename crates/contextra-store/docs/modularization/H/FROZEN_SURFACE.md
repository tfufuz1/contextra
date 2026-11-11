# Eingefrorene Außenfläche (Frozen Surface): STORE-MOD-H

Die Außenfläche umfasst alle Typen, Traits, Funktionen und Pfade im Eigentum von Agent H, die von externen Modulen oder Crates referenziert werden. Sie darf nicht semantisch gebrochen werden.

## 1. Kompatibilitäts-Fassaden

- `crate::engine::*` (`src/engine.rs`):
  - Fassade für `crate::compat::engine::*`
  - Konsumenten: `src/lsm/ops.rs` (`pub use crate::compat::engine::*;`), `tests/memory_only_crash_semantics.rs`
- `crate::scanner::*` (`src/scanner/mod.rs`):
  - Fassade für `crate::compat::scanner::*`
  - Re-exportiert `MemtableScanner`, `SstableScanner`, `WalScanner`, `StoreScanView`, `ScannerRuntime`, `lsm_scanners`, `lsm_scanners_with_runtime`, `lsm_purgers`, `lsm_purgers_with_runtime`
  - Konsumenten: `tests/determinism_injection_test.rs` (`use contextra_store::scanner::SstableScanner;`), `src/lsm/engine.rs`

## 2. Kanonische Brücken-Module (`crate::compat::*`)

- `crate::compat::engine`:
  - `InMemoryStorageEngine`: `contextra_ports::StorageEngine` Implementierung für Memory-Only
  - `TenantScopedStorageEngine`: Mandanten-isolierter Adapter
  - `LsmStorageEngine`: Wrapper um `Arc<LsmStorage>` als `StorageEngine`
- `crate::compat::scanner`:
  - Scanner-Implementierungen (`MemtableScanner`, `SstableScanner`, `WalScanner`)
  - Purger-Implementierung (`LsmPurger`)
  - `StoreScanView`
- `crate::compat::error`:
  - `From<StoreError> for ContextraError`
- `crate::compat::license`:
  - Lizenz-Validierungsfunktionen für Storage-Features

## 3. Beispiele (`examples/`)

- `examples/chaos_writer.rs`:
  - Externe Prozess-Schnittstelle!
  - Konsument: `tests/chaos_power_cut.rs` ruft `chaos_writer <storage_dir> <n_writes> <ground_truth_log_path>` auf.
  - CLI-Signatur und Ausgabeverhalten sind strikt eingefroren.
- `examples/standalone.rs`:
  - Autarkes Minimal-Beispiel.
- `examples/store_standalone.rs`:
  - Standalone-Nutzung des LSM-Stores.
