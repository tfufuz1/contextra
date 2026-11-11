# Archivierte verwaiste Dateien (archive/contextra-store-orphans)

Dieses Archiv enthält nachweislich unerreichbare, verwaiste Duplikate aus `crates/contextra-store/src/scanner/` und `crates/contextra-store/src/engine/`.

## Hintergrund & Begründung

In Phase S-70 / STORE-6 wurden die Implementierungen von Scanner, Purger und Engine nach `crates/contextra-store/src/compat/` migriert (`src/compat/scanner/` und `src/compat/engine/`).
Die Dateien in `src/scanner/` und `src/engine/` wurden dabei nicht mehr über `mod`-Deklarationen referenziert, sondern `src/scanner/mod.rs` und `src/engine.rs` fungieren als quellkompatible Fassaden (`pub use crate::compat::scanner::*;` bzw. `pub use crate::compat::engine::*;`).
Die verbliebenen Dateien unter `src/scanner/*.rs` und `src/engine/*.rs` waren tote Duplikate, die zudem die G-L2 Entkopplungs-Regel verletzten (unisolierte `contextra_types` / `contextra_ports` Importe im Kern).

Gemäß Root-`AGENTS.md` Regel 3 ("Code wird nie vernichtet, sondern bei Bedarf nach archive/ verschoben") wurden folgende Dateien archiviert:

- `src/scanner/memtable.rs` (Kanonische Version in `crates/contextra-store/src/compat/scanner/memtable.rs`)
- `src/scanner/purge.rs` (Kanonische Version in `crates/contextra-store/src/compat/scanner/purge.rs`)
- `src/scanner/runtime.rs` (Byte-identisch zu `crates/contextra-store/src/compat/scanner/runtime.rs`)
- `src/scanner/sstable.rs` (Byte-identisch zu `crates/contextra-store/src/compat/scanner/sstable.rs`)
- `src/scanner/store_scan_view.rs` (Kanonische Version in `crates/contextra-store/src/compat/scanner/store_scan_view.rs`)
- `src/scanner/tests.rs` (Byte-identisch zu `crates/contextra-store/src/compat/scanner/tests.rs`)
- `src/scanner/wal.rs` (Byte-identisch zu `crates/contextra-store/src/compat/scanner/wal.rs`)
- `src/engine/memory_only.rs` (Kanonische Version in `crates/contextra-store/src/compat/engine/memory_only.rs`)
