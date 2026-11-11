# Architektur der Workspace-Brücke (compat/): STORE-MOD-H

Dieses Dokument beschreibt die Architektur und Invarianten der Workspace-Brücke `src/compat/` im Crate `contextra-store`.

## 1. Übersicht & Zielbild

`contextra-store` ist der durabilitätskritische LSM-Speicherkern (Ring 1) von Contextra.
Um die Modularisierung als Baukastensystem gemäß Regel R2 und G-L2 zu gewährleisten, ist der Kern des Crates frei von Workspace-Kopplungen (`contextra_types`, `contextra_ports`, `contextra_testkit`).
Alle Integrationen mit den gemeinsamen Port-Traits befinden sich strikt hinter dem Cargo-Feature `workspace` im Modul `src/compat/`.

## 2. Brückenkarte: Kern-Typen → Port-Implementierungen

| Kern-Komponente (`contextra-store`) | Port / Trait (`contextra-ports`) | Adapter-Implementierung in `compat/` |
|---|---|---|
| `LsmStorage` | `StorageEngine` | `compat::engine::lsm::LsmStorageEngine` |
| Memory-Only Store | `StorageEngine` | `compat::engine::memory_only::InMemoryStorageEngine` |
| `TenantScopedStorage` | `StorageEngine` | `compat::engine::tenant_storage::TenantScopedStorageEngine` |
| MemTable | `LayerScanner` | `compat::scanner::memtable::MemtableScanner` |
| SSTable | `LayerScanner` | `compat::scanner::sstable::SstableScanner` |
| WAL | `LayerScanner` | `compat::scanner::wal::WalScanner` |
| LSM Purge State Machine | `Purger` | `compat::scanner::purge::LsmPurger` |
| Fabrikfunktionen | `Vec<Arc<dyn LayerScanner>>` / `Purger` | `compat::scanner::factory::{lsm_scanners*, lsm_purgers*}` |
| `StoreError` | `ContextraError` (`contextra-types`) | `compat::error::impl From<StoreError> for ContextraError` |
| Feature-Prüfung | Storage-Feature Gate | `compat::license::validate_against_features` |

## 3. Fassaden zur Quellkompatibilität

- `src/engine.rs`: `pub use crate::compat::engine::*;`
- `src/scanner/mod.rs`: `pub use crate::compat::scanner::*;`

Konsumenten können weiterhin transparent auf `contextra_store::engine::*` und `contextra_store::scanner::*` zugreifen, ohne dass Quellcode-Brüche entstehen (INV-2).

## 4. Archivierte verwaiste Dateien

Die ehemals in `src/scanner/` und `src/engine/` verbliebenen Dateien wurden nach `archive/contextra-store-orphans/` verschoben:
- `archive/contextra-store-orphans/src/scanner/memtable.rs`
- `archive/contextra-store-orphans/src/scanner/purge.rs`
- `archive/contextra-store-orphans/src/scanner/runtime.rs`
- `archive/contextra-store-orphans/src/scanner/sstable.rs`
- `archive/contextra-store-orphans/src/scanner/store_scan_view.rs`
- `archive/contextra-store-orphans/src/scanner/tests.rs`
- `archive/contextra-store-orphans/src/scanner/wal.rs`
- `archive/contextra-store-orphans/src/engine/memory_only.rs`

Dadurch ist G-L2 im Kern des Crates vollständig erfüllt.
