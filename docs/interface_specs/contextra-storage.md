# Schnittstellenspezifikation: `contextra-storage`

> Erzeugt durch `tools/extract_interfaces.py` aus dem Quellcode. Keine Handpflege. Jede Angabe trägt Datei:Zeile. Maßgeblich bleibt der Quellcode.

## 1. Kopf

| Feld | Wert |
|---|---|
| Name | `contextra-storage` |
| Version | workspace |
| Pfad | `crates/contextra-storage/crates/contextra-storage` |
| Rolle | Speicherkern (Kern, verschachtelt unter crates/contextra-storage) |
| Bibliothekspfad | src/lib.rs (Standard) |

**Crate-Dokumentation (//!):**

> contextra-storage: Speicherkern. Fassade und Modulwurzel (SP-FACADE).
> Komponentenbesitz, Abhängigkeitsgraph und Implementierungsreihenfolge stehen in
> [`IMPLEMENTATION-FOUNDATION.md`](IMPLEMENTATION-FOUNDATION.md). Das Crate ist
> ein synchroner, eingebetteter Kern; Subsysteme werden erst nach ihren Recovery-
> und Durability-Verträgen über die öffentliche Fassade freigegeben.

## 2. Abhängigkeiten

| Name | Art | Quelle | optional | verwendet von (normal/dev) |
|---|---|---|---|---|
| `bytes` | normal | registry | nein |  |
| `contextra-crypto-ref` | dev | path | nein | contextra-storage |
| `contextra-storage-ports` | normal | path | nein | contextra-crypto-ref, contextra-keys-cohort, contextra-keys-file, contextra-storage, contextra-storage-bridge, contextra-storage-fs |
| `crossbeam-skiplist` | normal | workspace | nein |  |
| `thiserror` | normal | registry | nein |  |
| `zeroize` | normal | registry | nein |  |

**Verwendet von (Crate-Ebene):** `contextra-storage-bridge`

## 3. Features

| Feature | Aktiviert |
|---|---|
| `testing` | – |

## 4. Metriken (Quellcode)

| Kennzahl | Wert |
|---|---|
| Quelldateien | 111 |
| Quellzeilen | 8059 |
| Öffentliche Items (Modul-Ebene) | 352 |
| davon `fn` | 51 |
| davon `struct` | 106 |
| davon `enum` | 37 |
| davon `trait` | 6 |
| davon `type` | 1 |
| davon `const` | 41 |
| davon `mod` | 110 |
| Öffentliche Makros (`#[macro_export]`) | 0 |
| Öffentliche Re-Exports (`pub use`) | 5 |
| `unsafe`-Vorkommen (Produktionscode) | 0 |
| `.unwrap()` (Produktionscode) | 0 |
| `.expect(` (Produktionscode) | 0 |
| `panic!` (Produktionscode) | 0 |
| `todo!/unimplemented!/unreachable!` | 0 |
| Indexierungen `x[…]` (grob, Produktionscode) | 8 |
| `#[deprecated]` | 0 |
| `#[cfg(feature…)]`-Gates | 6 |
| Parser-Abdeckung (geparste `pub`-Deklarationen / Grep-Treffer) | 650 / 650 = 100 % |

## 5. Modulbaum und Dateien

| Modul | Datei | Zeilen |
|---|---|---|
| `catalog::edit` | `crates/contextra-storage/crates/contextra-storage/src/catalog/edit.rs` | 57 |
| `catalog::file` | `crates/contextra-storage/crates/contextra-storage/src/catalog/file.rs` | 269 |
| `catalog` | `crates/contextra-storage/crates/contextra-storage/src/catalog/mod.rs` | 12 |
| `catalog::ops` | `crates/contextra-storage/crates/contextra-storage/src/catalog/ops.rs` | 232 |
| `catalog::record` | `crates/contextra-storage/crates/contextra-storage/src/catalog/record.rs` | 134 |
| `catalog::reserve` | `crates/contextra-storage/crates/contextra-storage/src/catalog/reserve.rs` | 49 |
| `catalog::rollover` | `crates/contextra-storage/crates/contextra-storage/src/catalog/rollover.rs` | 78 |
| `catalog::state` | `crates/contextra-storage/crates/contextra-storage/src/catalog/state.rs` | 214 |
| `checkpoint::create` | `crates/contextra-storage/crates/contextra-storage/src/checkpoint/create.rs` | 4 |
| `checkpoint` | `crates/contextra-storage/crates/contextra-storage/src/checkpoint/mod.rs` | 3 |
| `checkpoint::restore` | `crates/contextra-storage/crates/contextra-storage/src/checkpoint/restore.rs` | 4 |
| `codec::checksums` | `crates/contextra-storage/crates/contextra-storage/src/codec/checksums.rs` | 23 |
| `codec::comparator` | `crates/contextra-storage/crates/contextra-storage/src/codec/comparator.rs` | 29 |
| `codec::format` | `crates/contextra-storage/crates/contextra-storage/src/codec/format.rs` | 75 |
| `codec::header` | `crates/contextra-storage/crates/contextra-storage/src/codec/header.rs` | 74 |
| `codec::keys_codec` | `crates/contextra-storage/crates/contextra-storage/src/codec/keys_codec.rs` | 100 |
| `codec` | `crates/contextra-storage/crates/contextra-storage/src/codec/mod.rs` | 9 |
| `codec::seal` | `crates/contextra-storage/crates/contextra-storage/src/codec/seal.rs` | 169 |
| `codec::tags` | `crates/contextra-storage/crates/contextra-storage/src/codec/tags.rs` | 52 |
| `codec::varint` | `crates/contextra-storage/crates/contextra-storage/src/codec/varint.rs` | 56 |
| `config` | `crates/contextra-storage/crates/contextra-storage/src/config.rs` | 124 |
| `engine::branches` | `crates/contextra-storage/crates/contextra-storage/src/engine/branches.rs` | 205 |
| `engine::cache` | `crates/contextra-storage/crates/contextra-storage/src/engine/cache.rs` | 4 |
| `engine::committer` | `crates/contextra-storage/crates/contextra-storage/src/engine/committer.rs` | 157 |
| `engine::cursor` | `crates/contextra-storage/crates/contextra-storage/src/engine/cursor.rs` | 4 |
| `engine::extension` | `crates/contextra-storage/crates/contextra-storage/src/engine/extension.rs` | 4 |
| `engine::feed` | `crates/contextra-storage/crates/contextra-storage/src/engine/feed.rs` | 4 |
| `engine::graveyard` | `crates/contextra-storage/crates/contextra-storage/src/engine/graveyard.rs` | 38 |
| `engine::guard_window` | `crates/contextra-storage/crates/contextra-storage/src/engine/guard_window.rs` | 95 |
| `engine::idem` | `crates/contextra-storage/crates/contextra-storage/src/engine/idem.rs` | 4 |
| `engine::memtable` | `crates/contextra-storage/crates/contextra-storage/src/engine/memtable.rs` | 118 |
| `engine` | `crates/contextra-storage/crates/contextra-storage/src/engine/mod.rs` | 18 |
| `engine::packgroups` | `crates/contextra-storage/crates/contextra-storage/src/engine/packgroups.rs` | 4 |
| `engine::page` | `crates/contextra-storage/crates/contextra-storage/src/engine/page.rs` | 4 |
| `engine::registry` | `crates/contextra-storage/crates/contextra-storage/src/engine/registry.rs` | 112 |
| `engine::segments` | `crates/contextra-storage/crates/contextra-storage/src/engine/segments.rs` | 4 |
| `engine::version` | `crates/contextra-storage/crates/contextra-storage/src/engine/version.rs` | 187 |
| `engine::views` | `crates/contextra-storage/crates/contextra-storage/src/engine/views.rs` | 97 |
| `engine::volatile` | `crates/contextra-storage/crates/contextra-storage/src/engine/volatile.rs` | 185 |
| `env::anchor` | `crates/contextra-storage/crates/contextra-storage/src/env/anchor.rs` | 37 |
| `env::budget` | `crates/contextra-storage/crates/contextra-storage/src/env/budget.rs` | 122 |
| `env::faultfs` | `crates/contextra-storage/crates/contextra-storage/src/env/faultfs.rs` | 239 |
| `env::governor` | `crates/contextra-storage/crates/contextra-storage/src/env/governor.rs` | 95 |
| `env::memfs` | `crates/contextra-storage/crates/contextra-storage/src/env/memfs.rs` | 244 |
| `env` | `crates/contextra-storage/crates/contextra-storage/src/env/mod.rs` | 7 |
| `env::sim` | `crates/contextra-storage/crates/contextra-storage/src/env/sim.rs` | 4 |
| `erase::cluster_table` | `crates/contextra-storage/crates/contextra-storage/src/erase/cluster_table.rs` | 94 |
| `erase::gate` | `crates/contextra-storage/crates/contextra-storage/src/erase/gate.rs` | 90 |
| `erase::ledger` | `crates/contextra-storage/crates/contextra-storage/src/erase/ledger.rs` | 165 |
| `erase::links` | `crates/contextra-storage/crates/contextra-storage/src/erase/links.rs` | 209 |
| `erase` | `crates/contextra-storage/crates/contextra-storage/src/erase/mod.rs` | 9 |
| `erase::protocol` | `crates/contextra-storage/crates/contextra-storage/src/erase/protocol.rs` | 9 |
| `erase::receipt` | `crates/contextra-storage/crates/contextra-storage/src/erase/receipt.rs` | 9 |
| `erase::restrictions` | `crates/contextra-storage/crates/contextra-storage/src/erase/restrictions.rs` | 174 |
| `erase::tickets` | `crates/contextra-storage/crates/contextra-storage/src/erase/tickets.rs` | 210 |
| `keys::cache` | `crates/contextra-storage/crates/contextra-storage/src/keys/cache.rs` | 217 |
| `keys` | `crates/contextra-storage/crates/contextra-storage/src/keys/mod.rs` | 3 |
| `keys::wrapper` | `crates/contextra-storage/crates/contextra-storage/src/keys/wrapper.rs` | 96 |
| `crate` | `crates/contextra-storage/crates/contextra-storage/src/lib.rs` | 72 |
| `maint::blob_gc` | `crates/contextra-storage/crates/contextra-storage/src/maint/blob_gc.rs` | 4 |
| `maint::compaction` | `crates/contextra-storage/crates/contextra-storage/src/maint/compaction.rs` | 4 |
| `maint::flush` | `crates/contextra-storage/crates/contextra-storage/src/maint/flush.rs` | 4 |
| `maint` | `crates/contextra-storage/crates/contextra-storage/src/maint/mod.rs` | 8 |
| `maint::purge` | `crates/contextra-storage/crates/contextra-storage/src/maint/purge.rs` | 4 |
| `maint::reserve` | `crates/contextra-storage/crates/contextra-storage/src/maint/reserve.rs` | 98 |
| `maint::scheduler` | `crates/contextra-storage/crates/contextra-storage/src/maint/scheduler.rs` | 4 |
| `maint::scrub` | `crates/contextra-storage/crates/contextra-storage/src/maint/scrub.rs` | 4 |
| `metrics` | `crates/contextra-storage/crates/contextra-storage/src/metrics.rs` | 108 |
| `recovery::integrity` | `crates/contextra-storage/crates/contextra-storage/src/recovery/integrity.rs` | 40 |
| `recovery` | `crates/contextra-storage/crates/contextra-storage/src/recovery/mod.rs` | 4 |
| `recovery::replay` | `crates/contextra-storage/crates/contextra-storage/src/recovery/replay.rs` | 133 |
| `recovery::scan` | `crates/contextra-storage/crates/contextra-storage/src/recovery/scan.rs` | 110 |
| `spaces` | `crates/contextra-storage/crates/contextra-storage/src/spaces.rs` | 130 |
| `store::blob::gc_index` | `crates/contextra-storage/crates/contextra-storage/src/store/blob/gc_index.rs` | 65 |
| `store::blob` | `crates/contextra-storage/crates/contextra-storage/src/store/blob/mod.rs` | 5 |
| `store::blob::reader` | `crates/contextra-storage/crates/contextra-storage/src/store/blob/reader.rs` | 4 |
| `store::blob::record` | `crates/contextra-storage/crates/contextra-storage/src/store/blob/record.rs` | 63 |
| `store::blob::writer` | `crates/contextra-storage/crates/contextra-storage/src/store/blob/writer.rs` | 4 |
| `store` | `crates/contextra-storage/crates/contextra-storage/src/store/mod.rs` | 4 |
| `store::redo::cut` | `crates/contextra-storage/crates/contextra-storage/src/store/redo/cut.rs` | 4 |
| `store::redo::frame` | `crates/contextra-storage/crates/contextra-storage/src/store/redo/frame.rs` | 204 |
| `store::redo` | `crates/contextra-storage/crates/contextra-storage/src/store/redo/mod.rs` | 8 |
| `store::redo::reader` | `crates/contextra-storage/crates/contextra-storage/src/store/redo/reader.rs` | 162 |
| `store::redo::reserve` | `crates/contextra-storage/crates/contextra-storage/src/store/redo/reserve.rs` | 47 |
| `store::redo::segment` | `crates/contextra-storage/crates/contextra-storage/src/store/redo/segment.rs` | 22 |
| `store::redo::torn_tail` | `crates/contextra-storage/crates/contextra-storage/src/store/redo/torn_tail.rs` | 10 |
| `store::redo::writer` | `crates/contextra-storage/crates/contextra-storage/src/store/redo/writer.rs` | 128 |
| `store::run::block` | `crates/contextra-storage/crates/contextra-storage/src/store/run/block.rs` | 219 |
| `store::run::builder` | `crates/contextra-storage/crates/contextra-storage/src/store/run/builder.rs` | 121 |
| `store::run::copy` | `crates/contextra-storage/crates/contextra-storage/src/store/run/copy.rs` | 36 |
| `store::run::directory` | `crates/contextra-storage/crates/contextra-storage/src/store/run/directory.rs` | 134 |
| `store::run::extension` | `crates/contextra-storage/crates/contextra-storage/src/store/run/extension.rs` | 95 |
| `store::run::filter` | `crates/contextra-storage/crates/contextra-storage/src/store/run/filter.rs` | 164 |
| `store::run::format` | `crates/contextra-storage/crates/contextra-storage/src/store/run/format.rs` | 74 |
| `store::run` | `crates/contextra-storage/crates/contextra-storage/src/store/run/mod.rs` | 13 |
| `store::run::rangedel` | `crates/contextra-storage/crates/contextra-storage/src/store/run/rangedel.rs` | 132 |
| `store::run::reader` | `crates/contextra-storage/crates/contextra-storage/src/store/run/reader.rs` | 107 |
| `store::run::skip_scan` | `crates/contextra-storage/crates/contextra-storage/src/store/run/skip_scan.rs` | 30 |
| `store::run::subindex` | `crates/contextra-storage/crates/contextra-storage/src/store/run/subindex.rs` | 151 |
| `store::run::table_cache` | `crates/contextra-storage/crates/contextra-storage/src/store/run/table_cache.rs` | 109 |
| `sync::epoch` | `crates/contextra-storage/crates/contextra-storage/src/sync/epoch.rs` | 51 |
| `sync` | `crates/contextra-storage/crates/contextra-storage/src/sync/mod.rs` | 3 |
| `sync::ranked` | `crates/contextra-storage/crates/contextra-storage/src/sync/ranked.rs` | 83 |
| `testing::canary` | `crates/contextra-storage/crates/contextra-storage/src/testing/canary.rs` | 4 |
| `testing::crash_sweep` | `crates/contextra-storage/crates/contextra-storage/src/testing/crash_sweep.rs` | 4 |
| `testing::ghost` | `crates/contextra-storage/crates/contextra-storage/src/testing/ghost.rs` | 4 |
| `testing::inspector` | `crates/contextra-storage/crates/contextra-storage/src/testing/inspector.rs` | 4 |
| `testing` | `crates/contextra-storage/crates/contextra-storage/src/testing/mod.rs` | 8 |
| `testing::model` | `crates/contextra-storage/crates/contextra-storage/src/testing/model.rs` | 4 |
| `testing::oracle` | `crates/contextra-storage/crates/contextra-storage/src/testing/oracle.rs` | 4 |
| `testing::workload` | `crates/contextra-storage/crates/contextra-storage/src/testing/workload.rs` | 4 |

## 6. Re-Exports (`pub use`)

- `pub use ops::` — `crates/contextra-storage/crates/contextra-storage/src/catalog/mod.rs:10` (Modul `catalog`)
- `pub use record::` — `crates/contextra-storage/crates/contextra-storage/src/catalog/mod.rs:11` (Modul `catalog`)
- `pub use state::` — `crates/contextra-storage/crates/contextra-storage/src/catalog/mod.rs:12` (Modul `catalog`)
- `pub use contextra_storage_ports as ports` — `crates/contextra-storage/crates/contextra-storage/src/lib.rs:8` (Modul `crate`)
- `pub use contextra_storage_ports::DataClass` — `crates/contextra-storage/crates/contextra-storage/src/spaces.rs:6` (Modul `spaces`)

## 8. Öffentliche Schnittstelle je Item

### Modul `catalog`

#### `mod edit`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/catalog/mod.rs:2`
```rust
pub mod edit
```

#### `mod file`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/catalog/mod.rs:3`
```rust
pub mod file
```

#### `mod ops`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/catalog/mod.rs:4`
```rust
pub mod ops
```

#### `mod record`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/catalog/mod.rs:5`
```rust
pub mod record
```

#### `mod reserve`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/catalog/mod.rs:6`
```rust
pub mod reserve
```

#### `mod rollover`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/catalog/mod.rs:7`
```rust
pub mod rollover
```

#### `mod state`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/catalog/mod.rs:8`
```rust
pub mod state
```

### Modul `catalog::edit`

#### `struct PurgeBatchEdit`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/catalog/edit.rs:7`
```rust
pub struct PurgeBatchEdit
```
- Impl-Block `crates/contextra-storage/crates/contextra-storage/src/catalog/edit.rs:9`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn replace(ticket: [u8` | `crates/contextra-storage/crates/contextra-storage/src/catalog/edit.rs:10` |  |
| `pub fn remove(ticket: [u8` | `crates/contextra-storage/crates/contextra-storage/src/catalog/edit.rs:14` |  |
| `pub fn ticket(&self) -> &[u8` | `crates/contextra-storage/crates/contextra-storage/src/catalog/edit.rs:17` |  |
| `pub fn into_op(self) -> CatalogOp` | `crates/contextra-storage/crates/contextra-storage/src/catalog/edit.rs:18` |  |

#### `struct PurgeCompleteEdit`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/catalog/edit.rs:33`
```rust
pub struct PurgeCompleteEdit
```
- Impl-Block `crates/contextra-storage/crates/contextra-storage/src/catalog/edit.rs:35`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn after_owner_attestations(ticket: [u8` | `crates/contextra-storage/crates/contextra-storage/src/catalog/edit.rs:38` | The state fold additionally rejects completion while any source file remains in the ticket's persistent graveyard. |
| `pub fn into_op(self) -> CatalogOp` | `crates/contextra-storage/crates/contextra-storage/src/catalog/edit.rs:39` |  |

#### `struct UnlinkAckEdit`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/catalog/edit.rs:24`
```rust
pub struct UnlinkAckEdit
```
- Impl-Block `crates/contextra-storage/crates/contextra-storage/src/catalog/edit.rs:26`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn after_durable_unlink(ticket: [u8` | `crates/contextra-storage/crates/contextra-storage/src/catalog/edit.rs:28` | Construct only after `unlink` and the containing `sync_dir` succeeded. |
| `pub fn into_op(self) -> CatalogOp` | `crates/contextra-storage/crates/contextra-storage/src/catalog/edit.rs:29` |  |

#### `enum CatalogEdit`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/catalog/edit.rs:43`
```rust
pub enum CatalogEdit
```
- Impl-Block `crates/contextra-storage/crates/contextra-storage/src/catalog/edit.rs:49`

Varianten:

- `PurgeBatch(PurgeBatchEdit)` — `crates/contextra-storage/crates/contextra-storage/src/catalog/edit.rs:44`
- `UnlinkAck(UnlinkAckEdit)` — `crates/contextra-storage/crates/contextra-storage/src/catalog/edit.rs:45`
- `PurgeComplete(PurgeCompleteEdit)` — `crates/contextra-storage/crates/contextra-storage/src/catalog/edit.rs:46`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn into_op(self) -> CatalogOp` | `crates/contextra-storage/crates/contextra-storage/src/catalog/edit.rs:50` |  |

### Modul `catalog::file`

#### `struct CatalogFile`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/catalog/file.rs:16`
```rust
pub struct CatalogFile
```
- Impl-Block `crates/contextra-storage/crates/contextra-storage/src/catalog/file.rs:29`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn open_current( fs: &dyn Fs, root: &Path, mode: OpenMode, integrity: CatalogIntegrity, key: Option<&Key32>, mac: Option<&dyn Mac>, ) -> Result<Self, Error>` | `crates/contextra-storage/crates/contextra-storage/src/catalog/file.rs:30` |  |
| `pub fn open( fs: &dyn Fs, path: &Path, mode: OpenMode, integrity: CatalogIntegrity, key: Option<&Key32>, mac: Option<&dyn Mac>, ) -> Result<Self, Error>` | `crates/contextra-storage/crates/contextra-storage/src/catalog/file.rs:50` |  |
| `pub fn append_durable( &mut self, ops: &[super::ops::CatalogOp], integrity: CatalogIntegrity, key: Option<&Key32>, mac: Option<&dyn Mac>, ) -> Result<u64, Error>` | `crates/contextra-storage/crates/contextra-storage/src/catalog/file.rs:126` | Convenience path for one caller; the group writer below is the shared durability primitive. |
| `pub fn append_group_durable( &mut self, edits: &[Vec<super::ops::CatalogOp>], integrity: CatalogIntegrity, key: Option<&Key32>, mac: Option<&dyn Mac>, ) -> Result<Vec<u64>, Error>` | `crates/contextra-storage/crates/contextra-storage/src/catalog/file.rs:139` | Installs several waiting logical edits in one record, append, and sync. Operation patterns that must remain coupled are built before submission. |
| `pub fn append_edits_durable( &mut self, edits: &[super::edit::CatalogEdit], integrity: CatalogIntegrity, key: Option<&Key32>, mac: Option<&dyn Mac>, ) -> Result<Vec<u64>, Error>` | `crates/contextra-storage/crates/contextra-storage/src/catalog/file.rs:210` | Persists typed production edits. PurgeBatch, UnlinkAck and PurgeComplete are intentionally distinct operations and retain their per-file lifecycle in the folded |
| `pub fn reserve_space_ids_durable( &mut self, amount: u64, integrity: CatalogIntegrity, key: Option<&Key32>, mac: Option<&dyn Mac>, ) -> Result<super::reserve::Reservation, Error>` | `crates/contextra-storage/crates/contextra-storage/src/catalog/file.rs:224` | Durably reserves a user SpaceId interval before returning it to an allocator. Callers must serialize allocation through this mutable CatalogFile handle; after r |
| `pub fn path(&self) -> &Path` | `crates/contextra-storage/crates/contextra-storage/src/catalog/file.rs:238` |  |
| `pub fn state(&self) -> &CatalogState` | `crates/contextra-storage/crates/contextra-storage/src/catalog/file.rs:239` |  |
| `pub fn next_sequence(&self) -> u64` | `crates/contextra-storage/crates/contextra-storage/src/catalog/file.rs:240` |  |
| `pub fn append_offset(&self) -> u64` | `crates/contextra-storage/crates/contextra-storage/src/catalog/file.rs:241` |  |
| `pub fn is_writable(&self) -> bool` | `crates/contextra-storage/crates/contextra-storage/src/catalog/file.rs:242` |  |
| `pub fn has_torn_tail(&self) -> bool` | `crates/contextra-storage/crates/contextra-storage/src/catalog/file.rs:243` |  |

#### `enum OpenMode`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/catalog/file.rs:14`
```rust
pub enum OpenMode
```

### Modul `catalog::ops`

#### `struct FileEntry`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/catalog/ops.rs:23`
```rust
pub struct FileEntry
```

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `file_id` | `u64` | `crates/contextra-storage/crates/contextra-storage/src/catalog/ops.rs:24` |
| `space` | `SpaceId` | `crates/contextra-storage/crates/contextra-storage/src/catalog/ops.rs:25` |
| `cluster` | `Option<ClusterId>` | `crates/contextra-storage/crates/contextra-storage/src/catalog/ops.rs:26` |
| `kind` | `FileKind` | `crates/contextra-storage/crates/contextra-storage/src/catalog/ops.rs:27` |
| `level` | `u8` | `crates/contextra-storage/crates/contextra-storage/src/catalog/ops.rs:28` |
| `bytes` | `u64` | `crates/contextra-storage/crates/contextra-storage/src/catalog/ops.rs:29` |
| `min_stamp` | `Stamp` | `crates/contextra-storage/crates/contextra-storage/src/catalog/ops.rs:30` |
| `max_stamp` | `Stamp` | `crates/contextra-storage/crates/contextra-storage/src/catalog/ops.rs:31` |

#### `enum CatalogOp`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/catalog/ops.rs:35`
```rust
pub enum CatalogOp
```
- Impl-Block `crates/contextra-storage/crates/contextra-storage/src/catalog/ops.rs:53`

Varianten:

- `SpaceAdd{ space: SpaceId, protection: Protection }` — `crates/contextra-storage/crates/contextra-storage/src/catalog/ops.rs:36`
- `ClusterRegister{ cluster: ClusterId, protection: Protection }` — `crates/contextra-storage/crates/contextra-storage/src/catalog/ops.rs:37`
- `ClusterState{ cluster: ClusterId, state: ClusterLifecycle }` — `crates/contextra-storage/crates/contextra-storage/src/catalog/ops.rs:38`
- `FilesAdd{ file_id: u64, space: SpaceId, cluster: Option<ClusterId>, kind: FileKind, level: u8, bytes: u64, min_stamp: Stamp, max_stamp: Stamp }` — `crates/contextra-storage/crates/contextra-storage/src/catalog/ops.rs:39`
- `FilesRemove{ file_id: u64 }` — `crates/contextra-storage/crates/contextra-storage/src/catalog/ops.rs:40`
- `ReplayFloor{ segment: u64, offset: u64 }` — `crates/contextra-storage/crates/contextra-storage/src/catalog/ops.rs:41`
- `RedoMark{ segment: u64, offset: u64, stamp: Stamp }` — `crates/contextra-storage/crates/contextra-storage/src/catalog/ops.rs:42`
- `ReserveStamps{ through: Stamp }` — `crates/contextra-storage/crates/contextra-storage/src/catalog/ops.rs:43`
- `ReserveIds{ through: u64 }` — `crates/contextra-storage/crates/contextra-storage/src/catalog/ops.rs:44`
- `ReserveSpaceIds{ through: u64 }` — `crates/contextra-storage/crates/contextra-storage/src/catalog/ops.rs:45`
- `PurgeBatch{ ticket: [u8; 16], remove_file_id: u64, replacement: Option<FileEntry> }` — `crates/contextra-storage/crates/contextra-storage/src/catalog/ops.rs:48`
- `UnlinkAck{ ticket: [u8; 16], file_id: u64 }` — `crates/contextra-storage/crates/contextra-storage/src/catalog/ops.rs:49`
- `PurgeComplete{ ticket: [u8; 16] }` — `crates/contextra-storage/crates/contextra-storage/src/catalog/ops.rs:50`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn encode(&self, out: &mut Vec<u8>) -> Result<(), Error>` | `crates/contextra-storage/crates/contextra-storage/src/catalog/ops.rs:54` |  |
| `pub fn decode(input: &[u8]) -> Result<(Self, usize), Error>` | `crates/contextra-storage/crates/contextra-storage/src/catalog/ops.rs:118` |  |

#### `enum FileKind`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/catalog/ops.rs:20`
```rust
pub enum FileKind
```

### Modul `catalog::record`

#### `struct CatalogRecord`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/catalog/record.rs:15`
```rust
pub struct CatalogRecord
```

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `sequence` | `u64` | `crates/contextra-storage/crates/contextra-storage/src/catalog/record.rs:16` |
| `previous_crc` | `u32` | `crates/contextra-storage/crates/contextra-storage/src/catalog/record.rs:17` |
| `ops` | `Vec<CatalogOp>` | `crates/contextra-storage/crates/contextra-storage/src/catalog/record.rs:18` |
| `crc` | `u32` | `crates/contextra-storage/crates/contextra-storage/src/catalog/record.rs:19` |
| `chain_mac` | `Option<[u8; 32]>` | `crates/contextra-storage/crates/contextra-storage/src/catalog/record.rs:20` |

#### `enum CatalogIntegrity`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/catalog/record.rs:12`
```rust
pub enum CatalogIntegrity
```

#### `fn decode_record`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/catalog/record.rs:61`
```rust
pub fn decode_record( input: &[u8], expected_sequence: u64, expected_previous_crc: u32, integrity: CatalogIntegrity, key: Option<&Key32>, previous_mac: &[u8
```

#### `fn encode_record`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/catalog/record.rs:23`
```rust
pub fn encode_record( sequence: u64, previous_crc: u32, ops: &[CatalogOp], integrity: CatalogIntegrity, key: Option<&Key32>, previous_mac: &[u8
```

### Modul `catalog::reserve`

#### `struct Reservation`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/catalog/reserve.rs:11`
```rust
pub struct Reservation
```
- Impl-Block `crates/contextra-storage/crates/contextra-storage/src/catalog/reserve.rs:13`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn contains(&self, value: u64) -> bool` | `crates/contextra-storage/crates/contextra-storage/src/catalog/reserve.rs:14` |  |

#### `fn after_reopen`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/catalog/reserve.rs:43`
```rust
pub fn after_reopen(stamp_limit: Stamp, id_limit: u64, space_id_limit: u64) -> Result<(Stamp, u64, u64), Error>
```
Dokumentation: Recovery starts past the entire persisted range, preventing stamp reuse even when only a prefix of that range reached Redo before the crash.

#### `fn next_id_reservation`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/catalog/reserve.rs:27`
```rust
pub fn next_id_reservation(current_limit: u64, amount: u64) -> Result<(CatalogOp, Reservation), Error>
```

#### `fn next_space_id_reservation`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/catalog/reserve.rs:35`
```rust
pub fn next_space_id_reservation(current_limit: u64, amount: u64) -> Result<(CatalogOp, Reservation), Error>
```
Dokumentation: User-space IDs start after the reserved system range. The high-water mark is persisted before any ID in the returned interval may be issued.

#### `fn next_stamp_reservation`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/catalog/reserve.rs:19`
```rust
pub fn next_stamp_reservation(current_limit: Stamp, amount: Stamp) -> Result<(CatalogOp, Reservation), Error>
```
Dokumentation: Prepare a monotone reservation edit; the caller must durably commit the returned op before using any ID in the range.

#### `const ID_RESERVATION`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/catalog/reserve.rs:7`
```rust
pub const ID_RESERVATION: u64 = 1 << 16
```

#### `const SPACE_ID_RESERVATION`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/catalog/reserve.rs:8`
```rust
pub const SPACE_ID_RESERVATION: u64 = 1 << 16
```

#### `const STAMP_RESERVATION`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/catalog/reserve.rs:6`
```rust
pub const STAMP_RESERVATION: Stamp = 1 << 20
```

### Modul `catalog::rollover`

#### `struct CurrentPointer`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/catalog/rollover.rs:12`
```rust
pub struct CurrentPointer
```
- Impl-Block `crates/contextra-storage/crates/contextra-storage/src/catalog/rollover.rs:14`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn encode(self) -> Result<[u8` | `crates/contextra-storage/crates/contextra-storage/src/catalog/rollover.rs:15` |  |
| `pub fn decode(bytes: &[u8]) -> Result<Self, Error>` | `crates/contextra-storage/crates/contextra-storage/src/catalog/rollover.rs:28` |  |

#### `fn publish_current`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/catalog/rollover.rs:42`
```rust
pub fn publish_current(fs: &dyn Fs, root: &Path, pointer: CurrentPointer) -> Result<(), Error>
```

#### `fn read_current`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/catalog/rollover.rs:65`
```rust
pub fn read_current(fs: &dyn Fs, root: &Path) -> Result<CurrentPointer, Error>
```

### Modul `catalog::state`

#### `struct CatalogCluster`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/catalog/state.rs:11`
```rust
pub struct CatalogCluster
```

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `protection` | `Protection` | `crates/contextra-storage/crates/contextra-storage/src/catalog/state.rs:12` |
| `lifecycle` | `ClusterLifecycle` | `crates/contextra-storage/crates/contextra-storage/src/catalog/state.rs:13` |

#### `struct CatalogState`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/catalog/state.rs:23`
```rust
pub struct CatalogState
```
- Impl-Block (Trait `Default`) `crates/contextra-storage/crates/contextra-storage/src/catalog/state.rs:38`
- Impl-Block `crates/contextra-storage/crates/contextra-storage/src/catalog/state.rs:48`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn fold(records: &[CatalogRecord]) -> Result<Self, Error>` | `crates/contextra-storage/crates/contextra-storage/src/catalog/state.rs:49` |  |
| `pub fn apply(&mut self, op: &CatalogOp) -> Result<(), Error>` | `crates/contextra-storage/crates/contextra-storage/src/catalog/state.rs:63` |  |
| `pub fn spaces(&self) -> &BTreeMap<SpaceId, Protection>` | `crates/contextra-storage/crates/contextra-storage/src/catalog/state.rs:177` |  |
| `pub fn clusters(&self) -> &BTreeMap<ClusterId, CatalogCluster>` | `crates/contextra-storage/crates/contextra-storage/src/catalog/state.rs:178` |  |
| `pub fn files(&self) -> &BTreeMap<u64, FileMeta>` | `crates/contextra-storage/crates/contextra-storage/src/catalog/state.rs:179` |  |
| `pub fn graveyard(&self) -> &BTreeMap<([u8` | `crates/contextra-storage/crates/contextra-storage/src/catalog/state.rs:180` |  |
| `pub fn purge_is_complete(&self, ticket: &[u8` | `crates/contextra-storage/crates/contextra-storage/src/catalog/state.rs:181` |  |
| `pub fn replay_floor(&self) -> (u64, u64)` | `crates/contextra-storage/crates/contextra-storage/src/catalog/state.rs:182` |  |
| `pub fn redo_mark(&self) -> (u64, u64, Stamp)` | `crates/contextra-storage/crates/contextra-storage/src/catalog/state.rs:183` |  |
| `pub fn stamp_limit(&self) -> Stamp` | `crates/contextra-storage/crates/contextra-storage/src/catalog/state.rs:184` |  |
| `pub fn id_limit(&self) -> u64` | `crates/contextra-storage/crates/contextra-storage/src/catalog/state.rs:185` |  |
| `pub fn space_id_limit(&self) -> u64` | `crates/contextra-storage/crates/contextra-storage/src/catalog/state.rs:186` |  |
| `pub fn epoch(&self) -> u64` | `crates/contextra-storage/crates/contextra-storage/src/catalog/state.rs:187` |  |

#### `struct GraveyardEntry`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/catalog/state.rs:17`
```rust
pub struct GraveyardEntry
```

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `ticket` | `[u8; 16]` | `crates/contextra-storage/crates/contextra-storage/src/catalog/state.rs:18` |
| `file` | `FileMeta` | `crates/contextra-storage/crates/contextra-storage/src/catalog/state.rs:19` |

#### `type FileMeta`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/catalog/state.rs:8`
```rust
pub type FileMeta = FileEntry
```

### Modul `checkpoint`

#### `mod create`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/checkpoint/mod.rs:2`
```rust
pub mod create
```

#### `mod restore`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/checkpoint/mod.rs:3`
```rust
pub mod restore
```

### Modul `codec`

#### `mod checksums`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/codec/mod.rs:2`
```rust
pub mod checksums
```

#### `mod comparator`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/codec/mod.rs:3`
```rust
pub mod comparator
```

#### `mod format`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/codec/mod.rs:4`
```rust
pub mod format
```

#### `mod header`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/codec/mod.rs:5`
```rust
pub mod header
```

#### `mod keys_codec`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/codec/mod.rs:6`
```rust
pub mod keys_codec
```

#### `mod seal`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/codec/mod.rs:7`
```rust
pub mod seal
```

#### `mod tags`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/codec/mod.rs:8`
```rust
pub mod tags
```

#### `mod varint`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/codec/mod.rs:9`
```rust
pub mod varint
```

### Modul `codec::checksums`

#### `fn append_crc32c`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/codec/checksums.rs:17`
```rust
pub fn append_crc32c(out: &mut Vec<u8>, bytes: &[u8])
```

#### `fn crc32c`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/codec/checksums.rs:5`
```rust
pub fn crc32c(bytes: &[u8]) -> u32
```

#### `fn verify_crc32c`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/codec/checksums.rs:21`
```rust
pub fn verify_crc32c(bytes: &[u8], expected_le: [u8
```

### Modul `codec::comparator`

#### `struct EntryKey`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/codec/comparator.rs:8`
```rust
pub struct EntryKey<'a>
```
Dokumentation: Sortierschlüssel für einen Eintrag. `tag` wird als opaker Wire-Tag verglichen.
- Impl-Block (Trait `core::fmt::Debug`) `crates/contextra-storage/crates/contextra-storage/src/codec/comparator.rs:15`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `cluster` | `ClusterId` | `crates/contextra-storage/crates/contextra-storage/src/codec/comparator.rs:9` |
| `branch` | `BranchId` | `crates/contextra-storage/crates/contextra-storage/src/codec/comparator.rs:10` |
| `user_key` | `&'a [u8]` | `crates/contextra-storage/crates/contextra-storage/src/codec/comparator.rs:11` |
| `tag` | `u64` | `crates/contextra-storage/crates/contextra-storage/src/codec/comparator.rs:12` |

#### `fn cmp_entry`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/codec/comparator.rs:22`
```rust
pub fn cmp_entry(left: &EntryKey<'_>, right: &EntryKey<'_>) -> Ordering
```
Dokumentation: Vergleicht Felder einzeln; Konkatenation wäre wegen Präfixen mehrdeutig.

### Modul `codec::format`

#### `struct FormatRecord`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/codec/format.rs:25`
```rust
pub struct FormatRecord
```

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `integrity` | `IntegrityMode` | `crates/contextra-storage/crates/contextra-storage/src/codec/format.rs:26` |
| `feature_bits` | `u32` | `crates/contextra-storage/crates/contextra-storage/src/codec/format.rs:27` |
| `instance_id` | `[u8; 16]` | `crates/contextra-storage/crates/contextra-storage/src/codec/format.rs:28` |

#### `enum IntegrityMode`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/codec/format.rs:15`
```rust
pub enum IntegrityMode
```
- Impl-Block (Trait `TryFrom<u8>`) `crates/contextra-storage/crates/contextra-storage/src/codec/format.rs:17`

#### `fn decode`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/codec/format.rs:46`
```rust
pub fn decode(input: &[u8], root_key: &Key32, kdf: &dyn Kdf, mac: &dyn Mac, supported_features: u32) -> Result<FormatRecord, Error>
```

#### `fn encode`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/codec/format.rs:31`
```rust
pub fn encode(record: FormatRecord, root_key: &Key32, kdf: &dyn Kdf, mac: &dyn Mac) -> Result<Vec<u8>, Error>
```

### Modul `codec::header`

#### `struct FileHeader`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/codec/header.rs:9`
```rust
pub struct FileHeader
```
- Impl-Block `crates/contextra-storage/crates/contextra-storage/src/codec/header.rs:18`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `magic` | `[u8; 8]` | `crates/contextra-storage/crates/contextra-storage/src/codec/header.rs:10` |
| `file_kind` | `u8` | `crates/contextra-storage/crates/contextra-storage/src/codec/header.rs:11` |
| `format_ver` | `u16` | `crates/contextra-storage/crates/contextra-storage/src/codec/header.rs:12` |
| `feature_bits` | `u32` | `crates/contextra-storage/crates/contextra-storage/src/codec/header.rs:13` |
| `file_id` | `u64` | `crates/contextra-storage/crates/contextra-storage/src/codec/header.rs:14` |
| `instance_id` | `[u8; 16]` | `crates/contextra-storage/crates/contextra-storage/src/codec/header.rs:15` |

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn encode(&self) -> Result<[u8` | `crates/contextra-storage/crates/contextra-storage/src/codec/header.rs:19` |  |
| `pub fn decode(input: &[u8], expected_magic: [u8` | `crates/contextra-storage/crates/contextra-storage/src/codec/header.rs:34` |  |

#### `const HEADER_LEN`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/codec/header.rs:5`
```rust
pub const HEADER_LEN: usize = 43
```

#### `const HEADER_VERSION`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/codec/header.rs:6`
```rust
pub const HEADER_VERSION: u16 = 1
```

### Modul `codec::keys_codec`

#### `struct KeysRecord`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/codec/keys_codec.rs:14`
```rust
pub struct KeysRecord
```

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `wrapped_root` | `Vec<u8>` | `crates/contextra-storage/crates/contextra-storage/src/codec/keys_codec.rs:15` |

#### `fn decode`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/codec/keys_codec.rs:36`
```rust
pub fn decode(input: &[u8], root_key: &Key32, mac: &dyn Mac) -> Result<KeysRecord, Error>
```

#### `fn encode`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/codec/keys_codec.rs:19`
```rust
pub fn encode(record: &KeysRecord, root_key: &Key32, mac: &dyn Mac) -> Result<Vec<u8>, Error>
```

### Modul `codec::seal`

#### `enum Aad`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/codec/seal.rs:17`
```rust
pub enum Aad<'a>
```
- Impl-Block `crates/contextra-storage/crates/contextra-storage/src/codec/seal.rs:23`

Varianten:

- `Block{ instance: &'a [u8; 16], space: SpaceId, cluster: ClusterId, key_epoch: u32, block_uid: &'a [u8; 16] }` — `crates/contextra-storage/crates/contextra-storage/src/codec/seal.rs:18`
- `RedoOp{ instance: &'a [u8; 16], space: SpaceId, cluster: ClusterId, key_epoch: u32, op_uid: &'a [u8; 16], op_index: u16 }` — `crates/contextra-storage/crates/contextra-storage/src/codec/seal.rs:19`
- `BlobRecord{ instance: &'a [u8; 16], space: SpaceId, cluster: ClusterId, key_epoch: u32, record_uid: &'a [u8; 16] }` — `crates/contextra-storage/crates/contextra-storage/src/codec/seal.rs:20`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn encode(self) -> Vec<u8>` | `crates/contextra-storage/crates/contextra-storage/src/codec/seal.rs:28` |  |

#### `enum SealMode`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/codec/seal.rs:14`
```rust
pub enum SealMode
```

#### `fn seal_frame`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/codec/seal.rs:49`
```rust
pub fn seal_frame( mode: SealMode, key_epoch: u32, source: Option<&Key32>, nonce: &[u8
```

#### `fn unseal_frame`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/codec/seal.rs:90`
```rust
pub fn unseal_frame( mode: SealMode, source: Option<&Key32>, aad: Aad<'_>, frame: &[u8], cipher: &dyn Cipher, mac: &dyn Mac, checksum: &dyn Checksum, ) -> Result<(u32, Zeroizing<Vec<u8>>), Error>
```

### Modul `codec::tags`

#### `enum Kind`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/codec/tags.rs:7`
```rust
pub enum Kind
```

Varianten:

- `Put= 1` — `crates/contextra-storage/crates/contextra-storage/src/codec/tags.rs:8`
- `Delete= 2` — `crates/contextra-storage/crates/contextra-storage/src/codec/tags.rs:9`
- `RangeDelete= 3` — `crates/contextra-storage/crates/contextra-storage/src/codec/tags.rs:10`
- `EraseRange= 4` — `crates/contextra-storage/crates/contextra-storage/src/codec/tags.rs:11`
- `PutRef= 5` — `crates/contextra-storage/crates/contextra-storage/src/codec/tags.rs:12`

#### `enum TagError`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/codec/tags.rs:16`
```rust
pub enum TagError
```

Varianten:

- `StampZero` — `crates/contextra-storage/crates/contextra-storage/src/codec/tags.rs:17`
- `StampOutOfRange` — `crates/contextra-storage/crates/contextra-storage/src/codec/tags.rs:18`
- `UnknownKind(u8)` — `crates/contextra-storage/crates/contextra-storage/src/codec/tags.rs:19`
- `ReservedMerge` — `crates/contextra-storage/crates/contextra-storage/src/codec/tags.rs:20`

#### `fn tag`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/codec/tags.rs:25`
```rust
pub fn tag(stamp: Stamp, kind: Kind) -> Result<u64, TagError>
```
Dokumentation: Kodiert Stamp und Operationstyp. Stampwerte oberhalb der 56-Bit-Grenze werden abgewiesen, damit ein Überlauf nie einen alten Tag reproduziert.

#### `fn untag`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/codec/tags.rs:36`
```rust
pub fn untag(value: u64) -> Result<(Stamp, Kind), TagError>
```
Dokumentation: Dekodiert nur bekannte kanonische Tags; Typ 6 ist explizit reserviert.

### Modul `codec::varint`

#### `enum VarintError`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/codec/varint.rs:7`
```rust
pub enum VarintError
```
Dokumentation: Fehler beim Lesen eines kanonischen varint.
- Impl-Block (Trait `fmt::Display`) `crates/contextra-storage/crates/contextra-storage/src/codec/varint.rs:13`
- Impl-Block (Trait `std::error::Error`) `crates/contextra-storage/crates/contextra-storage/src/codec/varint.rs:23`

Varianten:

- `Truncated` — `crates/contextra-storage/crates/contextra-storage/src/codec/varint.rs:8`
- `Overflow` — `crates/contextra-storage/crates/contextra-storage/src/codec/varint.rs:9`
- `NonCanonical` — `crates/contextra-storage/crates/contextra-storage/src/codec/varint.rs:10`

#### `fn varint_get`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/codec/varint.rs:35`
```rust
pub fn varint_get(input: &[u8]) -> Result<(u64, usize), VarintError>
```
Dokumentation: Liest genau ein kanonisches unsigned LEB128 und liefert Wert sowie Bytezahl.

#### `fn varint_put`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/codec/varint.rs:26`
```rust
pub fn varint_put(out: &mut Vec<u8>, mut value: u64)
```
Dokumentation: Hängt `value` als kleinstmögliche unsigned LEB128-Kodierung an.

### Modul `config`

#### `struct Config`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/config.rs:10`
```rust
pub struct Config
```
- Impl-Block (Trait `fmt::Debug`) `crates/contextra-storage/crates/contextra-storage/src/config.rs:28`
- Impl-Block (Trait `Default`) `crates/contextra-storage/crates/contextra-storage/src/config.rs:48`
- Impl-Block `crates/contextra-storage/crates/contextra-storage/src/config.rs:69`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `max_body_bytes` | `usize` | `crates/contextra-storage/crates/contextra-storage/src/config.rs:11` |
| `max_change_bytes` | `usize` | `crates/contextra-storage/crates/contextra-storage/src/config.rs:12` |
| `max_group_bytes` | `usize` | `crates/contextra-storage/crates/contextra-storage/src/config.rs:13` |
| `redo_segment_bytes` | `u64` | `crates/contextra-storage/crates/contextra-storage/src/config.rs:14` |
| `max_clusters` | `u64` | `crates/contextra-storage/crates/contextra-storage/src/config.rs:15` |
| `max_open_files` | `usize` | `crates/contextra-storage/crates/contextra-storage/src/config.rs:16` |
| `erase_reserve_bytes` | `u64` | `crates/contextra-storage/crates/contextra-storage/src/config.rs:17` |
| `max_lag_bytes` | `u64` | `crates/contextra-storage/crates/contextra-storage/src/config.rs:18` |
| `key_grace_secs` | `u64` | `crates/contextra-storage/crates/contextra-storage/src/config.rs:19` |
| `root` | `Option<PathBuf>` | `crates/contextra-storage/crates/contextra-storage/src/config.rs:21` |
| `fs` | `Option<Arc<dyn Fs>>` | `crates/contextra-storage/crates/contextra-storage/src/config.rs:22` |
| `catalog_key` | `Option<Arc<Key32>>` | `crates/contextra-storage/crates/contextra-storage/src/config.rs:23` |
| `catalog_mac` | `Option<Arc<dyn Mac>>` | `crates/contextra-storage/crates/contextra-storage/src/config.rs:24` |
| `catalog_integrity` | `crate::catalog::CatalogIntegrity` | `crates/contextra-storage/crates/contextra-storage/src/config.rs:25` |

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn with_filesystem(mut self, root: PathBuf, fs: Arc<dyn Fs>) -> Self` | `crates/contextra-storage/crates/contextra-storage/src/config.rs:70` |  |
| `pub fn with_catalog_auth(mut self, key: Arc<Key32>, mac: Arc<dyn Mac>) -> Self` | `crates/contextra-storage/crates/contextra-storage/src/config.rs:76` |  |
| `pub fn validate(&self) -> Result<(), Error>` | `crates/contextra-storage/crates/contextra-storage/src/config.rs:83` |  |

### Modul `crate`

#### `mod catalog`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/lib.rs:10`
```rust
pub mod catalog
```

#### `mod checkpoint`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/lib.rs:11`
```rust
pub mod checkpoint
```

#### `mod codec`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/lib.rs:12`
```rust
pub mod codec
```

#### `mod config`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/lib.rs:13`
```rust
pub mod config
```

#### `mod engine`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/lib.rs:14`
```rust
pub mod engine
```

#### `mod env`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/lib.rs:15`
```rust
pub mod env
```

#### `mod erase`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/lib.rs:16`
```rust
pub mod erase
```

#### `mod keys`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/lib.rs:17`
```rust
pub mod keys
```

#### `mod maint`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/lib.rs:18`
```rust
pub mod maint
```

#### `mod metrics`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/lib.rs:19`
```rust
pub mod metrics
```

#### `mod recovery`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/lib.rs:20`
```rust
pub mod recovery
```

#### `mod spaces`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/lib.rs:21`
```rust
pub mod spaces
```

#### `mod store`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/lib.rs:22`
```rust
pub mod store
```

#### `mod sync`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/lib.rs:23`
```rust
pub mod sync
```

#### `mod testing`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/lib.rs:25`
```rust
pub mod testing
```
- Gate: `#[cfg(feature = "testing")]`

#### `struct Storage`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/lib.rs:34`
```rust
pub struct Storage
```
Dokumentation: Public storage façade. Persistent lifecycle operations are owned by their SP modules.
- Impl-Block `crates/contextra-storage/crates/contextra-storage/src/lib.rs:40`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn open(config: Config) -> Result<Storage, Error>` | `crates/contextra-storage/crates/contextra-storage/src/lib.rs:42` | Validates configuration before opening the durable Catalog. |
| `pub fn catalog(&self) -> &catalog::state::CatalogState` | `crates/contextra-storage/crates/contextra-storage/src/lib.rs:71` |  |

### Modul `engine`

#### `mod branches`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/engine/mod.rs:2`
```rust
pub mod branches
```

#### `mod cache`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/engine/mod.rs:3`
```rust
pub mod cache
```

#### `mod committer`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/engine/mod.rs:4`
```rust
pub mod committer
```

#### `mod cursor`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/engine/mod.rs:5`
```rust
pub mod cursor
```

#### `mod extension`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/engine/mod.rs:6`
```rust
pub mod extension
```

#### `mod feed`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/engine/mod.rs:7`
```rust
pub mod feed
```

#### `mod graveyard`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/engine/mod.rs:8`
```rust
pub mod graveyard
```

#### `mod guard_window`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/engine/mod.rs:9`
```rust
pub mod guard_window
```

#### `mod idem`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/engine/mod.rs:10`
```rust
pub mod idem
```

#### `mod memtable`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/engine/mod.rs:11`
```rust
pub mod memtable
```

#### `mod packgroups`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/engine/mod.rs:12`
```rust
pub mod packgroups
```

#### `mod page`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/engine/mod.rs:13`
```rust
pub mod page
```

#### `mod registry`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/engine/mod.rs:14`
```rust
pub mod registry
```

#### `mod segments`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/engine/mod.rs:15`
```rust
pub mod segments
```

#### `mod version`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/engine/mod.rs:16`
```rust
pub mod version
```

#### `mod views`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/engine/mod.rs:17`
```rust
pub mod views
```

#### `mod volatile`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/engine/mod.rs:18`
```rust
pub mod volatile
```

### Modul `engine::branches`

#### `struct BranchMeta`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/engine/branches.rs:41`
```rust
pub struct BranchMeta
```
- Impl-Block `crates/contextra-storage/crates/contextra-storage/src/engine/branches.rs:52`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn id(&self) -> BranchId` | `crates/contextra-storage/crates/contextra-storage/src/engine/branches.rs:53` |  |
| `pub fn parent(&self) -> Option<BranchId>` | `crates/contextra-storage/crates/contextra-storage/src/engine/branches.rs:54` |  |
| `pub fn fork_stamp(&self) -> Stamp` | `crates/contextra-storage/crates/contextra-storage/src/engine/branches.rs:55` |  |
| `pub fn head_stamp(&self) -> Stamp` | `crates/contextra-storage/crates/contextra-storage/src/engine/branches.rs:56` |  |
| `pub fn depth(&self) -> u8` | `crates/contextra-storage/crates/contextra-storage/src/engine/branches.rs:57` |  |
| `pub fn state(&self) -> BranchState` | `crates/contextra-storage/crates/contextra-storage/src/engine/branches.rs:58` |  |
| `pub fn name(&self) -> &[u8]` | `crates/contextra-storage/crates/contextra-storage/src/engine/branches.rs:59` |  |
| `pub fn ttl(&self) -> Option<Duration>` | `crates/contextra-storage/crates/contextra-storage/src/engine/branches.rs:60` |  |
| `pub fn is_live(&self) -> bool` | `crates/contextra-storage/crates/contextra-storage/src/engine/branches.rs:61` |  |

#### `struct BranchOptions`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/engine/branches.rs:23`
```rust
pub struct BranchOptions
```
- Impl-Block (Trait `Default`) `crates/contextra-storage/crates/contextra-storage/src/engine/branches.rs:29`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `name` | `Vec<u8>` | `crates/contextra-storage/crates/contextra-storage/src/engine/branches.rs:25` |
| `ttl` | `Option<Duration>` | `crates/contextra-storage/crates/contextra-storage/src/engine/branches.rs:26` |

#### `struct BranchTable`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/engine/branches.rs:67`
```rust
pub struct BranchTable
```
Dokumentation: Per-cluster branch projection. IDs are monotone and deliberately not recycled after drop. The owning storage layer serializes mutations through its single writer and persists each mutation in the same Redo frame.
- Impl-Block `crates/contextra-storage/crates/contextra-storage/src/engine/branches.rs:75`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new(history_floor: Stamp, main_head: Stamp, max_branches: usize, max_depth: u8) -> Result<Self, Error>` | `crates/contextra-storage/crates/contextra-storage/src/engine/branches.rs:76` |  |
| `pub fn history_floor(&self) -> Stamp` | `crates/contextra-storage/crates/contextra-storage/src/engine/branches.rs:97` |  |
| `pub fn len(&self) -> usize` | `crates/contextra-storage/crates/contextra-storage/src/engine/branches.rs:98` |  |
| `pub fn is_empty(&self) -> bool` | `crates/contextra-storage/crates/contextra-storage/src/engine/branches.rs:99` |  |
| `pub fn get(&self, id: BranchId) -> Option<&BranchMeta>` | `crates/contextra-storage/crates/contextra-storage/src/engine/branches.rs:100` |  |
| `pub fn iter(&self) -> impl Iterator<Item = &BranchMeta>` | `crates/contextra-storage/crates/contextra-storage/src/engine/branches.rs:101` |  |
| `pub fn fork(&mut self, id: BranchId, parent: BranchId, point: ForkPoint, options: BranchOptions, parent_head: Stamp, commit_stamp: Stamp) -> Result<Stamp, Error>` | `crates/contextra-storage/crates/contextra-storage/src/engine/branches.rs:107` | Creates an O(1) child descriptor. `id` is consumed monotonically; in a durable integration it must come from a completed durable reservation. `commit_stamp` is  |
| `pub fn advance_head(&mut self, id: BranchId, expected: Stamp, next: Stamp) -> Result<(), Error>` | `crates/contextra-storage/crates/contextra-storage/src/engine/branches.rs:135` | Advances a live branch head after its commit has been durably accepted. |
| `pub fn mark_merged(&mut self, source: BranchId, target: BranchId, expected_source_head: Stamp, expected_target_head: Stamp, at: Stamp) -> Result<(), Error>` | `crates/contextra-storage/crates/contextra-storage/src/engine/branches.rs:146` | Merge is terminal for the source and valid only for a direct child. The caller must atomically persist target mutations and this transition. |
| `pub fn drop_branch(&mut self, id: BranchId, at: Stamp) -> Result<(), Error>` | `crates/contextra-storage/crates/contextra-storage/src/engine/branches.rs:164` | Drops only a leaf. Merged branches remain readable until explicitly dropped. |
| `pub fn set_history_floor(&mut self, floor: Stamp) -> Result<(), Error>` | `crates/contextra-storage/crates/contextra-storage/src/engine/branches.rs:174` |  |
| `pub fn visibility_chain(&self, branch: BranchId, view_stamp: Stamp) -> Result<Vec<(BranchId, Stamp)>, Error>` | `crates/contextra-storage/crates/contextra-storage/src/engine/branches.rs:184` | Returns source-to-root visibility caps, including the requested branch. |

#### `enum BranchState`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/engine/branches.rs:34`
```rust
pub enum BranchState
```

Varianten:

- `Live` — `crates/contextra-storage/crates/contextra-storage/src/engine/branches.rs:35`
- `Merged{ at: Stamp }` — `crates/contextra-storage/crates/contextra-storage/src/engine/branches.rs:36`
- `Dropped{ at: Stamp }` — `crates/contextra-storage/crates/contextra-storage/src/engine/branches.rs:37`

#### `enum ForkPoint`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/engine/branches.rs:17`
```rust
pub enum ForkPoint
```

Varianten:

- `Head` — `crates/contextra-storage/crates/contextra-storage/src/engine/branches.rs:18`
- `At(Stamp)` — `crates/contextra-storage/crates/contextra-storage/src/engine/branches.rs:19`

#### `const DEFAULT_MAX_BRANCHES`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/engine/branches.rs:13`
```rust
pub const DEFAULT_MAX_BRANCHES: usize = 1024
```

#### `const DEFAULT_MAX_DEPTH`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/engine/branches.rs:14`
```rust
pub const DEFAULT_MAX_DEPTH: u8 = 8
```

#### `const MAIN_BRANCH`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/engine/branches.rs:12`
```rust
pub const MAIN_BRANCH: BranchId = BranchId(0)
```

### Modul `engine::committer`

#### `struct CommitMutation`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/engine/committer.rs:11`
```rust
pub struct CommitMutation
```

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `cluster` | `ClusterId` | `crates/contextra-storage/crates/contextra-storage/src/engine/committer.rs:12` |
| `branch` | `BranchId` | `crates/contextra-storage/crates/contextra-storage/src/engine/committer.rs:13` |
| `key` | `Vec<u8>` | `crates/contextra-storage/crates/contextra-storage/src/engine/committer.rs:14` |
| `kind` | `u8` | `crates/contextra-storage/crates/contextra-storage/src/engine/committer.rs:15` |
| `payload` | `Vec<u8>` | `crates/contextra-storage/crates/contextra-storage/src/engine/committer.rs:16` |

#### `struct Committer`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/engine/committer.rs:93`
```rust
pub struct Committer<'a>
```
- Impl-Block `crates/contextra-storage/crates/contextra-storage/src/engine/committer.rs:100`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new(writer: RedoWriter<'a>, publisher: Box<dyn CommitPublisher>) -> Self` | `crates/contextra-storage/crates/contextra-storage/src/engine/committer.rs:101` |  |
| `pub fn commit(&mut self, frames: &[EncodedFrame], durability: Durability) -> Result<GroupAck, Error>` | `crates/contextra-storage/crates/contextra-storage/src/engine/committer.rs:114` | Synchronous sequencer entry point. Group formation is intentionally left to the intake layer; this method guarantees one append and one sync per supplied group, |
| `pub fn last_published_stamp(&self) -> Stamp` | `crates/contextra-storage/crates/contextra-storage/src/engine/committer.rs:154` |  |
| `pub fn is_poisoned(&self) -> bool` | `crates/contextra-storage/crates/contextra-storage/src/engine/committer.rs:156` |  |

#### `struct MemtableCommitPublisher`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/engine/committer.rs:35`
```rust
pub struct MemtableCommitPublisher
```
Dokumentation: Concrete commit-to-memtable/MVCC adapter. The decoder owns operation format and key-scope policy; this publisher owns stamp ordering and post-durability visibility.
- Impl-Block `crates/contextra-storage/crates/contextra-storage/src/engine/committer.rs:41`
- Impl-Block (Trait `CommitPublisher`) `crates/contextra-storage/crates/contextra-storage/src/engine/committer.rs:47`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new(memtable: Arc<Memtable>, versions: Arc<VersionRegistry>, decoder: Arc<dyn CommitMutationDecoder>) -> Self` | `crates/contextra-storage/crates/contextra-storage/src/engine/committer.rs:42` |  |

#### `trait CommitMutationDecoder`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/engine/committer.rs:21`
```rust
pub trait CommitMutationDecoder: Send + Sync
```
Dokumentation: Decodes already-authenticated frame operations. A sealed implementation must verify the cluster generation before and after unsealing.

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `fn decode(&self, frame: &EncodedFrame) -> Result<Vec<CommitMutation>, Error>` | `crates/contextra-storage/crates/contextra-storage/src/engine/committer.rs:22` |  |

#### `trait CommitPublisher`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/engine/committer.rs:28`
```rust
pub trait CommitPublisher: Send + Sync
```
Dokumentation: Adapter applies committed frames to the memtable and publishes a new MVCC root. It is called only after the append/sync result is known and without storage-internal locks held by this module.

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `fn publish(&self, frames: &[EncodedFrame], ack: GroupAck) -> Result<(), Error>` | `crates/contextra-storage/crates/contextra-storage/src/engine/committer.rs:29` |  |

### Modul `engine::graveyard`

#### `struct Graveyard`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/engine/graveyard.rs:5`
```rust
pub struct Graveyard<T>
```
- Impl-Block (Trait `Default`) `crates/contextra-storage/crates/contextra-storage/src/engine/graveyard.rs:7`
- Impl-Block `crates/contextra-storage/crates/contextra-storage/src/engine/graveyard.rs:11`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new() -> Self` | `crates/contextra-storage/crates/contextra-storage/src/engine/graveyard.rs:12` |  |
| `pub fn retire(&mut self, object: Arc<T>)` | `crates/contextra-storage/crates/contextra-storage/src/engine/graveyard.rs:14` |  |
| `pub fn pending(&self) -> usize` | `crates/contextra-storage/crates/contextra-storage/src/engine/graveyard.rs:18` |  |
| `pub fn reclaim(&mut self) -> Vec<T>` | `crates/contextra-storage/crates/contextra-storage/src/engine/graveyard.rs:22` | Extracts objects whose sole remaining strong reference belongs to this graveyard. The caller may now unlink files or reclaim partition memory. |

### Modul `engine::guard_window`

#### `struct GuardMutation`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/engine/guard_window.rs:21`
```rust
pub struct GuardMutation
```

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `stamp` | `Stamp` | `crates/contextra-storage/crates/contextra-storage/src/engine/guard_window.rs:22` |
| `fingerprint` | `u64` | `crates/contextra-storage/crates/contextra-storage/src/engine/guard_window.rs:24` |
| `scope` | `GuardScope` | `crates/contextra-storage/crates/contextra-storage/src/engine/guard_window.rs:25` |

#### `struct GuardScope`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/engine/guard_window.rs:14`
```rust
pub struct GuardScope
```

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `space` | `SpaceId` | `crates/contextra-storage/crates/contextra-storage/src/engine/guard_window.rs:15` |
| `cluster` | `ClusterId` | `crates/contextra-storage/crates/contextra-storage/src/engine/guard_window.rs:16` |
| `branch` | `BranchId` | `crates/contextra-storage/crates/contextra-storage/src/engine/guard_window.rs:17` |

#### `struct GuardWindow`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/engine/guard_window.rs:31`
```rust
pub struct GuardWindow
```
Dokumentation: A bounded suffix of sequenced mutations, including mutations whose group has not reached its durability point yet. It must be advanced in exact stamp order by the single sequencer.
- Impl-Block `crates/contextra-storage/crates/contextra-storage/src/engine/guard_window.rs:37`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new(capacity: usize) -> Result<Self, Error>` | `crates/contextra-storage/crates/contextra-storage/src/engine/guard_window.rs:38` |  |
| `pub fn record(&mut self, mutation: GuardMutation) -> Result<(), Error>` | `crates/contextra-storage/crates/contextra-storage/src/engine/guard_window.rs:47` | Records one accepted mutation. Stamps must strictly increase; this prevents an out-of-order caller from making an old mutation invisible. |
| `pub fn check_point(&self, snapshot: Stamp, fingerprint: u64) -> Result<(), Error>` | `crates/contextra-storage/crates/contextra-storage/src/engine/guard_window.rs:61` | Returns `GuardRetry` if the proof predates retained history or the same key changed after the caller's snapshot. |
| `pub fn check_range(&self, snapshot: Stamp, scope: GuardScope) -> Result<(), Error>` | `crates/contextra-storage/crates/contextra-storage/src/engine/guard_window.rs:71` | Conservative range validation. Any mutation in the same partition after the snapshot requires exact revalidation by the caller. |
| `pub fn floor(&self) -> Stamp` | `crates/contextra-storage/crates/contextra-storage/src/engine/guard_window.rs:81` | Earliest snapshot stamp that can still be checked exactly by this ring. A snapshot equal to the floor is valid; an older one must retry Stufe A. |
| `pub fn len(&self) -> usize` | `crates/contextra-storage/crates/contextra-storage/src/engine/guard_window.rs:85` |  |
| `pub fn is_empty(&self) -> bool` | `crates/contextra-storage/crates/contextra-storage/src/engine/guard_window.rs:86` |  |

#### `const MAX_GUARD_WINDOW_ENTRIES`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/engine/guard_window.rs:11`
```rust
pub const MAX_GUARD_WINDOW_ENTRIES: usize = 1_000_000
```

### Modul `engine::memtable`

#### `struct DetachedPartition`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/engine/memtable.rs:108`
```rust
pub struct DetachedPartition
```
- Impl-Block `crates/contextra-storage/crates/contextra-storage/src/engine/memtable.rs:110`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn approx_bytes(&self) -> u64` | `crates/contextra-storage/crates/contextra-storage/src/engine/memtable.rs:111` |  |
| `pub fn is_empty(&self) -> bool` | `crates/contextra-storage/crates/contextra-storage/src/engine/memtable.rs:112` |  |
| `pub fn visit(&self, mut visitor: impl FnMut(&MemKey, &Arc<MemValue>))` | `crates/contextra-storage/crates/contextra-storage/src/engine/memtable.rs:113` |  |

#### `struct MemKey`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/engine/memtable.rs:12`
```rust
pub struct MemKey
```
- Impl-Block (Trait `PartialEq`) `crates/contextra-storage/crates/contextra-storage/src/engine/memtable.rs:19`
- Impl-Block (Trait `Eq`) `crates/contextra-storage/crates/contextra-storage/src/engine/memtable.rs:24`
- Impl-Block (Trait `PartialOrd`) `crates/contextra-storage/crates/contextra-storage/src/engine/memtable.rs:25`
- Impl-Block (Trait `Ord`) `crates/contextra-storage/crates/contextra-storage/src/engine/memtable.rs:28`
- Impl-Block (Trait `fmt::Debug`) `crates/contextra-storage/crates/contextra-storage/src/engine/memtable.rs:35`
- Impl-Block `crates/contextra-storage/crates/contextra-storage/src/engine/memtable.rs:39`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `branch` | `BranchId` | `crates/contextra-storage/crates/contextra-storage/src/engine/memtable.rs:13` |
| `user_key` | `Zeroizing<Vec<u8>>` | `crates/contextra-storage/crates/contextra-storage/src/engine/memtable.rs:14` |
| `tag` | `u64` | `crates/contextra-storage/crates/contextra-storage/src/engine/memtable.rs:16` |

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new(branch: BranchId, user_key: Vec<u8>, tag: u64) -> Self` | `crates/contextra-storage/crates/contextra-storage/src/engine/memtable.rs:40` |  |

#### `struct MemValue`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/engine/memtable.rs:45`
```rust
pub struct MemValue
```

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `kind` | `u8` | `crates/contextra-storage/crates/contextra-storage/src/engine/memtable.rs:46` |
| `payload` | `Zeroizing<Vec<u8>>` | `crates/contextra-storage/crates/contextra-storage/src/engine/memtable.rs:47` |

#### `struct Memtable`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/engine/memtable.rs:59`
```rust
pub struct Memtable
```
- Impl-Block (Trait `Default`) `crates/contextra-storage/crates/contextra-storage/src/engine/memtable.rs:63`
- Impl-Block `crates/contextra-storage/crates/contextra-storage/src/engine/memtable.rs:67`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new() -> Self` | `crates/contextra-storage/crates/contextra-storage/src/engine/memtable.rs:68` |  |
| `pub fn insert(&self, cluster: ClusterId, key: MemKey, kind: u8, payload: &[u8]) -> Result<(), Error>` | `crates/contextra-storage/crates/contextra-storage/src/engine/memtable.rs:70` |  |
| `pub fn get(&self, cluster: ClusterId, key: &MemKey) -> Result<Option<Arc<MemValue>>, Error>` | `crates/contextra-storage/crates/contextra-storage/src/engine/memtable.rs:83` |  |
| `pub fn detach_cluster(&self, cluster: ClusterId) -> Result<Option<DetachedPartition>, Error>` | `crates/contextra-storage/crates/contextra-storage/src/engine/memtable.rs:91` | Entfernt die Partition O(1); der zurückgegebene Arc wird auf urgent-Lane reclaimed. |
| `pub fn approx_bytes(&self) -> Result<u64, Error>` | `crates/contextra-storage/crates/contextra-storage/src/engine/memtable.rs:98` |  |
| `pub fn cluster_count(&self) -> Result<usize, Error>` | `crates/contextra-storage/crates/contextra-storage/src/engine/memtable.rs:103` |  |

### Modul `engine::registry`

#### `struct VersionRegistry`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/engine/registry.rs:14`
```rust
pub struct VersionRegistry
```
- Impl-Block (Trait `Default`) `crates/contextra-storage/crates/contextra-storage/src/engine/registry.rs:16`
- Impl-Block `crates/contextra-storage/crates/contextra-storage/src/engine/registry.rs:20`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new() -> Self` | `crates/contextra-storage/crates/contextra-storage/src/engine/registry.rs:21` |  |
| `pub fn snapshot(&self) -> Result<Arc<Version>, Error>` | `crates/contextra-storage/crates/contextra-storage/src/engine/registry.rs:28` |  |
| `pub fn publish(&self, version: Arc<Version>) -> Result<(), Error>` | `crates/contextra-storage/crates/contextra-storage/src/engine/registry.rs:36` | Publishes immutable roots and visibility/durability watermarks as one coherent state. Durable contents are supplied independently so Relaxed writes can never le |
| `pub fn publish_group(&self, versions: &[Arc<Version>]) -> Result<(), Error>` | `crates/contextra-storage/crates/contextra-storage/src/engine/registry.rs:40` |  |
| `pub fn view_at(&self, stamp: Stamp) -> Result<Arc<Version>, Error>` | `crates/contextra-storage/crates/contextra-storage/src/engine/registry.rs:68` |  |
| `pub fn view_durable(&self) -> Result<Arc<Version>, Error>` | `crates/contextra-storage/crates/contextra-storage/src/engine/registry.rs:77` |  |
| `pub fn advance_recovered(&self, stamp: Stamp) -> Result<(), Error>` | `crates/contextra-storage/crates/contextra-storage/src/engine/registry.rs:82` |  |
| `pub fn prune_before(&self, floor: Stamp) -> Result<(), Error>` | `crates/contextra-storage/crates/contextra-storage/src/engine/registry.rs:92` | Drops versions strictly older than the durable history floor. Active readers retain their roots through Arc references. |

### Modul `engine::version`

#### `struct Version`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/engine/version.rs:27`
```rust
pub struct Version
```
- Impl-Block `crates/contextra-storage/crates/contextra-storage/src/engine/version.rs:37`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn empty() -> Self` | `crates/contextra-storage/crates/contextra-storage/src/engine/version.rs:38` |  |
| `pub fn new( visible_stamp: Stamp, durable_stamp: Stamp, gc_floor: Stamp, partitions: Vec<Arc<DetachedPartition>>, durable_partitions: Vec<Arc<DetachedPartition>>, ) -> Result<Self, Error>` | `crates/contextra-storage/crates/contextra-storage/src/engine/version.rs:40` |  |
| `pub fn commit( parent: Arc<Version>, visible_stamp: Stamp, durable_stamp: Stamp, changes: Vec<VersionEntry>, ) -> Result<Self, Error>` | `crates/contextra-storage/crates/contextra-storage/src/engine/version.rs:53` |  |
| `pub fn visible_stamp(&self) -> Stamp` | `crates/contextra-storage/crates/contextra-storage/src/engine/version.rs:76` |  |
| `pub fn durable_stamp(&self) -> Stamp` | `crates/contextra-storage/crates/contextra-storage/src/engine/version.rs:77` |  |
| `pub fn gc_floor(&self) -> Stamp` | `crates/contextra-storage/crates/contextra-storage/src/engine/version.rs:78` |  |
| `pub fn partitions(&self) -> &[Arc<DetachedPartition>]` | `crates/contextra-storage/crates/contextra-storage/src/engine/version.rs:79` |  |
| `pub fn durable_partitions(&self) -> &[Arc<DetachedPartition>]` | `crates/contextra-storage/crates/contextra-storage/src/engine/version.rs:80` |  |
| `pub fn changes(&self) -> &[VersionEntry]` | `crates/contextra-storage/crates/contextra-storage/src/engine/version.rs:81` |  |
| `pub fn get(&self, cluster: ClusterId, branch: BranchId, key: &[u8], cap: Stamp) -> Option<&VersionEntry>` | `crates/contextra-storage/crates/contextra-storage/src/engine/version.rs:85` | Returns the newest point mutation at or below `cap`. Tombstone kinds are returned to the read layer for visibility resolution. |
| `pub fn get_on_branch<'a>( &'a self, branches: &super::branches::BranchTable, cluster: ClusterId, branch: BranchId, key: &[u8], view_stamp: Stamp, ) -> Result<Option<&'a VersionEntry>, Error>` | `crates/contextra-storage/crates/contextra-storage/src/engine/version.rs:102` | Resolves point mutations through a cluster-local branch ancestry. The first branch in the chain wins even when the entry is a tombstone; callers must apply tomb |
| `pub fn at_stamp(&self, stamp: Stamp) -> Result<Self, Error>` | `crates/contextra-storage/crates/contextra-storage/src/engine/version.rs:118` |  |
| `pub fn with_gc_floor(&self, gc_floor: Stamp) -> Result<Self, Error>` | `crates/contextra-storage/crates/contextra-storage/src/engine/version.rs:133` |  |
| `pub fn compact_history(&self, gc_floor: Stamp) -> Result<Self, Error>` | `crates/contextra-storage/crates/contextra-storage/src/engine/version.rs:150` | Rebuilds a bounded immutable delta chain. The newest value at or below the floor is retained per key, along with every later mutation. |

#### `struct VersionEntry`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/engine/version.rs:11`
```rust
pub struct VersionEntry
```
- Impl-Block `crates/contextra-storage/crates/contextra-storage/src/engine/version.rs:20`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `cluster` | `ClusterId` | `crates/contextra-storage/crates/contextra-storage/src/engine/version.rs:12` |
| `branch` | `BranchId` | `crates/contextra-storage/crates/contextra-storage/src/engine/version.rs:13` |
| `key` | `Zeroizing<Vec<u8>>` | `crates/contextra-storage/crates/contextra-storage/src/engine/version.rs:14` |
| `stamp` | `Stamp` | `crates/contextra-storage/crates/contextra-storage/src/engine/version.rs:15` |
| `kind` | `u8` | `crates/contextra-storage/crates/contextra-storage/src/engine/version.rs:16` |
| `payload` | `Zeroizing<Vec<u8>>` | `crates/contextra-storage/crates/contextra-storage/src/engine/version.rs:17` |

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new(cluster: ClusterId, branch: BranchId, key: Vec<u8>, stamp: Stamp, kind: u8, payload: Vec<u8>) -> Self` | `crates/contextra-storage/crates/contextra-storage/src/engine/version.rs:21` |  |

### Modul `engine::views`

#### `struct VersionView`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/engine/views.rs:32`
```rust
pub struct VersionView
```
Dokumentation: Stable root and read cap captured at one registry observation. This primitive deliberately exposes no direct `Storage` facade method, so callers must opt into the internal view contract explicitly. It is not a fully filtered public storage view.
- Impl-Block `crates/contextra-storage/crates/contextra-storage/src/engine/views.rs:37`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn capture( registry: &VersionRegistry, point: SnapshotPoint, ) -> Result<Self, Error>` | `crates/contextra-storage/crates/contextra-storage/src/engine/views.rs:38` |  |
| `pub fn point(&self) -> SnapshotPoint` | `crates/contextra-storage/crates/contextra-storage/src/engine/views.rs:71` |  |
| `pub fn stamp(&self) -> Stamp` | `crates/contextra-storage/crates/contextra-storage/src/engine/views.rs:75` | Stamp represented by this view. For an `At` view this is the requested stamp; for visible and durable views it is the captured root's stamp. |
| `pub fn durable_stamp(&self) -> Stamp` | `crates/contextra-storage/crates/contextra-storage/src/engine/views.rs:79` | Durable watermark available in the captured root. It can be lower than `stamp()` only for a visible view that includes relaxed commits. |
| `pub fn root(&self) -> &Arc<Version>` | `crates/contextra-storage/crates/contextra-storage/src/engine/views.rs:81` |  |
| `pub fn point_mutation( &self, cluster: ClusterId, branch: BranchId, key: &[u8], ) -> Option<VersionEntry>` | `crates/contextra-storage/crates/contextra-storage/src/engine/views.rs:87` | Returns the newest point mutation for a cluster/branch/key at this snapshot. The caller must still resolve range tombstones and branch inheritance, then apply G |

#### `enum SnapshotPoint`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/engine/views.rs:17`
```rust
pub enum SnapshotPoint
```
Dokumentation: The source from which an immutable snapshot root is selected.

Varianten:

- `Visible` — `crates/contextra-storage/crates/contextra-storage/src/engine/views.rs:19`
- `Durable` — `crates/contextra-storage/crates/contextra-storage/src/engine/views.rs:21`
- `At(Stamp)` — `crates/contextra-storage/crates/contextra-storage/src/engine/views.rs:23`

### Modul `engine::volatile`

#### `struct ChangeDisposition`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/engine/volatile.rs:15`
```rust
pub struct ChangeDisposition
```

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `durable` | `bool` | `crates/contextra-storage/crates/contextra-storage/src/engine/volatile.rs:18` |
| `has_volatile` | `bool` | `crates/contextra-storage/crates/contextra-storage/src/engine/volatile.rs:19` |
| `has_durable` | `bool` | `crates/contextra-storage/crates/contextra-storage/src/engine/volatile.rs:20` |

#### `struct VolatileStore`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/engine/volatile.rs:98`
```rust
pub struct VolatileStore
```
Dokumentation: Process-local point-value store. The rank-5 lock protects only its own map; methods do not perform I/O or invoke caller-provided callbacks while holding it. No read API retains a lock after returning.
- Impl-Block (Trait `Default`) `crates/contextra-storage/crates/contextra-storage/src/engine/volatile.rs:102`
- Impl-Block `crates/contextra-storage/crates/contextra-storage/src/engine/volatile.rs:106`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new() -> Self` | `crates/contextra-storage/crates/contextra-storage/src/engine/volatile.rs:107` |  |
| `pub fn put( &self, space: SpaceId, cluster: ClusterId, branch: BranchId, key: &[u8], header: &[u8], body: &[u8], weights: [u32` | `crates/contextra-storage/crates/contextra-storage/src/engine/volatile.rs:109` |  |
| `pub fn get( &self, space: SpaceId, cluster: ClusterId, branch: BranchId, key: &[u8], ) -> Result<Option<VolatileValue>, Error>` | `crates/contextra-storage/crates/contextra-storage/src/engine/volatile.rs:131` | Returns an owned, zeroizing copy so callers cannot pin the store lock. |
| `pub fn delete( &self, space: SpaceId, cluster: ClusterId, branch: BranchId, key: &[u8], ) -> Result<bool, Error>` | `crates/contextra-storage/crates/contextra-storage/src/engine/volatile.rs:149` |  |
| `pub fn erase_space(&self, space: SpaceId) -> Result<usize, Error>` | `crates/contextra-storage/crates/contextra-storage/src/engine/volatile.rs:162` |  |
| `pub fn erase_cluster(&self, cluster: ClusterId) -> Result<usize, Error>` | `crates/contextra-storage/crates/contextra-storage/src/engine/volatile.rs:169` |  |
| `pub fn clear(&self) -> Result<usize, Error>` | `crates/contextra-storage/crates/contextra-storage/src/engine/volatile.rs:176` |  |
| `pub fn len(&self) -> Result<usize, Error>` | `crates/contextra-storage/crates/contextra-storage/src/engine/volatile.rs:183` |  |
| `pub fn is_empty(&self) -> Result<bool, Error>` | `crates/contextra-storage/crates/contextra-storage/src/engine/volatile.rs:184` |  |

#### `struct VolatileValue`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/engine/volatile.rs:86`
```rust
pub struct VolatileValue
```
Dokumentation: A volatile value owns zeroizing copies of the header and body.

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `header` | `Zeroizing<Vec<u8>>` | `crates/contextra-storage/crates/contextra-storage/src/engine/volatile.rs:87` |
| `body` | `Zeroizing<Vec<u8>>` | `crates/contextra-storage/crates/contextra-storage/src/engine/volatile.rs:88` |
| `weights` | `[u32; 4]` | `crates/contextra-storage/crates/contextra-storage/src/engine/volatile.rs:89` |

#### `fn validate_change`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/engine/volatile.rs:24`
```rust
pub fn validate_change( change: &Change, mut class_of: impl FnMut(SpaceId) -> Result<SpaceClass, Error>, ) -> Result<ChangeDisposition, Error>
```
Dokumentation: Validates a Change's Space mix before mutation or I/O.

### Modul `env`

#### `mod anchor`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/env/mod.rs:2`
```rust
pub mod anchor
```

#### `mod budget`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/env/mod.rs:3`
```rust
pub mod budget
```

#### `mod faultfs`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/env/mod.rs:4`
```rust
pub mod faultfs
```

#### `mod governor`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/env/mod.rs:5`
```rust
pub mod governor
```

#### `mod memfs`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/env/mod.rs:6`
```rust
pub mod memfs
```

#### `mod sim`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/env/mod.rs:7`
```rust
pub mod sim
```

### Modul `env::anchor`

#### `struct MemAnchor`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/env/anchor.rs:7`
```rust
pub struct MemAnchor
```
- Impl-Block `crates/contextra-storage/crates/contextra-storage/src/env/anchor.rs:9`
- Impl-Block (Trait `Default`) `crates/contextra-storage/crates/contextra-storage/src/env/anchor.rs:13`
- Impl-Block (Trait `Anchor`) `crates/contextra-storage/crates/contextra-storage/src/env/anchor.rs:17`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new(initial: AnchorValue) -> Self` | `crates/contextra-storage/crates/contextra-storage/src/env/anchor.rs:10` |  |

### Modul `env::budget`

#### `struct Budget`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/env/budget.rs:52`
```rust
pub struct Budget
```
- Impl-Block `crates/contextra-storage/crates/contextra-storage/src/env/budget.rs:54`
- Impl-Block (Trait `MemoryBudget`) `crates/contextra-storage/crates/contextra-storage/src/env/budget.rs:91`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new(limits: BudgetLimits) -> Result<Self, Error>` | `crates/contextra-storage/crates/contextra-storage/src/env/budget.rs:55` |  |
| `pub fn from_total(total_bytes: u64) -> Result<Self, Error>` | `crates/contextra-storage/crates/contextra-storage/src/env/budget.rs:65` |  |
| `pub fn used(&self) -> u64` | `crates/contextra-storage/crates/contextra-storage/src/env/budget.rs:69` |  |
| `pub fn class_used(&self, class: ResourceClass) -> u64` | `crates/contextra-storage/crates/contextra-storage/src/env/budget.rs:70` |  |
| `pub fn class_limit(&self, class: ResourceClass) -> u64` | `crates/contextra-storage/crates/contextra-storage/src/env/budget.rs:73` |  |

#### `struct BudgetLimits`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/env/budget.rs:11`
```rust
pub struct BudgetLimits
```
- Impl-Block `crates/contextra-storage/crates/contextra-storage/src/env/budget.rs:16`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `total_bytes` | `u64` | `crates/contextra-storage/crates/contextra-storage/src/env/budget.rs:12` |
| `class_bytes` | `[u64; CLASS_COUNT]` | `crates/contextra-storage/crates/contextra-storage/src/env/budget.rs:13` |

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn from_total(total_bytes: u64) -> Self` | `crates/contextra-storage/crates/contextra-storage/src/env/budget.rs:19` | Default shares from SPEC §19. PurgeScratch is separately backed by the durable disk reserve and therefore receives no RAM share by default. |
| `pub fn with_class(mut self, class: ResourceClass, bytes: u64) -> Self` | `crates/contextra-storage/crates/contextra-storage/src/env/budget.rs:30` |  |

### Modul `env::faultfs`

#### `struct FaultFs`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/env/faultfs.rs:36`
```rust
pub struct FaultFs<F: Fs>
```
- Impl-Block `crates/contextra-storage/crates/contextra-storage/src/env/faultfs.rs:41`
- Impl-Block `crates/contextra-storage/crates/contextra-storage/src/env/faultfs.rs:75`
- Impl-Block (Trait `Fs`) `crates/contextra-storage/crates/contextra-storage/src/env/faultfs.rs:152`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new(inner: F, plan: FaultPlan) -> Self` | `crates/contextra-storage/crates/contextra-storage/src/env/faultfs.rs:42` |  |
| `pub fn op_count(&self) -> u64` | `crates/contextra-storage/crates/contextra-storage/src/env/faultfs.rs:55` |  |
| `pub fn bytes_read(&self, role: FileRole) -> u64` | `crates/contextra-storage/crates/contextra-storage/src/env/faultfs.rs:59` |  |
| `pub fn inner(&self) -> &F` | `crates/contextra-storage/crates/contextra-storage/src/env/faultfs.rs:65` |  |
| `pub fn ghost_journal(&self) -> Result<Vec<Vec<u8>>, Error>` | `crates/contextra-storage/crates/contextra-storage/src/env/faultfs.rs:68` |  |
| `pub fn crash_image(&self) -> Result<super::memfs::MemFs, Error>` | `crates/contextra-storage/crates/contextra-storage/src/env/faultfs.rs:76` |  |

#### `struct FaultPlan`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/env/faultfs.rs:22`
```rust
pub struct FaultPlan
```

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `at_op` | `u64` | `crates/contextra-storage/crates/contextra-storage/src/env/faultfs.rs:24` |
| `fault` | `Fault` | `crates/contextra-storage/crates/contextra-storage/src/env/faultfs.rs:25` |

#### `enum Fault`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/env/faultfs.rs:11`
```rust
pub enum Fault
```

Varianten:

- `Eio` — `crates/contextra-storage/crates/contextra-storage/src/env/faultfs.rs:12`
- `Enospc` — `crates/contextra-storage/crates/contextra-storage/src/env/faultfs.rs:13`
- `TornWrite{ keep: usize }` — `crates/contextra-storage/crates/contextra-storage/src/env/faultfs.rs:14`
- `LostFsync` — `crates/contextra-storage/crates/contextra-storage/src/env/faultfs.rs:15`
- `RenameReorder` — `crates/contextra-storage/crates/contextra-storage/src/env/faultfs.rs:16`
- `ShortRead` — `crates/contextra-storage/crates/contextra-storage/src/env/faultfs.rs:17`
- `BitFlip{ offset: u64 }` — `crates/contextra-storage/crates/contextra-storage/src/env/faultfs.rs:18`

### Modul `env::governor`

#### `struct Governor`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/env/governor.rs:41`
```rust
pub struct Governor
```
Dokumentation: Lock-free disk pressure state. User commits and ordinary maintenance are rejected while full; reads and urgent erasure remain available.
- Impl-Block `crates/contextra-storage/crates/contextra-storage/src/env/governor.rs:46`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new(config: GovernorConfig) -> Option<Self>` | `crates/contextra-storage/crates/contextra-storage/src/env/governor.rs:47` |  |
| `pub fn observe_free_space(&self, free_bytes: u64) -> bool` | `crates/contextra-storage/crates/contextra-storage/src/env/governor.rs:56` | Update the pressure state from an adapter's free-space observation. Entry is immediate; exit requires reserve plus configured headroom. |
| `pub fn is_disk_full(&self) -> bool` | `crates/contextra-storage/crates/contextra-storage/src/env/governor.rs:77` |  |
| `pub fn admits(&self, work: WorkClass) -> bool` | `crates/contextra-storage/crates/contextra-storage/src/env/governor.rs:83` | Whether work can start now. This is an admission decision only; callers perform any backpressure wait before acquiring a core lock. |
| `pub const fn maintenance_io_basis_points(&self) -> u16` | `crates/contextra-storage/crates/contextra-storage/src/env/governor.rs:92` | Configured cap for the maintenance scheduler, expressed in basis points of the adapter-measured available I/O rate. |

#### `struct GovernorConfig`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/env/governor.rs:23`
```rust
pub struct GovernorConfig
```
Dokumentation: Disk-full hysteresis and maintenance I/O pacing configuration.
- Impl-Block `crates/contextra-storage/crates/contextra-storage/src/env/governor.rs:33`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `reserve_bytes` | `u64` | `crates/contextra-storage/crates/contextra-storage/src/env/governor.rs:25` |
| `recovery_headroom_bytes` | `u64` | `crates/contextra-storage/crates/contextra-storage/src/env/governor.rs:27` |
| `maintenance_io_basis_points` | `u16` | `crates/contextra-storage/crates/contextra-storage/src/env/governor.rs:30` |

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn validate(self) -> bool` | `crates/contextra-storage/crates/contextra-storage/src/env/governor.rs:34` |  |

#### `enum WorkClass`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/env/governor.rs:12`
```rust
pub enum WorkClass
```
Dokumentation: Work classes use the SPEC priority order. The urgent class is reserved for erase gate, key destruction, and purge progress.

Varianten:

- `UserCommit` — `crates/contextra-storage/crates/contextra-storage/src/env/governor.rs:13`
- `Flush` — `crates/contextra-storage/crates/contextra-storage/src/env/governor.rs:14`
- `Compaction` — `crates/contextra-storage/crates/contextra-storage/src/env/governor.rs:15`
- `BlobGc` — `crates/contextra-storage/crates/contextra-storage/src/env/governor.rs:16`
- `UrgentErase` — `crates/contextra-storage/crates/contextra-storage/src/env/governor.rs:17`
- `Read` — `crates/contextra-storage/crates/contextra-storage/src/env/governor.rs:18`

### Modul `env::memfs`

#### `struct MemFs`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/env/memfs.rs:11`
```rust
pub struct MemFs
```
- Impl-Block `crates/contextra-storage/crates/contextra-storage/src/env/memfs.rs:51`
- Impl-Block (Trait `Default`) `crates/contextra-storage/crates/contextra-storage/src/env/memfs.rs:89`
- Impl-Block (Trait `Fs`) `crates/contextra-storage/crates/contextra-storage/src/env/memfs.rs:161`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new() -> Self` | `crates/contextra-storage/crates/contextra-storage/src/env/memfs.rs:52` |  |
| `pub fn crash_image(&self) -> Result<Self, Error>` | `crates/contextra-storage/crates/contextra-storage/src/env/memfs.rs:58` | Erstellt ein neues Abbild wie nach einem Absturz: nicht synchronisierte Bytes und nicht durch `sync_dir` bestätigte Namensänderungen entfallen. |
| `pub fn set_free_space(&self, bytes: u64)` | `crates/contextra-storage/crates/contextra-storage/src/env/memfs.rs:82` |  |

### Modul `erase`

#### `mod cluster_table`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/erase/mod.rs:2`
```rust
pub mod cluster_table
```

#### `mod gate`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/erase/mod.rs:3`
```rust
pub mod gate
```

#### `mod ledger`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/erase/mod.rs:4`
```rust
pub mod ledger
```

#### `mod links`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/erase/mod.rs:5`
```rust
pub mod links
```

#### `mod protocol`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/erase/mod.rs:6`
```rust
pub mod protocol
```

#### `mod receipt`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/erase/mod.rs:7`
```rust
pub mod receipt
```

#### `mod restrictions`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/erase/mod.rs:8`
```rust
pub mod restrictions
```

#### `mod tickets`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/erase/mod.rs:9`
```rust
pub mod tickets
```

### Modul `erase::cluster_table`

#### `struct ClusterEntry`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/erase/cluster_table.rs:10`
```rust
pub struct ClusterEntry
```

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `id` | `ClusterId` | `crates/contextra-storage/crates/contextra-storage/src/erase/cluster_table.rs:11` |
| `protection` | `Protection` | `crates/contextra-storage/crates/contextra-storage/src/erase/cluster_table.rs:12` |
| `state` | `ClusterState` | `crates/contextra-storage/crates/contextra-storage/src/erase/cluster_table.rs:13` |
| `state_stamp` | `Stamp` | `crates/contextra-storage/crates/contextra-storage/src/erase/cluster_table.rs:14` |
| `generation` | `u64` | `crates/contextra-storage/crates/contextra-storage/src/erase/cluster_table.rs:15` |

#### `struct ClusterTable`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/erase/cluster_table.rs:21`
```rust
pub struct ClusterTable
```
Dokumentation: Mutation methods require exclusive ownership so the commit sequencer or recovery fold can publish lifecycle changes atomically with their records.
- Impl-Block `crates/contextra-storage/crates/contextra-storage/src/erase/cluster_table.rs:23`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new() -> Self` | `crates/contextra-storage/crates/contextra-storage/src/erase/cluster_table.rs:24` |  |
| `pub fn register(&mut self, id: ClusterId, protection: Protection) -> Result<&ClusterEntry, Error>` | `crates/contextra-storage/crates/contextra-storage/src/erase/cluster_table.rs:26` |  |
| `pub fn get(&self, id: ClusterId) -> Option<&ClusterEntry>` | `crates/contextra-storage/crates/contextra-storage/src/erase/cluster_table.rs:36` |  |
| `pub fn len(&self) -> usize` | `crates/contextra-storage/crates/contextra-storage/src/erase/cluster_table.rs:37` |  |
| `pub fn is_empty(&self) -> bool` | `crates/contextra-storage/crates/contextra-storage/src/erase/cluster_table.rs:38` |  |
| `pub fn iter(&self) -> impl Iterator<Item = &ClusterEntry>` | `crates/contextra-storage/crates/contextra-storage/src/erase/cluster_table.rs:39` |  |
| `pub fn mark_retired(&mut self, id: ClusterId, stamp: Stamp) -> Result<(), Error>` | `crates/contextra-storage/crates/contextra-storage/src/erase/cluster_table.rs:41` |  |
| `pub fn mark_shredded(&mut self, id: ClusterId) -> Result<(), Error>` | `crates/contextra-storage/crates/contextra-storage/src/erase/cluster_table.rs:50` |  |
| `pub fn mark_purged(&mut self, id: ClusterId) -> Result<(), Error>` | `crates/contextra-storage/crates/contextra-storage/src/erase/cluster_table.rs:59` |  |
| `pub fn require_live(&self, id: ClusterId) -> Result<&ClusterEntry, Error>` | `crates/contextra-storage/crates/contextra-storage/src/erase/cluster_table.rs:68` |  |

#### `enum ClusterState`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/erase/cluster_table.rs:7`
```rust
pub enum ClusterState
```

### Modul `erase::gate`

#### `struct GateIndex`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/erase/gate.rs:14`
```rust
pub struct GateIndex
```
- Impl-Block `crates/contextra-storage/crates/contextra-storage/src/erase/gate.rs:19`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new() -> Self` | `crates/contextra-storage/crates/contextra-storage/src/erase/gate.rs:20` |  |
| `pub fn generation(&self) -> u64` | `crates/contextra-storage/crates/contextra-storage/src/erase/gate.rs:21` |  |
| `pub fn erased_at(&self, cluster: ClusterId) -> Option<Stamp>` | `crates/contextra-storage/crates/contextra-storage/src/erase/gate.rs:22` |  |
| `pub fn len(&self) -> usize` | `crates/contextra-storage/crates/contextra-storage/src/erase/gate.rs:23` |  |
| `pub fn is_empty(&self) -> bool` | `crates/contextra-storage/crates/contextra-storage/src/erase/gate.rs:24` |  |
| `pub fn check_visible(&self, cluster: ClusterId, _view_stamp: Stamp) -> Result<(), Error>` | `crates/contextra-storage/crates/contextra-storage/src/erase/gate.rs:28` | Gate semantics override historical snapshots: once present, a cluster is never visible through any view, including a view below its gate stamp. |
| `pub fn apply_cluster_gate( &mut self, clusters: &mut ClusterTable, links: &LinkGraph, restrictions: &BTreeMap<ClusterId, RestrictionTable>, members: &[ClusterId], stamp: Stamp, monotone_now: u64, max_gate_clusters: usize, ) -> Result<(), Error>` | `crates/contextra-storage/crates/contextra-storage/src/erase/gate.rs:38` | Applies one durable gate frame atomically to both the lifecycle table and this read filter. The scope is canonicalized and bounded before any mutation; restrict |

### Modul `erase::ledger`

#### `struct LedgerEvent`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/erase/ledger.rs:39`
```rust
pub struct LedgerEvent
```

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `kind` | `LedgerEventKind` | `crates/contextra-storage/crates/contextra-storage/src/erase/ledger.rs:40` |
| `tomb_epoch` | `u32` | `crates/contextra-storage/crates/contextra-storage/src/erase/ledger.rs:41` |
| `ticket` | `Ticket` | `crates/contextra-storage/crates/contextra-storage/src/erase/ledger.rs:42` |
| `tombs` | `Vec<TombId>` | `crates/contextra-storage/crates/contextra-storage/src/erase/ledger.rs:44` |
| `evidence_digest` | `[u8; 32]` | `crates/contextra-storage/crates/contextra-storage/src/erase/ledger.rs:45` |
| `scope_ciphertext` | `Vec<u8>` | `crates/contextra-storage/crates/contextra-storage/src/erase/ledger.rs:47` |

#### `struct LedgerRecord`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/erase/ledger.rs:51`
```rust
pub struct LedgerRecord
```

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `sequence` | `u64` | `crates/contextra-storage/crates/contextra-storage/src/erase/ledger.rs:52` |
| `previous_hash` | `[u8; 32]` | `crates/contextra-storage/crates/contextra-storage/src/erase/ledger.rs:53` |
| `clock_sample` | `u64` | `crates/contextra-storage/crates/contextra-storage/src/erase/ledger.rs:54` |
| `event` | `LedgerEvent` | `crates/contextra-storage/crates/contextra-storage/src/erase/ledger.rs:55` |
| `mac` | `[u8; 32]` | `crates/contextra-storage/crates/contextra-storage/src/erase/ledger.rs:56` |

#### `struct LedgerWriter`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/erase/ledger.rs:61`
```rust
pub struct LedgerWriter<'a>
```
Dokumentation: Single-owner durable ledger appender. It has no internal lock: ownership is held by the urgent ledger lane, as required by SPEC §3.4 and §9.7.
- Impl-Block `crates/contextra-storage/crates/contextra-storage/src/erase/ledger.rs:70`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn recover(file: &'a dyn File, key: &'a Key32, mac_impl: &'a dyn Mac) -> Result<(Self, Vec<LedgerRecord>), Error>` | `crates/contextra-storage/crates/contextra-storage/src/erase/ledger.rs:71` |  |
| `pub fn append(&mut self, clock_sample: u64, event: LedgerEvent) -> Result<LedgerRecord, Error>` | `crates/contextra-storage/crates/contextra-storage/src/erase/ledger.rs:93` |  |
| `pub fn head(&self) -> [u8` | `crates/contextra-storage/crates/contextra-storage/src/erase/ledger.rs:106` |  |
| `pub fn next_sequence(&self) -> u64` | `crates/contextra-storage/crates/contextra-storage/src/erase/ledger.rs:107` |  |
| `pub fn is_poisoned(&self) -> bool` | `crates/contextra-storage/crates/contextra-storage/src/erase/ledger.rs:108` |  |

#### `enum LedgerEventKind`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/erase/ledger.rs:17`
```rust
pub enum LedgerEventKind
```
- Impl-Block (Trait `TryFrom<u8>`) `crates/contextra-storage/crates/contextra-storage/src/erase/ledger.rs:24`

Varianten:

- `EraseIntent= 1, GateApplied, KeyDestroyIntent, KeysDestroyed, PurgeProgress` — `crates/contextra-storage/crates/contextra-storage/src/erase/ledger.rs:18`
- `UnlinkProgress, ExtAck, PurgeVerified, Completed, RestrictionSet, RestrictionLifted` — `crates/contextra-storage/crates/contextra-storage/src/erase/ledger.rs:19`
- `BranchEraseDone, SpaceDropIntent, SpaceScopeFrozen, SpaceDropped, RestoreReconciled` — `crates/contextra-storage/crates/contextra-storage/src/erase/ledger.rs:20`
- `ReceiptIssued, TombEpochOpened, TombEpochClosed, TombEpochRetireIntent, TombEpochRetired` — `crates/contextra-storage/crates/contextra-storage/src/erase/ledger.rs:21`

#### `fn decode_record`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/erase/ledger.rs:121`
```rust
pub fn decode_record(input: &[u8], expected_sequence: u64, previous_hash: [u8
```

#### `fn encode_record`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/erase/ledger.rs:111`
```rust
pub fn encode_record(record: &LedgerRecord, key: &Key32, mac: &dyn Mac) -> Result<Vec<u8>, Error>
```

#### `const MAX_INTENT_TOMBS`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/erase/ledger.rs:10`
```rust
pub const MAX_INTENT_TOMBS: usize = 4096
```

#### `const MAX_LEDGER_BYTES`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/erase/ledger.rs:11`
```rust
pub const MAX_LEDGER_BYTES: u64 = 64 * 1024 * 1024
```

#### `const MAX_LEDGER_RECORD_BYTES`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/erase/ledger.rs:9`
```rust
pub const MAX_LEDGER_RECORD_BYTES: usize = 256 * 1024
```

### Modul `erase::links`

#### `struct LinkGraph`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/erase/links.rs:30`
```rust
pub struct LinkGraph
```
- Impl-Block (Trait `Default`) `crates/contextra-storage/crates/contextra-storage/src/erase/links.rs:39`
- Impl-Block `crates/contextra-storage/crates/contextra-storage/src/erase/links.rs:50`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new(max_links: usize, max_gate_clusters: usize, max_depth: usize) -> Result<Self, Error>` | `crates/contextra-storage/crates/contextra-storage/src/erase/links.rs:51` |  |
| `pub fn generation(&self) -> u64` | `crates/contextra-storage/crates/contextra-storage/src/erase/links.rs:58` |  |
| `pub fn direct_sources(&self, derived: ClusterId) -> impl Iterator<Item = ClusterId> + '_` | `crates/contextra-storage/crates/contextra-storage/src/erase/links.rs:59` |  |
| `pub fn plan_link(&self, clusters: &ClusterTable, derived: ClusterId, sources: &[ClusterId]) -> Result<LinkPlan, Error>` | `crates/contextra-storage/crates/contextra-storage/src/erase/links.rs:65` | Computes, but does not publish, a link operation. The candidate graph is validated for cycles and depth before a plan can reach the writer. |
| `pub fn apply_link(&mut self, clusters: &ClusterTable, plan: LinkPlan) -> Result<(), Error>` | `crates/contextra-storage/crates/contextra-storage/src/erase/links.rs:93` | Publishes a previously planned link iff no intervening graph mutation invalidated its cycle/fanout proof. |
| `pub fn erase_closure(&self, source: ClusterId) -> Result<Vec<ClusterId>, Error>` | `crates/contextra-storage/crates/contextra-storage/src/erase/links.rs:118` | Returns source plus every transitive derived cluster. A child with multiple sources is included when any one source is in the closure. |
| `pub fn unlink(&mut self, expected_generation: u64, derived: ClusterId) -> Result<(), Error>` | `crates/contextra-storage/crates/contextra-storage/src/erase/links.rs:126` | Unlink is a durable control mutation; this method only applies its replayed/in-memory projection and rejects stale graph generations. |

#### `struct LinkPlan`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/erase/links.rs:17`
```rust
pub struct LinkPlan
```
- Impl-Block `crates/contextra-storage/crates/contextra-storage/src/erase/links.rs:23`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn generation(&self) -> u64` | `crates/contextra-storage/crates/contextra-storage/src/erase/links.rs:24` |  |
| `pub fn derived(&self) -> ClusterId` | `crates/contextra-storage/crates/contextra-storage/src/erase/links.rs:25` |  |
| `pub fn sources(&self) -> &[ClusterId]` | `crates/contextra-storage/crates/contextra-storage/src/erase/links.rs:26` |  |

#### `const DEFAULT_MAX_GATE_CLUSTERS`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/erase/links.rs:13`
```rust
pub const DEFAULT_MAX_GATE_CLUSTERS: usize = 4096
```

#### `const DEFAULT_MAX_LINKS`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/erase/links.rs:12`
```rust
pub const DEFAULT_MAX_LINKS: usize = 8
```

#### `const DEFAULT_MAX_LINK_DEPTH`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/erase/links.rs:14`
```rust
pub const DEFAULT_MAX_LINK_DEPTH: usize = 64
```

### Modul `erase::restrictions`

#### `struct AdminCap`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/erase/restrictions.rs:76`
```rust
pub struct AdminCap
```
Dokumentation: Non-cloneable capability minted by the embedding authorization layer.
- Impl-Block `crates/contextra-storage/crates/contextra-storage/src/erase/restrictions.rs:78`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn from_authorized_context() -> Self` | `crates/contextra-storage/crates/contextra-storage/src/erase/restrictions.rs:81` | Construct only after the caller has authenticated its administrative principal. The storage core does not implement identity policy. |

#### `struct Effects`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/erase/restrictions.rs:12`
```rust
pub struct Effects(u8)
```
- Impl-Block `crates/contextra-storage/crates/contextra-storage/src/erase/restrictions.rs:14`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new(bits: u8) -> Result<Self, Error>` | `crates/contextra-storage/crates/contextra-storage/src/erase/restrictions.rs:20` |  |
| `pub fn blocks_erase(self) -> bool` | `crates/contextra-storage/crates/contextra-storage/src/erase/restrictions.rs:27` |  |
| `pub fn blocks_write(self) -> bool` | `crates/contextra-storage/crates/contextra-storage/src/erase/restrictions.rs:28` |  |
| `pub fn blocks_read(self) -> bool` | `crates/contextra-storage/crates/contextra-storage/src/erase/restrictions.rs:29` |  |
| `pub fn bits(self) -> u8` | `crates/contextra-storage/crates/contextra-storage/src/erase/restrictions.rs:30` |  |

Assoziierte Konstanten/Typen:

- `pub const BLOCK_ERASE: u8 = 0b001` — `crates/contextra-storage/crates/contextra-storage/src/erase/restrictions.rs:15`
- `pub const BLOCK_WRITE: u8 = 0b010` — `crates/contextra-storage/crates/contextra-storage/src/erase/restrictions.rs:16`
- `pub const BLOCK_READ: u8 = 0b100` — `crates/contextra-storage/crates/contextra-storage/src/erase/restrictions.rs:17`

#### `struct Restriction`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/erase/restrictions.rs:44`
```rust
pub struct Restriction
```
- Impl-Block `crates/contextra-storage/crates/contextra-storage/src/erase/restrictions.rs:54`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn id(&self) -> u64` | `crates/contextra-storage/crates/contextra-storage/src/erase/restrictions.rs:55` |  |
| `pub fn reason(&self) -> u16` | `crates/contextra-storage/crates/contextra-storage/src/erase/restrictions.rs:56` |  |
| `pub fn effects(&self) -> Effects` | `crates/contextra-storage/crates/contextra-storage/src/erase/restrictions.rs:57` |  |
| `pub fn set_stamp(&self) -> Stamp` | `crates/contextra-storage/crates/contextra-storage/src/erase/restrictions.rs:58` |  |
| `pub fn lifted_stamp(&self) -> Option<Stamp>` | `crates/contextra-storage/crates/contextra-storage/src/erase/restrictions.rs:59` |  |
| `pub fn is_active_at(&self, monotone_now: u64) -> bool` | `crates/contextra-storage/crates/contextra-storage/src/erase/restrictions.rs:60` |  |

#### `struct RestrictionSpec`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/erase/restrictions.rs:36`
```rust
pub struct RestrictionSpec
```

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `reason` | `u16` | `crates/contextra-storage/crates/contextra-storage/src/erase/restrictions.rs:37` |
| `owner` | `Principal` | `crates/contextra-storage/crates/contextra-storage/src/erase/restrictions.rs:38` |
| `effects` | `Effects` | `crates/contextra-storage/crates/contextra-storage/src/erase/restrictions.rs:39` |
| `expires_at` | `Option<u64>` | `crates/contextra-storage/crates/contextra-storage/src/erase/restrictions.rs:41` |

#### `struct RestrictionTable`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/erase/restrictions.rs:84`
```rust
pub struct RestrictionTable
```
- Impl-Block `crates/contextra-storage/crates/contextra-storage/src/erase/restrictions.rs:91`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new(policy: LiftPolicy) -> Self` | `crates/contextra-storage/crates/contextra-storage/src/erase/restrictions.rs:92` |  |
| `pub fn get(&self, id: u64) -> Option<&Restriction>` | `crates/contextra-storage/crates/contextra-storage/src/erase/restrictions.rs:96` |  |
| `pub fn iter(&self) -> impl Iterator<Item = &Restriction>` | `crates/contextra-storage/crates/contextra-storage/src/erase/restrictions.rs:97` |  |
| `pub fn active_at(&self, now: u64) -> impl Iterator<Item = &Restriction>` | `crates/contextra-storage/crates/contextra-storage/src/erase/restrictions.rs:98` |  |
| `pub fn ensure_erase_allowed(&self, now: u64) -> Result<(), Error>` | `crates/contextra-storage/crates/contextra-storage/src/erase/restrictions.rs:102` |  |
| `pub fn ensure_write_allowed(&self, now: u64) -> Result<(), Error>` | `crates/contextra-storage/crates/contextra-storage/src/erase/restrictions.rs:109` |  |
| `pub fn ensure_read_allowed(&self, now: u64) -> Result<(), Error>` | `crates/contextra-storage/crates/contextra-storage/src/erase/restrictions.rs:116` |  |
| `pub fn apply_set(&mut self, id: u64, stamp: Stamp, spec: RestrictionSpec) -> Result<(), Error>` | `crates/contextra-storage/crates/contextra-storage/src/erase/restrictions.rs:124` | Applies a previously validated, durable RestrictionSet control record. |
| `pub fn validate_lift( &self, id: u64, stamp: Stamp, authorization: LiftAuthorization<'_>, authority: Option<&dyn Authority>, ) -> Result<(), Error>` | `crates/contextra-storage/crates/contextra-storage/src/erase/restrictions.rs:141` | Applies a durable RestrictionLift. Authorization is checked before the caller writes the frame and must be revalidated by its commit sequencer. |
| `pub fn apply_lift(&mut self, id: u64, stamp: Stamp) -> Result<(), Error>` | `crates/contextra-storage/crates/contextra-storage/src/erase/restrictions.rs:163` |  |

#### `enum LiftAuthorization`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/erase/restrictions.rs:65`
```rust
pub enum LiftAuthorization<'a>
```

Varianten:

- `Owner(&'a Principal)` — `crates/contextra-storage/crates/contextra-storage/src/erase/restrictions.rs:66`
- `Admin(&'a AdminCap)` — `crates/contextra-storage/crates/contextra-storage/src/erase/restrictions.rs:67`
- `FourEyes{` — `crates/contextra-storage/crates/contextra-storage/src/erase/restrictions.rs:68`

#### `enum LiftPolicy`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/erase/restrictions.rs:34`
```rust
pub enum LiftPolicy
```

### Modul `erase::tickets`

#### `struct CompletionEvidence`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/erase/tickets.rs:46`
```rust
pub struct CompletionEvidence
```

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `anchor_advanced` | `bool` | `crates/contextra-storage/crates/contextra-storage/src/erase/tickets.rs:47` |
| `verified` | `bool` | `crates/contextra-storage/crates/contextra-storage/src/erase/tickets.rs:48` |
| `owners_attested` | `bool` | `crates/contextra-storage/crates/contextra-storage/src/erase/tickets.rs:49` |
| `graveyard_empty` | `bool` | `crates/contextra-storage/crates/contextra-storage/src/erase/tickets.rs:50` |

#### `struct EraseTicket`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/erase/tickets.rs:71`
```rust
pub struct EraseTicket
```
- Impl-Block `crates/contextra-storage/crates/contextra-storage/src/erase/tickets.rs:83`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new(ticket: Ticket, scope: EraseScope, protection: Protection, level: EraseLevel) -> Result<Self, Error>` | `crates/contextra-storage/crates/contextra-storage/src/erase/tickets.rs:84` |  |
| `pub fn ticket(&self) -> Ticket` | `crates/contextra-storage/crates/contextra-storage/src/erase/tickets.rs:92` |  |
| `pub fn scope(&self) -> EraseScope` | `crates/contextra-storage/crates/contextra-storage/src/erase/tickets.rs:93` |  |
| `pub fn level(&self) -> EraseLevel` | `crates/contextra-storage/crates/contextra-storage/src/erase/tickets.rs:94` |  |
| `pub fn stage(&self) -> TicketStage` | `crates/contextra-storage/crates/contextra-storage/src/erase/tickets.rs:95` |  |
| `pub fn status(&self) -> TicketStatus` | `crates/contextra-storage/crates/contextra-storage/src/erase/tickets.rs:96` |  |
| `pub fn gate_stamp(&self) -> Option<Stamp>` | `crates/contextra-storage/crates/contextra-storage/src/erase/tickets.rs:97` |  |
| `pub fn last_sequence(&self) -> u64` | `crates/contextra-storage/crates/contextra-storage/src/erase/tickets.rs:98` |  |
| `pub fn escalate(&mut self, level: EraseLevel) -> Result<(), Error>` | `crates/contextra-storage/crates/contextra-storage/src/erase/tickets.rs:101` | Raises a non-terminal target level before it reaches `Done`. |
| `pub fn apply(&mut self, seq: u64, event: TicketEvidence) -> Result<(), Error>` | `crates/contextra-storage/crates/contextra-storage/src/erase/tickets.rs:112` | Folds a single durable ledger event. Replaying the same sequence/event is idempotent; a different event at that sequence signals corruption. |

#### `enum EraseLevel`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/erase/tickets.rs:10`
```rust
pub enum EraseLevel
```

#### `enum EraseScope`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/erase/tickets.rs:13`
```rust
pub enum EraseScope
```

#### `enum Residual`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/erase/tickets.rs:28`
```rust
pub enum Residual
```

Varianten:

- `LogicalOnly` — `crates/contextra-storage/crates/contextra-storage/src/erase/tickets.rs:29`
- `BoundedBackupKeys{ until_utc: u64 }` — `crates/contextra-storage/crates/contextra-storage/src/erase/tickets.rs:30`
- `WeakKeyAssurance` — `crates/contextra-storage/crates/contextra-storage/src/erase/tickets.rs:31`
- `DeclaredHostOrDeviceExposure` — `crates/contextra-storage/crates/contextra-storage/src/erase/tickets.rs:32`

#### `enum TicketEvidence`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/erase/tickets.rs:56`
```rust
pub enum TicketEvidence
```
Dokumentation: Events have already been made durable. `seq` is the ledger sequence and gives replay an unambiguous, monotone ordering.

Varianten:

- `IntentRecorded` — `crates/contextra-storage/crates/contextra-storage/src/erase/tickets.rs:57`
- `GateApplied{ stamp: Stamp }` — `crates/contextra-storage/crates/contextra-storage/src/erase/tickets.rs:58`
- `KeyDestroyIntent` — `crates/contextra-storage/crates/contextra-storage/src/erase/tickets.rs:59`
- `KeysDestroyed` — `crates/contextra-storage/crates/contextra-storage/src/erase/tickets.rs:60`
- `PurgeStarted` — `crates/contextra-storage/crates/contextra-storage/src/erase/tickets.rs:61`
- `Purged` — `crates/contextra-storage/crates/contextra-storage/src/erase/tickets.rs:62`
- `Completed{ evidence: CompletionEvidence, residual: Option<Residual> }` — `crates/contextra-storage/crates/contextra-storage/src/erase/tickets.rs:64`
- `Blocked` — `crates/contextra-storage/crates/contextra-storage/src/erase/tickets.rs:65`
- `Unblocked` — `crates/contextra-storage/crates/contextra-storage/src/erase/tickets.rs:66`
- `Incomplete` — `crates/contextra-storage/crates/contextra-storage/src/erase/tickets.rs:67`
- `Failed` — `crates/contextra-storage/crates/contextra-storage/src/erase/tickets.rs:68`

#### `enum TicketStage`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/erase/tickets.rs:16`
```rust
pub enum TicketStage
```

Varianten:

- `IntentRecorded` — `crates/contextra-storage/crates/contextra-storage/src/erase/tickets.rs:18`
- `Gated` — `crates/contextra-storage/crates/contextra-storage/src/erase/tickets.rs:19`
- `KeyDestroying` — `crates/contextra-storage/crates/contextra-storage/src/erase/tickets.rs:20`
- `KeysDestroyed` — `crates/contextra-storage/crates/contextra-storage/src/erase/tickets.rs:21`
- `Purging` — `crates/contextra-storage/crates/contextra-storage/src/erase/tickets.rs:22`
- `Purged` — `crates/contextra-storage/crates/contextra-storage/src/erase/tickets.rs:23`
- `Done` — `crates/contextra-storage/crates/contextra-storage/src/erase/tickets.rs:24`

#### `enum TicketStatus`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/erase/tickets.rs:36`
```rust
pub enum TicketStatus
```

Varianten:

- `Pending` — `crates/contextra-storage/crates/contextra-storage/src/erase/tickets.rs:37`
- `Blocked` — `crates/contextra-storage/crates/contextra-storage/src/erase/tickets.rs:38`
- `Incomplete` — `crates/contextra-storage/crates/contextra-storage/src/erase/tickets.rs:39`
- `Complete` — `crates/contextra-storage/crates/contextra-storage/src/erase/tickets.rs:40`
- `CompleteWithResidual(Residual)` — `crates/contextra-storage/crates/contextra-storage/src/erase/tickets.rs:41`
- `Failed` — `crates/contextra-storage/crates/contextra-storage/src/erase/tickets.rs:42`

### Modul `keys`

#### `mod cache`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/keys/mod.rs:2`
```rust
pub mod cache
```

#### `mod wrapper`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/keys/mod.rs:3`
```rust
pub mod wrapper
```

### Modul `keys::cache`

#### `struct Key32Handle`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/keys/cache.rs:37`
```rust
pub struct Key32Handle
```
- Impl-Block `crates/contextra-storage/crates/contextra-storage/src/keys/cache.rs:43`
- Impl-Block (Trait `fmt::Debug`) `crates/contextra-storage/crates/contextra-storage/src/keys/cache.rs:59`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn with_key<T>(&self, use_key: impl FnOnce(&Key32) -> Result<T, Error>) -> Result<T, Error>` | `crates/contextra-storage/crates/contextra-storage/src/keys/cache.rs:47` | The closure cannot keep a reference to key material beyond this call. A concurrent invalidation fences the result even if the operation was already using its bo |

#### `struct KeyCache`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/keys/cache.rs:28`
```rust
pub struct KeyCache
```
- Impl-Block `crates/contextra-storage/crates/contextra-storage/src/keys/cache.rs:63`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new( provider: Arc<dyn KeyProvider>, clock: Arc<dyn Clock>, ttl_nanos: u64, grace_nanos: u64, max_clusters: usize, ) -> Result<Self, Error>` | `crates/contextra-storage/crates/contextra-storage/src/keys/cache.rs:64` |  |
| `pub fn get(&self, cluster: ClusterId, epoch: u32) -> Result<Key32Handle, Error>` | `crates/contextra-storage/crates/contextra-storage/src/keys/cache.rs:78` |  |
| `pub fn prefetch(&self, request: &[(ClusterId, u32)]) -> Result<(), Error>` | `crates/contextra-storage/crates/contextra-storage/src/keys/cache.rs:133` | Fetches bounded groups of misses without holding the cache lock across provider calls. |
| `pub fn invalidate(&self, cluster: ClusterId) -> Result<u64, Error>` | `crates/contextra-storage/crates/contextra-storage/src/keys/cache.rs:174` |  |

### Modul `keys::wrapper`

#### `fn unwrap_root`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/keys/wrapper.rs:22`
```rust
pub fn unwrap_root(cipher: &dyn Cipher, kek: &Key32, wrapped: &[u8]) -> Result<Key32, Error>
```

#### `fn unwrap_space_dek`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/keys/wrapper.rs:38`
```rust
pub fn unwrap_space_dek( cipher: &dyn Cipher, root: &Key32, wrapped: &[u8], space: SpaceId, ) -> Result<Key32, Error>
```

#### `fn wrap_root`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/keys/wrapper.rs:13`
```rust
pub fn wrap_root( cipher: &dyn Cipher, kek: &Key32, root: &Key32, entropy: &dyn Entropy, ) -> Result<Vec<u8>, Error>
```

#### `fn wrap_space_dek`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/keys/wrapper.rs:26`
```rust
pub fn wrap_space_dek( cipher: &dyn Cipher, root: &Key32, dek: &Key32, entropy: &dyn Entropy, space: SpaceId, ) -> Result<Vec<u8>, Error>
```

### Modul `maint`

#### `mod blob_gc`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/maint/mod.rs:2`
```rust
pub mod blob_gc
```

#### `mod compaction`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/maint/mod.rs:3`
```rust
pub mod compaction
```

#### `mod flush`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/maint/mod.rs:4`
```rust
pub mod flush
```

#### `mod purge`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/maint/mod.rs:5`
```rust
pub mod purge
```

#### `mod reserve`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/maint/mod.rs:6`
```rust
pub mod reserve
```

#### `mod scheduler`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/maint/mod.rs:7`
```rust
pub mod scheduler
```

#### `mod scrub`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/maint/mod.rs:8`
```rust
pub mod scrub
```

### Modul `maint::reserve`

#### `struct ScratchReserve`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/maint/reserve.rs:15`
```rust
pub struct ScratchReserve
```
Dokumentation: A crash-recovery slot is held aside; ordinary writers can lease only slot 0. Both backing files are durably preallocated when this reserve opens.
- Impl-Block `crates/contextra-storage/crates/contextra-storage/src/maint/reserve.rs:21`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn open(fs: &dyn Fs, root: &Path, slot_bytes: u64) -> Result<Self, Error>` | `crates/contextra-storage/crates/contextra-storage/src/maint/reserve.rs:22` |  |
| `pub fn slot_bytes(&self) -> u64` | `crates/contextra-storage/crates/contextra-storage/src/maint/reserve.rs:68` |  |
| `pub fn writer_slot(&self) -> Result<ScratchSlot, Error>` | `crates/contextra-storage/crates/contextra-storage/src/maint/reserve.rs:70` |  |
| `pub fn recovery_path(&self) -> PathBuf` | `crates/contextra-storage/crates/contextra-storage/src/maint/reserve.rs:78` | Reserved for recovery from a killed writer; not available to concurrent flush/compaction or another purge operation. |

#### `struct ScratchSlot`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/maint/reserve.rs:81`
```rust
pub struct ScratchSlot
```
- Impl-Block `crates/contextra-storage/crates/contextra-storage/src/maint/reserve.rs:86`
- Impl-Block (Trait `Drop`) `crates/contextra-storage/crates/contextra-storage/src/maint/reserve.rs:90`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn path(&self) -> &Path` | `crates/contextra-storage/crates/contextra-storage/src/maint/reserve.rs:87` |  |

#### `const REQUIRED_PURGE_RESERVE_BYTES`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/maint/reserve.rs:9`
```rust
pub const REQUIRED_PURGE_RESERVE_BYTES: u64 = 128 * 1024 * 1024
```

### Modul `metrics`

#### `struct MetricDef`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/metrics.rs:22`
```rust
pub struct MetricDef
```

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `name` | `MetricName` | `crates/contextra-storage/crates/contextra-storage/src/metrics.rs:23` |
| `kind` | `MetricKind` | `crates/contextra-storage/crates/contextra-storage/src/metrics.rs:24` |
| `unit` | `MetricUnit` | `crates/contextra-storage/crates/contextra-storage/src/metrics.rs:25` |
| `labels` | `&'static [MetricLabelKey]` | `crates/contextra-storage/crates/contextra-storage/src/metrics.rs:26` |

#### `enum MetricKind`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/metrics.rs:6`
```rust
pub enum MetricKind
```

Varianten:

- `Counter` — `crates/contextra-storage/crates/contextra-storage/src/metrics.rs:7`
- `Gauge` — `crates/contextra-storage/crates/contextra-storage/src/metrics.rs:8`
- `Histogram` — `crates/contextra-storage/crates/contextra-storage/src/metrics.rs:9`

#### `enum MetricUnit`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/metrics.rs:13`
```rust
pub enum MetricUnit
```

Varianten:

- `Count` — `crates/contextra-storage/crates/contextra-storage/src/metrics.rs:14`
- `Bytes` — `crates/contextra-storage/crates/contextra-storage/src/metrics.rs:15`
- `Nanoseconds` — `crates/contextra-storage/crates/contextra-storage/src/metrics.rs:16`
- `RatioBasisPoints` — `crates/contextra-storage/crates/contextra-storage/src/metrics.rs:17`
- `Stamp` — `crates/contextra-storage/crates/contextra-storage/src/metrics.rs:18`

#### `fn validate_labels`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/metrics.rs:85`
```rust
pub fn validate_labels(name: MetricName, labels: &[MetricLabel]) -> bool
```
Dokumentation: Validates that a call uses the exact catalog label dimensions and no duplicates. Label values are already restricted by their Rust enums.

#### `fn validate_sample`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/metrics.rs:97`
```rust
pub fn validate_sample(name: MetricName, sample: MetricSample) -> bool
```
Dokumentation: Checks sample representation against the catalog's aggregation kind.

#### `const METRICS`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/metrics.rs:46`
```rust
pub const METRICS: &[MetricDef] = &[ MetricDef
```

### Modul `recovery`

#### `mod integrity`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/recovery/mod.rs:2`
```rust
pub mod integrity
```

#### `mod replay`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/recovery/mod.rs:3`
```rust
pub mod replay
```

#### `mod scan`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/recovery/mod.rs:4`
```rust
pub mod scan
```

### Modul `recovery::integrity`

#### `struct IntegrityDecision`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/recovery/integrity.rs:21`
```rust
pub struct IntegrityDecision
```

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `writable` | `bool` | `crates/contextra-storage/crates/contextra-storage/src/recovery/integrity.rs:22` |
| `rollback_detected` | `bool` | `crates/contextra-storage/crates/contextra-storage/src/recovery/integrity.rs:23` |

#### `enum KeyRecoveryState`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/recovery/integrity.rs:6`
```rust
pub enum KeyRecoveryState
```

#### `fn check_anchor`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/recovery/integrity.rs:28`
```rust
pub fn check_anchor( anchor_catalog_epoch: u64, catalog_epoch: u64, anchor_ledger_epoch: u64, ledger_epoch: u64, read_only_requested: bool, ) -> Result<IntegrityDecision, Error>
```
Dokumentation: A monotone anchor ahead of either durable metadata source indicates a rollback. The caller may expose diagnostics read-only, but cannot write.

#### `fn reconcile_cluster_key`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/recovery/integrity.rs:11`
```rust
pub fn reconcile_cluster_key(probe: KeyState, erase_proven: bool) -> Result<KeyRecoveryState, Error>
```
Dokumentation: A provider error must be propagated as `KeyUnavailable` by its adapter. Absence alone is never evidence of deletion: only a durable erase record permits recovery to classify it as Shredded.

### Modul `recovery::replay`

#### `struct EngineReplaySink`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/recovery/replay.rs:30`
```rust
pub struct EngineReplaySink
```
Dokumentation: Concrete replay bridge into the same memtable/version publication path used by commits. Control and Gate state remain owned by the erase/cluster adapter.
- Impl-Block `crates/contextra-storage/crates/contextra-storage/src/recovery/replay.rs:37`
- Impl-Block (Trait `ReplaySink`) `crates/contextra-storage/crates/contextra-storage/src/recovery/replay.rs:53`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new( memtable: Arc<Memtable>, versions: Arc<VersionRegistry>, decoder: Arc<dyn CommitMutationDecoder>, controls: Arc<dyn ControlReplayApplier>, ) -> Self` | `crates/contextra-storage/crates/contextra-storage/src/recovery/replay.rs:38` |  |

#### `struct ReplaySummary`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/recovery/replay.rs:88`
```rust
pub struct ReplaySummary
```

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `applied_frames` | `u64` | `crates/contextra-storage/crates/contextra-storage/src/recovery/replay.rs:89` |
| `skipped_retired_ops` | `u64` | `crates/contextra-storage/crates/contextra-storage/src/recovery/replay.rs:90` |
| `last_stamp` | `Stamp` | `crates/contextra-storage/crates/contextra-storage/src/recovery/replay.rs:91` |

#### `trait ControlReplayApplier`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/recovery/replay.rs:24`
```rust
pub trait ControlReplayApplier: Send + Sync
```

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `fn apply_control(&self, frame: &DecodedFrame<'_>) -> Result<(), Error>` | `crates/contextra-storage/crates/contextra-storage/src/recovery/replay.rs:25` |  |

#### `trait ReplaySink`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/recovery/replay.rs:15`
```rust
pub trait ReplaySink
```
Dokumentation: Engine adapter for recovery. Implementations decrypt and apply Change payloads; this layer controls ordering and suppresses retired-cluster data.

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `fn apply_change(&mut self, frame: &DecodedFrame<'_>, live_ops: &[usize]) -> Result<(), Error>` | `crates/contextra-storage/crates/contextra-storage/src/recovery/replay.rs:19` | Apply all live ops from this committed frame as one recovery unit. Retired ops are omitted by the visibility barrier, while live ops retain the original frame's |
| `fn apply_control(&mut self, frame: &DecodedFrame<'_>) -> Result<(), Error>` | `crates/contextra-storage/crates/contextra-storage/src/recovery/replay.rs:20` |  |
| `fn finish(&mut self, last_stamp: Stamp) -> Result<(), Error>` | `crates/contextra-storage/crates/contextra-storage/src/recovery/replay.rs:21` |  |

#### `fn replay`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/recovery/replay.rs:97`
```rust
pub fn replay(scan: &RecoveryScan<'_>, sink: &mut dyn ReplaySink) -> Result<ReplaySummary, Error>
```
Dokumentation: Replays only after the complete scan has established the final retired set. A mixed frame is passed intact to the sink only when all its ops are live; sinks must implement atomic per-frame application for partial retirement.

#### `fn replay_with_retired`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/recovery/replay.rs:101`
```rust
pub fn replay_with_retired( scan: &RecoveryScan<'_>, retired: &BTreeSet<ClusterId>, sink: &mut dyn ReplaySink, ) -> Result<ReplaySummary, Error>
```

### Modul `recovery::scan`

#### `struct RecoveryScan`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/recovery/scan.rs:10`
```rust
pub struct RecoveryScan<'a>
```

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `frames` | `Vec<DecodedFrame<'a>>` | `crates/contextra-storage/crates/contextra-storage/src/recovery/scan.rs:11` |
| `retired_clusters` | `BTreeSet<ClusterId>` | `crates/contextra-storage/crates/contextra-storage/src/recovery/scan.rs:12` |
| `tail` | `TailState` | `crates/contextra-storage/crates/contextra-storage/src/recovery/scan.rs:13` |
| `valid_bytes` | `usize` | `crates/contextra-storage/crates/contextra-storage/src/recovery/scan.rs:14` |
| `last_stamp` | `Stamp` | `crates/contextra-storage/crates/contextra-storage/src/recovery/scan.rs:15` |
| `last_mac` | `[u8; 32]` | `crates/contextra-storage/crates/contextra-storage/src/recovery/scan.rs:16` |

#### `trait GateScopeResolver`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/recovery/scan.rs:19`
```rust
pub trait GateScopeResolver
```

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `fn clusters_in_space(&self, space: SpaceId) -> Result<Vec<ClusterId>, Error>` | `crates/contextra-storage/crates/contextra-storage/src/recovery/scan.rs:20` |  |

#### `fn scan`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/recovery/scan.rs:25`
```rust
pub fn scan<'a>( reader: &RedoReader, bytes: &'a [u8], key: &Key32, mac: &dyn Mac, previous_mac: [u8
```
Dokumentation: Pass 1 checks framing, CRC, chain MAC and stamp monotonicity. Pass 2 extracts the durable retired-cluster set before callers expose or replay data frames.

#### `fn scan_with_resolver`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/recovery/scan.rs:36`
```rust
pub fn scan_with_resolver<'a>( reader: &RedoReader, bytes: &'a [u8], key: &Key32, mac: &dyn Mac, previous_mac: [u8
```

### Modul `spaces`

#### `struct SpaceOptions`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/spaces.rs:36`
```rust
pub struct SpaceOptions
```
- Impl-Block (Trait `Default`) `crates/contextra-storage/crates/contextra-storage/src/spaces.rs:51`
- Impl-Block `crates/contextra-storage/crates/contextra-storage/src/spaces.rs:70`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `class` | `SpaceClass` | `crates/contextra-storage/crates/contextra-storage/src/spaces.rs:37` |
| `clustered` | `bool` | `crates/contextra-storage/crates/contextra-storage/src/spaces.rs:38` |
| `data_class` | `DataClass` | `crates/contextra-storage/crates/contextra-storage/src/spaces.rs:39` |
| `packing` | `PackPolicy` | `crates/contextra-storage/crates/contextra-storage/src/spaces.rs:40` |
| `pure_threshold` | `u64` | `crates/contextra-storage/crates/contextra-storage/src/spaces.rs:41` |
| `seal` | `SealPolicy` | `crates/contextra-storage/crates/contextra-storage/src/spaces.rs:42` |
| `weight_slots` | `u8` | `crates/contextra-storage/crates/contextra-storage/src/spaces.rs:43` |
| `history` | `HistoryPolicy` | `crates/contextra-storage/crates/contextra-storage/src/spaces.rs:44` |
| `value_separation_min` | `u32` | `crates/contextra-storage/crates/contextra-storage/src/spaces.rs:45` |
| `max_key_bytes` | `u32` | `crates/contextra-storage/crates/contextra-storage/src/spaces.rs:46` |
| `compression` | `Compression` | `crates/contextra-storage/crates/contextra-storage/src/spaces.rs:47` |
| `durability_default` | `Durability` | `crates/contextra-storage/crates/contextra-storage/src/spaces.rs:48` |

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn volatile() -> Self` | `crates/contextra-storage/crates/contextra-storage/src/spaces.rs:71` |  |
| `pub fn validate(&self, integrity: IntegrityMode) -> Result<(), Error>` | `crates/contextra-storage/crates/contextra-storage/src/spaces.rs:76` | Validates the immutable option set before any catalog, key, or file I/O. |
| `pub fn protection_allowed(&self, protection: Protection) -> bool` | `crates/contextra-storage/crates/contextra-storage/src/spaces.rs:109` |  |

#### `struct SpaceSpec`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/spaces.rs:116`
```rust
pub struct SpaceSpec
```
- Impl-Block `crates/contextra-storage/crates/contextra-storage/src/spaces.rs:122`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `id` | `SpaceId` | `crates/contextra-storage/crates/contextra-storage/src/spaces.rs:117` |
| `name` | `String` | `crates/contextra-storage/crates/contextra-storage/src/spaces.rs:118` |
| `options` | `SpaceOptions` | `crates/contextra-storage/crates/contextra-storage/src/spaces.rs:119` |

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn validate(&self, integrity: IntegrityMode) -> Result<(), Error>` | `crates/contextra-storage/crates/contextra-storage/src/spaces.rs:123` |  |

#### `enum Compression`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/spaces.rs:33`
```rust
pub enum Compression
```

#### `enum ExpiryBucket`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/spaces.rs:18`
```rust
pub enum ExpiryBucket
```

#### `enum HistoryPolicy`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/spaces.rs:30`
```rust
pub enum HistoryPolicy
```

#### `enum PackPolicy`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/spaces.rs:21`
```rust
pub enum PackPolicy
```

Varianten:

- `None` — `crates/contextra-storage/crates/contextra-storage/src/spaces.rs:22`
- `ByTenantAndExpiry{ bucket: ExpiryBucket, shards: u8 }` — `crates/contextra-storage/crates/contextra-storage/src/spaces.rs:23`

#### `enum SealPolicy`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/spaces.rs:27`
```rust
pub enum SealPolicy
```

#### `enum SpaceClass`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/spaces.rs:15`
```rust
pub enum SpaceClass
```

#### `const DEFAULT_MAX_KEY_BYTES`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/spaces.rs:10`
```rust
pub const DEFAULT_MAX_KEY_BYTES: u32 = 1024
```

#### `const DEFAULT_PURE_THRESHOLD`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/spaces.rs:8`
```rust
pub const DEFAULT_PURE_THRESHOLD: u64 = 8 * 1024 * 1024
```

#### `const DEFAULT_VALUE_SEPARATION_MIN`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/spaces.rs:9`
```rust
pub const DEFAULT_VALUE_SEPARATION_MIN: u32 = 1024
```

#### `const MAX_KEY_BYTES`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/spaces.rs:11`
```rust
pub const MAX_KEY_BYTES: u32 = 4096
```

#### `const MAX_WEIGHT_SLOTS`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/spaces.rs:12`
```rust
pub const MAX_WEIGHT_SLOTS: u8 = 4
```

### Modul `store`

#### `mod blob`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/store/mod.rs:2`
```rust
pub mod blob
```

#### `mod redo`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/store/mod.rs:3`
```rust
pub mod redo
```

#### `mod run`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/store/mod.rs:4`
```rust
pub mod run
```

### Modul `store::blob`

#### `mod gc_index`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/store/blob/mod.rs:2`
```rust
pub mod gc_index
```

#### `mod reader`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/store/blob/mod.rs:3`
```rust
pub mod reader
```

#### `mod record`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/store/blob/mod.rs:4`
```rust
pub mod record
```

#### `mod writer`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/store/blob/mod.rs:5`
```rust
pub mod writer
```

### Modul `store::blob::gc_index`

#### `struct BlobGcIndex`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/store/blob/gc_index.rs:29`
```rust
pub struct BlobGcIndex
```
Dokumentation: Tracks references discovered in live Runs. Rebuild this index from Runs after recovery; it is deliberately not an authority for deletion.
- Impl-Block `crates/contextra-storage/crates/contextra-storage/src/store/blob/gc_index.rs:33`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new() -> Self` | `crates/contextra-storage/crates/contextra-storage/src/store/blob/gc_index.rs:34` |  |
| `pub fn add_reference(&mut self, reference: BlobRef) -> Result<(), Error>` | `crates/contextra-storage/crates/contextra-storage/src/store/blob/gc_index.rs:36` |  |
| `pub fn remove_reference(&mut self, reference: BlobRef) -> Result<(), Error>` | `crates/contextra-storage/crates/contextra-storage/src/store/blob/gc_index.rs:46` |  |
| `pub fn remove_candidates(&self, known_files: impl IntoIterator<Item = (BlobId, BlobIncarnation)>) -> Vec<(BlobId, BlobIncarnation)>` | `crates/contextra-storage/crates/contextra-storage/src/store/blob/gc_index.rs:56` | Lists file incarnations with no references in the currently indexed live Runs. It does not unlink them or assert that they are purgeable. |
| `pub fn live_reference_count(&self, blob: BlobId, incarnation: BlobIncarnation) -> u64` | `crates/contextra-storage/crates/contextra-storage/src/store/blob/gc_index.rs:62` |  |

#### `struct BlobId`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/store/blob/gc_index.rs:10`
```rust
pub struct BlobId(pub u64)
```

#### `struct BlobIncarnation`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/store/blob/gc_index.rs:13`
```rust
pub struct BlobIncarnation(pub u64)
```

#### `struct BlobRef`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/store/blob/gc_index.rs:16`
```rust
pub struct BlobRef
```

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `blob_id` | `BlobId` | `crates/contextra-storage/crates/contextra-storage/src/store/blob/gc_index.rs:17` |
| `incarnation` | `BlobIncarnation` | `crates/contextra-storage/crates/contextra-storage/src/store/blob/gc_index.rs:18` |
| `offset` | `u64` | `crates/contextra-storage/crates/contextra-storage/src/store/blob/gc_index.rs:19` |
| `len` | `u64` | `crates/contextra-storage/crates/contextra-storage/src/store/blob/gc_index.rs:20` |

### Modul `store::blob::record`

#### `struct BlobRecord`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/store/blob/record.rs:11`
```rust
pub struct BlobRecord
```
- Impl-Block (Trait `fmt::Debug`) `crates/contextra-storage/crates/contextra-storage/src/store/blob/record.rs:16`
- Impl-Block `crates/contextra-storage/crates/contextra-storage/src/store/blob/record.rs:23`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `record_uid` | `[u8; RECORD_UID_LEN]` | `crates/contextra-storage/crates/contextra-storage/src/store/blob/record.rs:12` |
| `seal_frame` | `Zeroizing<Vec<u8>>` | `crates/contextra-storage/crates/contextra-storage/src/store/blob/record.rs:13` |

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn encode(&self, max_frame_bytes: usize) -> Result<Vec<u8>, Error>` | `crates/contextra-storage/crates/contextra-storage/src/store/blob/record.rs:24` |  |
| `pub fn decode(bytes: &[u8], max_frame_bytes: usize) -> Result<Self, Error>` | `crates/contextra-storage/crates/contextra-storage/src/store/blob/record.rs:39` |  |

#### `const RECORD_OVERHEAD`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/store/blob/record.rs:8`
```rust
pub const RECORD_OVERHEAD: usize = RECORD_UID_LEN + 4 + 4
```

#### `const RECORD_UID_LEN`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/store/blob/record.rs:7`
```rust
pub const RECORD_UID_LEN: usize = 16
```

### Modul `store::redo`

#### `mod cut`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/store/redo/mod.rs:2`
```rust
pub mod cut
```

#### `mod frame`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/store/redo/mod.rs:3`
```rust
pub mod frame
```

#### `mod reader`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/store/redo/mod.rs:4`
```rust
pub mod reader
```

#### `mod reserve`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/store/redo/mod.rs:5`
```rust
pub mod reserve
```

#### `mod segment`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/store/redo/mod.rs:6`
```rust
pub mod segment
```

#### `mod torn_tail`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/store/redo/mod.rs:7`
```rust
pub mod torn_tail
```

#### `mod writer`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/store/redo/mod.rs:8`
```rust
pub mod writer
```

### Modul `store::redo::frame`

#### `struct DecodedFrame`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/store/redo/frame.rs:52`
```rust
pub struct DecodedFrame<'a>
```

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `head` | `FrameHead` | `crates/contextra-storage/crates/contextra-storage/src/store/redo/frame.rs:53` |
| `payloads` | `Vec<OpPayload<'a>>` | `crates/contextra-storage/crates/contextra-storage/src/store/redo/frame.rs:54` |
| `chain_mac` | `[u8; 32]` | `crates/contextra-storage/crates/contextra-storage/src/store/redo/frame.rs:55` |

#### `struct EncodedFrame`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/store/redo/frame.rs:39`
```rust
pub struct EncodedFrame
```

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `head` | `FrameHead` | `crates/contextra-storage/crates/contextra-storage/src/store/redo/frame.rs:40` |
| `payloads` | `Vec<Vec<u8>>` | `crates/contextra-storage/crates/contextra-storage/src/store/redo/frame.rs:42` |

#### `struct FrameHead`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/store/redo/frame.rs:27`
```rust
pub struct FrameHead
```

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `stamp` | `Stamp` | `crates/contextra-storage/crates/contextra-storage/src/store/redo/frame.rs:28` |
| `kind` | `FrameKind` | `crates/contextra-storage/crates/contextra-storage/src/store/redo/frame.rs:29` |
| `idem` | `[u8; 16]` | `crates/contextra-storage/crates/contextra-storage/src/store/redo/frame.rs:30` |
| `epoch` | `u64` | `crates/contextra-storage/crates/contextra-storage/src/store/redo/frame.rs:31` |
| `flags` | `u8` | `crates/contextra-storage/crates/contextra-storage/src/store/redo/frame.rs:32` |
| `caller` | `CallerMeta` | `crates/contextra-storage/crates/contextra-storage/src/store/redo/frame.rs:33` |
| `ttl_bucket` | `u32` | `crates/contextra-storage/crates/contextra-storage/src/store/redo/frame.rs:34` |
| `ops` | `Vec<OpHead>` | `crates/contextra-storage/crates/contextra-storage/src/store/redo/frame.rs:35` |

#### `struct OpHead`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/store/redo/frame.rs:16`
```rust
pub struct OpHead
```

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `space` | `SpaceId` | `crates/contextra-storage/crates/contextra-storage/src/store/redo/frame.rs:17` |
| `cluster` | `ClusterId` | `crates/contextra-storage/crates/contextra-storage/src/store/redo/frame.rs:18` |
| `branch` | `BranchId` | `crates/contextra-storage/crates/contextra-storage/src/store/redo/frame.rs:19` |
| `kind` | `u8` | `crates/contextra-storage/crates/contextra-storage/src/store/redo/frame.rs:20` |
| `key_epoch` | `u32` | `crates/contextra-storage/crates/contextra-storage/src/store/redo/frame.rs:21` |
| `op_uid` | `[u8; 16]` | `crates/contextra-storage/crates/contextra-storage/src/store/redo/frame.rs:22` |
| `payload_len` | `u32` | `crates/contextra-storage/crates/contextra-storage/src/store/redo/frame.rs:23` |

#### `struct OpPayload`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/store/redo/frame.rs:46`
```rust
pub struct OpPayload<'a>
```

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `head` | `OpHead` | `crates/contextra-storage/crates/contextra-storage/src/store/redo/frame.rs:47` |
| `bytes` | `&'a [u8]` | `crates/contextra-storage/crates/contextra-storage/src/store/redo/frame.rs:48` |

#### `enum FrameKind`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/store/redo/frame.rs:13`
```rust
pub enum FrameKind
```
- Impl-Block `crates/contextra-storage/crates/contextra-storage/src/store/redo/frame.rs:58`

#### `fn decode_frame`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/store/redo/frame.rs:120`
```rust
pub fn decode_frame<'a>(input: &'a [u8], previous: &[u8
```
Dokumentation: Prüft Grenzen, CRC und Ketten-MAC, bevor Payloads als Borrowed-Slices geliefert werden.

#### `fn encode_frame`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/store/redo/frame.rs:73`
```rust
pub fn encode_frame(frame: &EncodedFrame, previous: &[u8
```
Dokumentation: Kodiert einen vollständigen Frame einschließlich CRC und Ketten-MAC. `previous` ist der MAC des vorherigen Frames; der erste Frame nutzt Nullen.

### Modul `store::redo::reader`

#### `struct RedoReader`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/store/redo/reader.rs:21`
```rust
pub struct RedoReader
```
- Impl-Block `crates/contextra-storage/crates/contextra-storage/src/store/redo/reader.rs:26`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `alignment` | `u64` | `crates/contextra-storage/crates/contextra-storage/src/store/redo/reader.rs:22` |
| `max_group_bytes` | `usize` | `crates/contextra-storage/crates/contextra-storage/src/store/redo/reader.rs:23` |

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn scan<'a>( &self, segment: &'a [u8], key: &Key32, mac: &dyn Mac, previous_mac: [u8` | `crates/contextra-storage/crates/contextra-storage/src/store/redo/reader.rs:27` |  |
| `pub fn heads_only<'a>(&self, result: &'a ScanResult<'a>) -> impl Iterator<Item = (&'a super::frame::FrameHead, Stamp)> + 'a` | `crates/contextra-storage/crates/contextra-storage/src/store/redo/reader.rs:124` |  |

#### `struct ScanResult`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/store/redo/reader.rs:13`
```rust
pub struct ScanResult<'a>
```

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `frames` | `Vec<DecodedFrame<'a>>` | `crates/contextra-storage/crates/contextra-storage/src/store/redo/reader.rs:14` |
| `tail` | `TailState` | `crates/contextra-storage/crates/contextra-storage/src/store/redo/reader.rs:15` |
| `valid_bytes` | `usize` | `crates/contextra-storage/crates/contextra-storage/src/store/redo/reader.rs:16` |
| `last_stamp` | `Stamp` | `crates/contextra-storage/crates/contextra-storage/src/store/redo/reader.rs:17` |
| `last_mac` | `[u8; 32]` | `crates/contextra-storage/crates/contextra-storage/src/store/redo/reader.rs:18` |

### Modul `store::redo::reserve`

#### `struct ReserveWriter`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/store/redo/reserve.rs:8`
```rust
pub struct ReserveWriter
```
- Impl-Block `crates/contextra-storage/crates/contextra-storage/src/store/redo/reserve.rs:15`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn initialize(file: Box<dyn File>, capacity: u64) -> Result<Self, Error>` | `crates/contextra-storage/crates/contextra-storage/src/store/redo/reserve.rs:18` | Initialisiert die Reserve mit Nullen und macht die Vorallokation durabel. Der Aufrufer erzeugt/öffnet exklusiv die zugehörige Reservendatei. |
| `pub fn remaining(&self) -> u64` | `crates/contextra-storage/crates/contextra-storage/src/store/redo/reserve.rs:25` |  |
| `pub fn append_durable(&mut self, record: &[u8]) -> Result<u64, Error>` | `crates/contextra-storage/crates/contextra-storage/src/store/redo/reserve.rs:28` | Emergency-Datensatz durabel anhängen; Fehler vergiften diesen Handle. |

### Modul `store::redo::segment`

#### `fn align_up`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/store/redo/segment.rs:10`
```rust
pub fn align_up(value: u64, alignment: u64) -> Result<u64, Error>
```

#### `fn group_padding`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/store/redo/segment.rs:19`
```rust
pub fn group_padding(offset: u64, alignment: u64) -> Result<usize, Error>
```

#### `const DEFAULT_ALIGNMENT`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/store/redo/segment.rs:7`
```rust
pub const DEFAULT_ALIGNMENT: u64 = 4096
```

#### `const DEFAULT_MARK_BYTES`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/store/redo/segment.rs:6`
```rust
pub const DEFAULT_MARK_BYTES: u64 = 8 * 1024 * 1024
```

#### `const DEFAULT_SEGMENT_BYTES`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/store/redo/segment.rs:5`
```rust
pub const DEFAULT_SEGMENT_BYTES: u64 = 64 * 1024 * 1024
```

#### `const GROUP_MAGIC`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/store/redo/segment.rs:8`
```rust
pub const GROUP_MAGIC: &[u8
```

### Modul `store::redo::torn_tail`

#### `enum TailState`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/store/redo/torn_tail.rs:4`
```rust
pub enum TailState
```

#### `fn tail_state`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/store/redo/torn_tail.rs:8`
```rust
pub fn tail_state(has_later_group: bool) -> TailState
```
Dokumentation: Ein ungültiger Record ist Torn-Tail, sofern dahinter keine syntaktisch vollständige Gruppe mehr gefunden wird.

### Modul `store::redo::writer`

#### `struct GroupAck`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/store/redo/writer.rs:12`
```rust
pub struct GroupAck
```

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `last_stamp` | `Stamp` | `crates/contextra-storage/crates/contextra-storage/src/store/redo/writer.rs:13` |
| `durable` | `bool` | `crates/contextra-storage/crates/contextra-storage/src/store/redo/writer.rs:14` |
| `end_offset` | `u64` | `crates/contextra-storage/crates/contextra-storage/src/store/redo/writer.rs:15` |

#### `struct RedoWriter`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/store/redo/writer.rs:18`
```rust
pub struct RedoWriter<'a>
```
- Impl-Block `crates/contextra-storage/crates/contextra-storage/src/store/redo/writer.rs:30`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new( file: Box<dyn File>, key: Key32, mac: &'a dyn Mac, previous_mac: [u8` | `crates/contextra-storage/crates/contextra-storage/src/store/redo/writer.rs:31` |  |
| `pub fn durable_stamp(&self) -> Stamp` | `crates/contextra-storage/crates/contextra-storage/src/store/redo/writer.rs:46` |  |
| `pub fn is_poisoned(&self) -> bool` | `crates/contextra-storage/crates/contextra-storage/src/store/redo/writer.rs:50` | True only when an append or sync error made the writer unsafe to reuse. Validation/rotation errors before I/O leave the writer healthy. |
| `pub fn append_group(&mut self, frames: &[EncodedFrame], durability: Durability) -> Result<GroupAck, Error>` | `crates/contextra-storage/crates/contextra-storage/src/store/redo/writer.rs:52` |  |
| `pub fn needs_rotation(&self, additional_bytes: u64) -> Result<bool, Error>` | `crates/contextra-storage/crates/contextra-storage/src/store/redo/writer.rs:124` |  |

### Modul `store::run`

#### `mod block`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/store/run/mod.rs:2`
```rust
pub mod block
```

#### `mod builder`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/store/run/mod.rs:3`
```rust
pub mod builder
```

#### `mod copy`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/store/run/mod.rs:4`
```rust
pub mod copy
```

#### `mod directory`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/store/run/mod.rs:5`
```rust
pub mod directory
```

#### `mod extension`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/store/run/mod.rs:6`
```rust
pub mod extension
```

#### `mod filter`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/store/run/mod.rs:7`
```rust
pub mod filter
```

#### `mod format`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/store/run/mod.rs:8`
```rust
pub mod format
```

#### `mod rangedel`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/store/run/mod.rs:9`
```rust
pub mod rangedel
```

#### `mod reader`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/store/run/mod.rs:10`
```rust
pub mod reader
```

#### `mod skip_scan`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/store/run/mod.rs:11`
```rust
pub mod skip_scan
```

#### `mod subindex`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/store/run/mod.rs:12`
```rust
pub mod subindex
```

#### `mod table_cache`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/store/run/mod.rs:13`
```rust
pub mod table_cache
```

### Modul `store::run::block`

#### `struct BlockEntry`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/store/run/block.rs:19`
```rust
pub struct BlockEntry
```
- Impl-Block (Trait `fmt::Debug`) `crates/contextra-storage/crates/contextra-storage/src/store/run/block.rs:27`
- Impl-Block `crates/contextra-storage/crates/contextra-storage/src/store/run/block.rs:34`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `branch` | `BranchId` | `crates/contextra-storage/crates/contextra-storage/src/store/run/block.rs:20` |
| `key` | `Zeroizing<Vec<u8>>` | `crates/contextra-storage/crates/contextra-storage/src/store/run/block.rs:21` |
| `tag` | `u64` | `crates/contextra-storage/crates/contextra-storage/src/store/run/block.rs:22` |
| `header` | `Zeroizing<Vec<u8>>` | `crates/contextra-storage/crates/contextra-storage/src/store/run/block.rs:23` |
| `body` | `Zeroizing<Vec<u8>>` | `crates/contextra-storage/crates/contextra-storage/src/store/run/block.rs:24` |

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn stamp(&self) -> Stamp` | `crates/contextra-storage/crates/contextra-storage/src/store/run/block.rs:35` |  |

#### `fn compare`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/store/run/block.rs:172`
```rust
pub fn compare(left: &BlockEntry, right: &BlockEntry) -> std::cmp::Ordering
```

#### `fn decode`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/store/run/block.rs:89`
```rust
pub fn decode(bytes: &[u8], max_bytes: usize) -> Result<Vec<BlockEntry>, Error>
```
Dokumentation: Decodes exactly one block. Every wire length is checked against both the remaining input and the caller's configured allocation limit.

#### `fn encode`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/store/run/block.rs:41`
```rust
pub fn encode(entries: &[BlockEntry], max_bytes: usize) -> Result<Vec<u8>, Error>
```
Dokumentation: Encodes sorted entries, using a full key at every restart entry. `max_bytes` allows the builder's single-entry oversize progress rule while remaining bounded by the configured change limit.

#### `const MAX_DECODED_BLOCK_BYTES`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/store/run/block.rs:15`
```rust
pub const MAX_DECODED_BLOCK_BYTES: usize = 8 * 1024 * 1024
```

#### `const TARGET_BLOCK_BYTES`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/store/run/block.rs:16`
```rust
pub const TARGET_BLOCK_BYTES: usize = 16 * 1024
```

### Modul `store::run::builder`

#### `struct BuiltRun`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/store/run/builder.rs:19`
```rust
pub struct BuiltRun
```

#### `struct RunFileBuilder`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/store/run/builder.rs:21`
```rust
pub struct RunFileBuilder
```
- Impl-Block `crates/contextra-storage/crates/contextra-storage/src/store/run/builder.rs:31`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn create(fs: &dyn Fs, path: &Path, header: &FileHeader) -> Result<Self, Error>` | `crates/contextra-storage/crates/contextra-storage/src/store/run/builder.rs:32` |  |
| `pub fn append_section( &mut self, cluster: ClusterId, section_bytes: &[u8], block_count: u32, entry_count: u64, min_stamp: Stamp, max_stamp: Stamp, ) -> Result<(), Error>` | `crates/contextra-storage/crates/contextra-storage/src/store/run/builder.rs:42` | Appends one complete, already encoded ClusterSection. The section bytes stay cluster-local and are copied without interpreting payloads. |
| `pub fn finish(mut self, pack_meta: &[u8], extensions: &[ExtensionBlock]) -> Result<BuiltRun, Error>` | `crates/contextra-storage/crates/contextra-storage/src/store/run/builder.rs:75` | Finishes the immutable file. `pack_meta` is opaque here because its owner-SP has not frozen its field layout yet. |

#### `const MAX_PACK_META_BYTES`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/store/run/builder.rs:17`
```rust
pub const MAX_PACK_META_BYTES: usize = 1024 * 1024
```

#### `const RUN_PREALLOC_BYTES`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/store/run/builder.rs:16`
```rust
pub const RUN_PREALLOC_BYTES: u64 = 4 * 1024 * 1024
```

### Modul `store::run::copy`

#### `trait RawSectionChecksumVerifier`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/store/run/copy.rs:9`
```rust
pub trait RawSectionChecksumVerifier
```
Dokumentation: Format-specific scanner that checks only integrity fields available without a key (for example CRCs), never an AEAD/MAC authentication tag.

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `fn verify_raw_checksums(&self, section: &[u8]) -> Result<(), Error>` | `crates/contextra-storage/crates/contextra-storage/src/store/run/copy.rs:10` |  |

#### `fn copy_sections`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/store/run/copy.rs:16`
```rust
pub fn copy_sections( src: &RunReader, keep: &[ClusterId], dst: &mut RunFileBuilder, max_section_bytes: usize, verifier: &dyn RawSectionChecksumVerifier, ) -> Result<(), Error>
```
Dokumentation: Copies selected complete cluster sections byte-for-byte. The verifier is required so callers cannot accidentally treat an unchecked relocation as a scrubbed copy. Any verifier failure aborts before that section is added.

### Modul `store::run::directory`

#### `struct ClusterDirectory`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/store/run/directory.rs:24`
```rust
pub struct ClusterDirectory
```
- Impl-Block `crates/contextra-storage/crates/contextra-storage/src/store/run/directory.rs:28`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new(entries: Vec<ClusterDirectoryEntry>) -> Result<Self, Error>` | `crates/contextra-storage/crates/contextra-storage/src/store/run/directory.rs:29` |  |
| `pub fn entries(&self) -> &[ClusterDirectoryEntry]` | `crates/contextra-storage/crates/contextra-storage/src/store/run/directory.rs:34` |  |
| `pub fn find(&self, cluster: ClusterId) -> Option<&ClusterDirectoryEntry>` | `crates/contextra-storage/crates/contextra-storage/src/store/run/directory.rs:36` |  |
| `pub fn encode(&self) -> Result<Vec<u8>, Error>` | `crates/contextra-storage/crates/contextra-storage/src/store/run/directory.rs:41` |  |
| `pub fn decode(bytes: &[u8], max_entries: usize) -> Result<Self, Error>` | `crates/contextra-storage/crates/contextra-storage/src/store/run/directory.rs:65` |  |

#### `struct ClusterDirectoryEntry`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/store/run/directory.rs:13`
```rust
pub struct ClusterDirectoryEntry
```

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `cluster` | `ClusterId` | `crates/contextra-storage/crates/contextra-storage/src/store/run/directory.rs:14` |
| `section_offset` | `u64` | `crates/contextra-storage/crates/contextra-storage/src/store/run/directory.rs:15` |
| `section_len` | `u64` | `crates/contextra-storage/crates/contextra-storage/src/store/run/directory.rs:16` |
| `block_count` | `u32` | `crates/contextra-storage/crates/contextra-storage/src/store/run/directory.rs:17` |
| `entry_count` | `u64` | `crates/contextra-storage/crates/contextra-storage/src/store/run/directory.rs:18` |
| `min_stamp` | `Stamp` | `crates/contextra-storage/crates/contextra-storage/src/store/run/directory.rs:19` |
| `max_stamp` | `Stamp` | `crates/contextra-storage/crates/contextra-storage/src/store/run/directory.rs:20` |

#### `const MAX_DIRECTORY_ENTRIES`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/store/run/directory.rs:10`
```rust
pub const MAX_DIRECTORY_ENTRIES: usize = 1_000_000
```

### Modul `store::run::extension`

#### `struct ExtensionBlock`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/store/run/extension.rs:11`
```rust
pub struct ExtensionBlock
```
- Impl-Block (Trait `fmt::Debug`) `crates/contextra-storage/crates/contextra-storage/src/store/run/extension.rs:17`
- Impl-Block `crates/contextra-storage/crates/contextra-storage/src/store/run/extension.rs:24`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `tag` | `u16` | `crates/contextra-storage/crates/contextra-storage/src/store/run/extension.rs:12` |
| `version` | `u16` | `crates/contextra-storage/crates/contextra-storage/src/store/run/extension.rs:13` |
| `payload` | `Zeroizing<Vec<u8>>` | `crates/contextra-storage/crates/contextra-storage/src/store/run/extension.rs:14` |

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new(tag: u16, version: u16, payload: Vec<u8>) -> Result<Self, Error>` | `crates/contextra-storage/crates/contextra-storage/src/store/run/extension.rs:25` |  |

#### `fn decode`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/store/run/extension.rs:53`
```rust
pub fn decode(bytes: &[u8], max_bytes: usize, max_blocks: usize) -> Result<Vec<ExtensionBlock>, Error>
```

#### `fn encode`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/store/run/extension.rs:31`
```rust
pub fn encode(blocks: &[ExtensionBlock], max_bytes: usize) -> Result<Vec<u8>, Error>
```

#### `const MAX_EXTENSION_BLOCKS`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/store/run/extension.rs:8`
```rust
pub const MAX_EXTENSION_BLOCKS: usize = 4096
```

### Modul `store::run::filter`

#### `struct BloomFilter`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/store/run/filter.rs:13`
```rust
pub struct BloomFilter
```
- Impl-Block (Trait `fmt::Debug`) `crates/contextra-storage/crates/contextra-storage/src/store/run/filter.rs:20`
- Impl-Block `crates/contextra-storage/crates/contextra-storage/src/store/run/filter.rs:28`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new(expected_items: u32, false_positive_rate: f64, salt: [u8` | `crates/contextra-storage/crates/contextra-storage/src/store/run/filter.rs:29` |  |
| `pub fn insert(&mut self, key: &[u8])` | `crates/contextra-storage/crates/contextra-storage/src/store/run/filter.rs:46` |  |
| `pub fn may_contain(&self, key: &[u8]) -> bool` | `crates/contextra-storage/crates/contextra-storage/src/store/run/filter.rs:55` |  |
| `pub fn encode(&self) -> Result<Vec<u8>, Error>` | `crates/contextra-storage/crates/contextra-storage/src/store/run/filter.rs:64` |  |
| `pub fn decode(bytes: &[u8], max_bytes: usize) -> Result<Self, Error>` | `crates/contextra-storage/crates/contextra-storage/src/store/run/filter.rs:79` |  |

#### `const FILTER_KIND_BLOOM_V1`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/store/run/filter.rs:8`
```rust
pub const FILTER_KIND_BLOOM_V1: u8 = 1
```

#### `const MAX_FILTER_BYTES`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/store/run/filter.rs:9`
```rust
pub const MAX_FILTER_BYTES: usize = 8 * 1024 * 1024
```

### Modul `store::run::format`

#### `struct RunFooter`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/store/run/format.rs:9`
```rust
pub struct RunFooter
```
- Impl-Block `crates/contextra-storage/crates/contextra-storage/src/store/run/format.rs:16`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `directory_offset` | `u64` | `crates/contextra-storage/crates/contextra-storage/src/store/run/format.rs:10` |
| `directory_len` | `u64` | `crates/contextra-storage/crates/contextra-storage/src/store/run/format.rs:11` |
| `pack_offset` | `u64` | `crates/contextra-storage/crates/contextra-storage/src/store/run/format.rs:12` |
| `extension_offset` | `u64` | `crates/contextra-storage/crates/contextra-storage/src/store/run/format.rs:13` |

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn encode(self, file_len: u64) -> Result<[u8` | `crates/contextra-storage/crates/contextra-storage/src/store/run/format.rs:17` |  |
| `pub fn decode(bytes: &[u8], file_len: u64) -> Result<Self, Error>` | `crates/contextra-storage/crates/contextra-storage/src/store/run/format.rs:30` |  |

#### `const RUN_FOOTER_LEN`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/store/run/format.rs:6`
```rust
pub const RUN_FOOTER_LEN: usize = 44
```

#### `const RUN_FOOTER_MAGIC`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/store/run/format.rs:5`
```rust
pub const RUN_FOOTER_MAGIC: [u8
```

### Modul `store::run::rangedel`

#### `struct RangeMarker`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/store/run/rangedel.rs:12`
```rust
pub struct RangeMarker
```
- Impl-Block (Trait `fmt::Debug`) `crates/contextra-storage/crates/contextra-storage/src/store/run/rangedel.rs:20`
- Impl-Block `crates/contextra-storage/crates/contextra-storage/src/store/run/rangedel.rs:28`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `branch` | `BranchId` | `crates/contextra-storage/crates/contextra-storage/src/store/run/rangedel.rs:13` |
| `start` | `Zeroizing<Vec<u8>>` | `crates/contextra-storage/crates/contextra-storage/src/store/run/rangedel.rs:14` |
| `end` | `Zeroizing<Vec<u8>>` | `crates/contextra-storage/crates/contextra-storage/src/store/run/rangedel.rs:16` |
| `tag` | `u64` | `crates/contextra-storage/crates/contextra-storage/src/store/run/rangedel.rs:17` |

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn stamp(&self) -> Result<Stamp, Error>` | `crates/contextra-storage/crates/contextra-storage/src/store/run/rangedel.rs:29` |  |

#### `fn decode`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/store/run/rangedel.rs:60`
```rust
pub fn decode(bytes: &[u8], max_bytes: usize, max_markers: usize) -> Result<Vec<RangeMarker>, Error>
```

#### `fn encode`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/store/run/rangedel.rs:39`
```rust
pub fn encode(markers: &[RangeMarker], max_bytes: usize) -> Result<Vec<u8>, Error>
```

#### `const MAX_RANGE_MARKERS`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/store/run/rangedel.rs:9`
```rust
pub const MAX_RANGE_MARKERS: usize = 1_000_000
```

### Modul `store::run::reader`

#### `struct RunReader`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/store/run/reader.rs:17`
```rust
pub struct RunReader
```
- Impl-Block `crates/contextra-storage/crates/contextra-storage/src/store/run/reader.rs:27`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn open( fs: &dyn Fs, cache: &TableCache, path: &Path, expected_magic: [u8` | `crates/contextra-storage/crates/contextra-storage/src/store/run/reader.rs:28` |  |
| `pub fn header(&self) -> &FileHeader` | `crates/contextra-storage/crates/contextra-storage/src/store/run/reader.rs:77` |  |
| `pub fn footer(&self) -> RunFooter` | `crates/contextra-storage/crates/contextra-storage/src/store/run/reader.rs:78` |  |
| `pub fn file_len(&self) -> u64` | `crates/contextra-storage/crates/contextra-storage/src/store/run/reader.rs:79` |  |
| `pub fn directory(&self) -> &ClusterDirectory` | `crates/contextra-storage/crates/contextra-storage/src/store/run/reader.rs:80` |  |
| `pub fn pack_meta(&self) -> &[u8]` | `crates/contextra-storage/crates/contextra-storage/src/store/run/reader.rs:81` |  |
| `pub fn extensions(&self) -> &[ExtensionBlock]` | `crates/contextra-storage/crates/contextra-storage/src/store/run/reader.rs:82` |  |
| `pub fn read_section(&self, cluster: ClusterId, max_bytes: usize) -> Result<Option<Zeroizing<Vec<u8>>>, Error>` | `crates/contextra-storage/crates/contextra-storage/src/store/run/reader.rs:84` |  |
| `pub fn section_meta(&self, cluster: ClusterId) -> Option<&ClusterDirectoryEntry>` | `crates/contextra-storage/crates/contextra-storage/src/store/run/reader.rs:91` |  |

#### `const MAX_EXTENSION_REGION_BYTES`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/store/run/reader.rs:15`
```rust
pub const MAX_EXTENSION_REGION_BYTES: usize = 16 * 1024 * 1024
```

#### `const MAX_PACK_META_BYTES`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/store/run/reader.rs:14`
```rust
pub const MAX_PACK_META_BYTES: usize = 1024 * 1024
```

### Modul `store::run::skip_scan`

#### `struct BlockSkipQuery`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/store/run/skip_scan.rs:6`
```rust
pub struct BlockSkipQuery<'a>
```

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `start_inclusive` | `Option<&'a [u8]>` | `crates/contextra-storage/crates/contextra-storage/src/store/run/skip_scan.rs:7` |
| `end_exclusive` | `Option<&'a [u8]>` | `crates/contextra-storage/crates/contextra-storage/src/store/run/skip_scan.rs:8` |
| `min_stamp` | `Stamp` | `crates/contextra-storage/crates/contextra-storage/src/store/run/skip_scan.rs:9` |
| `max_stamp` | `Stamp` | `crates/contextra-storage/crates/contextra-storage/src/store/run/skip_scan.rs:10` |

#### `fn block_skip_scan`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/store/run/skip_scan.rs:15`
```rust
pub fn block_skip_scan<'a>( subindex: &'a SubIndex, query: BlockSkipQuery<'a>, ) -> impl Iterator<Item = &'a SubIndexEntry> + 'a
```
Dokumentation: Yields only summaries whose key interval and stamp range intersect the query. A block represents `(previous.last_key, current.last_key]`.

### Modul `store::run::subindex`

#### `struct SubIndex`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/store/run/subindex.rs:32`
```rust
pub struct SubIndex
```
- Impl-Block `crates/contextra-storage/crates/contextra-storage/src/store/run/subindex.rs:34`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new(entries: Vec<SubIndexEntry>) -> Result<Self, Error>` | `crates/contextra-storage/crates/contextra-storage/src/store/run/subindex.rs:35` |  |
| `pub fn entries(&self) -> &[SubIndexEntry]` | `crates/contextra-storage/crates/contextra-storage/src/store/run/subindex.rs:40` |  |
| `pub fn candidate(&self, key: &[u8], stamp_cap: Stamp) -> Option<&SubIndexEntry>` | `crates/contextra-storage/crates/contextra-storage/src/store/run/subindex.rs:42` |  |
| `pub fn encode(&self, max_bytes: usize) -> Result<Vec<u8>, Error>` | `crates/contextra-storage/crates/contextra-storage/src/store/run/subindex.rs:47` |  |
| `pub fn decode(bytes: &[u8], max_bytes: usize, max_entries: usize) -> Result<Self, Error>` | `crates/contextra-storage/crates/contextra-storage/src/store/run/subindex.rs:73` |  |

#### `struct SubIndexEntry`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/store/run/subindex.rs:11`
```rust
pub struct SubIndexEntry
```
- Impl-Block (Trait `fmt::Debug`) `crates/contextra-storage/crates/contextra-storage/src/store/run/subindex.rs:22`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `last_key` | `Zeroizing<Vec<u8>>` | `crates/contextra-storage/crates/contextra-storage/src/store/run/subindex.rs:12` |
| `offset` | `u64` | `crates/contextra-storage/crates/contextra-storage/src/store/run/subindex.rs:13` |
| `len` | `u32` | `crates/contextra-storage/crates/contextra-storage/src/store/run/subindex.rs:14` |
| `entries` | `u32` | `crates/contextra-storage/crates/contextra-storage/src/store/run/subindex.rs:15` |
| `weight_sum` | `[u64; 4]` | `crates/contextra-storage/crates/contextra-storage/src/store/run/subindex.rs:16` |
| `min_stamp` | `Stamp` | `crates/contextra-storage/crates/contextra-storage/src/store/run/subindex.rs:17` |
| `max_stamp` | `Stamp` | `crates/contextra-storage/crates/contextra-storage/src/store/run/subindex.rs:18` |
| `block_uid` | `[u8; 16]` | `crates/contextra-storage/crates/contextra-storage/src/store/run/subindex.rs:19` |

### Modul `store::run::table_cache`

#### `struct TableCache`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/store/run/table_cache.rs:23`
```rust
pub struct TableCache
```
Dokumentation: Cache capacity is a hard bound, including opens in progress. An entry is evictable only when the cache owns its sole strong reference.
- Impl-Block `crates/contextra-storage/crates/contextra-storage/src/store/run/table_cache.rs:29`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new(max_open_files: usize) -> Result<Self, Error>` | `crates/contextra-storage/crates/contextra-storage/src/store/run/table_cache.rs:30` |  |
| `pub fn pin(&self, fs: &dyn Fs, path: &Path) -> Result<Arc<dyn File>, Error>` | `crates/contextra-storage/crates/contextra-storage/src/store/run/table_cache.rs:35` |  |
| `pub fn resident(&self) -> Result<usize, Error>` | `crates/contextra-storage/crates/contextra-storage/src/store/run/table_cache.rs:85` |  |
| `pub fn capacity(&self) -> usize` | `crates/contextra-storage/crates/contextra-storage/src/store/run/table_cache.rs:86` |  |
| `pub fn evict_unpinned(&self) -> Result<usize, Error>` | `crates/contextra-storage/crates/contextra-storage/src/store/run/table_cache.rs:89` | Drops every unpinned handle. Returns the number of handles closed. |

### Modul `sync`

#### `mod epoch`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/sync/mod.rs:2`
```rust
pub mod epoch
```

#### `mod ranked`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/sync/mod.rs:3`
```rust
pub mod ranked
```

### Modul `sync::epoch`

#### `struct Epoch`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/sync/epoch.rs:13`
```rust
pub struct Epoch(AtomicU64)
```
- Impl-Block `crates/contextra-storage/crates/contextra-storage/src/sync/epoch.rs:15`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub const fn new(state: u8, generation: u64) -> Option<Self>` | `crates/contextra-storage/crates/contextra-storage/src/sync/epoch.rs:16` |  |
| `pub fn load(&self) -> EpochValue` | `crates/contextra-storage/crates/contextra-storage/src/sync/epoch.rs:24` |  |
| `pub fn transition(&self, expected: EpochValue, next_state: u8) -> Result<EpochValue, EpochValue>` | `crates/contextra-storage/crates/contextra-storage/src/sync/epoch.rs:30` | Ändert den Zustand nur dann, wenn Zustand und Generation noch passen. Generation wird strikt erhöht und kann nicht umbrechen. |

#### `struct EpochValue`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/sync/epoch.rs:8`
```rust
pub struct EpochValue
```

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `state` | `u8` | `crates/contextra-storage/crates/contextra-storage/src/sync/epoch.rs:9` |
| `generation` | `u64` | `crates/contextra-storage/crates/contextra-storage/src/sync/epoch.rs:10` |

### Modul `sync::ranked`

#### `struct RankStack`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/sync/ranked.rs:60`
```rust
pub struct RankStack
```
Dokumentation: Diagnose the held-rank stack without exposing thread-local state.
- Impl-Block `crates/contextra-storage/crates/contextra-storage/src/sync/ranked.rs:62`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn current() -> Vec<u8>` | `crates/contextra-storage/crates/contextra-storage/src/sync/ranked.rs:63` |  |

#### `struct Ranked`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/sync/ranked.rs:12`
```rust
pub struct Ranked<T, const RANK: u8>
```
- Impl-Block `crates/contextra-storage/crates/contextra-storage/src/sync/ranked.rs:16`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub const fn new(value: T) -> Self` | `crates/contextra-storage/crates/contextra-storage/src/sync/ranked.rs:17` |  |
| `pub fn lock(&self) -> Result<RankedGuard<'_, T, RANK>, Error>` | `crates/contextra-storage/crates/contextra-storage/src/sync/ranked.rs:21` |  |
| `pub fn into_inner(self) -> Result<T, Error>` | `crates/contextra-storage/crates/contextra-storage/src/sync/ranked.rs:34` |  |

#### `struct RankedGuard`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/sync/ranked.rs:39`
```rust
pub struct RankedGuard<'a, T, const RANK: u8>
```
- Impl-Block (Trait `Deref`) `crates/contextra-storage/crates/contextra-storage/src/sync/ranked.rs:44`
- Impl-Block (Trait `DerefMut`) `crates/contextra-storage/crates/contextra-storage/src/sync/ranked.rs:49`
- Impl-Block (Trait `Drop`) `crates/contextra-storage/crates/contextra-storage/src/sync/ranked.rs:53`

### Modul `testing`

#### `mod canary`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/testing/mod.rs:2`
```rust
pub mod canary
```

#### `mod crash_sweep`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/testing/mod.rs:3`
```rust
pub mod crash_sweep
```

#### `mod ghost`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/testing/mod.rs:4`
```rust
pub mod ghost
```

#### `mod inspector`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/testing/mod.rs:5`
```rust
pub mod inspector
```

#### `mod model`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/testing/mod.rs:6`
```rust
pub mod model
```

#### `mod oracle`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/testing/mod.rs:7`
```rust
pub mod oracle
```

#### `mod workload`
- Ort: `crates/contextra-storage/crates/contextra-storage/src/testing/mod.rs:8`
```rust
pub mod workload
```
