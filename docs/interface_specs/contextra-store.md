# Schnittstellenspezifikation: `contextra-store`

> Erzeugt durch `tools/extract_interfaces.py` aus dem Quellcode. Keine Handpflege. Jede Angabe trägt Datei:Zeile. Maßgeblich bleibt der Quellcode.

## 1. Kopf

| Feld | Wert |
|---|---|
| Name | `contextra-store` |
| Version | workspace |
| Pfad | `crates/contextra-store` |
| Rolle | Altstand LSM (bleibt bis Cutover) |
| Bibliothekspfad | src/lib.rs (Standard) |

**Crate-Dokumentation (//!):**

> # contextra-store (Ring 1)
>
> Die Crate `contextra-store` implementiert das persistente LSM-Tree-Speichertriebwerk
> (Log-Structured Merge-tree) für die Contextra-Plattform. Sie stellt die primäre
> Datenspeicherebene bereit und bietet atomare Schreibvorgänge, Konsistenz
> über Snapshots sowie Ausfallsicherheit durch Write-Ahead-Logging (WAL).
>
> ## Architektur & Datenpfad
> - **Client / Transaktion:** Sendet atomare Schreiboperationen (Put, Delete) über Transaktionspuffer.
> - **WAL (Write-Ahead-Log):** Persistiert alle Mutationen sequentiell auf Festplatte inklusive
> CRC32C- und HMAC-SHA256-Integritätsprüfung, bevor sie im RAM bestätigt werden.
> - **MemTable:** Haltebereich im Arbeitsspeicher (geschardete BTreeMap), der schnelle

## 2. Abhängigkeiten

| Name | Art | Quelle | optional | verwendet von (normal/dev) |
|---|---|---|---|---|
| `ahash` | normal | workspace | nein |  |
| `blake3` | normal | workspace | nein |  |
| `bytes` | normal | workspace | nein |  |
| `contextra-crypto` | normal | workspace | nein | contextra-crypto, contextra-crypto-fuzz, contextra-infer-candle, contextra-kvcache, contextra-mcp, contextra-store |
| `contextra-env` | dev | workspace | nein | contextra-checkpoint, contextra-crypto, contextra-graph, contextra-infer-candle, contextra-kvcache, contextra-mvcc, contextra-ports, contextra-store, contextra-text, contextra-vector |
| `contextra-ids` | normal | workspace | nein | contextra-checkpoint, contextra-crypto, contextra-graph, contextra-mvcc, contextra-store, contextra-text, contextra-types, contextra-vector |
| `contextra-mvcc` | normal | workspace | nein | contextra-db, contextra-store, contextra-vector, contextra-vector-fuzz |
| `contextra-ports` | normal | workspace | ja | contextra, contextra-adapt, contextra-agent, contextra-checkpoint, contextra-cognition, contextra-crypto, contextra-crypto-fuzz, contextra-db, contextra-engine, contextra-graph, contextra-graph-fuzz, contextra-infer-candle, contextra-infer-ollama, contextra-infer-onnx, contextra-kvcache, contextra-mcp, contextra-mvcc, contextra-rank, contextra-router, contextra-store, contextra-store-fuzz, contextra-testkit, contextra-text, contextra-vector, contextra-vector-fuzz |
| `contextra-sys` | normal | workspace | nein | contextra-bench, contextra-store, contextra-vault, contextra-vector |
| `contextra-testkit` | dev | workspace | nein | contextra-agent, contextra-checkpoint, contextra-cognition, contextra-crypto-fuzz, contextra-db, contextra-engine, contextra-store |
| `contextra-types` | normal | workspace | ja | contextra, contextra-adapt, contextra-agent, contextra-bench, contextra-checkpoint, contextra-cognition, contextra-crypto, contextra-crypto-fuzz, contextra-db, contextra-engine, contextra-graph, contextra-graph-fuzz, contextra-infer-candle, contextra-infer-ollama, contextra-infer-onnx, contextra-ingest, contextra-kvcache, contextra-mcp, contextra-mvcc, contextra-ports, contextra-py, contextra-rank, contextra-router, contextra-simd, contextra-store, contextra-store-fuzz, contextra-testkit, contextra-text, contextra-vault, contextra-vector, contextra-vector-fuzz |
| `crc32fast` | normal | workspace | nein |  |
| `criterion` | dev | registry | nein |  |
| `futures` | dev | registry | nein |  |
| `libc` | dev | registry | nein |  |
| `loom` | dev | registry | nein |  |
| `lru` | normal | registry | nein |  |
| `memmap2` | normal | workspace | nein |  |
| `parking_lot` | normal | workspace | nein |  |
| `proptest` | dev | registry | nein |  |
| `quick_cache` | normal | registry | ja |  |
| `rand` | normal | workspace | nein |  |
| `redb` | dev | registry | nein |  |
| `serde` | normal | workspace | nein |  |
| `serde_json` | dev | workspace | nein |  |
| `tempfile` | dev | workspace | nein |  |
| `thiserror` | normal | workspace | nein |  |
| `tokio` | dev | workspace | nein |  |
| `tokio-util` | normal | registry | nein |  |
| `tracing` | normal | workspace | nein |  |
| `uuid` | normal | registry | nein |  |

**Verwendet von (Crate-Ebene):** `contextra-agent`, `contextra-bench`, `contextra-db`, `contextra-engine`, `contextra-store-fuzz`

## 3. Features

| Feature | Aktiviert |
|---|---|
| `block-cache-v2` | `dep:quick_cache` |
| `default` | – |
| `docid-128` | `contextra-ids/docid-128` |
| `fault-injection` | – |
| `inspect` | – |
| `legacy-wal-key` | – |
| `memory-only-storage` | – |
| `sieve-cache` | – |
| `test-hooks` | – |
| `workspace` | `dep:contextra-types`, `dep:contextra-ports` |

## 4. Metriken (Quellcode)

| Kennzahl | Wert |
|---|---|
| Quelldateien | 161 |
| Quellzeilen | 32015 |
| Öffentliche Items (Modul-Ebene) | 293 |
| davon `fn` | 51 |
| davon `struct` | 66 |
| davon `enum` | 15 |
| davon `trait` | 16 |
| davon `type` | 9 |
| davon `const` | 47 |
| davon `static` | 7 |
| davon `mod` | 82 |
| Öffentliche Makros (`#[macro_export]`) | 0 |
| Öffentliche Re-Exports (`pub use`) | 81 |
| `unsafe`-Vorkommen (Produktionscode) | 0 |
| `.unwrap()` (Produktionscode) | 513 |
| `.expect(` (Produktionscode) | 747 |
| `panic!` (Produktionscode) | 14 |
| `todo!/unimplemented!/unreachable!` | 0 |
| Indexierungen `x[…]` (grob, Produktionscode) | 333 |
| `#[deprecated]` | 4 |
| `#[cfg(feature…)]`-Gates | 48 |
| Parser-Abdeckung (geparste `pub`-Deklarationen / Grep-Treffer) | 679 / 679 = 100 % |

## 5. Modulbaum und Dateien

| Modul | Datei | Zeilen |
|---|---|---|
| `budget` | `crates/contextra-store/src/budget.rs` | 91 |
| `compaction::adaptive::cost_based` | `crates/contextra-store/src/compaction/adaptive/cost_based.rs` | 243 |
| `compaction::adaptive::metrics` | `crates/contextra-store/src/compaction/adaptive/metrics.rs` | 80 |
| `compaction::adaptive::planner` | `crates/contextra-store/src/compaction/adaptive/planner.rs` | 68 |
| `compaction::adaptive` | `crates/contextra-store/src/compaction/adaptive.rs` | 15 |
| `compaction::config` | `crates/contextra-store/src/compaction/config.rs` | 101 |
| `compaction::engine::install` | `crates/contextra-store/src/compaction/engine/install.rs` | 18 |
| `compaction::engine::select` | `crates/contextra-store/src/compaction/engine/select.rs` | 39 |
| `compaction::engine::throttle` | `crates/contextra-store/src/compaction/engine/throttle.rs` | 29 |
| `compaction::engine` | `crates/contextra-store/src/compaction/engine.rs` | 966 |
| `compaction::merge_operator` | `crates/contextra-store/src/compaction/merge_operator.rs` | 16 |
| `compaction::retention` | `crates/contextra-store/src/compaction/retention.rs` | 84 |
| `compaction::tests::advanced` | `crates/contextra-store/src/compaction/tests/advanced.rs` | 1044 |
| `compaction::tests::basic` | `crates/contextra-store/src/compaction/tests/basic.rs` | 1196 |
| `compaction::tests::candidate_selection` | `crates/contextra-store/src/compaction/tests/candidate_selection.rs` | 24 |
| `compaction::tests` | `crates/contextra-store/src/compaction/tests.rs` | 41 |
| `compaction` | `crates/contextra-store/src/compaction.rs` | 20 |
| `compat::engine::lsm` | `crates/contextra-store/src/compat/engine/lsm.rs` | 222 |
| `compat::engine::memory_only` | `crates/contextra-store/src/compat/engine/memory_only.rs` | 366 |
| `compat::engine` | `crates/contextra-store/src/compat/engine/mod.rs` | 9 |
| `compat::engine::tenant_storage` | `crates/contextra-store/src/compat/engine/tenant_storage.rs` | 265 |
| `compat::error::tests` | `crates/contextra-store/src/compat/error/tests.rs` | 109 |
| `compat::error` | `crates/contextra-store/src/compat/error.rs` | 46 |
| `compat::license` | `crates/contextra-store/src/compat/license.rs` | 40 |
| `compat` | `crates/contextra-store/src/compat/mod.rs` | 8 |
| `compat::scanner::factory` | `crates/contextra-store/src/compat/scanner/factory.rs` | 75 |
| `compat::scanner::memtable` | `crates/contextra-store/src/compat/scanner/memtable.rs` | 110 |
| `compat::scanner` | `crates/contextra-store/src/compat/scanner/mod.rs` | 26 |
| `compat::scanner::purge` | `crates/contextra-store/src/compat/scanner/purge.rs` | 290 |
| `compat::scanner::runtime` | `crates/contextra-store/src/compat/scanner/runtime.rs` | 123 |
| `compat::scanner::sstable` | `crates/contextra-store/src/compat/scanner/sstable.rs` | 197 |
| `compat::scanner::store_scan_view` | `crates/contextra-store/src/compat/scanner/store_scan_view.rs` | 120 |
| `compat::scanner::tests` | `crates/contextra-store/src/compat/scanner/tests.rs` | 466 |
| `compat::scanner::wal` | `crates/contextra-store/src/compat/scanner/wal.rs` | 178 |
| `engine` | `crates/contextra-store/src/engine.rs` | 3 |
| `error` | `crates/contextra-store/src/error.rs` | 180 |
| `fsync` | `crates/contextra-store/src/fsync.rs` | 128 |
| `io_seam::real_fs` | `crates/contextra-store/src/io_seam/real_fs.rs` | 121 |
| `io_seam::store_io` | `crates/contextra-store/src/io_seam/store_io.rs` | 153 |
| `io_seam::types` | `crates/contextra-store/src/io_seam/types.rs` | 91 |
| `io_seam` | `crates/contextra-store/src/io_seam.rs` | 397 |
| `key_source` | `crates/contextra-store/src/key_source.rs` | 67 |
| `crate` | `crates/contextra-store/src/lib.rs` | 110 |
| `lsm::commit` | `crates/contextra-store/src/lsm/commit.rs` | 143 |
| `lsm::config` | `crates/contextra-store/src/lsm/config.rs` | 155 |
| `lsm::engine::facade` | `crates/contextra-store/src/lsm/engine/facade.rs` | 130 |
| `lsm::engine::migration` | `crates/contextra-store/src/lsm/engine/migration.rs` | 134 |
| `lsm::engine` | `crates/contextra-store/src/lsm/engine.rs` | 434 |
| `lsm::fence` | `crates/contextra-store/src/lsm/fence.rs` | 113 |
| `lsm::flush` | `crates/contextra-store/src/lsm/flush.rs` | 22 |
| `lsm::group_commit` | `crates/contextra-store/src/lsm/group_commit.rs` | 83 |
| `lsm::guard` | `crates/contextra-store/src/lsm/guard.rs` | 17 |
| `lsm::lock_order` | `crates/contextra-store/src/lsm/lock_order.rs` | 107 |
| `lsm` | `crates/contextra-store/src/lsm/mod.rs` | 119 |
| `lsm::observer::adapter` | `crates/contextra-store/src/lsm/observer/adapter.rs` | 50 |
| `lsm::observer::registry` | `crates/contextra-store/src/lsm/observer/registry.rs` | 273 |
| `lsm::observer::types` | `crates/contextra-store/src/lsm/observer/types.rs` | 92 |
| `lsm::observer::worker` | `crates/contextra-store/src/lsm/observer/worker.rs` | 97 |
| `lsm::observer` | `crates/contextra-store/src/lsm/observer.rs` | 31 |
| `lsm::ops::compaction` | `crates/contextra-store/src/lsm/ops/compaction.rs` | 281 |
| `lsm::ops::maintenance` | `crates/contextra-store/src/lsm/ops/maintenance.rs` | 69 |
| `lsm::ops::read` | `crates/contextra-store/src/lsm/ops/read.rs` | 337 |
| `lsm::ops::write::batch` | `crates/contextra-store/src/lsm/ops/write/batch.rs` | 55 |
| `lsm::ops::write::commit_pipeline` | `crates/contextra-store/src/lsm/ops/write/commit_pipeline.rs` | 162 |
| `lsm::ops::write::group` | `crates/contextra-store/src/lsm/ops/write/group.rs` | 319 |
| `lsm::ops::write::guards` | `crates/contextra-store/src/lsm/ops/write/guards.rs` | 125 |
| `lsm::ops::write::put_delete` | `crates/contextra-store/src/lsm/ops/write/put_delete.rs` | 157 |
| `lsm::ops::write::single` | `crates/contextra-store/src/lsm/ops/write/single.rs` | 102 |
| `lsm::ops::write` | `crates/contextra-store/src/lsm/ops/write.rs` | 19 |
| `lsm::ops` | `crates/contextra-store/src/lsm/ops.rs` | 14 |
| `lsm::recovery::rollback` | `crates/contextra-store/src/lsm/recovery/rollback.rs` | 228 |
| `lsm::recovery::salt` | `crates/contextra-store/src/lsm/recovery/salt.rs` | 138 |
| `lsm::recovery::startup` | `crates/contextra-store/src/lsm/recovery/startup.rs` | 584 |
| `lsm::recovery` | `crates/contextra-store/src/lsm/recovery.rs` | 18 |
| `lsm::scan` | `crates/contextra-store/src/lsm/scan.rs` | 212 |
| `lsm::tests::commit_tests::basic` | `crates/contextra-store/src/lsm/tests/commit_tests/basic.rs` | 300 |
| `lsm::tests::commit_tests::durability` | `crates/contextra-store/src/lsm/tests/commit_tests/durability.rs` | 215 |
| `lsm::tests::commit_tests` | `crates/contextra-store/src/lsm/tests/commit_tests.rs` | 10 |
| `lsm::tests::flush_tests` | `crates/contextra-store/src/lsm/tests/flush_tests.rs` | 257 |
| `lsm::tests::mvcc_tests::isolation` | `crates/contextra-store/src/lsm/tests/mvcc_tests/isolation.rs` | 339 |
| `lsm::tests::mvcc_tests::put_if_absent` | `crates/contextra-store/src/lsm/tests/mvcc_tests/put_if_absent.rs` | 272 |
| `lsm::tests::mvcc_tests` | `crates/contextra-store/src/lsm/tests/mvcc_tests.rs` | 10 |
| `lsm::tests::recovery_tests::rollback` | `crates/contextra-store/src/lsm/tests/recovery_tests/rollback.rs` | 403 |
| `lsm::tests::recovery_tests::startup` | `crates/contextra-store/src/lsm/tests/recovery_tests/startup.rs` | 341 |
| `lsm::tests::recovery_tests` | `crates/contextra-store/src/lsm/tests/recovery_tests.rs` | 13 |
| `lsm::tests::scan_tests::bounded` | `crates/contextra-store/src/lsm/tests/scan_tests/bounded.rs` | 249 |
| `lsm::tests::scan_tests::prefix` | `crates/contextra-store/src/lsm/tests/scan_tests/prefix.rs` | 193 |
| `lsm::tests::scan_tests::prop` | `crates/contextra-store/src/lsm/tests/scan_tests/prop.rs` | 111 |
| `lsm::tests::scan_tests` | `crates/contextra-store/src/lsm/tests/scan_tests.rs` | 13 |
| `lsm::tests` | `crates/contextra-store/src/lsm/tests.rs` | 37 |
| `lsm::validate` | `crates/contextra-store/src/lsm/validate.rs` | 55 |
| `manifest::apply` | `crates/contextra-store/src/manifest/apply.rs` | 61 |
| `manifest::cleanup` | `crates/contextra-store/src/manifest/cleanup.rs` | 109 |
| `manifest::core` | `crates/contextra-store/src/manifest/core.rs` | 156 |
| `manifest::entry` | `crates/contextra-store/src/manifest/entry.rs` | 370 |
| `manifest::handle` | `crates/contextra-store/src/manifest/handle.rs` | 111 |
| `manifest` | `crates/contextra-store/src/manifest/mod.rs` | 27 |
| `manifest::recovery` | `crates/contextra-store/src/manifest/recovery.rs` | 130 |
| `manifest::rollover` | `crates/contextra-store/src/manifest/rollover.rs` | 197 |
| `manifest::tests` | `crates/contextra-store/src/manifest/tests.rs` | 444 |
| `memtable::iter` | `crates/contextra-store/src/memtable/iter.rs` | 180 |
| `memtable::mvcc` | `crates/contextra-store/src/memtable/mvcc.rs` | 174 |
| `memtable::shard` | `crates/contextra-store/src/memtable/shard.rs` | 144 |
| `memtable::tests` | `crates/contextra-store/src/memtable/tests.rs` | 478 |
| `memtable::tombstone` | `crates/contextra-store/src/memtable/tombstone.rs` | 28 |
| `memtable` | `crates/contextra-store/src/memtable.rs` | 103 |
| `scanner` | `crates/contextra-store/src/scanner/mod.rs` | 3 |
| `sstable::block_cache::lru` | `crates/contextra-store/src/sstable/block_cache/lru.rs` | 75 |
| `sstable::block_cache` | `crates/contextra-store/src/sstable/block_cache/mod.rs` | 178 |
| `sstable::block_cache::quick` | `crates/contextra-store/src/sstable/block_cache/quick.rs` | 68 |
| `sstable::block_cache::sieve` | `crates/contextra-store/src/sstable/block_cache/sieve.rs` | 126 |
| `sstable::block_cache::traits` | `crates/contextra-store/src/sstable/block_cache/traits.rs` | 38 |
| `sstable::block_search` | `crates/contextra-store/src/sstable/block_search.rs` | 224 |
| `sstable::bloom` | `crates/contextra-store/src/sstable/bloom.rs` | 177 |
| `sstable::builder::block` | `crates/contextra-store/src/sstable/builder/block.rs` | 180 |
| `sstable::builder::meta` | `crates/contextra-store/src/sstable/builder/meta.rs` | 17 |
| `sstable::builder` | `crates/contextra-store/src/sstable/builder/mod.rs` | 12 |
| `sstable::builder::writer` | `crates/contextra-store/src/sstable/builder/writer.rs` | 436 |
| `sstable::format` | `crates/contextra-store/src/sstable/format.rs` | 204 |
| `sstable::io` | `crates/contextra-store/src/sstable/io.rs` | 25 |
| `sstable::reader::block` | `crates/contextra-store/src/sstable/reader/block.rs` | 99 |
| `sstable::reader::iter` | `crates/contextra-store/src/sstable/reader/iter.rs` | 129 |
| `sstable::reader::lookup` | `crates/contextra-store/src/sstable/reader/lookup.rs` | 219 |
| `sstable::reader` | `crates/contextra-store/src/sstable/reader/mod.rs` | 271 |
| `sstable::reader::open` | `crates/contextra-store/src/sstable/reader/open.rs` | 451 |
| `sstable::reader_ext` | `crates/contextra-store/src/sstable/reader_ext.rs` | 303 |
| `sstable::stream` | `crates/contextra-store/src/sstable/stream.rs` | 138 |
| `sstable::tests` | `crates/contextra-store/src/sstable/tests.rs` | 918 |
| `sstable` | `crates/contextra-store/src/sstable.rs` | 22 |
| `stats` | `crates/contextra-store/src/stats.rs` | 12 |
| `system_pressure::calculation` | `crates/contextra-store/src/system_pressure/calculation.rs` | 55 |
| `system_pressure::monitor` | `crates/contextra-store/src/system_pressure/monitor.rs` | 188 |
| `system_pressure::signal` | `crates/contextra-store/src/system_pressure/signal.rs` | 36 |
| `system_pressure` | `crates/contextra-store/src/system_pressure.rs` | 175 |
| `tenant_codec::storage_engine` | `crates/contextra-store/src/tenant_codec/storage_engine.rs` | 3 |
| `tenant_codec::tests` | `crates/contextra-store/src/tenant_codec/tests.rs` | 43 |
| `tenant_codec` | `crates/contextra-store/src/tenant_codec.rs` | 200 |
| `traits` | `crates/contextra-store/src/traits.rs` | 139 |
| `util` | `crates/contextra-store/src/util.rs` | 155 |
| `wal::config` | `crates/contextra-store/src/wal/config.rs` | 59 |
| `wal::core` | `crates/contextra-store/src/wal/core.rs` | 667 |
| `wal::encode` | `crates/contextra-store/src/wal/encode.rs` | 447 |
| `wal::flusher::command` | `crates/contextra-store/src/wal/flusher/command.rs` | 65 |
| `wal::flusher` | `crates/contextra-store/src/wal/flusher.rs` | 771 |
| `wal::fs` | `crates/contextra-store/src/wal/fs.rs` | 267 |
| `wal::hmac` | `crates/contextra-store/src/wal/hmac.rs` | 570 |
| `wal::io::acl` | `crates/contextra-store/src/wal/io/acl.rs` | 63 |
| `wal::io::append` | `crates/contextra-store/src/wal/io/append.rs` | 317 |
| `wal::io` | `crates/contextra-store/src/wal/io/mod.rs` | 12 |
| `wal::io::scan` | `crates/contextra-store/src/wal/io/scan.rs` | 467 |
| `wal::io::truncate` | `crates/contextra-store/src/wal/io/truncate.rs` | 202 |
| `wal` | `crates/contextra-store/src/wal/mod.rs` | 34 |
| `wal::open_heal_tests` | `crates/contextra-store/src/wal/open_heal_tests.rs` | 363 |
| `wal::poison` | `crates/contextra-store/src/wal/poison.rs` | 45 |
| `wal::replay` | `crates/contextra-store/src/wal/replay.rs` | 729 |
| `wal::tests::encode_tests` | `crates/contextra-store/src/wal/tests/encode_tests.rs` | 259 |
| `wal::tests::flusher_tests` | `crates/contextra-store/src/wal/tests/flusher_tests.rs` | 406 |
| `wal::tests::hmac_tests` | `crates/contextra-store/src/wal/tests/hmac_tests.rs` | 416 |
| `wal::tests::io_tests` | `crates/contextra-store/src/wal/tests/io_tests.rs` | 572 |
| `wal::tests::replay_tests` | `crates/contextra-store/src/wal/tests/replay_tests.rs` | 1622 |
| `wal::tests` | `crates/contextra-store/src/wal/tests.rs` | 18 |

## 6. Re-Exports (`pub use`)

- `pub use adaptive::*` — `crates/contextra-store/src/compaction.rs:12` (Modul `compaction`)
- `pub use config::*` — `crates/contextra-store/src/compaction.rs:13` (Modul `compaction`)
- `pub use engine::*` — `crates/contextra-store/src/compaction.rs:14` (Modul `compaction`)
- `pub use merge_operator::*` — `crates/contextra-store/src/compaction.rs:15` (Modul `compaction`)
- `pub use retention::*` — `crates/contextra-store/src/compaction.rs:16` (Modul `compaction`)
- `pub use cost_based::*` — `crates/contextra-store/src/compaction/adaptive.rs:13` (Modul `compaction::adaptive`)
- `pub use metrics::*` — `crates/contextra-store/src/compaction/adaptive.rs:14` (Modul `compaction::adaptive`)
- `pub use planner::*` — `crates/contextra-store/src/compaction/adaptive.rs:15` (Modul `compaction::adaptive`)
- `pub use lsm::*` — `crates/contextra-store/src/compat/engine/mod.rs:8` (Modul `compat::engine`)
- `pub use tenant_storage::*` — `crates/contextra-store/src/compat/engine/mod.rs:9` (Modul `compat::engine`)
- `pub use contextra_ports::license::FeatureRing` — `crates/contextra-store/src/compat/license.rs:4` (Modul `compat::license`)
- `pub use memtable::MemtableScanner` — `crates/contextra-store/src/compat/scanner/mod.rs:17` (Modul `compat::scanner`)
- `pub use sstable::SstableScanner` — `crates/contextra-store/src/compat/scanner/mod.rs:18` (Modul `compat::scanner`)
- `pub use wal::WalScanner` — `crates/contextra-store/src/compat/scanner/mod.rs:19` (Modul `compat::scanner`)
- `pub use runtime::ScannerRuntime` — `crates/contextra-store/src/compat/scanner/mod.rs:20` (Modul `compat::scanner`)
- `pub use store_scan_view::StoreScanView` — `crates/contextra-store/src/compat/scanner/mod.rs:21` (Modul `compat::scanner`)
- `pub use purge::LsmPurger` — `crates/contextra-store/src/compat/scanner/mod.rs:22` (Modul `compat::scanner`)
- `pub use factory::` — `crates/contextra-store/src/compat/scanner/mod.rs:23` (Modul `compat::scanner`)
- `pub use crate::compat::engine::*` — `crates/contextra-store/src/engine.rs:3` (Modul `engine`)
- `pub use error::` — `crates/contextra-store/src/lib.rs:56` (Modul `crate`)
- `pub use util::` — `crates/contextra-store/src/lib.rs:85` (Modul `crate`)
- `pub use compaction::` — `crates/contextra-store/src/lib.rs:88` (Modul `crate`)
- `pub use key_source::` — `crates/contextra-store/src/lib.rs:96` (Modul `crate`)
- `pub use lsm::` — `crates/contextra-store/src/lib.rs:98` (Modul `crate`)
- `pub use manifest::` — `crates/contextra-store/src/lib.rs:103` (Modul `crate`)
- `pub use system_pressure::` — `crates/contextra-store/src/lib.rs:105` (Modul `crate`)
- `pub use tenant_codec::` — `crates/contextra-store/src/lib.rs:107` (Modul `crate`)
- `pub use io_seam::` — `crates/contextra-store/src/lib.rs:110` (Modul `crate`)
- `pub use crate::compat::license::*` — `crates/contextra-store/src/lsm/config.rs:155` (Modul `lsm::config`)
- `pub use config::` — `crates/contextra-store/src/lsm/mod.rs:91` (Modul `lsm`)
- `pub use engine::` — `crates/contextra-store/src/lsm/mod.rs:92` (Modul `lsm`)
- `pub use observer::` — `crates/contextra-store/src/lsm/mod.rs:95` (Modul `lsm`)
- `pub use adapter::AsyncObserverAdapter` — `crates/contextra-store/src/lsm/observer.rs:15` (Modul `lsm::observer`)
- `pub use registry::ObserverRegistry` — `crates/contextra-store/src/lsm/observer.rs:16` (Modul `lsm::observer`)
- `pub use types::` — `crates/contextra-store/src/lsm/observer.rs:17` (Modul `lsm::observer`)
- `pub use crate::compat::engine::*` — `crates/contextra-store/src/lsm/ops.rs:14` (Modul `lsm::ops`)
- `pub use rollback::` — `crates/contextra-store/src/lsm/recovery.rs:18` (Modul `lsm::recovery`)
- `pub use super::*` — `crates/contextra-store/src/lsm/tests.rs:5` (Modul `lsm::tests`)
- `pub use crate::compaction::` — `crates/contextra-store/src/lsm/tests.rs:6` (Modul `lsm::tests`)
- `pub use contextra_ids::TOMBSTONE_BIT` — `crates/contextra-store/src/lsm/tests.rs:7` (Modul `lsm::tests`)
- `pub use std::sync::Arc` — `crates/contextra-store/src/lsm/tests.rs:8` (Modul `lsm::tests`)
- `pub use std::time::Duration` — `crates/contextra-store/src/lsm/tests.rs:9` (Modul `lsm::tests`)
- `pub use super::handle::Manifest` — `crates/contextra-store/src/manifest/core.rs:23` (Modul `manifest::core`)
- `pub use core::Manifest` — `crates/contextra-store/src/manifest/mod.rs:18` (Modul `manifest`)
- `pub use entry::ManifestEntry` — `crates/contextra-store/src/manifest/mod.rs:19` (Modul `manifest`)
- `pub use core::` — `crates/contextra-store/src/manifest/mod.rs:27` (Modul `manifest`)
- `pub use shard::` — `crates/contextra-store/src/memtable.rs:21` (Modul `memtable`)
- `pub use tombstone::` — `crates/contextra-store/src/memtable.rs:26` (Modul `memtable`)
- `pub use contextra_ids::TOMBSTONE_BIT` — `crates/contextra-store/src/memtable/tombstone.rs:6` (Modul `memtable::tombstone`)
- `pub use crate::compat::scanner::*` — `crates/contextra-store/src/scanner/mod.rs:3` (Modul `scanner`)
- `pub use block_cache::*` — `crates/contextra-store/src/sstable.rs:16` (Modul `sstable`)
- `pub use block_search::*` — `crates/contextra-store/src/sstable.rs:17` (Modul `sstable`)
- `pub use bloom::*` — `crates/contextra-store/src/sstable.rs:18` (Modul `sstable`)
- `pub use builder::*` — `crates/contextra-store/src/sstable.rs:19` (Modul `sstable`)
- `pub use format::*` — `crates/contextra-store/src/sstable.rs:20` (Modul `sstable`)
- `pub use reader::*` — `crates/contextra-store/src/sstable.rs:21` (Modul `sstable`)
- `pub use stream::*` — `crates/contextra-store/src/sstable.rs:22` (Modul `sstable`)
- `pub use lru::LruBlockCacheBackend` — `crates/contextra-store/src/sstable/block_cache/mod.rs:11` (Modul `sstable::block_cache`)
- `pub use quick::QuickCacheBlockCacheBackend` — `crates/contextra-store/src/sstable/block_cache/mod.rs:13` (Modul `sstable::block_cache`)
- `pub use sieve::SieveCacheBackend` — `crates/contextra-store/src/sstable/block_cache/mod.rs:14` (Modul `sstable::block_cache`)
- `pub use traits::BlockCacheBackend` — `crates/contextra-store/src/sstable/block_cache/mod.rs:15` (Modul `sstable::block_cache`)
- `pub use super::format::` — `crates/contextra-store/src/sstable/block_cache/mod.rs:17` (Modul `sstable::block_cache`)
- `pub use block::BlockBuilder` — `crates/contextra-store/src/sstable/builder/mod.rs:10` (Modul `sstable::builder`)
- `pub use meta::SstableMetadata` — `crates/contextra-store/src/sstable/builder/mod.rs:11` (Modul `sstable::builder`)
- `pub use writer::SstableBuilder` — `crates/contextra-store/src/sstable/builder/mod.rs:12` (Modul `sstable::builder`)
- `pub use calculation::*` — `crates/contextra-store/src/system_pressure.rs:11` (Modul `system_pressure`)
- `pub use monitor::*` — `crates/contextra-store/src/system_pressure.rs:12` (Modul `system_pressure`)
- `pub use signal::*` — `crates/contextra-store/src/system_pressure.rs:13` (Modul `system_pressure`)
- `pub use crate::compat::engine::tenant_storage::*` — `crates/contextra-store/src/tenant_codec.rs:116` (Modul `tenant_codec`)
- `pub use crate::compat::engine::tenant_storage::*` — `crates/contextra-store/src/tenant_codec/storage_engine.rs:3` (Modul `tenant_codec::storage_engine`)
- `pub use tokio::fs::*` — `crates/contextra-store/src/wal/fs.rs:6` (Modul `wal::fs`)
- `pub use loom_fs::*` — `crates/contextra-store/src/wal/fs.rs:267` (Modul `wal::fs`)
- `pub use contextra_crypto::crypto::KeyManager` — `crates/contextra-store/src/wal/mod.rs:9` (Modul `wal`)
- `pub use config::WalConfig` — `crates/contextra-store/src/wal/mod.rs:28` (Modul `wal`)
- `pub use core::*` — `crates/contextra-store/src/wal/mod.rs:29` (Modul `wal`)
- `pub use encode::*` — `crates/contextra-store/src/wal/mod.rs:30` (Modul `wal`)
- `pub use flusher::*` — `crates/contextra-store/src/wal/mod.rs:31` (Modul `wal`)
- `pub use hmac::*` — `crates/contextra-store/src/wal/mod.rs:32` (Modul `wal`)
- `pub use poison::PoisonCause` — `crates/contextra-store/src/wal/mod.rs:33` (Modul `wal`)
- `pub use replay::*` — `crates/contextra-store/src/wal/mod.rs:34` (Modul `wal`)
- `pub use super::*` — `crates/contextra-store/src/wal/tests.rs:5` (Modul `wal::tests`)

## 8. Öffentliche Schnittstelle je Item

### Modul `budget`

#### `struct ResourceBudget`
- Ort: `crates/contextra-store/src/budget.rs:8`
```rust
pub struct ResourceBudget
```
Dokumentation: Resource budget configuration.

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `memory_limit` | `usize` | `crates/contextra-store/src/budget.rs:10` |

#### `struct ResourceTracker`
- Ort: `crates/contextra-store/src/budget.rs:15`
```rust
pub struct ResourceTracker
```
Dokumentation: Thread-safe tracker for memory allocations.
- Impl-Block (Trait `Default`) `crates/contextra-store/src/budget.rs:20`
- Impl-Block `crates/contextra-store/src/budget.rs:26`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new(budget: ResourceBudget) -> Self` | `crates/contextra-store/src/budget.rs:28` | Creates a new ResourceTracker with the specified budget. |
| `pub fn consume_memory(&self, bytes: u64) -> StoreResult<()>` | `crates/contextra-store/src/budget.rs:36` | Consumes memory, failing if limit is exceeded. |
| `pub fn release_memory(&self, bytes: u64)` | `crates/contextra-store/src/budget.rs:63` | Releases previously consumed memory. |
| `pub fn memory_used(&self) -> u64` | `crates/contextra-store/src/budget.rs:80` | Returns the current memory used in bytes. |
| `pub fn has_memory_capacity(&self) -> bool` | `crates/contextra-store/src/budget.rs:85` | Returns true if memory limit is not reached. |

### Modul `compaction`

#### `mod adaptive`
- Ort: `crates/contextra-store/src/compaction.rs:3`
```rust
pub mod adaptive
```

#### `mod merge_operator`
- Ort: `crates/contextra-store/src/compaction.rs:6`
```rust
pub mod merge_operator
```

#### `mod retention`
- Ort: `crates/contextra-store/src/compaction.rs:7`
```rust
pub mod retention
```

#### `struct TtlMetadata`
- Ort: `crates/contextra-store/src/compaction.rs:20`
```rust
pub struct TtlMetadata
```
Dokumentation: Removed TTL feature stub per Decision Option B.

### Modul `compaction::adaptive`

#### `mod cost_based`
- Ort: `crates/contextra-store/src/compaction/adaptive.rs:9`
```rust
pub mod cost_based
```

#### `mod metrics`
- Ort: `crates/contextra-store/src/compaction/adaptive.rs:10`
```rust
pub mod metrics
```

#### `mod planner`
- Ort: `crates/contextra-store/src/compaction/adaptive.rs:11`
```rust
pub mod planner
```

### Modul `compaction::adaptive::cost_based`

#### `struct CostBasedAdaptivePlanner`
- Ort: `crates/contextra-store/src/compaction/adaptive/cost_based.rs:17`
```rust
pub struct CostBasedAdaptivePlanner
```
Dokumentation: Reference implementation of cost-based adaptive compaction (EcoTune / ArceKV inspired). Adjusts compaction aggressiveness based on read-vs-write workload ratio to minimize read amplification.
- Impl-Block `crates/contextra-store/src/compaction/adaptive/cost_based.rs:23`
- Impl-Block (Trait `Default`) `crates/contextra-store/src/compaction/adaptive/cost_based.rs:61`
- Impl-Block (Trait `AdaptiveCompactionPlanner`) `crates/contextra-store/src/compaction/adaptive/cost_based.rs:67`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new(read_ratio_threshold: f64, min_sstables_per_tier: usize, size_ratio: f64) -> Self` | `crates/contextra-store/src/compaction/adaptive/cost_based.rs:25` | Creates a new CostBasedAdaptivePlanner with configurable read ratio threshold. |
| `pub fn validate_tombstone_safety( candidates: &[Arc<SstableReader>], min_active_seqno: u64, is_full_compaction: bool, ) -> bool` | `crates/contextra-store/src/compaction/adaptive/cost_based.rs:36` | Hard invariant verification enforcing INV-COMPACTION-ADAPTIVE-1. Validates that no candidate tombstone with `seq >= min_active_seqno` can ever be garbage collec |

#### `fn compute_key_overlap_score`
- Ort: `crates/contextra-store/src/compaction/adaptive/cost_based.rs:137`
```rust
pub fn compute_key_overlap_score(window: &[Arc<SstableReader>]) -> usize
```
Dokumentation: Computes a score representing key range contiguity/overlap among SSTables in a window. Returns the number of adjacent overlapping pairs when sorted by first_key.

#### `fn select_stcs_candidates`
- Ort: `crates/contextra-store/src/compaction/adaptive/cost_based.rs:155`
```rust
pub fn select_stcs_candidates( ssts: &[Arc<SstableReader>], min_sstables_per_tier: usize, size_ratio: f64, ) -> Option<Vec<Arc<SstableReader>>>
```
Dokumentation: Helper function to perform standard Size-Tiered Compaction candidate selection. Prefers contiguous key ranges (overlapping or adjacent key spaces) when evaluating tiers.

### Modul `compaction::adaptive::metrics`

#### `struct WorkloadMetrics`
- Ort: `crates/contextra-store/src/compaction/adaptive/metrics.rs:11`
```rust
pub struct WorkloadMetrics
```
Dokumentation: Operational workload metrics captured deterministically without wall-clock time (P28 compliance). All trigger decisions are strictly derived from transaction/sequence numbers and operational counts.
- Impl-Block (Trait `Default`) `crates/contextra-store/src/compaction/adaptive/metrics.rs:17`
- Impl-Block `crates/contextra-store/src/compaction/adaptive/metrics.rs:23`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new() -> Self` | `crates/contextra-store/src/compaction/adaptive/metrics.rs:25` | Creates a new WorkloadMetrics counter instance initialized to zero. |
| `pub fn record_read(&self)` | `crates/contextra-store/src/compaction/adaptive/metrics.rs:35` | Records a read operation atomically. |
| `pub fn record_write(&self, seq_no: u64)` | `crates/contextra-store/src/compaction/adaptive/metrics.rs:41` | Records a write operation and updates the latest sequence number atomically. |
| `pub fn snapshot(&self) -> WorkloadMetricsSnapshot` | `crates/contextra-store/src/compaction/adaptive/metrics.rs:47` | Takes a deterministic snapshot of current metrics for compaction strategy evaluation. |

#### `struct WorkloadMetricsSnapshot`
- Ort: `crates/contextra-store/src/compaction/adaptive/metrics.rs:58`
```rust
pub struct WorkloadMetricsSnapshot
```
Dokumentation: Point-in-time deterministic snapshot of operational workload counters.
- Impl-Block `crates/contextra-store/src/compaction/adaptive/metrics.rs:67`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `read_count` | `u64` | `crates/contextra-store/src/compaction/adaptive/metrics.rs:60` |
| `write_count` | `u64` | `crates/contextra-store/src/compaction/adaptive/metrics.rs:62` |
| `last_seq_no` | `u64` | `crates/contextra-store/src/compaction/adaptive/metrics.rs:64` |

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn read_write_ratio(&self) -> f64` | `crates/contextra-store/src/compaction/adaptive/metrics.rs:69` | Computes the read/write operational ratio safely handling zero writes. |

### Modul `compaction::adaptive::planner`

#### `struct AdaptiveCompactionPlan`
- Ort: `crates/contextra-store/src/compaction/adaptive/planner.rs:24`
```rust
pub struct AdaptiveCompactionPlan
```
Dokumentation: Resulting plan proposed by adaptive compaction planner.
- Impl-Block (Trait `PartialEq`) `crates/contextra-store/src/compaction/adaptive/planner.rs:33`
- Impl-Block (Trait `Eq`) `crates/contextra-store/src/compaction/adaptive/planner.rs:46`
- Impl-Block (Trait `std::fmt::Debug`) `crates/contextra-store/src/compaction/adaptive/planner.rs:48`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `strategy` | `CompactionStrategy` | `crates/contextra-store/src/compaction/adaptive/planner.rs:26` |
| `candidates` | `Vec<Arc<SstableReader>>` | `crates/contextra-store/src/compaction/adaptive/planner.rs:28` |
| `is_full_compaction` | `bool` | `crates/contextra-store/src/compaction/adaptive/planner.rs:30` |

#### `enum CompactionStrategy`
- Ort: `crates/contextra-store/src/compaction/adaptive/planner.rs:13`
```rust
pub enum CompactionStrategy
```
Dokumentation: Dynamic compaction strategy chosen by the adaptive planner.

Varianten:

- `WriteOptimizedSTCS` — `crates/contextra-store/src/compaction/adaptive/planner.rs:15`
- `ReadOptimizedAggressive` — `crates/contextra-store/src/compaction/adaptive/planner.rs:17`
- `Balanced` — `crates/contextra-store/src/compaction/adaptive/planner.rs:19`

#### `trait AdaptiveCompactionPlanner`
- Ort: `crates/contextra-store/src/compaction/adaptive/planner.rs:59`
```rust
pub trait AdaptiveCompactionPlanner: Send + Sync
```
Dokumentation: Trait defining adaptive compaction strategy selection.

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `fn plan_compaction( &self, stats: &StorageStats, metrics: &WorkloadMetricsSnapshot, sstables: &[Arc<SstableReader>], min_active_seqno: u64, ) -> Result<Option<AdaptiveCompactionPlan>>` | `crates/contextra-store/src/compaction/adaptive/planner.rs:61` | Derives a compaction decision based on storage stats and operational workload metrics. |

### Modul `compaction::config`

#### `struct CompactionConfig`
- Ort: `crates/contextra-store/src/compaction/config.rs:46`
```rust
pub struct CompactionConfig
```
Dokumentation: Configuration for the compaction engine.
- Impl-Block (Trait `Default`) `crates/contextra-store/src/compaction/config.rs:82`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `min_sstables_per_tier` | `usize` | `crates/contextra-store/src/compaction/config.rs:48` |
| `l0_compaction_trigger` | `usize` | `crates/contextra-store/src/compaction/config.rs:50` |
| `level_size_multiplier` | `f64` | `crates/contextra-store/src/compaction/config.rs:52` |
| `size_ratio` | `f64` | `crates/contextra-store/src/compaction/config.rs:54` |
| `check_interval` | `Duration` | `crates/contextra-store/src/compaction/config.rs:56` |
| `yield_threshold` | `usize` | `crates/contextra-store/src/compaction/config.rs:58` |
| `max_memory_bytes` | `Option<u64>` | `crates/contextra-store/src/compaction/config.rs:60` |
| `max_peak_memory_bytes` | `Option<u64>` | `crates/contextra-store/src/compaction/config.rs:63` |
| `max_backpressure_wait` | `Duration` | `crates/contextra-store/src/compaction/config.rs:65` |
| `max_io_bytes_per_second` | `Option<u64>` | `crates/contextra-store/src/compaction/config.rs:71` |
| `enable_adaptive_compaction` | `bool` | `crates/contextra-store/src/compaction/config.rs:73` |
| `adaptive_read_ratio_threshold` | `f64` | `crates/contextra-store/src/compaction/config.rs:75` |
| `merge_max_fuel` | `u64` | `crates/contextra-store/src/compaction/config.rs:77` |
| `merge_wall_clock_timeout` | `Duration` | `crates/contextra-store/src/compaction/config.rs:79` |

### Modul `compaction::engine`

#### `mod install`
- Ort: `crates/contextra-store/src/compaction/engine.rs:7`
```rust
pub mod install
```

#### `mod select`
- Ort: `crates/contextra-store/src/compaction/engine.rs:5`
```rust
pub mod select
```

#### `mod throttle`
- Ort: `crates/contextra-store/src/compaction/engine.rs:6`
```rust
pub mod throttle
```

#### `struct CompactionEngine`
- Ort: `crates/contextra-store/src/compaction/engine.rs:30`
```rust
pub struct CompactionEngine
```
Dokumentation: Orchestriert die periodische Verdichtung und Bereinigung von SSTables.
- Impl-Block `crates/contextra-store/src/compaction/engine.rs:46`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `snapshot_registry` | `Arc<SnapshotRegistry>` | `crates/contextra-store/src/compaction/engine.rs:32` |
| `vfs` | `Arc<dyn Vfs>` | `crates/contextra-store/src/compaction/engine.rs:43` |

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new( config: CompactionConfig, snapshot_registry: Arc<SnapshotRegistry>, block_cache: Arc<BlockCache>, key_manager: Option<Arc<KeyManager>>, budget: Arc<crate::budget::ResourceTracker>, manifest: Option<Arc<crate::manifest::Manifest>>, ) -> Self` | `crates/contextra-store/src/compaction/engine.rs:48` | Creates a new compaction engine. |
| `pub fn with_vfs(mut self, vfs: Arc<dyn Vfs>) -> Self` | `crates/contextra-store/src/compaction/engine.rs:85` | Configures a custom VFS implementation for compaction file operations. |
| `pub fn with_merge_operator(mut self, merge_operator: Arc<dyn MergeOperator>) -> Self` | `crates/contextra-store/src/compaction/engine.rs:91` | Attaches a custom merge operator for compaction value merging (§4.12). |
| `pub fn with_clock(mut self, clock: Arc<dyn contextra_env::Clock>) -> Self` | `crates/contextra-store/src/compaction/engine.rs:97` | Attaches a custom clock port for deterministic execution timing (P28). |
| `pub fn with_pressure_rx( mut self, rx: tokio::sync::watch::Receiver<crate::system_pressure::SystemPressure>, ) -> Self` | `crates/contextra-store/src/compaction/engine.rs:103` | Attaches a system pressure watch receiver to enable pressure-aware compaction backpressure. |
| `pub fn with_adaptive_planner(mut self, planner: Arc<dyn AdaptiveCompactionPlanner>) -> Self` | `crates/contextra-store/src/compaction/engine.rs:112` | Attaches a custom adaptive compaction planner. |
| `pub fn record_read_op(&self)` | `crates/contextra-store/src/compaction/engine.rs:119` | Records a read operation for workload tracking. |
| `pub fn record_write_op(&self, seq_no: u64)` | `crates/contextra-store/src/compaction/engine.rs:125` | Records a write operation for workload tracking. |
| `pub async fn maybe_compact( &self, sstables: &RwLock<Vec<Arc<SstableReader>>>, data_path: &std::path::Path, ) -> Result<bool>` | `crates/contextra-store/src/compaction/engine.rs:134` |  |
| `pub async fn maybe_compact_with_cancel( &self, sstables: &RwLock<Vec<Arc<SstableReader>>>, data_path: &std::path::Path, cancel_token: Option<&tokio_util::sync::CancellationToken>, ) -> Result<bool>` | `crates/contextra-store/src/compaction/engine.rs:144` | Evaluates whether compaction should run and performs it with an optional cancellation token. |
| `pub async fn compact_all( &self, sstables: &RwLock<Vec<Arc<SstableReader>>>, data_path: &std::path::Path, ) -> Result<bool>` | `crates/contextra-store/src/compaction/engine.rs:226` | Forces a full compaction over all existing SSTables. This maintenance API merges all currently registered SSTables into a single consolidated SSTable, evaluatin |
| `pub async fn compact_range( &self, range: std::ops::RangeInclusive<&[u8]>, sstables: &RwLock<Vec<Arc<SstableReader>>>, data_path: &std::path::Path, ) -> Result<bool>` | `crates/contextra-store/src/compaction/engine.rs:248` | Compacts all SSTables that overlap with the specified key range `[start, end]`. Neutral maintenance API without domain-specific erasure/shredding semantics. |
| `pub fn select_compaction_candidates( &self, ssts: &[Arc<SstableReader>], ) -> Option<Vec<Arc<SstableReader>>>` | `crates/contextra-store/src/compaction/engine.rs:467` | Selects SSTables to compact using Size-Tiered strategy. Groups by size class and returns the group that meets the threshold, prioritizing key range overlap and  |
| `pub async fn merge_sstables( &self, inputs: &[Arc<SstableReader>], output_path: &std::path::Path, min_snapshot_seq: u64, is_full_compaction: bool, ) -> Result<()>` | `crates/contextra-store/src/compaction/engine.rs:491` | Performs a multi-way merge of input SSTables into a single output SSTable. During merge: - Duplicate keys: newest sequence number wins - Tombstones: removed if  |
| `pub async fn merge_sstables_with_cancel( &self, inputs: &[Arc<SstableReader>], output_path: &std::path::Path, min_snapshot_seq: u64, is_full_compaction: bool, cancel_token: Option<&tokio_util::sync::CancellationToken>, ) -> Result<()>` | `crates/contextra-store/src/compaction/engine.rs:509` | Performs a multi-way merge with an optional cancellation token. |
| `pub fn generate_sst_path(&self, data_path: &std::path::Path) -> Result<PathBuf>` | `crates/contextra-store/src/compaction/engine.rs:903` | Generates a unique SSTable file path using a monotonic counter. |
| `pub async fn run_loop( self: Arc<Self>, sstables: Arc<RwLock<Vec<Arc<SstableReader>>>>, data_path: PathBuf, shutdown: tokio_util::sync::CancellationToken, )` | `crates/contextra-store/src/compaction/engine.rs:911` | Runs the background compaction loop. Periodically checks if compaction is needed and performs it. Designed to be spawned via `tokio::spawn`. |

### Modul `compaction::engine::install`

#### `fn generate_sst_path`
- Ort: `crates/contextra-store/src/compaction/engine/install.rs:10`
```rust
pub fn generate_sst_path( data_path: &Path, global_counter: &AtomicU64, instance_counter: &AtomicU64, ) -> Result<PathBuf>
```
Dokumentation: Generiert einen eindeutigen Pfad für eine neue SSTable-Datei unter Verwendung monotoner Zähler.

### Modul `compaction::engine::select`

#### `fn estimate_compaction_peak_bytes`
- Ort: `crates/contextra-store/src/compaction/engine/select.rs:29`
```rust
pub fn estimate_compaction_peak_bytes(candidates: &[Arc<SstableReader>]) -> u64
```
Dokumentation: Schätzt den Spitzen-Speicherverbrauch einer Kompaktierung anhand der Kandidatengrößen.

#### `fn select_compaction_candidates`
- Ort: `crates/contextra-store/src/compaction/engine/select.rs:12`
```rust
pub fn select_compaction_candidates( sstables: &[Arc<SstableReader>], max_count: usize, ) -> Vec<Arc<SstableReader>>
```
Dokumentation: Wählt die besten SSTable-Kandidaten für einen Kompaktierungslauf aus. Folgt chronologischer Reihenfolge (kleinste min_tx_id zuerst) und garantiert Kontiguität der ausgewählten Sequenz.

### Modul `compaction::engine::throttle`

#### `fn calculate_throttle_delay`
- Ort: `crates/contextra-store/src/compaction/engine/throttle.rs:10`
```rust
pub fn calculate_throttle_delay( pressure_rx: Option<&Receiver<SystemPressure>>, ) -> Option<Duration>
```
Dokumentation: Berechnet eine Drossel-Verzögerung basierend auf dem aktuellen Systemdruck.

### Modul `compaction::merge_operator`

#### `trait MergeOperator`
- Ort: `crates/contextra-store/src/compaction/merge_operator.rs:7`
```rust
pub trait MergeOperator: Send + Sync
```
Dokumentation: Trait for merging two raw byte values (`existing_val` from older tier, `new_val` from newer tier) for the same key across different SSTable tiers.

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `fn merge(&self, existing_val: &[u8], new_val: &[u8]) -> Result<Vec<u8>>` | `crates/contextra-store/src/compaction/merge_operator.rs:15` | Merges two raw byte values for the same key. # Fail-Safe Requirement Implementations running untrusted guest code (e.g. WASM via `contextra-sandbox`) MUST imple |

### Modul `compaction::retention`

#### `fn retain_key_versions`
- Ort: `crates/contextra-store/src/compaction/retention.rs:16`
```rust
pub fn retain_key_versions<T, F>(entries: Vec<T>, get_seq: F, floor_seq: u64) -> Vec<T> where F: Fn(&T) -> u64,
```
Dokumentation: Filters a sequence of entries for a SINGLE key (sorted by raw sequence DESCENDING) according to MVCC retention rules given a `floor` sequence number. Rules: 1. Retain ALL versions with `raw_seq > floor_seq`. 2. Retain the FIRST (newest) version with `raw_seq <= floor_seq`. 3. Discard all older versions with `raw_seq <= floor_seq`. Note: Sequence numbers are compared after masking `TOMBSTONE_BIT` (`seq & !TOMBSTONE_BIT`).

### Modul `compat`

#### `mod engine`
- Ort: `crates/contextra-store/src/compat/mod.rs:4`
```rust
pub mod engine
```

#### `mod error`
- Ort: `crates/contextra-store/src/compat/mod.rs:2`
```rust
pub mod error
```
- Gate: `#[cfg(feature = "workspace")]`

#### `mod license`
- Ort: `crates/contextra-store/src/compat/mod.rs:8`
```rust
pub mod license
```
- Gate: `#[cfg(feature = "workspace")]`

#### `mod scanner`
- Ort: `crates/contextra-store/src/compat/mod.rs:6`
```rust
pub mod scanner
```

### Modul `compat::engine`

#### `mod lsm`
- Ort: `crates/contextra-store/src/compat/engine/mod.rs:6`
```rust
pub mod lsm
```

#### `mod memory_only`
- Ort: `crates/contextra-store/src/compat/engine/mod.rs:4`
```rust
pub mod memory_only
```
- Gate: `#[cfg(feature = "memory-only-storage")]`

#### `mod tenant_storage`
- Ort: `crates/contextra-store/src/compat/engine/mod.rs:7`
```rust
pub mod tenant_storage
```

### Modul `compat::engine::memory_only`

#### `struct InMemoryStorageEngine`
- Ort: `crates/contextra-store/src/compat/engine/memory_only.rs:32`
```rust
pub struct InMemoryStorageEngine
```
Dokumentation: An in-memory implementation of [`StorageEngine`] guaranteed to perform no disk I/O.
- Impl-Block (Trait `Default`) `crates/contextra-store/src/compat/engine/memory_only.rs:39`
- Impl-Block `crates/contextra-store/src/compat/engine/memory_only.rs:50`
- Impl-Block (Trait `StorageEngine`) `crates/contextra-store/src/compat/engine/memory_only.rs:57`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new() -> Self` | `crates/contextra-store/src/compat/engine/memory_only.rs:52` | Creates a new, empty in-memory storage engine. |

### Modul `compat::engine::tenant_storage`

#### `trait StorageEngineHandle`
- Ort: `crates/contextra-store/src/compat/engine/tenant_storage.rs:12`
```rust
pub trait StorageEngineHandle: Send + Sync + 'static
```
Dokumentation: Adapter trait for types that can act as backend for `TenantScopedStorage`.

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `fn get_handle<'a>(&'a self, key: &'a [u8]) -> BoxFuture<'a, Result<Option<Bytes>>>` | `crates/contextra-store/src/compat/engine/tenant_storage.rs:13` |  |
| `fn get_at_seq_handle<'a>(&'a self, key: &'a [u8], seq_no: u64) -> BoxFuture<'a, Result<Option<Bytes>>>` | `crates/contextra-store/src/compat/engine/tenant_storage.rs:14` |  |
| `fn get_tracked_handle<'a>(&'a self, tx_id: TxId, key: &'a [u8]) -> BoxFuture<'a, Result<Option<Bytes>>>` | `crates/contextra-store/src/compat/engine/tenant_storage.rs:15` |  |
| `fn get_at_seq_tracked_handle<'a>(&'a self, tx_id: TxId, key: &'a [u8], seq: u64) -> BoxFuture<'a, Result<Option<Bytes>>>` | `crates/contextra-store/src/compat/engine/tenant_storage.rs:16` |  |
| `fn scan_prefix_tracked_handle<'a>(&'a self, tx_id: TxId, prefix: &'a [u8]) -> BoxFuture<'a, Result<Vec<(Vec<u8>, Vec<u8>)>>>` | `crates/contextra-store/src/compat/engine/tenant_storage.rs:17` |  |
| `fn supports_ssi_tracking_handle(&self) -> bool` | `crates/contextra-store/src/compat/engine/tenant_storage.rs:18` |  |
| `fn put_handle<'a>(&'a self, tx_id: TxId, key: &'a [u8], value: &'a [u8]) -> BoxFuture<'a, Result<()>>` | `crates/contextra-store/src/compat/engine/tenant_storage.rs:19` |  |
| `fn put_if_absent_handle<'a>(&'a self, tx_id: TxId, key: &'a [u8], value: &'a [u8]) -> BoxFuture<'a, Result<bool>>` | `crates/contextra-store/src/compat/engine/tenant_storage.rs:20` |  |
| `fn put_batch_handle<'a>(&'a self, tx_id: TxId, entries: &'a [(Vec<u8>, Vec<u8>)]) -> BoxFuture<'a, Result<()>>` | `crates/contextra-store/src/compat/engine/tenant_storage.rs:21` |  |
| `fn delete_handle<'a>(&'a self, tx_id: TxId, key: &'a [u8]) -> BoxFuture<'a, Result<()>>` | `crates/contextra-store/src/compat/engine/tenant_storage.rs:22` |  |
| `fn delete_many_handle<'a>(&'a self, tx_id: TxId, keys: Vec<Vec<u8>>) -> BoxFuture<'a, Result<u64>>` | `crates/contextra-store/src/compat/engine/tenant_storage.rs:23` |  |
| `fn delete_prefix_handle<'a>(&'a self, tx_id: TxId, prefix: &'a [u8]) -> BoxFuture<'a, Result<u64>>` | `crates/contextra-store/src/compat/engine/tenant_storage.rs:24` |  |
| `fn commit_handle<'a>(&'a self, tx_id: TxId) -> BoxFuture<'a, Result<()>>` | `crates/contextra-store/src/compat/engine/tenant_storage.rs:25` |  |
| `fn rollback_handle<'a>(&'a self, tx_id: TxId) -> BoxFuture<'a, Result<()>>` | `crates/contextra-store/src/compat/engine/tenant_storage.rs:26` |  |
| `fn rollback_to_tx_handle<'a>(&'a self, tx_id: TxId) -> BoxFuture<'a, Result<()>>` | `crates/contextra-store/src/compat/engine/tenant_storage.rs:27` |  |
| `fn flush_handle<'a>(&'a self) -> BoxFuture<'a, Result<()>>` | `crates/contextra-store/src/compat/engine/tenant_storage.rs:28` |  |
| `fn stats_handle<'a>(&'a self) -> BoxFuture<'a, Result<StorageStats>>` | `crates/contextra-store/src/compat/engine/tenant_storage.rs:29` |  |
| `fn last_seq_no_handle<'a>(&'a self) -> BoxFuture<'a, Result<u64>>` | `crates/contextra-store/src/compat/engine/tenant_storage.rs:30` |  |
| `fn last_tx_id_handle<'a>(&'a self) -> BoxFuture<'a, Result<TxId>>` | `crates/contextra-store/src/compat/engine/tenant_storage.rs:31` |  |
| `fn pin_checkpoint_handle<'a>(&'a self, seq_no: u64) -> BoxFuture<'a, Result<()>>` | `crates/contextra-store/src/compat/engine/tenant_storage.rs:32` |  |
| `fn unpin_checkpoint_handle<'a>(&'a self, seq_no: u64) -> BoxFuture<'a, Result<()>>` | `crates/contextra-store/src/compat/engine/tenant_storage.rs:33` |  |
| `fn scan_prefix_handle<'a>(&'a self, prefix: &'a [u8]) -> BoxFuture<'a, Result<Vec<(Vec<u8>, Vec<u8>)>>>` | `crates/contextra-store/src/compat/engine/tenant_storage.rs:34` |  |
| `fn scan_prefix_bounded_handle<'a>(&'a self, prefix: &'a [u8], limit: usize, cursor: Option<&'a [u8]>) -> BoxFuture<'a, Result<(Vec<(Vec<u8>, Vec<u8>)>, Option<Vec<u8>>)>>` | `crates/contextra-store/src/compat/engine/tenant_storage.rs:35` |  |
| `fn scan_prefix_at_handle<'a>(&'a self, prefix: &'a [u8], seq_no: u64) -> BoxFuture<'a, Result<Vec<(Vec<u8>, Vec<u8>)>>>` | `crates/contextra-store/src/compat/engine/tenant_storage.rs:36` |  |
| `fn scan_handle<'a>(&'a self, start: Bound<&'a [u8]>, end: Bound<&'a [u8]>, limit: Option<usize>) -> BoxFuture<'a, Result<Vec<(Vec<u8>, Vec<u8>)>>>` | `crates/contextra-store/src/compat/engine/tenant_storage.rs:37` |  |
| `fn scan_bounded_handle<'a>(&'a self, start: Bound<&'a [u8]>, end: Bound<&'a [u8]>, limit: usize, cursor: Option<&'a [u8]>) -> BoxFuture<'a, Result<(Vec<(Vec<u8>, Vec<u8>)>, Option<Vec<u8>>)>>` | `crates/contextra-store/src/compat/engine/tenant_storage.rs:38` |  |

### Modul `compat::scanner`

#### `mod factory`
- Ort: `crates/contextra-store/src/compat/scanner/mod.rs:12`
```rust
pub mod factory
```

#### `mod memtable`
- Ort: `crates/contextra-store/src/compat/scanner/mod.rs:6`
```rust
pub mod memtable
```

#### `mod purge`
- Ort: `crates/contextra-store/src/compat/scanner/mod.rs:11`
```rust
pub mod purge
```

#### `mod runtime`
- Ort: `crates/contextra-store/src/compat/scanner/mod.rs:9`
```rust
pub mod runtime
```

#### `mod sstable`
- Ort: `crates/contextra-store/src/compat/scanner/mod.rs:7`
```rust
pub mod sstable
```

#### `mod store_scan_view`
- Ort: `crates/contextra-store/src/compat/scanner/mod.rs:10`
```rust
pub mod store_scan_view
```

#### `mod wal`
- Ort: `crates/contextra-store/src/compat/scanner/mod.rs:8`
```rust
pub mod wal
```

### Modul `compat::scanner::factory`

#### `fn lsm_purgers`
- Ort: `crates/contextra-store/src/compat/scanner/factory.rs:52`
```rust
pub fn lsm_purgers(storage: &Arc<LsmStorage>) -> Vec<Arc<dyn Purger>>
```
Dokumentation: Erzeugt die LSM-Purger unter Verwendung der aktuellen Tokio-Laufzeitumgebung.

#### `fn lsm_purgers_with_handle`
- Ort: `crates/contextra-store/src/compat/scanner/factory.rs:58`
```rust
pub fn lsm_purgers_with_handle( storage: &Arc<LsmStorage>, handle: tokio::runtime::Handle, ) -> Vec<Arc<dyn Purger>>
```
Dokumentation: Erzeugt die LSM-Purger unter Verwendung eines expliziten Tokio-Runtime-Handles.

#### `fn lsm_purgers_with_runtime`
- Ort: `crates/contextra-store/src/compat/scanner/factory.rs:67`
```rust
pub fn lsm_purgers_with_runtime( storage: &Arc<LsmStorage>, runtime: ScannerRuntime, ) -> Vec<Arc<dyn Purger>>
```
Dokumentation: Erzeugt die LSM-Purger unter Verwendung einer übergebenen [`ScannerRuntime`].

#### `fn lsm_scanners`
- Ort: `crates/contextra-store/src/compat/scanner/factory.rs:15`
```rust
pub fn lsm_scanners(storage: &Arc<LsmStorage>) -> Vec<Arc<dyn LayerScanner>>
```
Dokumentation: Erzeugt die Standard-LSM-Scanner unter Verwendung der aktuellen Tokio-Laufzeitumgebung.

#### `fn lsm_scanners_with_handle`
- Ort: `crates/contextra-store/src/compat/scanner/factory.rs:21`
```rust
pub fn lsm_scanners_with_handle( storage: &Arc<LsmStorage>, handle: tokio::runtime::Handle, ) -> Vec<Arc<dyn LayerScanner>>
```
Dokumentation: Erzeugt die Standard-LSM-Scanner unter Verwendung eines expliziten Tokio-Runtime-Handles.

#### `fn lsm_scanners_with_runtime`
- Ort: `crates/contextra-store/src/compat/scanner/factory.rs:30`
```rust
pub fn lsm_scanners_with_runtime( storage: &Arc<LsmStorage>, runtime: ScannerRuntime, ) -> Vec<Arc<dyn LayerScanner>>
```
Dokumentation: Erzeugt die Standard-LSM-Scanner unter Verwendung einer übergebenen [`ScannerRuntime`].

### Modul `compat::scanner::memtable`

#### `struct MemtableScanner`
- Ort: `crates/contextra-store/src/compat/scanner/memtable.rs:30`
```rust
pub struct MemtableScanner
```
Dokumentation: Scanner für die `LsmMemtable`-Schicht. Hält [`StoreScanView`] statt `Arc<LsmStorage>` — keine direkte Kopplung an die Speicher-Engine. Kann daher mit isolierten Test-Daten instanziiert werden.
- Impl-Block `crates/contextra-store/src/compat/scanner/memtable.rs:36`
- Impl-Block (Trait `LayerScanner`) `crates/contextra-store/src/compat/scanner/memtable.rs:77`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new(view: StoreScanView) -> Self` | `crates/contextra-store/src/compat/scanner/memtable.rs:38` | Erzeugt einen neuen `MemtableScanner` aus einer [`StoreScanView`]. |
| `pub fn with_runtime(view: StoreScanView, runtime: ScannerRuntime) -> Self` | `crates/contextra-store/src/compat/scanner/memtable.rs:43` | Erzeugt einen neuen `MemtableScanner` aus einer [`StoreScanView`] mit expliziter Runtime. |
| `pub fn from_storage_blocking(storage: Arc<LsmStorage>, runtime: ScannerRuntime) -> Result<Self>` | `crates/contextra-store/src/compat/scanner/memtable.rs:51` | Erzeugt einen `MemtableScanner` aus einem `Arc<LsmStorage>` (Kompatibilitätspfad). Intern wird [`LsmStorage::scan_view`] aufgerufen — die Kopplung bleibt auf di |

### Modul `compat::scanner::purge`

#### `struct LsmPurger`
- Ort: `crates/contextra-store/src/compat/scanner/purge.rs:24`
```rust
pub struct LsmPurger
```
Dokumentation: Purger für LSM Storage Engine.
- Impl-Block `crates/contextra-store/src/compat/scanner/purge.rs:29`
- Impl-Block (Trait `Purger`) `crates/contextra-store/src/compat/scanner/purge.rs:41`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new(storage: Arc<LsmStorage>) -> Self` | `crates/contextra-store/src/compat/scanner/purge.rs:31` | Erzeugt einen neuen `LsmPurger` mit implizit erfasster Laufzeit. |
| `pub fn with_runtime(storage: Arc<LsmStorage>, runtime: ScannerRuntime) -> Self` | `crates/contextra-store/src/compat/scanner/purge.rs:36` | Erzeugt einen neuen `LsmPurger` mit expliziter `ScannerRuntime`. |

### Modul `compat::scanner::runtime`

#### `struct ScannerRuntime`
- Ort: `crates/contextra-store/src/compat/scanner/runtime.rs:15`
```rust
pub struct ScannerRuntime
```
Dokumentation: Laufzeit-Ausführungsstrategie für synchrone Scanner- und Purger-Ports. SPEC §3.4 (Sync-Kern / Async-Schale) & SPEC E-07 (Zero-Panic-Doktrin): Synchron aufgerufene Port-Methoden wie `Purger::purge` und `LayerScanner::scan_units` müssen asynchrone I/O- und Speichertreiber-Operationen blockierend ausführen können, ohne zu panicken, ohne pro Aufruf eine Runtime aufzubauen und ohne Panics im Worker bis in den Aufrufer unkontrolliert hochzujagen (kein `resume_unwind`).
- Impl-Block (Trait `Default`) `crates/contextra-store/src/compat/scanner/runtime.rs:19`
- Impl-Block `crates/contextra-store/src/compat/scanner/runtime.rs:25`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn capture() -> Self` | `crates/contextra-store/src/compat/scanner/runtime.rs:27` | Erfasst das aktuelle Tokio-Runtime-Handle aus dem Aufrufkontext. |
| `pub fn with_handle(handle: Handle) -> Self` | `crates/contextra-store/src/compat/scanner/runtime.rs:34` | Erstellt eine `ScannerRuntime` mit explizit bereitgestelltem Handle. |
| `pub fn empty() -> Self` | `crates/contextra-store/src/compat/scanner/runtime.rs:41` | Erstellt eine leere `ScannerRuntime` (führt bei Port-Aufruf zu einem typisierten Fehler). |
| `pub fn handle(&self) -> Option<&Handle>` | `crates/contextra-store/src/compat/scanner/runtime.rs:46` | Liefert eine Referenz auf das Handle, falls vorhanden. |
| `pub fn block_on<F, T>(&self, future: F) -> Result<T> where F: Future<Output = Result<T>> + Send, T: Send,` | `crates/contextra-store/src/compat/scanner/runtime.rs:60` | Führt ein Future synchron aus. Strategie: 1. Kein Handle vorhanden: Sofortiger Fehler `ContextraError::Storage`. 2. Multi-Thread Runtime: Ausführung via `tokio: |

### Modul `compat::scanner::sstable`

#### `struct SstableScanner`
- Ort: `crates/contextra-store/src/compat/scanner/sstable.rs:23`
```rust
pub struct SstableScanner
```
Dokumentation: Scanner für die `SsTableAllLevels`-Schicht.
- Impl-Block `crates/contextra-store/src/compat/scanner/sstable.rs:28`
- Impl-Block (Trait `LayerScanner`) `crates/contextra-store/src/compat/scanner/sstable.rs:97`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new(view: StoreScanView) -> Self` | `crates/contextra-store/src/compat/scanner/sstable.rs:30` | Erzeugt einen neuen `SstableScanner` mit implizit erfasster Laufzeit. |
| `pub fn with_runtime(view: StoreScanView, runtime: ScannerRuntime) -> Self` | `crates/contextra-store/src/compat/scanner/sstable.rs:35` | Erzeugt einen neuen `SstableScanner` mit expliziter `ScannerRuntime`. |
| `pub fn from_storage_blocking(storage: Arc<LsmStorage>, runtime: ScannerRuntime) -> Result<Self>` | `crates/contextra-store/src/compat/scanner/sstable.rs:40` | Erzeugt einen `SstableScanner` aus einem `Arc<LsmStorage>` (Kompatibilitätspfad). |

### Modul `compat::scanner::store_scan_view`

#### `struct StoreScanView`
- Ort: `crates/contextra-store/src/compat/scanner/store_scan_view.rs:39`
```rust
pub struct StoreScanView
```
Dokumentation: Schlanke, entkoppelte Lesezugriffssicht auf LSM-Speicherschichten für Scanner und Purger. Kann direkt aus einem `Arc<LsmStorage>` erzeugt oder für Tests mit isolierten Daten befüllt werden.
- Impl-Block `crates/contextra-store/src/compat/scanner/store_scan_view.rs:56`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `active_memtable` | `Arc<MemTable>` | `crates/contextra-store/src/compat/scanner/store_scan_view.rs:41` |
| `immutable_memtables` | `Vec<Arc<MemTable>>` | `crates/contextra-store/src/compat/scanner/store_scan_view.rs:43` |
| `sstables` | `Arc<RwLock<Vec<Arc<SstableReader>>>>` | `crates/contextra-store/src/compat/scanner/store_scan_view.rs:45` |
| `block_cache` | `Arc<BlockCache>` | `crates/contextra-store/src/compat/scanner/store_scan_view.rs:47` |
| `data_path` | `PathBuf` | `crates/contextra-store/src/compat/scanner/store_scan_view.rs:49` |
| `vfs` | `Arc<dyn Vfs>` | `crates/contextra-store/src/compat/scanner/store_scan_view.rs:51` |
| `snapshot_registry` | `Arc<SnapshotRegistry>` | `crates/contextra-store/src/compat/scanner/store_scan_view.rs:53` |

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new_for_test( active_memtable: Arc<MemTable>, immutable_memtables: Vec<Arc<MemTable>>, sstables: Arc<RwLock<Vec<Arc<SstableReader>>>>, block_cache: Arc<BlockCache>, data_path: PathBuf, vfs: Arc<dyn Vfs>, snapshot_registry: Arc<SnapshotRegistry>, ) -> Self` | `crates/contextra-store/src/compat/scanner/store_scan_view.rs:61` | Erstellt einen `StoreScanView` für Tests mit vollständig isolierten Daten. Ermöglicht Unit-Tests ohne `LsmStorage`-Instanz. |
| `pub fn active_memtable(&self) -> &Arc<MemTable>` | `crates/contextra-store/src/compat/scanner/store_scan_view.rs:82` | Gibt die aktive MemTable zurück. |
| `pub fn immutable_memtables(&self) -> &[Arc<MemTable>]` | `crates/contextra-store/src/compat/scanner/store_scan_view.rs:87` | Gibt alle immutablen MemTables zurück. |
| `pub fn all_memtables(&self) -> impl Iterator<Item = &Arc<MemTable>>` | `crates/contextra-store/src/compat/scanner/store_scan_view.rs:92` | Gibt einen Iterator über alle MemTables (aktiv + immutable) zurück. |
| `pub fn sstables_arc(&self) -> &Arc<RwLock<Vec<Arc<SstableReader>>>>` | `crates/contextra-store/src/compat/scanner/store_scan_view.rs:97` | Gibt die SSTable-Liste als `Arc<RwLock<...>>` zurück (für Purger-Schreibzugriff). |
| `pub fn block_cache(&self) -> &Arc<BlockCache>` | `crates/contextra-store/src/compat/scanner/store_scan_view.rs:102` | Gibt den gemeinsamen Block-Cache zurück. |
| `pub fn data_path(&self) -> &std::path::Path` | `crates/contextra-store/src/compat/scanner/store_scan_view.rs:107` | Gibt den Pfad zum Datenverzeichnis zurück. |
| `pub fn vfs(&self) -> &Arc<dyn Vfs>` | `crates/contextra-store/src/compat/scanner/store_scan_view.rs:112` | Gibt die VFS-Instanz zurück. |
| `pub fn snapshot_registry(&self) -> &Arc<SnapshotRegistry>` | `crates/contextra-store/src/compat/scanner/store_scan_view.rs:117` | Gibt die SnapshotRegistry zurück (für Purge-Sperrprüfung). |

### Modul `compat::scanner::wal`

#### `struct WalScanner`
- Ort: `crates/contextra-store/src/compat/scanner/wal.rs:22`
```rust
pub struct WalScanner
```
Dokumentation: Scanner für die `WalAllSegments`-Schicht.
- Impl-Block `crates/contextra-store/src/compat/scanner/wal.rs:27`
- Impl-Block (Trait `LayerScanner`) `crates/contextra-store/src/compat/scanner/wal.rs:110`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new(view: StoreScanView) -> Self` | `crates/contextra-store/src/compat/scanner/wal.rs:29` | Erzeugt einen neuen `WalScanner` mit implizit erfasster Laufzeit. |
| `pub fn with_runtime(view: StoreScanView, runtime: ScannerRuntime) -> Self` | `crates/contextra-store/src/compat/scanner/wal.rs:34` | Erzeugt einen neuen `WalScanner` mit expliziter `ScannerRuntime`. |
| `pub fn from_storage_blocking(storage: Arc<LsmStorage>, runtime: ScannerRuntime) -> Result<Self>` | `crates/contextra-store/src/compat/scanner/wal.rs:39` | Erzeugt einen `WalScanner` aus einem `Arc<LsmStorage>` (Kompatibilitätspfad). |

### Modul `crate`

#### `mod budget`
- Ort: `crates/contextra-store/src/lib.rs:57`
```rust
pub mod budget
```

#### `mod compaction`
- Ort: `crates/contextra-store/src/lib.rs:61`
```rust
pub mod compaction
```

#### `mod compat`
- Ort: `crates/contextra-store/src/lib.rs:55`
```rust
pub mod compat
```

#### `mod engine`
- Ort: `crates/contextra-store/src/lib.rs:64`
```rust
pub mod engine
```
- Gate: `#[cfg(feature = "workspace")]`

#### `mod error`
- Ort: `crates/contextra-store/src/lib.rs:53`
```rust
pub mod error
```

#### `mod fsync`
- Ort: `crates/contextra-store/src/lib.rs:65`
```rust
pub mod fsync
```

#### `mod io_seam`
- Ort: `crates/contextra-store/src/lib.rs:109`
```rust
pub mod io_seam
```

#### `mod key_source`
- Ort: `crates/contextra-store/src/lib.rs:66`
```rust
pub mod key_source
```

#### `mod lsm`
- Ort: `crates/contextra-store/src/lib.rs:68`
```rust
pub mod lsm
```

#### `mod manifest`
- Ort: `crates/contextra-store/src/lib.rs:70`
```rust
pub mod manifest
```

#### `mod memtable`
- Ort: `crates/contextra-store/src/lib.rs:72`
```rust
pub mod memtable
```

#### `mod scanner`
- Ort: `crates/contextra-store/src/lib.rs:75`
```rust
pub mod scanner
```
- Gate: `#[cfg(feature = "workspace")]`

#### `mod sstable`
- Ort: `crates/contextra-store/src/lib.rs:77`
```rust
pub mod sstable
```

#### `mod stats`
- Ort: `crates/contextra-store/src/lib.rs:58`
```rust
pub mod stats
```

#### `mod system_pressure`
- Ort: `crates/contextra-store/src/lib.rs:79`
```rust
pub mod system_pressure
```

#### `mod tenant_codec`
- Ort: `crates/contextra-store/src/lib.rs:81`
```rust
pub mod tenant_codec
```

#### `mod traits`
- Ort: `crates/contextra-store/src/lib.rs:54`
```rust
pub mod traits
```

#### `mod util`
- Ort: `crates/contextra-store/src/lib.rs:82`
```rust
pub mod util
```

#### `mod wal`
- Ort: `crates/contextra-store/src/lib.rs:83`
```rust
pub mod wal
```

### Modul `error`

#### `enum StoreError`
- Ort: `crates/contextra-store/src/error.rs:8`
```rust
pub enum StoreError
```
Dokumentation: Lokaler Fehlertyp des LSM-Speicherkerns `contextra-store`.
- Impl-Block `crates/contextra-store/src/error.rs:115`
- Impl-Block (Trait `From<contextra_crypto::CryptoError>`) `crates/contextra-store/src/error.rs:148`
- Impl-Block (Trait `From<contextra_mvcc::MvccError>`) `crates/contextra-store/src/error.rs:169`

Varianten:

- `Storage(String)` — `crates/contextra-store/src/error.rs:11`
- `Io(#[from] std::io::Error)` — `crates/contextra-store/src/error.rs:15`
- `WalCorruption{` — `crates/contextra-store/src/error.rs:19`
- `ChecksumMismatch{` — `crates/contextra-store/src/error.rs:28`
- `WalTruncationDetected{` — `crates/contextra-store/src/error.rs:37`
- `CrossDeviceLink{` — `crates/contextra-store/src/error.rs:46`
- `Transaction(String)` — `crates/contextra-store/src/error.rs:55`
- `Conflict(String)` — `crates/contextra-store/src/error.rs:59`
- `CommitTimeout{` — `crates/contextra-store/src/error.rs:63`
- `Serialization(String)` — `crates/contextra-store/src/error.rs:70`
- `ParseError(String)` — `crates/contextra-store/src/error.rs:74`
- `InvalidInput(String)` — `crates/contextra-store/src/error.rs:78`
- `NotFound(String)` — `crates/contextra-store/src/error.rs:82`
- `Internal(String)` — `crates/contextra-store/src/error.rs:86`
- `MemoryBudgetExceeded{` — `crates/contextra-store/src/error.rs:90`
- `LimitExceeded{` — `crates/contextra-store/src/error.rs:99`
- `DurabilityConfig(String)` — `crates/contextra-store/src/error.rs:108`
- `Sandbox(String)` — `crates/contextra-store/src/error.rs:112`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn wal_corruption(offset: u64, reason: impl Into<String>) -> Self` | `crates/contextra-store/src/error.rs:118` | Erzeugt einen neuen `StoreError::WalCorruption`. |
| `pub fn checksum_mismatch(path: impl Into<String>, block_id: u64) -> Self` | `crates/contextra-store/src/error.rs:127` | Erzeugt einen neuen `StoreError::ChecksumMismatch`. |
| `pub fn wal_truncation_detected(expected_hmac: [u8` | `crates/contextra-store/src/error.rs:136` | Erzeugt einen neuen `StoreError::InvalidInput`. |
| `pub fn invalid_input(reason: impl Into<String>) -> Self` | `crates/contextra-store/src/error.rs:143` |  |

#### `type StoreResult`
- Ort: `crates/contextra-store/src/error.rs:167`
```rust
pub type StoreResult<T> = std::result::Result<T, StoreError>
```
Dokumentation: Ergebnis-Typ fuer Speicheroperationen.

### Modul `fsync`

#### `fn fsync_parent_dir`
- Ort: `crates/contextra-store/src/fsync.rs:83`
```rust
pub async fn fsync_parent_dir(path: &Path) -> Result<()>
```
Dokumentation: Alias for `sync_parent_dir` for backward compatibility across storage modules.

#### `fn sync_file`
- Ort: `crates/contextra-store/src/fsync.rs:14`
```rust
pub async fn sync_file(path: &Path) -> Result<()>
```
Dokumentation: Performs fsync on a single file at `path`. # Errors Returns `StoreError::Storage` on Unix if opening or syncing fails. Returns `StoreError::Io` with `ErrorKind::Unsupported` on Windows (E20).

#### `fn sync_parent_dir`
- Ort: `crates/contextra-store/src/fsync.rs:46`
```rust
pub async fn sync_parent_dir(path: &Path) -> Result<()>
```
Dokumentation: Performs fsync on the parent directory of `path`. Directory fsync is required on POSIX filesystems to guarantee that newly created files or directory entries are durably persisted to disk. # Errors Returns `StoreError::Storage` on Unix if opening or syncing fails. Returns `StoreError::Io` with `ErrorKind::Unsupported` on Windows (E20).

### Modul `io_seam`

#### `struct RealFs`
- Ort: `crates/contextra-store/src/io_seam.rs:66`
```rust
pub struct RealFs
```
Dokumentation: Default POSIX/OS filesystem implementation of [] for contextra-store.

#### `struct RealFsFile`
- Ort: `crates/contextra-store/src/io_seam.rs:179`
```rust
pub struct RealFsFile
```
Dokumentation: Standard [] handle based on [].

#### `struct StoreIo`
- Ort: `crates/contextra-store/src/io_seam.rs:249`
```rust
pub struct StoreIo
```
Dokumentation: Central Store I/O abstraction () wrapping .

#### `struct VfsOpenOptions`
- Ort: `crates/contextra-store/src/io_seam.rs:17`
```rust
pub struct VfsOpenOptions
```
Dokumentation: Options configuring how a file is opened via [].

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `read` | `bool` | `crates/contextra-store/src/io_seam.rs:18` |
| `write` | `bool` | `crates/contextra-store/src/io_seam.rs:19` |
| `create` | `bool` | `crates/contextra-store/src/io_seam.rs:20` |
| `create_new` | `bool` | `crates/contextra-store/src/io_seam.rs:21` |
| `truncate` | `bool` | `crates/contextra-store/src/io_seam.rs:22` |

#### `enum VerifiedVia`
- Ort: `crates/contextra-store/src/io_seam.rs:9`
```rust
pub enum VerifiedVia
```
Dokumentation: Method via which medium verification was performed.

Varianten:

- `Direct` — `crates/contextra-store/src/io_seam.rs:10`
- `CacheDropped` — `crates/contextra-store/src/io_seam.rs:11`
- `Cached` — `crates/contextra-store/src/io_seam.rs:12`

#### `trait Vfs`
- Ort: `crates/contextra-store/src/io_seam.rs:42`
```rust
pub trait Vfs: Send + Sync + 'static
```
Dokumentation: Abstract virtual file system for contextra-store.

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `fn open(&self, path: &Path, opts: &VfsOpenOptions) -> Result<Box<dyn VfsFile>>` | `crates/contextra-store/src/io_seam.rs:43` |  |
| `fn write_atomic(&self, path: &Path, data: &[u8]) -> Result<()>` | `crates/contextra-store/src/io_seam.rs:44` |  |
| `fn create_dir_all(&self, path: &Path) -> Result<()>` | `crates/contextra-store/src/io_seam.rs:45` |  |
| `fn remove_file(&self, path: &Path) -> Result<()>` | `crates/contextra-store/src/io_seam.rs:46` |  |
| `fn rename(&self, from: &Path, to: &Path) -> Result<()>` | `crates/contextra-store/src/io_seam.rs:47` |  |
| `fn read_dir(&self, path: &Path) -> Result<Vec<PathBuf>>` | `crates/contextra-store/src/io_seam.rs:48` |  |
| `fn exists(&self, path: &Path) -> Result<bool>` | `crates/contextra-store/src/io_seam.rs:49` |  |
| `fn len(&self, path: &Path) -> Result<u64>` | `crates/contextra-store/src/io_seam.rs:50` |  |
| `fn list_dir(&self, path: &Path) -> Result<Vec<String>>` | `crates/contextra-store/src/io_seam.rs:51` |  |
| `fn sync_dir(&self, path: &Path) -> Result<()>` | `crates/contextra-store/src/io_seam.rs:52` |  |
| `fn read(&self, path: &Path) -> Result<Vec<u8>>` | `crates/contextra-store/src/io_seam.rs:53` |  |

#### `trait VfsFile`
- Ort: `crates/contextra-store/src/io_seam.rs:32`
```rust
pub trait VfsFile: Send + Sync + 'static
```
Dokumentation: Abstract positional file handle.

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `fn read_at(&self, buf: &mut [u8], offset: u64) -> std::io::Result<usize>` | `crates/contextra-store/src/io_seam.rs:33` |  |
| `fn write_at(&self, buf: &[u8], offset: u64) -> std::io::Result<usize>` | `crates/contextra-store/src/io_seam.rs:34` |  |
| `fn set_len(&self, len: u64) -> std::io::Result<()>` | `crates/contextra-store/src/io_seam.rs:35` |  |
| `fn len(&self) -> std::io::Result<u64>` | `crates/contextra-store/src/io_seam.rs:36` |  |
| `fn sync_data(&self) -> std::io::Result<()>` | `crates/contextra-store/src/io_seam.rs:37` |  |
| `fn sync_all(&self) -> std::io::Result<()>` | `crates/contextra-store/src/io_seam.rs:38` |  |

### Modul `io_seam::real_fs`

#### `struct RealFs`
- Ort: `crates/contextra-store/src/io_seam/real_fs.rs:14`
```rust
pub struct RealFs
```
Dokumentation: Standard POSIX/Betriebssystem-Dateisystemimplementierung für `contextra-store`.
- Impl-Block (Trait `Vfs`) `crates/contextra-store/src/io_seam/real_fs.rs:16`
- Impl-Block (Trait `Vfs`) `crates/contextra-store/src/io_seam.rs:68`

#### `struct RealFsFile`
- Ort: `crates/contextra-store/src/io_seam/real_fs.rs:95`
```rust
pub struct RealFsFile(pub File)
```
Dokumentation: Dateihandle für `RealFs`.
- Impl-Block (Trait `VfsFile`) `crates/contextra-store/src/io_seam/real_fs.rs:97`
- Impl-Block `crates/contextra-store/src/io_seam.rs:183`
- Impl-Block (Trait `VfsFile`) `crates/contextra-store/src/io_seam.rs:195`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new(file: std::fs::File) -> Self` | `crates/contextra-store/src/io_seam.rs:185` | Creates a new  from an open file handle. |
| `pub fn file(&self) -> &std::fs::File` | `crates/contextra-store/src/io_seam.rs:190` | Returns a reference to the inner file. |

### Modul `io_seam::store_io`

#### `struct StoreIo`
- Ort: `crates/contextra-store/src/io_seam/store_io.rs:11`
```rust
pub struct StoreIo
```
Dokumentation: Konkrete asynchrone I/O-Hülle für den LSM-Kern.
- Impl-Block (Trait `Default`) `crates/contextra-store/src/io_seam/store_io.rs:15`
- Impl-Block `crates/contextra-store/src/io_seam/store_io.rs:23`
- Impl-Block (Trait `std::fmt::Debug`) `crates/contextra-store/src/io_seam.rs:253`
- Impl-Block (Trait `Default`) `crates/contextra-store/src/io_seam.rs:259`
- Impl-Block `crates/contextra-store/src/io_seam.rs:267`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new(vfs: Arc<dyn Vfs>) -> Self` | `crates/contextra-store/src/io_seam/store_io.rs:25` | Erzeugt ein neues `StoreIo` mit dem angegebenen VFS. |
| `pub fn vfs(&self) -> &Arc<dyn Vfs>` | `crates/contextra-store/src/io_seam/store_io.rs:30` | Liefert eine Referenz auf das innere VFS. |
| `pub fn open(&self, path: &Path, opts: &VfsOpenOptions) -> Result<Box<dyn VfsFile>>` | `crates/contextra-store/src/io_seam/store_io.rs:35` | Öffnet eine Datei unter Verwendung von `VfsOpenOptions`. |
| `pub async fn read(&self, path: &Path) -> Result<Vec<u8>>` | `crates/contextra-store/src/io_seam/store_io.rs:40` | Liest eine Datei vollständig asynchron ein. |
| `pub async fn read_at(&self, path: &Path, offset: u64, len: usize) -> Result<Vec<u8>>` | `crates/contextra-store/src/io_seam/store_io.rs:49` | Liest Daten positionell ab `offset` mit Länge `len` asynchron ein. |
| `pub async fn write_atomic(&self, path: &Path, data: Vec<u8>) -> Result<()>` | `crates/contextra-store/src/io_seam/store_io.rs:82` | Schreibt Daten atomar und crash-konsistent. |
| `pub async fn create_dir_all(&self, path: &Path) -> Result<()>` | `crates/contextra-store/src/io_seam/store_io.rs:91` | Erzeugt Verzeichnisse rekursiv. |
| `pub async fn remove_file(&self, path: &Path) -> Result<()>` | `crates/contextra-store/src/io_seam/store_io.rs:100` | Löscht eine Datei. |
| `pub async fn rename(&self, from: &Path, to: &Path) -> Result<()>` | `crates/contextra-store/src/io_seam/store_io.rs:109` | Benennt eine Datei atomar um. |
| `pub async fn exists(&self, path: &Path) -> Result<bool>` | `crates/contextra-store/src/io_seam/store_io.rs:119` | Prüft auf Existenz einer Datei. |
| `pub async fn len(&self, path: &Path) -> Result<u64>` | `crates/contextra-store/src/io_seam/store_io.rs:128` | Liefert die Dateilänge in Bytes. |
| `pub async fn read_dir(&self, path: &Path) -> Result<Vec<PathBuf>>` | `crates/contextra-store/src/io_seam/store_io.rs:137` | Liest Verzeichniseinträge als Pfadliste. |
| `pub async fn sync_dir(&self, path: &Path) -> Result<()>` | `crates/contextra-store/src/io_seam/store_io.rs:146` | Synchronisiert Verzeichnis-Metadaten. |
| `pub fn new(vfs: Arc<dyn Vfs>) -> Self` | `crates/contextra-store/src/io_seam.rs:269` | Creates a new  with the specified . |
| `pub fn vfs(&self) -> &Arc<dyn Vfs>` | `crates/contextra-store/src/io_seam.rs:274` | Returns a reference to the inner . |
| `pub fn open(&self, path: &Path, opts: &VfsOpenOptions) -> Result<Box<dyn VfsFile>>` | `crates/contextra-store/src/io_seam.rs:279` | Opens a file using positional VfsFile options. |
| `pub async fn read(&self, path: &Path) -> Result<Vec<u8>>` | `crates/contextra-store/src/io_seam.rs:284` | Reads an entire file to a vector of bytes. |
| `pub async fn read_at(&self, path: &Path, offset: u64, len: usize) -> Result<Vec<u8>>` | `crates/contextra-store/src/io_seam.rs:293` | Reads data positionally from  into . |
| `pub async fn write_atomic(&self, path: &Path, data: Vec<u8>) -> Result<()>` | `crates/contextra-store/src/io_seam.rs:326` | Atomically writes data to  with crash-consistency. |
| `pub async fn create_dir_all(&self, path: &Path) -> Result<()>` | `crates/contextra-store/src/io_seam.rs:335` | Creates directories along . |
| `pub async fn remove_file(&self, path: &Path) -> Result<()>` | `crates/contextra-store/src/io_seam.rs:344` | Removes a file at . |
| `pub async fn rename(&self, from: &Path, to: &Path) -> Result<()>` | `crates/contextra-store/src/io_seam.rs:353` | Atomically renames a file. |
| `pub async fn exists(&self, path: &Path) -> Result<bool>` | `crates/contextra-store/src/io_seam.rs:363` | Checks if a file exists. |
| `pub async fn len(&self, path: &Path) -> Result<u64>` | `crates/contextra-store/src/io_seam.rs:372` | Returns the length in bytes of the file at . |
| `pub async fn read_dir(&self, path: &Path) -> Result<Vec<PathBuf>>` | `crates/contextra-store/src/io_seam.rs:381` | Reads directory entries as PathBufs. |
| `pub async fn sync_dir(&self, path: &Path) -> Result<()>` | `crates/contextra-store/src/io_seam.rs:390` | Syncs directory metadata. |

### Modul `io_seam::types`

#### `struct VfsOpenOptions`
- Ort: `crates/contextra-store/src/io_seam/types.rs:23`
```rust
pub struct VfsOpenOptions
```
Dokumentation: Optionen zum Öffnen von Dateien über das VFS.
- Impl-Block `crates/contextra-store/src/io_seam/types.rs:36`
- Impl-Block `crates/contextra-store/src/io_seam.rs:25`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `read` | `bool` | `crates/contextra-store/src/io_seam/types.rs:25` |
| `write` | `bool` | `crates/contextra-store/src/io_seam/types.rs:27` |
| `create` | `bool` | `crates/contextra-store/src/io_seam/types.rs:29` |
| `create_new` | `bool` | `crates/contextra-store/src/io_seam/types.rs:31` |
| `truncate` | `bool` | `crates/contextra-store/src/io_seam/types.rs:33` |

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn read_only() -> Self` | `crates/contextra-store/src/io_seam/types.rs:38` | Standardoptionen für Nur-Lese-Zugriff. |
| `pub fn read_only() -> Self` | `crates/contextra-store/src/io_seam.rs:26` |  |

#### `enum VerifiedVia`
- Ort: `crates/contextra-store/src/io_seam/types.rs:12`
```rust
pub enum VerifiedVia
```
Dokumentation: Methode, über welche die Medienverifikation durchgeführt wurde.

Varianten:

- `Direct` — `crates/contextra-store/src/io_seam/types.rs:14`
- `CacheDropped` — `crates/contextra-store/src/io_seam/types.rs:16`
- `Cached` — `crates/contextra-store/src/io_seam/types.rs:18`

#### `trait Vfs`
- Ort: `crates/contextra-store/src/io_seam/types.rs:60`
```rust
pub trait Vfs: Send + Sync + 'static
```
Dokumentation: Abstraktes virtuelles Dateisystem für `contextra-store`.

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `fn open(&self, path: &Path, opts: &VfsOpenOptions) -> Result<Box<dyn VfsFile>>` | `crates/contextra-store/src/io_seam/types.rs:62` | Öffnet eine Datei mit den übergebenen Optionen. |
| `fn write_atomic(&self, path: &Path, data: &[u8]) -> Result<()>` | `crates/contextra-store/src/io_seam/types.rs:64` | Schreibt Daten atomar über temporäre Datei und Rename. |
| `fn create_dir_all(&self, path: &Path) -> Result<()>` | `crates/contextra-store/src/io_seam/types.rs:66` | Erzeugt ein Verzeichnis rekursiv. |
| `fn remove_file(&self, path: &Path) -> Result<()>` | `crates/contextra-store/src/io_seam/types.rs:68` | Entfernt eine Datei. |
| `fn rename(&self, from: &Path, to: &Path) -> Result<()>` | `crates/contextra-store/src/io_seam/types.rs:70` | Benennt eine Datei atomar um. |
| `fn read_dir(&self, path: &Path) -> Result<Vec<PathBuf>>` | `crates/contextra-store/src/io_seam/types.rs:72` | Liest Einträge eines Verzeichnisses als Pfade. |
| `fn exists(&self, path: &Path) -> Result<bool>` | `crates/contextra-store/src/io_seam/types.rs:74` | Prüft auf Existenz eines Pfads. |
| `fn len(&self, path: &Path) -> Result<u64>` | `crates/contextra-store/src/io_seam/types.rs:76` | Liefert die Dateigröße. |
| `fn list_dir(&self, path: &Path) -> Result<Vec<String>>` | `crates/contextra-store/src/io_seam/types.rs:78` | Listet Dateinamen in einem Verzeichnis als Strings auf. |
| `fn sync_dir(&self, path: &Path) -> Result<()>` | `crates/contextra-store/src/io_seam/types.rs:80` | Synchronisiert Verzeichnis-Metadaten. |
| `fn read(&self, path: &Path) -> Result<Vec<u8>>` | `crates/contextra-store/src/io_seam/types.rs:82` | Liest eine Datei vollständig ein. |

#### `trait VfsFile`
- Ort: `crates/contextra-store/src/io_seam/types.rs:44`
```rust
pub trait VfsFile: Send + Sync + 'static
```
Dokumentation: Abstraktes positionelles Dateihandle.

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `fn read_at(&self, buf: &mut [u8], offset: u64) -> std::io::Result<usize>` | `crates/contextra-store/src/io_seam/types.rs:46` | Liest positionell ab `offset` in `buf`. |
| `fn write_at(&self, buf: &[u8], offset: u64) -> std::io::Result<usize>` | `crates/contextra-store/src/io_seam/types.rs:48` | Schreibt positionell ab `offset` aus `buf`. |
| `fn set_len(&self, len: u64) -> std::io::Result<()>` | `crates/contextra-store/src/io_seam/types.rs:50` | Setzt die Dateilänge. |
| `fn len(&self) -> std::io::Result<u64>` | `crates/contextra-store/src/io_seam/types.rs:52` | Liefert die Dateilänge in Bytes. |
| `fn sync_data(&self) -> std::io::Result<()>` | `crates/contextra-store/src/io_seam/types.rs:54` | Synchronisiert Datenblöcke auf Platte (`fdatasync`). |
| `fn sync_all(&self) -> std::io::Result<()>` | `crates/contextra-store/src/io_seam/types.rs:56` | Synchronisiert Datenblöcke und Metadaten auf Platte (`fsync`). |

### Modul `key_source`

#### `struct OsKeyMaterialSource`
- Ort: `crates/contextra-store/src/key_source.rs:14`
```rust
pub struct OsKeyMaterialSource
```
Dokumentation: Default production implementation using OS CSPRNG.
- Impl-Block (Trait `KeyMaterialSource`) `crates/contextra-store/src/key_source.rs:16`

#### `struct StaticKeyMaterialSource`
- Ort: `crates/contextra-store/src/key_source.rs:29`
```rust
pub struct StaticKeyMaterialSource
```
Dokumentation: Fixed deterministic key material source for reproducible testing and fault simulation.
- Impl-Block `crates/contextra-store/src/key_source.rs:33`
- Impl-Block (Trait `KeyMaterialSource`) `crates/contextra-store/src/key_source.rs:42`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new(bytes: Vec<u8>) -> Self` | `crates/contextra-store/src/key_source.rs:35` | Creates a static key material source repeating or providing fixed byte slices. |

#### `trait KeyMaterialSource`
- Ort: `crates/contextra-store/src/key_source.rs:7`
```rust
pub trait KeyMaterialSource: Send + Sync + Debug + 'static
```
Dokumentation: Port trait for key and salt material generation.

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `fn fill_key_bytes(&self, dest: &mut [u8]) -> Result<()>` | `crates/contextra-store/src/key_source.rs:9` | Fills dest with random bytes. |

### Modul `lsm`

#### `mod commit`
- Ort: `crates/contextra-store/src/lsm/mod.rs:78`
```rust
pub mod commit
```

#### `mod config`
- Ort: `crates/contextra-store/src/lsm/mod.rs:72`
```rust
pub mod config
```

#### `mod fence`
- Ort: `crates/contextra-store/src/lsm/mod.rs:79`
```rust
pub mod fence
```

#### `mod flush`
- Ort: `crates/contextra-store/src/lsm/mod.rs:80`
```rust
pub mod flush
```

#### `mod group_commit`
- Ort: `crates/contextra-store/src/lsm/mod.rs:81`
```rust
pub mod group_commit
```

#### `mod lock_order`
- Ort: `crates/contextra-store/src/lsm/mod.rs:82`
```rust
pub mod lock_order
```

#### `mod observer`
- Ort: `crates/contextra-store/src/lsm/mod.rs:75`
```rust
pub mod observer
```

#### `mod ops`
- Ort: `crates/contextra-store/src/lsm/mod.rs:89`
```rust
pub mod ops
```

#### `mod recovery`
- Ort: `crates/contextra-store/src/lsm/mod.rs:83`
```rust
pub mod recovery
```

#### `mod scan`
- Ort: `crates/contextra-store/src/lsm/mod.rs:84`
```rust
pub mod scan
```

#### `const MAX_BATCH_SIZE`
- Ort: `crates/contextra-store/src/lsm/mod.rs:108`
```rust
pub const MAX_BATCH_SIZE: usize = 10_000
```
Dokumentation: Maximum batch size for `delete_many` operations (10,000 items).

#### `const MAX_GROUP_COMMIT_BATCH_SIZE`
- Ort: `crates/contextra-store/src/lsm/mod.rs:114`
```rust
pub const MAX_GROUP_COMMIT_BATCH_SIZE: usize = 1_000
```
Dokumentation: Maximum batch size for group commits (1,000 transactions).

#### `const MAX_INTERNAL_MERGE_ENTRIES_FACTOR`
- Ort: `crates/contextra-store/src/lsm/mod.rs:111`
```rust
pub const MAX_INTERNAL_MERGE_ENTRIES_FACTOR: usize = 8
```
Dokumentation: Maximum factor for internal merge set size relative to limit in bounded scans.

#### `const MAX_KEY_SIZE`
- Ort: `crates/contextra-store/src/lsm/mod.rs:102`
```rust
pub const MAX_KEY_SIZE: usize = 65_535
```
Dokumentation: Maximum key size allowed for LSM operations (65,535 bytes).

#### `const MAX_VALUE_SIZE`
- Ort: `crates/contextra-store/src/lsm/mod.rs:105`
```rust
pub const MAX_VALUE_SIZE: usize = 134_217_728
```
Dokumentation: Maximum value size allowed for LSM operations (128MB).

#### `const MIN_ENTRIES_FOR_SSTABLE_REBUILD`
- Ort: `crates/contextra-store/src/lsm/mod.rs:119`
```rust
pub const MIN_ENTRIES_FOR_SSTABLE_REBUILD: usize = 8
```
Dokumentation: Minimum surviving entry threshold required to rebuild a new SSTable during rollback. Below this threshold (1..7 entries), surviving entries from a spanning SSTable are inserted directly into the MemTable instead of allocating a full new SSTable/manifest pipeline.

### Modul `lsm::config`

#### `struct LsmConfig`
- Ort: `crates/contextra-store/src/lsm/config.rs:74`
```rust
pub struct LsmConfig
```
Dokumentation: Configuration for the LSM storage engine.
- Impl-Block (Trait `Default`) `crates/contextra-store/src/lsm/config.rs:108`
- Impl-Block `crates/contextra-store/src/lsm/config.rs:136`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `path` | `PathBuf` | `crates/contextra-store/src/lsm/config.rs:76` |
| `memtable_size_limit` | `usize` | `crates/contextra-store/src/lsm/config.rs:78` |
| `max_ram_mb` | `u64` | `crates/contextra-store/src/lsm/config.rs:80` |
| `tx_timeout` | `Duration` | `crates/contextra-store/src/lsm/config.rs:82` |
| `compaction` | `CompactionConfig` | `crates/contextra-store/src/lsm/config.rs:84` |
| `encryption_passphrase` | `Option<String>` | `crates/contextra-store/src/lsm/config.rs:85` |
| `group_commit_window_micros` | `u64` | `crates/contextra-store/src/lsm/config.rs:88` |
| `block_cache_shards` | `usize` | `crates/contextra-store/src/lsm/config.rs:91` |
| `durability_mode` | `DurabilityMode` | `crates/contextra-store/src/lsm/config.rs:93` |
| `ssi_max_tracked_keys` | `usize` | `crates/contextra-store/src/lsm/config.rs:96` |
| `vfs` | `Option<crate::io_seam::StoreIo>` | `crates/contextra-store/src/lsm/config.rs:99` |
| `clock` | `std::sync::Arc<dyn contextra_env::Clock>` | `crates/contextra-store/src/lsm/config.rs:102` |
| `key_source` | `std::sync::Arc<dyn crate::key_source::KeyMaterialSource>` | `crates/contextra-store/src/lsm/config.rs:105` |

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn with_clock(mut self, clock: std::sync::Arc<dyn contextra_env::Clock>) -> Self` | `crates/contextra-store/src/lsm/config.rs:138` | Attaches an injected clock port fluently to `LsmConfig`. |
| `pub fn with_key_source( mut self, key_source: std::sync::Arc<dyn crate::key_source::KeyMaterialSource>, ) -> Self` | `crates/contextra-store/src/lsm/config.rs:144` | Attaches an injected key material source fluently to `LsmConfig` (for deterministic testing). |

#### `enum DurabilityConfigError`
- Ort: `crates/contextra-store/src/lsm/config.rs:25`
```rust
pub enum DurabilityConfigError
```

Varianten:

- `IncompatibleCombination{` — `crates/contextra-store/src/lsm/config.rs:27`

#### `enum DurabilityMode`
- Ort: `crates/contextra-store/src/lsm/config.rs:12`
```rust
pub enum DurabilityMode
```
- Impl-Block `crates/contextra-store/src/compat/license.rs:6`
- Impl-Block (Trait `Default`) `crates/contextra-store/src/lsm/config.rs:18`
- Impl-Block `crates/contextra-store/src/lsm/config.rs:36`

Varianten:

- `Full` — `crates/contextra-store/src/lsm/config.rs:13`
- `WalNoHmac` — `crates/contextra-store/src/lsm/config.rs:14`
- `MemoryOnly` — `crates/contextra-store/src/lsm/config.rs:15`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn validate_against_features( self, deletion_proof_active: bool, feature_ring: contextra_ports::license::FeatureRing, ) -> Result<(), DurabilityConfigError>` | `crates/contextra-store/src/compat/license.rs:12` |  |
| `pub fn validate_durability( self, integrity_chain_required: bool, require_persistence: bool, ) -> Result<(), DurabilityConfigError>` | `crates/contextra-store/src/lsm/config.rs:38` | Validates the durability mode against required integrity guarantees and persistence constraints. |

### Modul `lsm::engine`

#### `mod facade`
- Ort: `crates/contextra-store/src/lsm/engine.rs:6`
```rust
pub mod facade
```

#### `mod migration`
- Ort: `crates/contextra-store/src/lsm/engine.rs:7`
```rust
pub mod migration
```

#### `struct LsmStorage`
- Ort: `crates/contextra-store/src/lsm/engine.rs:37`
```rust
pub struct LsmStorage
```
Dokumentation: Primaere Schnittstelle fuer den LSM-Baum mit get, put, delete und scan.
- Impl-Block (Trait `StorageEngine`) `crates/contextra-store/src/compat/engine/lsm.rs:8`
- Impl-Block (Trait `StorageEngineHandle`) `crates/contextra-store/src/compat/engine/tenant_storage.rs:41`
- Impl-Block `crates/contextra-store/src/lsm/commit.rs:14`
- Impl-Block `crates/contextra-store/src/lsm/engine/facade.rs:10`
- Impl-Block `crates/contextra-store/src/lsm/engine/migration.rs:12`
- Impl-Block (Trait `Drop`) `crates/contextra-store/src/lsm/engine.rs:81`
- Impl-Block `crates/contextra-store/src/lsm/engine.rs:87`
- Impl-Block `crates/contextra-store/src/lsm/flush.rs:8`
- Impl-Block `crates/contextra-store/src/lsm/recovery/rollback.rs:21`
- Impl-Block `crates/contextra-store/src/lsm/recovery/startup.rs:37`
- Impl-Block `crates/contextra-store/src/lsm/scan.rs:49`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `config` | `LsmConfig` | `crates/contextra-store/src/lsm/engine.rs:38` |
| `key_manager` | `Option<Arc<KeyManager>>` | `crates/contextra-store/src/lsm/engine.rs:39` |
| `state` | `RwLock<LsmState>` | `crates/contextra-store/src/lsm/engine.rs:40` |
| `sstables` | `Arc<RwLock<Vec<Arc<SstableReader>>>>` | `crates/contextra-store/src/lsm/engine.rs:42` |
| `budget` | `Arc<crate::budget::ResourceTracker>` | `crates/contextra-store/src/lsm/engine.rs:44` |
| `block_cache` | `Arc<BlockCache>` | `crates/contextra-store/src/lsm/engine.rs:45` |
| `wal` | `RwLock<Arc<Wal>>` | `crates/contextra-store/src/lsm/engine.rs:46` |
| `snapshot_registry` | `Arc<SnapshotRegistry>` | `crates/contextra-store/src/lsm/engine.rs:47` |
| `compaction_engine` | `Arc<CompactionEngine>` | `crates/contextra-store/src/lsm/engine.rs:50` |
| `manifest` | `Arc<crate::manifest::Manifest>` | `crates/contextra-store/src/lsm/engine.rs:51` |
| `next_seq_no` | `AtomicU64` | `crates/contextra-store/src/lsm/engine.rs:52` |
| `last_committed_tx` | `AtomicU64` | `crates/contextra-store/src/lsm/engine.rs:53` |
| `last_applied_seq` | `AtomicU64` | `crates/contextra-store/src/lsm/engine.rs:54` |
| `flush_notify` | `Arc<tokio::sync::Notify>` | `crates/contextra-store/src/lsm/engine.rs:55` |
| `health` | `Arc<parking_lot::RwLock<StorageHealth>>` | `crates/contextra-store/src/lsm/engine.rs:56` |
| `commit_mutex` | `tokio::sync::Mutex<()>` | `crates/contextra-store/src/lsm/engine.rs:58` |
| `flush_mutex` | `tokio::sync::Mutex<()>` | `crates/contextra-store/src/lsm/engine.rs:60` |
| `flush_purge_fence` | `super::fence::FlushPurgeFence` | `crates/contextra-store/src/lsm/engine.rs:62` |
| `cancel_token` | `tokio_util::sync::CancellationToken` | `crates/contextra-store/src/lsm/engine.rs:65` |
| `task_tracker` | `tokio_util::task::TaskTracker` | `crates/contextra-store/src/lsm/engine.rs:66` |
| `flush_counter` | `AtomicU64` | `crates/contextra-store/src/lsm/engine.rs:67` |
| `segment_counter` | `AtomicU64` | `crates/contextra-store/src/lsm/engine.rs:68` |
| `budget_tracking_drift_bytes` | `std::sync::atomic::AtomicU64` | `crates/contextra-store/src/lsm/engine.rs:69` |
| `pending_commit_queue` | `tokio::sync::Mutex<Option<PendingCommitQueue>>` | `crates/contextra-store/src/lsm/engine.rs:70` |
| `wal_queue_depth` | `Arc<std::sync::atomic::AtomicUsize>` | `crates/contextra-store/src/lsm/engine.rs:71` |
| `pressure_rx` | `tokio::sync::watch::Receiver<crate::system_pressure::SystemPressure>` | `crates/contextra-store/src/lsm/engine.rs:72` |
| `observer_registry` | `ObserverRegistry` | `crates/contextra-store/src/lsm/engine.rs:74` |
| `clock` | `parking_lot::RwLock<Arc<dyn contextra_env::Clock>>` | `crates/contextra-store/src/lsm/engine.rs:75` |
| `ssi_validator` | `Arc<dyn contextra_mvcc::SsiValidator + Send + Sync>` | `crates/contextra-store/src/lsm/engine.rs:76` |
| `sequence_log_ssi_validator` | `Option<Arc<contextra_mvcc::SequenceLogSsiValidator>>` | `crates/contextra-store/src/lsm/engine.rs:77` |
| `metrics_sink` | `Arc<parking_lot::RwLock<Arc<dyn contextra_env::MetricsSink>>>` | `crates/contextra-store/src/lsm/engine.rs:78` |

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn cleanup_intent_locks_where<F>(&self, mut predicate: F) where F: FnMut(TxId) -> bool,` | `crates/contextra-store/src/lsm/commit.rs:34` | Clears intent locks matching a predicate, gracefully recovering from poisoned locks. |
| `pub fn clear_intent_locks_for_tx(&self, tx_id: TxId)` | `crates/contextra-store/src/lsm/commit.rs:43` | Clears intent locks for a specific transaction ID. |
| `pub fn clear_intent_locks_above_tx(&self, target_tx: TxId)` | `crates/contextra-store/src/lsm/commit.rs:48` | Removes all intent lock registrations associated with transactions strictly newer than target_tx. |
| `pub async fn get<'a>(&'a self, key: &'a [u8]) -> Result<Option<bytes::Bytes>>` | `crates/contextra-store/src/lsm/engine/facade.rs:12` | Inherent read operations for standalone usage. |
| `pub async fn get_at_seq<'a>(&'a self, key: &'a [u8], seq_no: u64) -> Result<Option<bytes::Bytes>>` | `crates/contextra-store/src/lsm/engine/facade.rs:17` | Punktlesen zu einer bestimmten Sequenznummer. |
| `pub async fn scan_prefix<'a>(&'a self, prefix: &'a [u8]) -> Result<Vec<(Vec<u8>, Vec<u8>)>>` | `crates/contextra-store/src/lsm/engine/facade.rs:22` | Praefix-Scan ueber alle Schichten. |
| `pub async fn scan_prefix_bounded<'a>( &'a self, prefix: &'a [u8], limit: usize, cursor: Option<&'a [u8]>, ) -> Result<(Vec<(Vec<u8>, Vec<u8>)>, Option<Vec<u8>>)>` | `crates/contextra-store/src/lsm/engine/facade.rs:27` | Begrenzter Praefix-Scan mit Paginierungs-Cursor. |
| `pub async fn scan_prefix_at<'a>( &'a self, prefix: &'a [u8], seq_no: u64, ) -> Result<Vec<(Vec<u8>, Vec<u8>)>>` | `crates/contextra-store/src/lsm/engine/facade.rs:37` | Praefix-Scan zu einer bestimmten Sequenznummer. |
| `pub async fn scan_bounded<'a>( &'a self, start: std::ops::Bound<&'a [u8]>, end: std::ops::Bound<&'a [u8]>, limit: usize, cursor: Option<&'a [u8]>, ) -> Result<(Vec<(Vec<u8>, Vec<u8>)>, Option<Vec<u8>>)>` | `crates/contextra-store/src/lsm/engine/facade.rs:46` | Bereichs-Scan mit Grenzen und Paginierungs-Cursor. |
| `pub async fn scan<'a>( &'a self, start: std::ops::Bound<&'a [u8]>, end: std::ops::Bound<&'a [u8]>, limit: Option<usize>, ) -> Result<Vec<(Vec<u8>, Vec<u8>)>>` | `crates/contextra-store/src/lsm/engine/facade.rs:57` | Bereichs-Scan ueber Schichten mit optionalem Limit. |
| `pub async fn put<'a>(&'a self, tx_id: TxId, key: &'a [u8], value: &'a [u8]) -> Result<()>` | `crates/contextra-store/src/lsm/engine/facade.rs:67` | Inherent write and maintenance operations. |
| `pub async fn put_if_absent<'a>(&'a self, tx_id: TxId, key: &'a [u8], value: &'a [u8]) -> Result<bool>` | `crates/contextra-store/src/lsm/engine/facade.rs:72` | Einfuegen nur wenn Schluessel nicht vorhanden. |
| `pub async fn delete_many<'a>(&'a self, tx_id: TxId, keys: Vec<Vec<u8>>) -> Result<u64>` | `crates/contextra-store/src/lsm/engine/facade.rs:77` | Loeschen mehrerer Schluessel. |
| `pub async fn delete_prefix<'a>(&'a self, tx_id: TxId, prefix: &'a [u8]) -> Result<u64>` | `crates/contextra-store/src/lsm/engine/facade.rs:82` | Loeschen aller Schluessel mit Praefix. |
| `pub async fn delete<'a>(&'a self, tx_id: TxId, key: &'a [u8]) -> Result<()>` | `crates/contextra-store/src/lsm/engine/facade.rs:87` | Loeschen eines einzelnen Schluessels. |
| `pub async fn commit<'a>(&'a self, tx_id: TxId) -> Result<()>` | `crates/contextra-store/src/lsm/engine/facade.rs:92` | Transaktion committen. |
| `pub async fn rollback<'a>(&'a self, tx_id: TxId) -> Result<()>` | `crates/contextra-store/src/lsm/engine/facade.rs:97` | Transaktion zurueckrollen. |
| `pub async fn pin_checkpoint<'a>(&'a self, seq_no: u64) -> Result<()>` | `crates/contextra-store/src/lsm/engine/facade.rs:102` | Checkpoint pinnen. |
| `pub async fn unpin_checkpoint<'a>(&'a self, seq_no: u64) -> Result<()>` | `crates/contextra-store/src/lsm/engine/facade.rs:107` | Checkpoint freigeben. |
| `pub async fn flush<'a>(&'a self) -> Result<()>` | `crates/contextra-store/src/lsm/engine/facade.rs:112` | MemTable synchron flushen. |
| `pub async fn stats<'a>(&'a self) -> Result<crate::stats::StorageStats>` | `crates/contextra-store/src/lsm/engine/facade.rs:117` | Speicherstatistiken ermitteln. |
| `pub async fn last_seq_no<'a>(&'a self) -> Result<u64>` | `crates/contextra-store/src/lsm/engine/facade.rs:122` | Letzte Sequenznummer abfragen. |
| `pub async fn last_tx_id<'a>(&'a self) -> Result<TxId>` | `crates/contextra-store/src/lsm/engine/facade.rs:127` | Letzte Transaktions-ID abfragen. |
| `pub async fn has_pending_legacy_wal_migration_path(path: &std::path::Path) -> Result<bool>` | `crates/contextra-store/src/lsm/engine/migration.rs:14` | Checks if a database directory contains any legacy WAL files requiring explicit migration. |
| `pub async fn has_pending_legacy_wal_migration(&self) -> Result<bool>` | `crates/contextra-store/src/lsm/engine/migration.rs:61` | Checks if the current storage instance's data directory contains any pending legacy WAL files. |
| `pub async fn migrate_legacy_wal_keys( config: &LsmConfig, clock: Arc<dyn contextra_env::Clock>, ) -> Result<usize>` | `crates/contextra-store/src/lsm/engine/migration.rs:66` | Explicitly migrates all legacy WAL files in the specified directory using the provided clock port. |
| `pub async fn open(config: LsmConfig) -> Result<Self>` | `crates/contextra-store/src/lsm/engine.rs:103` | Opens or recovers an LSM storage instance with the given configuration. # Errors Gibt `ContextraError::Storage` zurück, falls I/O-Fehler, Verzeichnis-Lock-Konfl |
| `pub async fn open_with_merge_operator( config: LsmConfig, merge_operator: Arc<dyn MergeOperator>, ) -> Result<Self>` | `crates/contextra-store/src/lsm/engine.rs:117` | Opens or recovers an LSM storage instance with the given configuration and a `MergeOperator`. # Errors Gibt `ContextraError::Storage` zurück bei Initialisierung |
| `pub fn pressure_receiver( &self, ) -> tokio::sync::watch::Receiver<crate::system_pressure::SystemPressure>` | `crates/contextra-store/src/lsm/engine.rs:125` | Returns a watch receiver for monitoring system pressure levels. |
| `pub fn shutdown(&self)` | `crates/contextra-store/src/lsm/engine.rs:132` | Signals the background compaction engine to stop. |
| `pub async fn wait_shutdown(&self)` | `crates/contextra-store/src/lsm/engine.rs:137` | Waits for all spawned tasks to shut down fully. |
| `pub async fn close(&self) -> Result<()>` | `crates/contextra-store/src/lsm/engine.rs:152` | Gracefully closes the storage engine, stopping background tasks and flushing active memtable to disk. # Errors Gibt `ContextraError::Storage` zurück, falls beim |
| `pub fn spawn_tracked<F>(&self, future: F) where F: std::future::Future<Output = ()> + Send + 'static,` | `crates/contextra-store/src/lsm/engine.rs:159` | Spawns a background task tracked by this storage instance. |
| `pub async fn simulate_wal_append_failure_for_test(&self)` | `crates/contextra-store/src/lsm/engine.rs:168` |  |
| `pub async fn simulate_wal_append_failure_for_tx_for_test(&self, _tx_id: u64)` | `crates/contextra-store/src/lsm/engine.rs:175` |  |
| `pub async fn restore_wal_file_handle_for_test(&self)` | `crates/contextra-store/src/lsm/engine.rs:182` |  |
| `pub fn budget_tracking_drift_bytes(&self) -> u64` | `crates/contextra-store/src/lsm/engine.rs:189` | Returns the accumulated total memory budget tracking drift in bytes caused by unbudgeted memtable puts during commit when memory limit was exceeded. |
| `pub fn register_observer(&self, observer: Arc<dyn WalObserver>)` | `crates/contextra-store/src/lsm/engine.rs:199` | Registers a new WAL observer. |
| `pub fn deregister_observer(&self, observer: &Arc<dyn WalObserver>)` | `crates/contextra-store/src/lsm/engine.rs:204` | Deregisters an observer matching the provided `Arc` reference via pointer equality. |
| `pub fn set_max_observer_latency(&self, latency: std::time::Duration)` | `crates/contextra-store/src/lsm/engine.rs:209` | Sets the maximum latency threshold for observer callbacks. |
| `pub fn max_observer_latency(&self) -> std::time::Duration` | `crates/contextra-store/src/lsm/engine.rs:214` | Returns the current maximum latency threshold for observer callbacks. |
| `pub fn metrics_sink(&self) -> Arc<dyn contextra_env::MetricsSink>` | `crates/contextra-store/src/lsm/engine.rs:219` | Returns the currently configured [`contextra_env::MetricsSink`]. |
| `pub fn set_metrics_sink(&self, sink: Arc<dyn contextra_env::MetricsSink>)` | `crates/contextra-store/src/lsm/engine.rs:224` | Sets the [`contextra_env::MetricsSink`] for recording operational metrics. |
| `pub fn with_metrics_sink(self, sink: Arc<dyn contextra_env::MetricsSink>) -> Self` | `crates/contextra-store/src/lsm/engine.rs:229` | Configures the [`contextra_env::MetricsSink`] fluently on `self` and returns `self`. |
| `pub fn clock(&self) -> Arc<dyn contextra_env::Clock>` | `crates/contextra-store/src/lsm/engine.rs:235` | Returns the currently configured clock port. |
| `pub fn set_clock(&self, clock: Arc<dyn contextra_env::Clock>)` | `crates/contextra-store/src/lsm/engine.rs:240` | Sets the injected clock port for deterministic execution timing (P28). |
| `pub fn with_clock(self, clock: Arc<dyn contextra_env::Clock>) -> Self` | `crates/contextra-store/src/lsm/engine.rs:246` | Configures the clock port fluently on `self` and returns `self`. |
| `pub fn set_ssi_validator( &mut self, validator: Arc<dyn contextra_mvcc::SsiValidator + Send + Sync>, )` | `crates/contextra-store/src/lsm/engine.rs:252` | Sets or replaces the [`contextra_mvcc::SsiValidator`] port for transaction conflict detection. |
| `pub fn with_ssi_validator( mut self, validator: Arc<dyn contextra_mvcc::SsiValidator + Send + Sync>, ) -> Self` | `crates/contextra-store/src/lsm/engine.rs:261` | Configures the [`contextra_mvcc::SsiValidator`] fluently on `self` and returns `self`. |
| `pub async fn get_tracked(&self, tx_id: TxId, key: &[u8]) -> Result<Option<Bytes>>` | `crates/contextra-store/src/lsm/engine.rs:279` | Performs a tracked read for transaction `tx_id`. Evaluates `snapshot_seq = next_seq_no - 1` ONCE (K2), registers the read key in `TxBuffer`, and reads the key a |
| `pub async fn get_at_seq_tracked( &self, tx_id: TxId, key: &[u8], snapshot_seq: u64, ) -> Result<Option<Bytes>>` | `crates/contextra-store/src/lsm/engine.rs:292` | Performs a tracked read for transaction `tx_id` at an explicit `snapshot_seq`. Registers the read key in `TxBuffer` at `snapshot_seq` and reads the key at `snap |
| `pub async fn scan_prefix_tracked( &self, tx_id: TxId, prefix: &[u8], ) -> Result<Vec<(Vec<u8>, Vec<u8>)>>` | `crates/contextra-store/src/lsm/engine.rs:305` | Performs a tracked prefix scan for transaction `tx_id`. Evaluates `snapshot_seq = last_applied_seq` ONCE, scans all matching entries, and registers each scanned |
| `pub fn last_applied_seq(&self) -> u64` | `crates/contextra-store/src/lsm/engine.rs:314` | Returns the highest sequence number applied to the MemTable and published. |
| `pub fn health(&self) -> StorageHealth` | `crates/contextra-store/src/lsm/engine.rs:320` | Returns the current storage health status. |
| `pub fn request_flush(&self)` | `crates/contextra-store/src/lsm/engine.rs:325` | Requests an asynchronous background flush without blocking the caller or holding commit locks. |
| `pub fn has_observers(&self) -> bool` | `crates/contextra-store/src/lsm/engine.rs:330` | Returns true if at least one observer is registered. |
| `pub fn next_seq_no_for_test(&self) -> u64` | `crates/contextra-store/src/lsm/engine.rs:337` |  |
| `pub fn sstables_for_test(&self) -> Arc<RwLock<Vec<Arc<SstableReader>>>>` | `crates/contextra-store/src/lsm/engine.rs:353` |  |
| `pub fn block_cache_for_test(&self) -> Arc<BlockCache>` | `crates/contextra-store/src/lsm/engine.rs:359` |  |
| `pub fn budget_for_test(&self) -> Arc<crate::budget::ResourceTracker>` | `crates/contextra-store/src/lsm/engine.rs:365` |  |
| `pub fn manifest_for_test(&self) -> Arc<crate::manifest::Manifest>` | `crates/contextra-store/src/lsm/engine.rs:371` |  |
| `pub async fn maybe_compact_for_test(&self) -> Result<bool>` | `crates/contextra-store/src/lsm/engine.rs:377` |  |
| `pub fn data_path(&self) -> &std::path::Path` | `crates/contextra-store/src/lsm/engine.rs:406` | Returns the data directory path. |
| `pub fn vfs(&self) -> &std::sync::Arc<dyn crate::io_seam::Vfs>` | `crates/contextra-store/src/lsm/engine.rs:411` | Returns the VFS instance. |
| `pub async fn scan_view(self: &Arc<Self>) -> crate::compat::scanner::StoreScanView` | `crates/contextra-store/src/lsm/engine.rs:420` | Erzeugt eine entkoppelte [`StoreScanView`] für Scanner und Purger. Dies ist die einzige erlaubte Stelle, an der `LsmStorage` seine Interna an Scanner weitergibt |
| `pub async fn force_flush(&self) -> Result<()>` | `crates/contextra-store/src/lsm/flush.rs:9` |  |
| `pub async fn maybe_compact(&self) -> Result<bool>` | `crates/contextra-store/src/lsm/flush.rs:17` | Evaluates whether compaction should run and performs it if needed. Uses the persistent CompactionEngine instance to maintain a stable counter across calls, prev |
| `pub async fn rollback_to_tx(&self, target_tx: TxId) -> Result<()>` | `crates/contextra-store/src/lsm/recovery/rollback.rs:23` | Fuehrt ein Rollback des Datenbestands auf `target_tx` aus. |
| `pub async fn new(config: LsmConfig) -> Result<Self>` | `crates/contextra-store/src/lsm/recovery/startup.rs:39` | Oeffnet oder erzeugt den LSM-Speicher mit der uebergebenen Konfiguration. |
| `pub async fn new_with_merge_operator( config: LsmConfig, merge_operator: Option<Arc<dyn MergeOperator>>, ) -> Result<Self>` | `crates/contextra-store/src/lsm/recovery/startup.rs:44` | Oeffnet oder erzeugt den LSM-Speicher mit optionalem MergeOperator fuer Compaction. |
| `pub async fn point_lookup_metrics(&self, key: &[u8]) -> (usize, usize, usize, usize, bool)` | `crates/contextra-store/src/lsm/scan.rs:51` | Evaluates detailed traversal metrics (evaluated_sstables, bloom_passes, range_passes, block_reads, found) for point lookups. |

Assoziierte Konstanten/Typen:

- `pub const ROLLBACK_INLINE_THRESHOLD_ENTRIES: usize = 1` — `crates/contextra-store/src/lsm/engine.rs:196`

#### `enum StorageHealth`
- Ort: `crates/contextra-store/src/lsm/engine.rs:29`
```rust
pub enum StorageHealth
```
Dokumentation: Health status of the storage engine.

Varianten:

- `Healthy` — `crates/contextra-store/src/lsm/engine.rs:30`
- `FlushFailing` — `crates/contextra-store/src/lsm/engine.rs:31`

### Modul `lsm::fence`

#### `struct FlushPurgeFence`
- Ort: `crates/contextra-store/src/lsm/fence.rs:24`
```rust
pub struct FlushPurgeFence
```
Dokumentation: Koordiniert das gegenseitige Fencing zwischen MemTable-Flush und Purge-Kompaktion. Atomares Fencing: Garantierter wechselseitiger Ausschluss über SeqCst-Flags mit striktem Tie-Breaking zugunsten von Purge.
- Impl-Block `crates/contextra-store/src/lsm/fence.rs:29`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub const fn new() -> Self` | `crates/contextra-store/src/lsm/fence.rs:31` | Erzeugt ein neues Fence-Paar. |
| `pub fn enter_flush(&self) -> bool` | `crates/contextra-store/src/lsm/fence.rs:41` | Markiert den Beginn eines Flush-Vorgangs. Gibt `true` zurück, wenn kein Purge aktiv ist und Flush erworben wurde. |
| `pub fn leave_flush(&self)` | `crates/contextra-store/src/lsm/fence.rs:52` | Beendet den Flush-Abschnitt. |
| `pub fn enter_purge(&self) -> bool` | `crates/contextra-store/src/lsm/fence.rs:59` | Markiert den Beginn einer Purge-Kompaktion (Priorität). Gibt `true` zurück, wenn Purge exklusiven Zugriff erlangt hat. |
| `pub fn leave_purge(&self)` | `crates/contextra-store/src/lsm/fence.rs:75` | Beendet den Purge-Abschnitt. |
| `pub fn is_purge_active(&self) -> bool` | `crates/contextra-store/src/lsm/fence.rs:81` | Prüft nicht-blockierend, ob ein Purge ansteht oder aktiv ist. |
| `pub fn is_flush_active(&self) -> bool` | `crates/contextra-store/src/lsm/fence.rs:87` | Prüft nicht-blockierend, ob ein Flush aktiv ist. |

#### `fn load_snapshot_tx`
- Ort: `crates/contextra-store/src/lsm/fence.rs:111`
```rust
pub fn load_snapshot_tx(target: &AtomicU64) -> u64
```
Dokumentation: Lädt den aktuellen Sichtbarkeitshorizont für einen konsistenten Snapshot mit `Acquire`-Semantik.

#### `fn publish_visibility`
- Ort: `crates/contextra-store/src/lsm/fence.rs:94`
```rust
pub fn publish_visibility(target: &AtomicU64, new_tx: u64)
```
Dokumentation: Publiziert Sichtbarkeitsfortschritte (`last_committed_tx`) mit `Release`-Semantik.

### Modul `lsm::guard`

#### `struct CommitGuard`
- Ort: `crates/contextra-store/src/lsm/guard.rs:10`
```rust
pub struct CommitGuard<'a>
```
Dokumentation: Proof that `commit_mutex` is currently held by the calling task. Can only be constructed while holding the mutex guard.

#### `struct LsmState`
- Ort: `crates/contextra-store/src/lsm/guard.rs:14`
```rust
pub struct LsmState
```

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `memtable` | `Arc<MemTable>` | `crates/contextra-store/src/lsm/guard.rs:15` |
| `immutable_memtables` | `Vec<Arc<MemTable>>` | `crates/contextra-store/src/lsm/guard.rs:16` |

### Modul `lsm::lock_order`

#### `const RANK_BLOCK_CACHE`
- Ort: `crates/contextra-store/src/lsm/lock_order.rs:89`
```rust
pub const RANK_BLOCK_CACHE: u32 = 950
```
Dokumentation: Rang 950: SSTable-Block-Cache-Sperre (`SieveCacheBackend::state`).

#### `const RANK_COMMIT_MUTEX`
- Ort: `crates/contextra-store/src/lsm/lock_order.rs:65`
```rust
pub const RANK_COMMIT_MUTEX: u32 = 200
```
Dokumentation: Rang 200: Globaler Commit-Mutex des Stores (`commit_mutex`), entspricht Commit-Barriere.

#### `const RANK_DOCUMENT_INTENT`
- Ort: `crates/contextra-store/src/lsm/lock_order.rs:62`
```rust
pub const RANK_DOCUMENT_INTENT: u32 = 100
```
Dokumentation: Rang 100: Transaktionale Absichtssperren auf Dokument-/Schlüsselebene (`intent_locks`).

#### `const RANK_FLUSH_MUTEX`
- Ort: `crates/contextra-store/src/lsm/lock_order.rs:68`
```rust
pub const RANK_FLUSH_MUTEX: u32 = 300
```
Dokumentation: Rang 300: Flush-Mutex des Stores (`flush_mutex`), serialisiert Flush-Vorgänge.

#### `const RANK_MANIFEST_MUTEX`
- Ort: `crates/contextra-store/src/lsm/lock_order.rs:83`
```rust
pub const RANK_MANIFEST_MUTEX: u32 = 800
```
Dokumentation: Rang 800: Manifest-Append-Sperre (`Manifest::file`).

#### `const RANK_MEMTABLE_SHARDS`
- Ort: `crates/contextra-store/src/lsm/lock_order.rs:80`
```rust
pub const RANK_MEMTABLE_SHARDS: u32 = 700
```
Dokumentation: Rang 700: MemTable-Shards (`MemTable::shards`), atomare RAM-Modifikationen.

#### `const RANK_OBSERVER_REGISTRY`
- Ort: `crates/contextra-store/src/lsm/lock_order.rs:86`
```rust
pub const RANK_OBSERVER_REGISTRY: u32 = 900
```
Dokumentation: Rang 900: Observer-Registry-Sperre (`ObserverRegistry::observers`).

#### `const RANK_SSTABLES_RWLOCK`
- Ort: `crates/contextra-store/src/lsm/lock_order.rs:77`
```rust
pub const RANK_SSTABLES_RWLOCK: u32 = 600
```
Dokumentation: Rang 600: SSTable-Katalog (`sstables`), schützt die Liste aktiver L0..Ln SSTables.

#### `const RANK_STATE_RWLOCK`
- Ort: `crates/contextra-store/src/lsm/lock_order.rs:71`
```rust
pub const RANK_STATE_RWLOCK: u32 = 400
```
Dokumentation: Rang 400: LSM-Zustand (`state`), schützt MemTable-Zeiger und unveränderliche MemTables.

#### `const RANK_WAL_TRUNCATE`
- Ort: `crates/contextra-store/src/lsm/lock_order.rs:74`
```rust
pub const RANK_WAL_TRUNCATE: u32 = 500
```
Dokumentation: Rang 500: WAL-Truncate-Sperre (`wal.truncate_lock`), serialisiert Append/Truncate im WAL.

### Modul `lsm::observer::adapter`

#### `struct AsyncObserverAdapter`
- Ort: `crates/contextra-store/src/lsm/observer/adapter.rs:13`
```rust
pub struct AsyncObserverAdapter
```
Dokumentation: Non-blocking, bounded async adapter wrapper for `WalObserver` implementations.
- Impl-Block `crates/contextra-store/src/lsm/observer/adapter.rs:18`
- Impl-Block (Trait `WalObserver`) `crates/contextra-store/src/lsm/observer/adapter.rs:40`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new( inner: Arc<dyn WalObserver>, capacity: usize, ) -> (Self, tokio::task::JoinHandle<()>)` | `crates/contextra-store/src/lsm/observer/adapter.rs:19` |  |

### Modul `lsm::observer::registry`

#### `struct ObserverRegistry`
- Ort: `crates/contextra-store/src/lsm/observer/registry.rs:19`
```rust
pub struct ObserverRegistry
```
Dokumentation: Configuration and handle for managing WAL observers with bounded delivery and circuit breaker.
- Impl-Block (Trait `Default`) `crates/contextra-store/src/lsm/observer/registry.rs:26`
- Impl-Block `crates/contextra-store/src/lsm/observer/registry.rs:37`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn is_empty(&self) -> bool` | `crates/contextra-store/src/lsm/observer/registry.rs:39` | Returns true if no observers are registered. |
| `pub fn new() -> Self` | `crates/contextra-store/src/lsm/observer/registry.rs:44` | Creates a new, empty `ObserverRegistry`. |
| `pub fn register_observer(&self, observer: Arc<dyn WalObserver>)` | `crates/contextra-store/src/lsm/observer/registry.rs:49` | Registers a new WAL observer. |
| `pub fn deregister_observer(&self, observer: &Arc<dyn WalObserver>)` | `crates/contextra-store/src/lsm/observer/registry.rs:56` | Deregisters an observer matching the provided `Arc` reference via pointer equality. |
| `pub fn set_max_observer_latency(&self, latency: Duration)` | `crates/contextra-store/src/lsm/observer/registry.rs:72` | Sets the maximum latency threshold for observer callbacks. |
| `pub fn max_observer_latency(&self) -> Duration` | `crates/contextra-store/src/lsm/observer/registry.rs:77` | Returns the current maximum latency threshold. |
| `pub fn set_clock(&self, clock: Arc<dyn Clock>)` | `crates/contextra-store/src/lsm/observer/registry.rs:82` | Sets the injected clock port for deterministic execution timing (P28). |
| `pub fn dropped_count(&self) -> u64` | `crates/contextra-store/src/lsm/observer/registry.rs:87` | Returns the total number of dropped events across all observers (including deregistered ones). |
| `pub fn dropped_count_for(&self, observer: &Arc<dyn WalObserver>) -> u64` | `crates/contextra-store/src/lsm/observer/registry.rs:97` | Returns the number of dropped events for a specific registered observer. |
| `pub fn is_circuit_breaker_open(&self, observer: &Arc<dyn WalObserver>) -> bool` | `crates/contextra-store/src/lsm/observer/registry.rs:107` | Returns whether the circuit breaker is currently open for a specific observer. |
| `pub fn is_any_circuit_breaker_open(&self) -> bool` | `crates/contextra-store/src/lsm/observer/registry.rs:119` | Returns whether any registered observer currently has an open circuit breaker. |
| `pub fn clear_circuit_breaker(&self, observer: &Arc<dyn WalObserver>)` | `crates/contextra-store/src/lsm/observer/registry.rs:127` | Resets the circuit breaker state for a specific observer. |
| `pub fn notify_with_context(&self, entries: &[WalEntry], seq_no: u64, ctx: CommitContext)` | `crates/contextra-store/src/lsm/observer/registry.rs:138` | Synchronously notifies all registered observers of a committed batch with explicit CommitContext. |
| `pub fn notify(&self, entries: &[WalEntry], seq_no: u64, tx_id: TxId, origin: WriteOrigin)` | `crates/contextra-store/src/lsm/observer/registry.rs:262` | Synchronously notifies all registered observers of a committed batch. |

### Modul `lsm::observer::types`

#### `struct CommitContext`
- Ort: `crates/contextra-store/src/lsm/observer/types.rs:63`
```rust
pub struct CommitContext
```
Dokumentation: Context describing a committed transaction passed to observers.

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `tx_id` | `TxId` | `crates/contextra-store/src/lsm/observer/types.rs:64` |
| `origin` | `WriteOrigin` | `crates/contextra-store/src/lsm/observer/types.rs:65` |
| `durable` | `bool` | `crates/contextra-store/src/lsm/observer/types.rs:66` |

#### `struct CommittedBatch`
- Ort: `crates/contextra-store/src/lsm/observer/types.rs:70`
```rust
pub struct CommittedBatch<'a>
```
Dokumentation: Batch of committed WAL entries passed to observers.

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `entries` | `Vec<WalEntryRef<'a>>` | `crates/contextra-store/src/lsm/observer/types.rs:71` |
| `origin` | `WriteOrigin` | `crates/contextra-store/src/lsm/observer/types.rs:72` |

#### `struct WalEntryRef`
- Ort: `crates/contextra-store/src/lsm/observer/types.rs:25`
```rust
pub struct WalEntryRef<'a>
```
Dokumentation: Lightweight view reference into a `WalEntry`.
- Impl-Block `crates/contextra-store/src/lsm/observer/types.rs:29`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `entry` | `&'a WalEntry` | `crates/contextra-store/src/lsm/observer/types.rs:26` |

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new(entry: &'a WalEntry) -> Self` | `crates/contextra-store/src/lsm/observer/types.rs:31` | Creates a new `WalEntryRef` from a `WalEntry`. |
| `pub fn op(&self) -> &'a WalOp` | `crates/contextra-store/src/lsm/observer/types.rs:36` | Returns a reference to the operation inside the entry. |
| `pub fn seq_no(&self) -> u64` | `crates/contextra-store/src/lsm/observer/types.rs:41` | Returns the sequence number of the entry. |
| `pub fn checksum(&self) -> &'a [u8` | `crates/contextra-store/src/lsm/observer/types.rs:46` | Returns a reference to the entry's checksum. |
| `pub fn prev_hmac(&self) -> &'a [u8` | `crates/contextra-store/src/lsm/observer/types.rs:51` | Returns a reference to the entry's previous HMAC checksum. |
| `pub fn tx_id(&self) -> TxId` | `crates/contextra-store/src/lsm/observer/types.rs:56` | Returns the transaction ID associated with this entry. |

#### `enum WriteOrigin`
- Ort: `crates/contextra-store/src/lsm/observer/types.rs:18`
```rust
pub enum WriteOrigin
```
Dokumentation: Origin tag indicating where a committed batch originated from.

Varianten:

- `UserWrite` — `crates/contextra-store/src/lsm/observer/types.rs:20`

#### `trait WalObserver`
- Ort: `crates/contextra-store/src/lsm/observer/types.rs:76`
```rust
pub trait WalObserver: Send + Sync
```
Dokumentation: Synchronous, deterministic observer interface for committed WAL batches.

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `fn requires_durability(&self) -> bool` | `crates/contextra-store/src/lsm/observer/types.rs:81` | Indicates whether this observer requires durable WAL commits. Returns `false` by default. Observers setting this to `false` will also receive notifications for  |
| `fn on_commit(&self, batch: &CommittedBatch<'_>, seq_no: u64, tx_id: TxId)` | `crates/contextra-store/src/lsm/observer/types.rs:91` | Notification callback executed upon batch commit. Invoked via a bounded, non-blocking dispatcher mechanism. The commit path enforces a configurable maximum late |

#### `const DEFAULT_MAX_OBSERVER_LATENCY`
- Ort: `crates/contextra-store/src/lsm/observer/types.rs:13`
```rust
pub const DEFAULT_MAX_OBSERVER_LATENCY: Duration = Duration::from_millis(1)
```
Dokumentation: Default maximum latency allowed for WAL observer callback execution (1 millisecond).

### Modul `lsm::recovery::rollback`

#### `const INTENT_MAGIC`
- Ort: `crates/contextra-store/src/lsm/recovery/rollback.rs:16`
```rust
pub const INTENT_MAGIC: &[u8] = b""
```
Dokumentation: Magische Konstante fuer Rollback-Intent-Dateien.

#### `const MIN_ENTRIES_FOR_SSTABLE_REBUILD`
- Ort: `crates/contextra-store/src/lsm/recovery/rollback.rs:19`
```rust
pub const MIN_ENTRIES_FOR_SSTABLE_REBUILD: usize = 8
```
Dokumentation: Mindestanzahl an ueberlebenden Eintraegen, um eine neue SSTable zu bauen.

### Modul `lsm::scan`

#### `const MAX_SCAN_MERGE_ACCUMULATOR`
- Ort: `crates/contextra-store/src/lsm/scan.rs:8`
```rust
pub const MAX_SCAN_MERGE_ACCUMULATOR: usize = 100_000
```

### Modul `lsm::tests::recovery_tests`

#### `mod rollback`
- Ort: `crates/contextra-store/src/lsm/tests/recovery_tests.rs:8`
```rust
pub mod rollback
```

#### `mod startup`
- Ort: `crates/contextra-store/src/lsm/tests/recovery_tests.rs:9`
```rust
pub mod startup
```

### Modul `lsm::tests::scan_tests`

#### `mod bounded`
- Ort: `crates/contextra-store/src/lsm/tests/scan_tests.rs:9`
```rust
pub mod bounded
```

#### `mod prefix`
- Ort: `crates/contextra-store/src/lsm/tests/scan_tests.rs:10`
```rust
pub mod prefix
```

#### `mod prop`
- Ort: `crates/contextra-store/src/lsm/tests/scan_tests.rs:11`
```rust
pub mod prop
```

### Modul `lsm::validate`

#### `fn derive_doc_id`
- Ort: `crates/contextra-store/src/lsm/validate.rs:29`
```rust
pub fn derive_doc_id(key: &[u8]) -> DocId
```
Dokumentation: Derives a deterministic `DocId` from key bytes using BLAKE3.

#### `fn derive_doc_id`
- Ort: `crates/contextra-store/src/lsm/validate.rs:38`
```rust
pub fn derive_doc_id(key: &[u8]) -> DocId
```
Dokumentation: Derives a deterministic 128-bit `DocId` from key bytes using BLAKE3.
- Gate: `#[cfg(feature = "docid-128")]`

#### `fn validate_key`
- Ort: `crates/contextra-store/src/lsm/validate.rs:13`
```rust
pub fn validate_key(key: &[u8]) -> Result<()>
```
Dokumentation: Validates that a key is non-empty and does not exceed `MAX_KEY_SIZE`.

#### `fn validate_value`
- Ort: `crates/contextra-store/src/lsm/validate.rs:46`
```rust
pub fn validate_value(value: &[u8]) -> Result<()>
```
Dokumentation: Validates that value bytes do not exceed `MAX_VALUE_SIZE`.

### Modul `manifest`

#### `const DEFAULT_ROLLOVER_THRESHOLD_BYTES`
- Ort: `crates/contextra-store/src/manifest/mod.rs:25`
```rust
pub const DEFAULT_ROLLOVER_THRESHOLD_BYTES: u64 = 64 * 1024
```
Dokumentation: Default file size threshold (64 KB) to trigger MANIFEST rollover.

#### `const MAX_MANIFEST_ENTRY_SIZE`
- Ort: `crates/contextra-store/src/manifest/mod.rs:22`
```rust
pub const MAX_MANIFEST_ENTRY_SIZE: u32 = 1024 * 1024
```
Dokumentation: Maximum allowed payload size for a single manifest entry (1 MB).

### Modul `manifest::core`

#### `const CURRENT_MANIFEST_VERSION`
- Ort: `crates/contextra-store/src/manifest/core.rs:15`
```rust
pub const CURRENT_MANIFEST_VERSION: u8 = 2
```

#### `const MANIFEST_HEADER_MAGIC`
- Ort: `crates/contextra-store/src/manifest/core.rs:14`
```rust
pub const MANIFEST_HEADER_MAGIC: &[u8
```

#### `const MANIFEST_VERSION_V1`
- Ort: `crates/contextra-store/src/manifest/core.rs:17`
```rust
pub const MANIFEST_VERSION_V1: u8 = 1
```

#### `const MANIFEST_VERSION_V2`
- Ort: `crates/contextra-store/src/manifest/core.rs:19`
```rust
pub const MANIFEST_VERSION_V2: u8 = 2
```

### Modul `manifest::entry`

#### `enum ManifestEntry`
- Ort: `crates/contextra-store/src/manifest/entry.rs:21`
```rust
pub enum ManifestEntry
```
Dokumentation: An entry in the SSTable manifest log.
- Impl-Block `crates/contextra-store/src/manifest/entry.rs:47`

Varianten:

- `Add{ path: PathBuf, max_tx: u64 }` — `crates/contextra-store/src/manifest/entry.rs:23`
- `Remove{ path: PathBuf }` — `crates/contextra-store/src/manifest/entry.rs:25`
- `RollbackComplete{ target_tx: u64 }` — `crates/contextra-store/src/manifest/entry.rs:27`
- `Replace{` — `crates/contextra-store/src/manifest/entry.rs:29`
- `WalCheckpoint{ hmac: [u8; 32] }` — `crates/contextra-store/src/manifest/entry.rs:44`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn to_bytes(&self) -> StoreResult<Vec<u8>>` | `crates/contextra-store/src/manifest/entry.rs:49` | Serializes the entry into binary format with a 4-byte total size prefix and a 4-byte CRC32. |
| `pub fn from_bytes(data: &[u8]) -> StoreResult<Self>` | `crates/contextra-store/src/manifest/entry.rs:118` | Deserializes a manifest entry from a slice containing `[crc32: u32 LE] [payload...]`. |

### Modul `manifest::handle`

#### `struct Manifest`
- Ort: `crates/contextra-store/src/manifest/handle.rs:15`
```rust
pub struct Manifest
```
Dokumentation: Append-only Manifest file handle for recording active SSTable set transitions.
- Impl-Block `crates/contextra-store/src/manifest/apply.rs:13`
- Impl-Block `crates/contextra-store/src/manifest/cleanup.rs:14`
- Impl-Block `crates/contextra-store/src/manifest/core.rs:25`
- Impl-Block (Trait `std::fmt::Debug`) `crates/contextra-store/src/manifest/handle.rs:22`
- Impl-Block `crates/contextra-store/src/manifest/handle.rs:30`
- Impl-Block `crates/contextra-store/src/manifest/rollover.rs:11`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `path` | `PathBuf` | `crates/contextra-store/src/manifest/handle.rs:16` |
| `file` | `tokio::sync::Mutex<tokio::fs::File>` | `crates/contextra-store/src/manifest/handle.rs:17` |
| `_dir_lock` | `Option<DirLock>` | `crates/contextra-store/src/manifest/handle.rs:18` |
| `is_poisoned` | `AtomicBool` | `crates/contextra-store/src/manifest/handle.rs:19` |

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub async fn append(&self, entry: &ManifestEntry) -> StoreResult<()>` | `crates/contextra-store/src/manifest/apply.rs:15` | Appends a new entry to the manifest and performs flush + fsync. |
| `pub async fn append_batch(&self, entries: &[ManifestEntry]) -> StoreResult<()>` | `crates/contextra-store/src/manifest/apply.rs:20` | Appends multiple entries to the manifest in a single atomic batch with flush + fsync. |
| `pub fn reconstruct_valid_sstables(entries: &[ManifestEntry]) -> Vec<(PathBuf, u64)>` | `crates/contextra-store/src/manifest/cleanup.rs:17` | Reconstructs the ordered list of currently valid SSTable file names (or path components) and their rank (shadowing order: smaller rank = older) from a sequence  |
| `pub fn reconstruct_dead_sstables(entries: &[ManifestEntry]) -> HashSet<PathBuf>` | `crates/contextra-store/src/manifest/cleanup.rs:64` | Reconstructs the set of SSTable file names (or path components) that are proven dead (explicitly removed via `Remove` or `Replace.removed`) and are not in the v |
| `pub fn extract_high_water_mark(entries: &[ManifestEntry]) -> Option<[u8` | `crates/contextra-store/src/manifest/cleanup.rs:101` | Extracts the most recent WAL high-water-mark HMAC from a list of manifest entries. |
| `pub async fn load(path: &Path) -> StoreResult<Vec<ManifestEntry>>` | `crates/contextra-store/src/manifest/core.rs:27` | Loads all valid manifest entries from `path`. |
| `pub async fn open(path: impl AsRef<Path>) -> StoreResult<Self>` | `crates/contextra-store/src/manifest/handle.rs:32` | Opens or creates an append-only Manifest file at `path`. |
| `pub fn is_poisoned(&self) -> bool` | `crates/contextra-store/src/manifest/handle.rs:97` | Checks if the manifest handle has been poisoned by a preceding fsync failure (INV-FSYNC-1). |
| `pub fn poison(&self)` | `crates/contextra-store/src/manifest/handle.rs:103` | Explicitly poisons the manifest handle (INV-FSYNC-1). |
| `pub fn path(&self) -> &Path` | `crates/contextra-store/src/manifest/handle.rs:108` | Returns the filesystem path to the manifest file. |
| `pub async fn rollover(&self, live_entries: &[ManifestEntry]) -> Result<()>` | `crates/contextra-store/src/manifest/rollover.rs:28` | Evaluates whether the MANIFEST file size exceeds `threshold_bytes` and performs an atomic rollover if needed. Rollover process: (a) Write current live entries ( |
| `pub async fn maybe_rollover( &self, live_entries: &[ManifestEntry], threshold_bytes: u64, ) -> Result<bool>` | `crates/contextra-store/src/manifest/rollover.rs:45` | Evaluates whether the MANIFEST file size exceeds `threshold_bytes` and performs an atomic rollover if needed. Rollover process: (a) Write current live entries ( |

### Modul `memtable`

#### `mod iter`
- Ort: `crates/contextra-store/src/memtable.rs:13`
```rust
pub mod iter
```

#### `mod mvcc`
- Ort: `crates/contextra-store/src/memtable.rs:14`
```rust
pub mod mvcc
```

#### `mod shard`
- Ort: `crates/contextra-store/src/memtable.rs:15`
```rust
pub mod shard
```

#### `mod tombstone`
- Ort: `crates/contextra-store/src/memtable.rs:16`
```rust
pub mod tombstone
```

#### `struct MemTable`
- Ort: `crates/contextra-store/src/memtable.rs:29`
```rust
pub struct MemTable
```
- Impl-Block `crates/contextra-store/src/memtable/iter.rs:14`
- Impl-Block `crates/contextra-store/src/memtable/mvcc.rs:14`
- Impl-Block `crates/contextra-store/src/memtable.rs:38`
- Impl-Block (Trait `Default`) `crates/contextra-store/src/memtable.rs:99`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `shards` | `[MemTableShard; SHARD_COUNT]` | `crates/contextra-store/src/memtable.rs:32` |
| `size` | `AtomicUsize` | `crates/contextra-store/src/memtable.rs:33` |
| `min_tx` | `AtomicU64` | `crates/contextra-store/src/memtable.rs:34` |
| `max_tx` | `AtomicU64` | `crates/contextra-store/src/memtable.rs:35` |

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn iter(&self) -> Vec<(Bytes, Bytes, u64, u64)>` | `crates/contextra-store/src/memtable/iter.rs:16` | Iterates all entries across all shards in ascending key order. |
| `pub fn scan_prefix_into( &self, prefix: &[u8], seq_no: u64, max_tx: TxId, target: &mut BTreeMap<Bytes, (Bytes, u64)>, )` | `crates/contextra-store/src/memtable/iter.rs:34` | Scans entries matching a key prefix into `target`. Higher `seq_no` versions shadow lower `seq_no` versions. |
| `pub fn scan_prefix_into_matching<F>( &self, prefix: &[u8], seq_no: u64, max_tx: TxId, target: &mut BTreeMap<Bytes, (Bytes, u64)>, predicate: F, ) where F: Fn(&[u8], u64, u64) -> bool,` | `crates/contextra-store/src/memtable/iter.rs:45` | Scans entries matching a key prefix into `target` with an additional matching predicate. |
| `pub fn scan_range_into( &self, start: Bound<&[u8]>, end: Bound<&[u8]>, seq_no: u64, max_tx: TxId, target: &mut BTreeMap<Bytes, (Bytes, u64)>, )` | `crates/contextra-store/src/memtable/iter.rs:100` | Scans entries matching a key range into `target` according to MVCC visibility rules. |
| `pub fn scan_range_into_matching<F>( &self, start: Bound<&[u8]>, end: Bound<&[u8]>, seq_no: u64, max_tx: TxId, target: &mut BTreeMap<Bytes, (Bytes, u64)>, predicate: F, ) where F: Fn(&[u8], u64, u64) -> bool,` | `crates/contextra-store/src/memtable/iter.rs:112` | Scans entries matching a key range into `target` with an additional matching predicate. |
| `pub fn iter_latest(&self) -> Vec<(Bytes, Bytes, u64, u64)>` | `crates/contextra-store/src/memtable/iter.rs:166` | Iterates over only the latest version of each key in sorted key order (including tombstones). |
| `pub fn put(&self, key: Bytes, value: Bytes, seq_no: u64, tx_id: u64)` | `crates/contextra-store/src/memtable/mvcc.rs:16` | Inserts a key-value pair with sequence number and transaction id into the appropriate shard. |
| `pub fn tx_range(&self) -> (u64, u64)` | `crates/contextra-store/src/memtable/mvcc.rs:75` | Returns the minimum and maximum transaction IDs contained in this memtable. |
| `pub fn rollback(&self, tx_id: u64)` | `crates/contextra-store/src/memtable/mvcc.rs:83` | Rollback: Removes all entries belonging to `tx_id` across all shards. |
| `pub fn get(&self, key: &[u8]) -> Option<(Bytes, u64)>` | `crates/contextra-store/src/memtable/mvcc.rs:129` | Retrieves the latest value and sequence number for a key. |
| `pub fn get_at_seq(&self, key: &[u8], seq_no: u64, max_tx: u64) -> Option<(Bytes, u64, u64)>` | `crates/contextra-store/src/memtable/mvcc.rs:139` | Retrieves a value, sequence number, and transaction ID by key at or below a specific sequence number and bounded by maximum transaction ID for Snapshot Isolatio |
| `pub fn new() -> Self` | `crates/contextra-store/src/memtable.rs:40` | Creates a new, empty sharded MemTable. |
| `pub fn size(&self) -> usize` | `crates/contextra-store/src/memtable.rs:62` | Returns the approximate size in bytes. |
| `pub fn is_empty(&self) -> bool` | `crates/contextra-store/src/memtable.rs:67` | Returns true if the memtable is empty. |
| `pub fn shard_entry_counts(&self) -> [usize` | `crates/contextra-store/src/memtable.rs:73` | Returns entry count for each shard (used for testing shard distribution). |
| `pub fn shard_for(key: &[u8]) -> usize` | `crates/contextra-store/src/memtable.rs:79` | Exposes shard calculation for testing key distribution. |
| `pub fn shard_range_for_prefix(prefix: &[u8]) -> std::ops::RangeInclusive<usize>` | `crates/contextra-store/src/memtable.rs:85` | Exposes shard range for prefix for testing and internal scanning. |
| `pub fn shard_range_for_bounds( start: std::ops::Bound<&[u8]>, end: std::ops::Bound<&[u8]>, ) -> std::ops::RangeInclusive<usize>` | `crates/contextra-store/src/memtable.rs:91` | Exposes shard range for bounds for testing and internal scanning. |

### Modul `memtable::shard`

#### `struct MemTableShard`
- Ort: `crates/contextra-store/src/memtable/shard.rs:64`
```rust
pub struct MemTableShard
```
Dokumentation: A single shard of the MemTable, holding a subset of the key space.
- Impl-Block `crates/contextra-store/src/memtable/shard.rs:68`
- Impl-Block (Trait `Default`) `crates/contextra-store/src/memtable/shard.rs:76`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `entries` | `RwLock<MemTableMap>` | `crates/contextra-store/src/memtable/shard.rs:65` |

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new() -> Self` | `crates/contextra-store/src/memtable/shard.rs:69` |  |

#### `fn hash_remainder`
- Ort: `crates/contextra-store/src/memtable/shard.rs:53`
```rust
pub fn hash_remainder(remainder: &[u8]) -> usize
```
Dokumentation: Helper to compute a deterministic hash for reserved system key remainder.

#### `fn matching_reserved_prefix`
- Ort: `crates/contextra-store/src/memtable/shard.rs:39`
```rust
pub fn matching_reserved_prefix(key: &[u8]) -> Option<&'static [u8]>
```
Dokumentation: Helper to detect if a key matches a reserved system prefix.

#### `fn shard_for`
- Ort: `crates/contextra-store/src/memtable/shard.rs:84`
```rust
pub fn shard_for(key: &[u8]) -> usize
```
Dokumentation: Maps a key to its owning shard index `0..SHARD_COUNT`.

#### `fn shard_range_for_bounds`
- Ort: `crates/contextra-store/src/memtable/shard.rs:115`
```rust
pub fn shard_range_for_bounds( start: Bound<&[u8]>, end: Bound<&[u8]>, ) -> std::ops::RangeInclusive<usize>
```
Dokumentation: Computes the contiguous shard range `[start, end]` that covers the given bounds.

#### `fn shard_range_for_prefix`
- Ort: `crates/contextra-store/src/memtable/shard.rs:98`
```rust
pub fn shard_range_for_prefix(prefix: &[u8]) -> std::ops::RangeInclusive<usize>
```
Dokumentation: Computes the contiguous shard range `[start, end]` that may contain keys with the given prefix.

#### `type MemTableEntry`
- Ort: `crates/contextra-store/src/memtable/shard.rs:13`
```rust
pub type MemTableEntry = (SequenceNumber, Bytes, TransactionId)
```

#### `type MemTableMap`
- Ort: `crates/contextra-store/src/memtable/shard.rs:14`
```rust
pub type MemTableMap = BTreeMap<Bytes, Vec<MemTableEntry>>
```

#### `type SequenceNumber`
- Ort: `crates/contextra-store/src/memtable/shard.rs:11`
```rust
pub type SequenceNumber = u64
```

#### `type TransactionId`
- Ort: `crates/contextra-store/src/memtable/shard.rs:12`
```rust
pub type TransactionId = u64
```

#### `const RESERVED_PREFIXES`
- Ort: `crates/contextra-store/src/memtable/shard.rs:22`
```rust
pub const RESERVED_PREFIXES: &[&[u8]] = &[ b"", b"", b"", b"", b"", b"", b"", b"", b"", b"", b"", b"", ]
```
Dokumentation: List of known reserved system key prefixes.

#### `const SHARD_COUNT`
- Ort: `crates/contextra-store/src/memtable/shard.rs:19`
```rust
pub const SHARD_COUNT: usize = 16
```
Dokumentation: Number of independent shards. 16 provides sufficient parallelism for the 8-concurrent-embed pipeline while keeping memory overhead negligible. Must be > 0 (compile-time const, enforced by type system).

### Modul `memtable::tombstone`

#### `fn is_tombstone`
- Ort: `crates/contextra-store/src/memtable/tombstone.rs:14`
```rust
pub const fn is_tombstone(seq: u64) -> bool
```
Dokumentation: Helper function to check if a sequence number has TOMBSTONE_BIT set.

#### `fn is_valid_range_bounds`
- Ort: `crates/contextra-store/src/memtable/tombstone.rs:20`
```rust
pub fn is_valid_range_bounds(start: Bound<&[u8]>, end: Bound<&[u8]>) -> bool
```
Dokumentation: Validates whether start and end key range bounds define a non-empty, valid interval (`start <= end`).

### Modul `sstable`

#### `mod block_cache`
- Ort: `crates/contextra-store/src/sstable.rs:3`
```rust
pub mod block_cache
```

#### `mod format`
- Ort: `crates/contextra-store/src/sstable.rs:7`
```rust
pub mod format
```

### Modul `sstable::block_cache`

#### `struct BlockCache`
- Ort: `crates/contextra-store/src/sstable/block_cache/mod.rs:32`
```rust
pub struct BlockCache
```
- Impl-Block `crates/contextra-store/src/sstable/block_cache/mod.rs:39`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new(capacity_per_shard: usize) -> Self` | `crates/contextra-store/src/sstable/block_cache/mod.rs:40` |  |
| `pub fn new_with_shards(capacity_per_shard: usize, num_shards: usize) -> Self` | `crates/contextra-store/src/sstable/block_cache/mod.rs:44` |  |
| `pub fn shard_idx(&self, file_id: u64, offset: u64) -> usize` | `crates/contextra-store/src/sstable/block_cache/mod.rs:74` |  |
| `pub fn get(&self, file_id: u64, offset: u64) -> Option<Bytes>` | `crates/contextra-store/src/sstable/block_cache/mod.rs:86` |  |
| `pub fn insert(&self, file_id: u64, offset: u64, data: Bytes)` | `crates/contextra-store/src/sstable/block_cache/mod.rs:98` |  |
| `pub fn len(&self) -> usize` | `crates/contextra-store/src/sstable/block_cache/mod.rs:103` |  |
| `pub fn is_empty(&self) -> bool` | `crates/contextra-store/src/sstable/block_cache/mod.rs:108` |  |
| `pub fn contains(&self, file_id: u64, offset: u64) -> bool` | `crates/contextra-store/src/sstable/block_cache/mod.rs:113` |  |
| `pub fn hits(&self) -> u64` | `crates/contextra-store/src/sstable/block_cache/mod.rs:119` | Returns the total number of cache hits recorded. |
| `pub fn misses(&self) -> u64` | `crates/contextra-store/src/sstable/block_cache/mod.rs:125` | Returns the total number of cache misses recorded. |
| `pub fn hit_ratio(&self) -> f64` | `crates/contextra-store/src/sstable/block_cache/mod.rs:130` | Returns the cache hit ratio (0.0 .. 1.0). |
| `pub fn reset_metrics(&self)` | `crates/contextra-store/src/sstable/block_cache/mod.rs:141` | Resets hit and miss metrics counters. |
| `pub fn current_bytes(&self) -> usize` | `crates/contextra-store/src/sstable/block_cache/mod.rs:147` | Returns current memory usage in bytes across all shards. |
| `pub fn total_capacity_bytes(&self) -> usize` | `crates/contextra-store/src/sstable/block_cache/mod.rs:152` | Returns total configured byte capacity across all shards. |

#### `fn create_block_cache`
- Ort: `crates/contextra-store/src/sstable/block_cache/mod.rs:164`
```rust
pub fn create_block_cache(capacity_mb: usize) -> Arc<BlockCache>
```
Dokumentation: Creates a new block cache instance with default shard count. Capacity is in MB (assuming 4KB blocks).

#### `fn create_block_cache_with_shards`
- Ort: `crates/contextra-store/src/sstable/block_cache/mod.rs:169`
```rust
pub fn create_block_cache_with_shards(capacity_mb: usize, num_shards: usize) -> Arc<BlockCache>
```
Dokumentation: Creates a new block cache instance with configurable shard count. Capacity is in MB.

#### `type BlockCacheShard`
- Ort: `crates/contextra-store/src/sstable/block_cache/mod.rs:27`
```rust
pub type BlockCacheShard = LruBlockCacheBackend
```

#### `type BlockCacheShard`
- Ort: `crates/contextra-store/src/sstable/block_cache/mod.rs:30`
```rust
pub type BlockCacheShard = QuickCacheBlockCacheBackend
```
- Gate: `#[cfg(feature = "block-cache-v2")]`

#### `const BLOCK_CACHE_SHARDS`
- Ort: `crates/contextra-store/src/sstable/block_cache/mod.rs:24`
```rust
pub const BLOCK_CACHE_SHARDS: usize = 64
```
Dokumentation: Sharded block cache: 64 independent shards to reduce contention. Shard selection uses ahash tuple hashing on (file_id, offset).

### Modul `sstable::block_cache::lru`

#### `struct LruBlockCacheBackend`
- Ort: `crates/contextra-store/src/sstable/block_cache/lru.rs:17`
```rust
pub struct LruBlockCacheBackend
```
- Impl-Block (Trait `BlockCacheBackend`) `crates/contextra-store/src/sstable/block_cache/lru.rs:21`

### Modul `sstable::block_cache::quick`

#### `struct QuickCacheBlockCacheBackend`
- Ort: `crates/contextra-store/src/sstable/block_cache/quick.rs:22`
```rust
pub struct QuickCacheBlockCacheBackend
```
Dokumentation: Lock-optimized block cache backend using `quick_cache::sync::Cache` (S3-FIFO / Clock-based eviction) with byte-based capacity.
- Gate: `#[cfg(feature = "block-cache-v2")]`
- Impl-Block (Trait `BlockCacheBackend`) `crates/contextra-store/src/sstable/block_cache/quick.rs:27`

### Modul `sstable::block_cache::sieve`

#### `struct SieveCacheBackend`
- Ort: `crates/contextra-store/src/sstable/block_cache/sieve.rs:11`
```rust
pub struct SieveCacheBackend
```
- Impl-Block (Trait `BlockCacheBackend`) `crates/contextra-store/src/sstable/block_cache/sieve.rs:29`

### Modul `sstable::block_cache::traits`

#### `trait BlockCacheBackend`
- Ort: `crates/contextra-store/src/sstable/block_cache/traits.rs:8`
```rust
pub trait BlockCacheBackend: Send + Sync
```

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `fn new(capacity: usize) -> Self where Self: Sized` | `crates/contextra-store/src/sstable/block_cache/traits.rs:10` | Creates a new cache backend instance with the specified capacity. |
| `fn get(&self, key: &(u64, u64)) -> Option<Bytes>` | `crates/contextra-store/src/sstable/block_cache/traits.rs:15` | Retrieves a cached block by `(file_id, offset)` key. |
| `fn insert(&self, key: (u64, u64), value: Bytes)` | `crates/contextra-store/src/sstable/block_cache/traits.rs:18` | Inserts a block into the cache. |
| `fn len(&self) -> usize` | `crates/contextra-store/src/sstable/block_cache/traits.rs:21` | Returns the number of cached blocks in this backend instance. |
| `fn is_empty(&self) -> bool` | `crates/contextra-store/src/sstable/block_cache/traits.rs:24` | Returns `true` if the backend contains no cached blocks. |
| `fn contains(&self, key: &(u64, u64)) -> bool` | `crates/contextra-store/src/sstable/block_cache/traits.rs:29` | Checks if a key is present in the cache. |
| `fn current_bytes(&self) -> usize` | `crates/contextra-store/src/sstable/block_cache/traits.rs:34` | Returns the current memory usage in bytes. |
| `fn capacity_bytes(&self) -> usize` | `crates/contextra-store/src/sstable/block_cache/traits.rs:37` | Returns the capacity limit in bytes. |

### Modul `sstable::block_search`

#### `fn binary_search_entry_in_block`
- Ort: `crates/contextra-store/src/sstable/block_search.rs:194`
```rust
pub fn binary_search_entry_in_block( block_data: &[u8], offsets_start: usize, num_offsets: usize, key: &[u8], is_v3: bool, ) -> Result<Option<(usize, usize)>>
```

#### `fn binary_search_first_index_in_block`
- Ort: `crates/contextra-store/src/sstable/block_search.rs:118`
```rust
pub fn binary_search_first_index_in_block( block_data: &[u8], offsets_start: usize, num_offsets: usize, key: &[u8], is_v3: bool, ) -> StoreResult<Option<usize>>
```

#### `fn binary_search_index_in_block`
- Ort: `crates/contextra-store/src/sstable/block_search.rs:150`
```rust
pub fn binary_search_index_in_block( block_data: &[u8], offsets_start: usize, num_offsets: usize, key: &[u8], is_v3: bool, ) -> Result<std::result::Result<usize, usize>>
```

#### `fn block_binary_search`
- Ort: `crates/contextra-store/src/sstable/block_search.rs:210`
```rust
pub fn block_binary_search( block_data: &[u8], offsets_start: usize, num_offsets: usize, key: &[u8], is_v3: bool, ) -> StoreResult<Option<usize>>
```

#### `fn get_entry_at_index`
- Ort: `crates/contextra-store/src/sstable/block_search.rs:10`
```rust
pub fn get_entry_at_index( block_data: &[u8], offsets_start: usize, idx: usize, is_v3: bool, ) -> StoreResult<(usize, usize)>
```
Dokumentation: Entry offset accessor supporting both 16-bit (legacy v0..v2) and 32-bit (v3+) block formats.

#### `fn get_entry_key_at_index`
- Ort: `crates/contextra-store/src/sstable/block_search.rs:95`
```rust
pub fn get_entry_key_at_index( block_data: &[u8], offsets_start: usize, idx: usize, is_v3: bool, ) -> StoreResult<&[u8]>
```

### Modul `sstable::bloom`

#### `struct BloomFilter`
- Ort: `crates/contextra-store/src/sstable/bloom.rs:14`
```rust
pub struct BloomFilter
```
Dokumentation: SPECCED: Speichereffizienter Bloom-Filter für SSTable-Pre-Checks. Ziel: False-Positive-Rate p ≤ 0.01. Formel: m = -n * ln(p) / (ln(2)^2) ≈ 9.6 * n Hashes: k = (m/n) * ln(2) ≈ 7
- Impl-Block `crates/contextra-store/src/sstable/bloom.rs:20`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `bits` | `Vec<u64>` | `crates/contextra-store/src/sstable/bloom.rs:15` |
| `num_hashes` | `usize` | `crates/contextra-store/src/sstable/bloom.rs:16` |
| `num_bits` | `usize` | `crates/contextra-store/src/sstable/bloom.rs:17` |

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new(expected_elements: usize, fpr: f64) -> Self` | `crates/contextra-store/src/sstable/bloom.rs:29` | Creates a new Bloom filter for the expected number of elements and target false positive rate (fpr). ## FPR Trade-off - `fpr = 0.01` (1%) uses ~9.6 bits/element |
| `pub fn insert(&mut self, key: &[u8])` | `crates/contextra-store/src/sstable/bloom.rs:53` | Inserts a key into the filter. |
| `pub fn may_contain(&self, key: &[u8]) -> bool` | `crates/contextra-store/src/sstable/bloom.rs:69` | Checks if a key might be in the filter. |
| `pub fn hash_pair(key: &[u8]) -> (u64, u64)` | `crates/contextra-store/src/sstable/bloom.rs:90` | Erzeugt zwei unabhängige 64-bit Hashes aus dem Blake3-Digest. h1 = erste 8 Bytes, h2 = Bytes 8-15 (oder fallback zu h1 ^ const) |
| `pub fn to_bytes(&self) -> Vec<u8>` | `crates/contextra-store/src/sstable/bloom.rs:103` |  |
| `pub fn from_bytes(data: &[u8]) -> Result<Self>` | `crates/contextra-store/src/sstable/bloom.rs:116` |  |

### Modul `sstable::builder::block`

#### `struct BlockBuilder`
- Ort: `crates/contextra-store/src/sstable/builder/block.rs:11`
```rust
pub struct BlockBuilder
```
Dokumentation: A builder for SSTable data blocks (Format v4 default, supports v3).
- Impl-Block `crates/contextra-store/src/sstable/builder/block.rs:20`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `block_size` | `usize` | `crates/contextra-store/src/sstable/builder/block.rs:14` |

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new(block_size: usize) -> Self` | `crates/contextra-store/src/sstable/builder/block.rs:21` |  |
| `pub fn new_with_version(block_size: usize, format_version: u16) -> Self` | `crates/contextra-store/src/sstable/builder/block.rs:25` |  |
| `pub fn add(&mut self, key: &[u8], value: &[u8], seq_no: u64, tx_id: u64) -> bool` | `crates/contextra-store/src/sstable/builder/block.rs:53` |  |
| `pub fn current_size(&self) -> usize` | `crates/contextra-store/src/sstable/builder/block.rs:107` |  |
| `pub fn is_empty(&self) -> bool` | `crates/contextra-store/src/sstable/builder/block.rs:131` |  |
| `pub fn build(mut self) -> Bytes` | `crates/contextra-store/src/sstable/builder/block.rs:136` | Finalizes the block and returns the bytes. |

### Modul `sstable::builder::meta`

#### `struct SstableMetadata`
- Ort: `crates/contextra-store/src/sstable/builder/meta.rs:9`
```rust
pub struct SstableMetadata
```

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `first_key` | `Bytes` | `crates/contextra-store/src/sstable/builder/meta.rs:10` |
| `last_key` | `Bytes` | `crates/contextra-store/src/sstable/builder/meta.rs:11` |
| `file_size` | `u64` | `crates/contextra-store/src/sstable/builder/meta.rs:12` |
| `min_tx_id` | `u64` | `crates/contextra-store/src/sstable/builder/meta.rs:13` |
| `max_tx_id` | `u64` | `crates/contextra-store/src/sstable/builder/meta.rs:14` |
| `min_seq` | `u64` | `crates/contextra-store/src/sstable/builder/meta.rs:15` |
| `max_seq` | `u64` | `crates/contextra-store/src/sstable/builder/meta.rs:16` |

### Modul `sstable::builder::writer`

#### `struct SstableBuilder`
- Ort: `crates/contextra-store/src/sstable/builder/writer.rs:19`
```rust
pub struct SstableBuilder
```
- Impl-Block `crates/contextra-store/src/sstable/builder/writer.rs:44`
- Impl-Block (Trait `Drop`) `crates/contextra-store/src/sstable/builder/writer.rs:430`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub async fn create(path: impl AsRef<Path>) -> Result<Self>` | `crates/contextra-store/src/sstable/builder/writer.rs:46` | Creates a new SstableBuilder that writes to the given file path. |
| `pub async fn create_with_key_manager( path: impl AsRef<Path>, key_manager: Option<Arc<KeyManager>>, ) -> Result<Self>` | `crates/contextra-store/src/sstable/builder/writer.rs:50` |  |
| `pub async fn create_with_vfs( path: impl AsRef<Path>, key_manager: Option<Arc<KeyManager>>, vfs: Arc<dyn Vfs>, ) -> Result<Self>` | `crates/contextra-store/src/sstable/builder/writer.rs:57` |  |
| `pub fn with_block_size(mut self, size: usize) -> Self` | `crates/contextra-store/src/sstable/builder/writer.rs:115` | Sets the target block size for data blocks. |
| `pub fn set_block_size(&mut self, size: usize)` | `crates/contextra-store/src/sstable/builder/writer.rs:121` | Configures the target block size for data blocks. |
| `pub fn with_bloom_fpr(mut self, expected_keys: usize, fpr: f64) -> Self` | `crates/contextra-store/src/sstable/builder/writer.rs:130` | Sets the whole-SSTable bloom filter parameters. |
| `pub fn set_bloom_filter(&mut self, expected_keys: usize, fpr: f64)` | `crates/contextra-store/src/sstable/builder/writer.rs:136` | Configures the whole-SSTable bloom filter parameters. |
| `pub fn set_format_version(&mut self, version: u16)` | `crates/contextra-store/src/sstable/builder/writer.rs:171` | Explicitly configures the SSTable format version to write (default: 4). |
| `pub async fn add(&mut self, key: &[u8], value: &[u8], seq_no: u64, tx_id: u64) -> Result<()>` | `crates/contextra-store/src/sstable/builder/writer.rs:177` | Adds a key-value pair to the SSTable being built. |
| `pub async fn finish(mut self) -> Result<SstableMetadata>` | `crates/contextra-store/src/sstable/builder/writer.rs:288` | Finalizes the SSTable and returns metadata. |

### Modul `sstable::format`

#### `struct SstableFooter`
- Ort: `crates/contextra-store/src/sstable/format.rs:37`
```rust
pub struct SstableFooter
```
Dokumentation: Strukturierte Repräsentation des SSTable-Footers / Trailers.
- Impl-Block `crates/contextra-store/src/sstable/format.rs:58`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `format_version` | `u16` | `crates/contextra-store/src/sstable/format.rs:39` |
| `is_mfsx` | `bool` | `crates/contextra-store/src/sstable/format.rs:41` |
| `bloom_offset` | `u64` | `crates/contextra-store/src/sstable/format.rs:43` |
| `index_offset` | `u64` | `crates/contextra-store/src/sstable/format.rs:45` |
| `min_tx_id` | `u64` | `crates/contextra-store/src/sstable/format.rs:47` |
| `max_tx_id` | `u64` | `crates/contextra-store/src/sstable/format.rs:49` |
| `min_seq` | `u64` | `crates/contextra-store/src/sstable/format.rs:51` |
| `max_seq` | `u64` | `crates/contextra-store/src/sstable/format.rs:53` |
| `trailer_len` | `usize` | `crates/contextra-store/src/sstable/format.rs:55` |

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn parse(trailer_data: &[u8], path_display: &str) -> StoreResult<Self>` | `crates/contextra-store/src/sstable/format.rs:60` | Parst und validiert den SSTable-Trailer aus den letzten Bytes der Datei. |

#### `const BLOCK_SIZE`
- Ort: `crates/contextra-store/src/sstable/format.rs:21`
```rust
pub const BLOCK_SIZE: usize = 4096
```
Dokumentation: Standard-Blockgröße für SSTable Data Blocks (4 KiB).

#### `const CURRENT_SSTABLE_FORMAT_VERSION`
- Ort: `crates/contextra-store/src/sstable/format.rs:24`
```rust
pub const CURRENT_SSTABLE_FORMAT_VERSION: u16 = 4
```
Dokumentation: Aktuelle unterstützte SSTable-Formatversion (V4).

#### `const LEGACY_TRAILER_LEN`
- Ort: `crates/contextra-store/src/sstable/format.rs:33`
```rust
pub const LEGACY_TRAILER_LEN: usize = 54
```
Dokumentation: Trailer-Länge für Legacy MFST (54 Bytes).

#### `const MFSX_TRAILER_LEN_V0`
- Ort: `crates/contextra-store/src/sstable/format.rs:30`
```rust
pub const MFSX_TRAILER_LEN_V0: usize = 52
```
Dokumentation: Trailer-Länge für MFSX Format Version 0 (52 Bytes).

#### `const MFSX_TRAILER_LEN_V1`
- Ort: `crates/contextra-store/src/sstable/format.rs:27`
```rust
pub const MFSX_TRAILER_LEN_V1: usize = 54
```
Dokumentation: Trailer-Länge für MFSX Format Version >= 1 (54 Bytes).

#### `const SSTABLE_MAGIC_LEGACY`
- Ort: `crates/contextra-store/src/sstable/format.rs:18`
```rust
pub const SSTABLE_MAGIC_LEGACY: u32 = 0x4D46_5354
```
Dokumentation: Magic-Bytes für Legacy SSTable: `MFST` in Hex (0x4D46_5354).

#### `const SSTABLE_MAGIC_MFSX`
- Ort: `crates/contextra-store/src/sstable/format.rs:15`
```rust
pub const SSTABLE_MAGIC_MFSX: u32 = 0x5853_464D
```
Dokumentation: Magic-Bytes für V4 SSTable: `MFSX` in Hex (0x5853_464D).

### Modul `sstable::reader`

#### `struct SstableReader`
- Ort: `crates/contextra-store/src/sstable/reader/mod.rs:225`
```rust
pub struct SstableReader
```
Dokumentation: Liest SSTable-Dateien mit Block-Cache und Bloom-Filter-Unterstuetzung.
- Impl-Block `crates/contextra-store/src/sstable/reader/block.rs:13`
- Impl-Block `crates/contextra-store/src/sstable/reader/iter.rs:13`
- Impl-Block `crates/contextra-store/src/sstable/reader/lookup.rs:15`
- Impl-Block `crates/contextra-store/src/sstable/reader/mod.rs:246`
- Impl-Block `crates/contextra-store/src/sstable/reader/open.rs:16`
- Impl-Block `crates/contextra-store/src/sstable/reader_ext.rs:10`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `file` | `Arc<dyn VfsFile>` | `crates/contextra-store/src/sstable/reader/mod.rs:226` |
| `metadata` | `SstableMetadata` | `crates/contextra-store/src/sstable/reader/mod.rs:228` |
| `index_offset` | `u64` | `crates/contextra-store/src/sstable/reader/mod.rs:230` |
| `file_path` | `PathBuf` | `crates/contextra-store/src/sstable/reader/mod.rs:232` |
| `file_id` | `u64` | `crates/contextra-store/src/sstable/reader/mod.rs:234` |
| `block_cache` | `Arc<BlockCache>` | `crates/contextra-store/src/sstable/reader/mod.rs:236` |
| `key_manager` | `Option<Arc<KeyManager>>` | `crates/contextra-store/src/sstable/reader/mod.rs:237` |
| `bloom_filter` | `Option<BloomFilter>` | `crates/contextra-store/src/sstable/reader/mod.rs:239` |
| `has_crc` | `bool` | `crates/contextra-store/src/sstable/reader/mod.rs:241` |
| `format_version` | `u16` | `crates/contextra-store/src/sstable/reader/mod.rs:243` |

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn metadata(&self) -> &SstableMetadata` | `crates/contextra-store/src/sstable/reader/iter.rs:14` |  |
| `pub async fn stream(self: &Arc<Self>) -> Result<SstableStream>` | `crates/contextra-store/src/sstable/reader/iter.rs:19` |  |
| `pub async fn iter(&self) -> Result<Vec<(Bytes, Bytes, u64)>>` | `crates/contextra-store/src/sstable/reader/iter.rs:31` | Iterates over all entries in sorted key order (allocates memory for all entries). |
| `pub async fn get_at( &self, key: &[u8], max_seq: u64, max_tx: u64, ) -> Result<Option<(Bytes, u64, u64)>>` | `crates/contextra-store/src/sstable/reader/lookup.rs:16` |  |
| `pub async fn get(&self, key: &[u8]) -> Result<Option<(Bytes, u64, u64)>>` | `crates/contextra-store/src/sstable/reader/lookup.rs:160` | Retrieves a value from the SSTable by key. |
| `pub async fn lookup_metrics(&self, key: &[u8]) -> (bool, bool, bool, bool)` | `crates/contextra-store/src/sstable/reader/lookup.rs:165` | Metrics result for point lookup instrumentation. |
| `pub fn first_key(&self) -> &Bytes` | `crates/contextra-store/src/sstable/reader/mod.rs:247` |  |
| `pub fn last_key(&self) -> &Bytes` | `crates/contextra-store/src/sstable/reader/mod.rs:251` |  |
| `pub fn min_tx_id(&self) -> u64` | `crates/contextra-store/src/sstable/reader/mod.rs:255` |  |
| `pub fn max_tx_id(&self) -> u64` | `crates/contextra-store/src/sstable/reader/mod.rs:259` |  |
| `pub fn file_path(&self) -> &Path` | `crates/contextra-store/src/sstable/reader/mod.rs:263` |  |
| `pub async fn open(path: impl AsRef<Path>, block_cache: Arc<BlockCache>) -> Result<Self>` | `crates/contextra-store/src/sstable/reader/open.rs:17` |  |
| `pub async fn open_with_key_manager( path: impl AsRef<Path>, block_cache: Arc<BlockCache>, key_manager: Option<Arc<KeyManager>>, ) -> Result<Self>` | `crates/contextra-store/src/sstable/reader/open.rs:21` |  |
| `pub async fn open_with_vfs( path: impl AsRef<Path>, block_cache: Arc<BlockCache>, key_manager: Option<Arc<KeyManager>>, vfs: Arc<dyn Vfs>, ) -> Result<Self>` | `crates/contextra-store/src/sstable/reader/open.rs:29` |  |
| `pub async fn scan_prefix(&self, prefix: &[u8]) -> Result<Vec<(Bytes, Bytes, u64, u64)>>` | `crates/contextra-store/src/sstable/reader_ext.rs:12` | Scans the SSTable for keys starting with the given prefix. |
| `pub async fn scan_range( &self, start: std::ops::Bound<&[u8]>, end: std::ops::Bound<&[u8]>, ) -> Result<Vec<(Bytes, Bytes, u64, u64)>>` | `crates/contextra-store/src/sstable/reader_ext.rs:158` | Scans all entries within a key range. |

#### `fn parse_block_trailer`
- Ort: `crates/contextra-store/src/sstable/reader/mod.rs:24`
```rust
pub fn parse_block_trailer( block_data: &[u8], format_version: u16, ) -> Result<(usize, usize, &[u8])>
```
Dokumentation: Helper function to parse block trailer across format versions v0..v4. Returns `(num_offsets, offsets_start, bloom_bytes)`.

### Modul `sstable::stream`

#### `struct SstableStream`
- Ort: `crates/contextra-store/src/sstable/stream.rs:11`
```rust
pub struct SstableStream
```
- Impl-Block `crates/contextra-store/src/sstable/stream.rs:20`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `reader` | `Arc<SstableReader>` | `crates/contextra-store/src/sstable/stream.rs:12` |
| `block_idx` | `usize` | `crates/contextra-store/src/sstable/stream.rs:13` |
| `entry_idx` | `usize` | `crates/contextra-store/src/sstable/stream.rs:14` |
| `current_block` | `Option<Bytes>` | `crates/contextra-store/src/sstable/stream.rs:15` |
| `num_offsets` | `usize` | `crates/contextra-store/src/sstable/stream.rs:16` |
| `offsets_start` | `usize` | `crates/contextra-store/src/sstable/stream.rs:17` |

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub async fn next(&mut self) -> Result<Option<(Bytes, Bytes, u64, u64)>>` | `crates/contextra-store/src/sstable/stream.rs:21` |  |
| `pub async fn next_entry(&mut self) -> Result<Option<(Bytes, Bytes, u64, u64)>>` | `crates/contextra-store/src/sstable/stream.rs:135` | Optimized for compaction: avoids copying if possible. |

### Modul `stats`

#### `struct StorageStats`
- Ort: `crates/contextra-store/src/stats.rs:5`
```rust
pub struct StorageStats
```
Dokumentation: Detailed operational and storage statistics.
- Impl-Block (Trait `From<crate::stats::StorageStats>`) `crates/contextra-store/src/compat/engine/lsm.rs:204`
- Impl-Block (Trait `From<StorageStats>`) `crates/contextra-store/src/compat/engine/lsm.rs:214`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `num_segments` | `usize` | `crates/contextra-store/src/stats.rs:7` |
| `total_size_bytes` | `u64` | `crates/contextra-store/src/stats.rs:9` |
| `memtable_size_bytes` | `u64` | `crates/contextra-store/src/stats.rs:11` |

### Modul `system_pressure`

#### `mod calculation`
- Ort: `crates/contextra-store/src/system_pressure.rs:7`
```rust
pub mod calculation
```

#### `mod monitor`
- Ort: `crates/contextra-store/src/system_pressure.rs:8`
```rust
pub mod monitor
```

#### `mod signal`
- Ort: `crates/contextra-store/src/system_pressure.rs:9`
```rust
pub mod signal
```

### Modul `system_pressure::calculation`

#### `struct SystemPressure`
- Ort: `crates/contextra-store/src/system_pressure/calculation.rs:12`
```rust
pub struct SystemPressure
```
Dokumentation: Aggregierter Systemdruck-Zustand.

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `wal_queue_depth` | `usize` | `crates/contextra-store/src/system_pressure/calculation.rs:14` |
| `scheduler_queue_depth` | `f32` | `crates/contextra-store/src/system_pressure/calculation.rs:16` |
| `blocking_util` | `f32` | `crates/contextra-store/src/system_pressure/calculation.rs:18` |
| `embedding_queue_depth` | `usize` | `crates/contextra-store/src/system_pressure/calculation.rs:20` |
| `pressure_level` | `PressureLevel` | `crates/contextra-store/src/system_pressure/calculation.rs:22` |

#### `fn compute_pressure`
- Ort: `crates/contextra-store/src/system_pressure/calculation.rs:26`
```rust
pub fn compute_pressure( wal_depth: usize, scheduler_queue_depth: f32, blocking_util: f32, embedding_queue: usize, max_permits: usize, ) -> SystemPressure
```
Dokumentation: Berechnet den Systemdruck anhand der Einzelsignale.

### Modul `system_pressure::monitor`

#### `struct SystemPressureMonitor`
- Ort: `crates/contextra-store/src/system_pressure/monitor.rs:12`
```rust
pub struct SystemPressureMonitor
```
Dokumentation: Periodischer Monitor zur Erfassung von Systemdruck-Signalen.
- Impl-Block `crates/contextra-store/src/system_pressure/monitor.rs:20`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `pressure_rx` | `tokio::sync::watch::Receiver<SystemPressure>` | `crates/contextra-store/src/system_pressure/monitor.rs:15` |
| `clock` | `Arc<dyn contextra_env::Clock>` | `crates/contextra-store/src/system_pressure/monitor.rs:17` |

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new(sampling_interval: Duration) -> Self` | `crates/contextra-store/src/system_pressure/monitor.rs:22` | Erstellt einen neuen Monitor mit SystemClock. |
| `pub fn new_with_clock( sampling_interval: Duration, clock: Arc<dyn contextra_env::Clock>, ) -> Self` | `crates/contextra-store/src/system_pressure/monitor.rs:30` | Erstellt einen neuen Monitor mit einer injizierten Clock. |
| `pub fn with_clock(mut self, clock: Arc<dyn contextra_env::Clock>) -> Self` | `crates/contextra-store/src/system_pressure/monitor.rs:51` | Ersetzt die Clock durch eine übergebene Instanz. |
| `pub fn compute_pressure( &self, wal_depth: usize, scheduler_queue_depth: f32, blocking_util: f32, embedding_queue: usize, max_permits: usize, ) -> SystemPressure` | `crates/contextra-store/src/system_pressure/monitor.rs:57` | Berechnet den aktuellen Druckzustand. |
| `pub async fn run( self, cancellation: CancellationToken, wal_queue_depth_fn: impl Fn() -> usize + Send + 'static, embedding_permits_fn: impl Fn() -> usize + Send + 'static, max_embedding_permits: usize, )` | `crates/contextra-store/src/system_pressure/monitor.rs:75` | Startet die periodische Hintergrundüberwachung. |
| `pub async fn run_with_metrics_sink( self, cancellation: CancellationToken, wal_queue_depth_fn: impl Fn() -> usize + Send + 'static, embedding_permits_fn: impl Fn() -> usize + Send + 'static, max_embedding_permits: usize, metrics_sink: Option<Arc<parking_lot::RwLock<Arc<dyn contextra_env::MetricsSink>>>>, )` | `crates/contextra-store/src/system_pressure/monitor.rs:93` | Startet die periodische Hintergrundüberwachung mit optionalem MetricsSink. |

### Modul `system_pressure::signal`

#### `enum PressureLevel`
- Ort: `crates/contextra-store/src/system_pressure/signal.rs:29`
```rust
pub enum PressureLevel
```
Dokumentation: Diskreter Systemdruck-Pegel für Backpressure-Steuerung.

Varianten:

- `Normal` — `crates/contextra-store/src/system_pressure/signal.rs:31`
- `Elevated` — `crates/contextra-store/src/system_pressure/signal.rs:33`
- `Critical` — `crates/contextra-store/src/system_pressure/signal.rs:35`

#### `trait SystemPressureSignal`
- Ort: `crates/contextra-store/src/system_pressure/signal.rs:19`
```rust
pub trait SystemPressureSignal: Send + Sync
```
Dokumentation: Injektions-Trait für externe Speicherdruck-Signale (ohne globale Zustände).

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `fn current_pressure(&self) -> PressureLevel` | `crates/contextra-store/src/system_pressure/signal.rs:21` | Liefert den aktuellen Druckpegel. |
| `fn advise_reduction_ratio(&self) -> f32` | `crates/contextra-store/src/system_pressure/signal.rs:24` | Gibt eine empfohlene Durchsatz-Reduktionsrate (0.0 bis 1.0) zurück. |

#### `const BLOCKING_UTIL_CRITICAL`
- Ort: `crates/contextra-store/src/system_pressure/signal.rs:14`
```rust
pub const BLOCKING_UTIL_CRITICAL: f32 = SCHEDULER_QUEUE_CRITICAL
```
Dokumentation: Kritischer Schwellenwert für Blocking-Pool Auslastung.

#### `const BLOCKING_UTIL_ELEVATED`
- Ort: `crates/contextra-store/src/system_pressure/signal.rs:16`
```rust
pub const BLOCKING_UTIL_ELEVATED: f32 = SCHEDULER_QUEUE_ELEVATED
```
Dokumentation: Erhöhter Schwellenwert für Blocking-Pool Auslastung.

#### `const SCHEDULER_QUEUE_CRITICAL`
- Ort: `crates/contextra-store/src/system_pressure/signal.rs:10`
```rust
pub const SCHEDULER_QUEUE_CRITICAL: f32 = 0.85
```
Dokumentation: Kritischer Schwellenwert für die Tokio-Scheduler-Warteschlange (Verhältnis).

#### `const SCHEDULER_QUEUE_ELEVATED`
- Ort: `crates/contextra-store/src/system_pressure/signal.rs:12`
```rust
pub const SCHEDULER_QUEUE_ELEVATED: f32 = 0.60
```
Dokumentation: Erhöhter Schwellenwert für die Tokio-Scheduler-Warteschlange (Verhältnis).

#### `const WAL_QUEUE_CRITICAL_THRESHOLD`
- Ort: `crates/contextra-store/src/system_pressure/signal.rs:6`
```rust
pub const WAL_QUEUE_CRITICAL_THRESHOLD: usize = 500
```
Dokumentation: Kritischer Schwellenwert für die WAL-Warteschlangentiefe.

#### `const WAL_QUEUE_ELEVATED_THRESHOLD`
- Ort: `crates/contextra-store/src/system_pressure/signal.rs:8`
```rust
pub const WAL_QUEUE_ELEVATED_THRESHOLD: usize = 100
```
Dokumentation: Erhöhter Schwellenwert für die WAL-Warteschlangentiefe.

### Modul `tenant_codec`

#### `struct TenantKeyCodec`
- Ort: `crates/contextra-store/src/tenant_codec.rs:29`
```rust
pub struct TenantKeyCodec
```
Dokumentation: Encodes LSM storage keys with tenant isolation prefixes.
- Impl-Block `crates/contextra-store/src/tenant_codec.rs:35`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new(tenant_id: TenantId) -> Self` | `crates/contextra-store/src/tenant_codec.rs:37` | Creates a new `TenantKeyCodec` for the given `TenantId`. |
| `pub fn encode_chunk_key(&self, collection: &CollectionId, doc_id: DocId) -> Vec<u8>` | `crates/contextra-store/src/tenant_codec.rs:47` | Encodes a document chunk key: `t:{tenant}:{collection}:chunk:{doc}`. |
| `pub fn encode_graph_key(&self, collection: &CollectionId, entity_id: u64) -> Vec<u8>` | `crates/contextra-store/src/tenant_codec.rs:59` | Encodes a graph entity key: `t:{tenant}:{collection}:graph:{entity}`. |
| `pub fn scan_prefix(&self) -> &[u8]` | `crates/contextra-store/src/tenant_codec.rs:72` | Globales Scan-Präfix für diesen Tenant (alle Collections): `t:{tenant}:` Cached — keine Allokation im Hot-Path. |
| `pub fn collection_prefix(&self, collection: &CollectionId) -> Vec<u8>` | `crates/contextra-store/src/tenant_codec.rs:77` | Collection-spezifisches Scan-Präfix: `t:{tenant}:{col}:` |
| `pub fn decode_tenant_id(key: &[u8]) -> Option<TenantId>` | `crates/contextra-store/src/tenant_codec.rs:83` | Dekodiert TenantId aus encoded Key. None bei ungültigem Format. Kein Unwrap — robuste Nutzung im Recovery-Pfad. |
| `pub fn decode_chunk_key(key: &[u8]) -> Option<(TenantId, CollectionId, DocId)>` | `crates/contextra-store/src/tenant_codec.rs:94` | Dekodiert einen Chunk-Key vollständig in (TenantId, CollectionId, DocId). Invariante: decode(encode(x)) == x (Injektivität). |

#### `struct TenantScopedStorage`
- Ort: `crates/contextra-store/src/tenant_codec.rs:118`
```rust
pub struct TenantScopedStorage<S>
```
- Impl-Block (Trait `StorageEngine`) `crates/contextra-store/src/compat/engine/tenant_storage.rs:99`
- Impl-Block `crates/contextra-store/src/tenant_codec.rs:124`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `inner` | `S` | `crates/contextra-store/src/tenant_codec.rs:120` |
| `codec` | `TenantKeyCodec` | `crates/contextra-store/src/tenant_codec.rs:121` |

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new(inner: S, tenant_id: TenantId) -> Self` | `crates/contextra-store/src/tenant_codec.rs:126` | Creates a new `TenantScopedStorage` wrapping an underlying `StorageEngine` or shared pointer. |
| `pub fn tenant_id(&self) -> TenantId` | `crates/contextra-store/src/tenant_codec.rs:134` | Returns the immutable `TenantId` bound to this storage wrapper. |

### Modul `traits`

#### `struct StoreFileOpenOptions`
- Ort: `crates/contextra-store/src/traits.rs:34`
```rust
pub struct StoreFileOpenOptions
```
Dokumentation: Öffnungs-Optionen für `StoreFile`.
- Impl-Block `crates/contextra-store/src/traits.rs:49`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `read` | `bool` | `crates/contextra-store/src/traits.rs:36` |
| `write` | `bool` | `crates/contextra-store/src/traits.rs:38` |
| `create` | `bool` | `crates/contextra-store/src/traits.rs:40` |
| `create_new` | `bool` | `crates/contextra-store/src/traits.rs:42` |
| `truncate` | `bool` | `crates/contextra-store/src/traits.rs:44` |
| `append` | `bool` | `crates/contextra-store/src/traits.rs:46` |

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn read_only() -> Self` | `crates/contextra-store/src/traits.rs:51` | Standardoptionen für reines Lesen. |
| `pub fn create_new() -> Self` | `crates/contextra-store/src/traits.rs:63` | Standardoptionen für reines Erzeugen / Schreiben. |

#### `trait StoreFile`
- Ort: `crates/contextra-store/src/traits.rs:12`
```rust
pub trait StoreFile: Send + Sync + Debug + 'static
```
Dokumentation: Positional I/O-Zugriff auf eine geöffnete Datei.

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `fn read_at(&self, buf: &mut [u8], offset: u64) -> std::io::Result<usize>` | `crates/contextra-store/src/traits.rs:14` | Liest positionell bei `offset` in `buf`. |
| `fn write_at(&self, buf: &[u8], offset: u64) -> std::io::Result<usize>` | `crates/contextra-store/src/traits.rs:17` | Schreibt positionell bei `offset` aus `buf`. |
| `fn set_len(&self, len: u64) -> std::io::Result<()>` | `crates/contextra-store/src/traits.rs:20` | Setzt die Dateilänge. |
| `fn len(&self) -> std::io::Result<u64>` | `crates/contextra-store/src/traits.rs:23` | Liefert die Dateilänge in Bytes. |
| `fn sync_data(&self) -> std::io::Result<()>` | `crates/contextra-store/src/traits.rs:26` | Synchronisiert Datenblöcke auf Platte (`fdatasync`). |
| `fn sync_all(&self) -> std::io::Result<()>` | `crates/contextra-store/src/traits.rs:29` | Synchronisiert Datenblöcke und Metadaten auf Platte (`fsync`). |

#### `trait StoreFs`
- Ort: `crates/contextra-store/src/traits.rs:76`
```rust
pub trait StoreFs: Send + Sync + 'static
```
Dokumentation: Lokale VFS-Naht für den Speicherkern.

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `fn open(&self, path: &Path, opts: &StoreFileOpenOptions) -> StoreResult<Box<dyn StoreFile>>` | `crates/contextra-store/src/traits.rs:78` | Öffnet eine Datei mit den gegebenen Optionen. |
| `fn read(&self, path: &Path) -> StoreResult<Vec<u8>>` | `crates/contextra-store/src/traits.rs:81` | Liest eine Datei vollständig ein. |
| `fn write_atomic(&self, path: &Path, data: &[u8]) -> StoreResult<()>` | `crates/contextra-store/src/traits.rs:84` | Schreibt Daten atomar und crash-konsistent. |
| `fn create_dir_all(&self, path: &Path) -> StoreResult<()>` | `crates/contextra-store/src/traits.rs:87` | Erzeugt ein Verzeichnis und alle Elternverzeichnisse. |
| `fn remove_file(&self, path: &Path) -> StoreResult<()>` | `crates/contextra-store/src/traits.rs:90` | Löscht eine Datei. |
| `fn rename(&self, from: &Path, to: &Path) -> StoreResult<()>` | `crates/contextra-store/src/traits.rs:93` | Benennt eine Datei atomar um. |
| `fn exists(&self, path: &Path) -> StoreResult<bool>` | `crates/contextra-store/src/traits.rs:96` | Prüft, ob ein Pfad existiert. |
| `fn len(&self, path: &Path) -> StoreResult<u64>` | `crates/contextra-store/src/traits.rs:99` | Liefert die Dateigröße. |
| `fn list_dir(&self, path: &Path) -> StoreResult<Vec<String>>` | `crates/contextra-store/src/traits.rs:102` | Listet Dateinamen in einem Verzeichnis auf. |
| `fn sync_dir(&self, path: &Path) -> StoreResult<()>` | `crates/contextra-store/src/traits.rs:105` | Synchronisiert Verzeichnis-Metadaten (`fsync` auf Directory). |

### Modul `util`

#### `struct SealedEnvelope`
- Ort: `crates/contextra-store/src/util.rs:22`
```rust
pub struct SealedEnvelope
```
Dokumentation: Kapselung eines versiegelten Datensatzes.
- Impl-Block `crates/contextra-store/src/util.rs:26`
- Impl-Block (Trait `fmt::Debug`) `crates/contextra-store/src/util.rs:40`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new(payload: Vec<u8>) -> Self` | `crates/contextra-store/src/util.rs:29` | Erzeugt einen neuen [`SealedEnvelope`]. |
| `pub fn payload(&self) -> &[u8]` | `crates/contextra-store/src/util.rs:35` | Liefert die Rohdaten des Umschlags. |

#### `trait RecordAuthenticator`
- Ort: `crates/contextra-store/src/util.rs:62`
```rust
pub trait RecordAuthenticator: Send + Sync
```
Dokumentation: Lokaler Trait zur Authentisierung von Datensätzen und Blöcken.

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `fn authenticate(&self, key: &[u8], data: &[u8]) -> Result<[u8` | `crates/contextra-store/src/util.rs:67` | Berechnet ein Authentisierungs-Tag über gegebene Daten. # Errors Liefert einen Fehler zurück, falls das Tag nicht erzeugt werden kann. |
| `fn verify(&self, key: &[u8], data: &[u8], expected_tag: &[u8` | `crates/contextra-store/src/util.rs:73` | Verifiziert ein Authentisierungs-Tag gegen gegebene Daten. # Errors Liefert einen Fehler zurück, falls die Verifikation fehlschlägt. |

#### `trait RecordSealer`
- Ort: `crates/contextra-store/src/util.rs:47`
```rust
pub trait RecordSealer: Send + Sync
```
Dokumentation: Lokaler Trait zur Versiegelung von Datensätzen (Ersatz für externe Krypto-Kopplungen).

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `fn seal(&self, key: &[u8], plaintext: &[u8]) -> Result<SealedEnvelope>` | `crates/contextra-store/src/util.rs:52` | Versiegelt Klartextdaten deterministisch oder mit Umschlag. # Errors Liefert einen Fehler zurück, falls die Versiegelung fehlschlägt. |
| `fn open(&self, key: &[u8], envelope: &[u8]) -> Result<Vec<u8>>` | `crates/contextra-store/src/util.rs:58` | Öffnet einen versiegelten Datensatz. # Errors Liefert einen Fehler zurück, falls die Entsiegelung fehlschlägt. |

### Modul `wal`

#### `mod config`
- Ort: `crates/contextra-store/src/wal/mod.rs:11`
```rust
pub mod config
```

#### `mod core`
- Ort: `crates/contextra-store/src/wal/mod.rs:12`
```rust
pub mod core
```

#### `mod encode`
- Ort: `crates/contextra-store/src/wal/mod.rs:13`
```rust
pub mod encode
```

#### `mod flusher`
- Ort: `crates/contextra-store/src/wal/mod.rs:14`
```rust
pub mod flusher
```

#### `mod fs`
- Ort: `crates/contextra-store/src/wal/mod.rs:15`
```rust
pub mod fs
```

#### `mod hmac`
- Ort: `crates/contextra-store/src/wal/mod.rs:16`
```rust
pub mod hmac
```

#### `mod io`
- Ort: `crates/contextra-store/src/wal/mod.rs:17`
```rust
pub mod io
```

#### `mod poison`
- Ort: `crates/contextra-store/src/wal/mod.rs:18`
```rust
pub mod poison
```

#### `mod replay`
- Ort: `crates/contextra-store/src/wal/mod.rs:19`
```rust
pub mod replay
```

### Modul `wal::config`

#### `struct WalConfig`
- Ort: `crates/contextra-store/src/wal/config.rs:14`
```rust
pub struct WalConfig
```
Dokumentation: Konfigurationsoptionen für das Write-Ahead-Log (WAL). Steuert Schlüsselmanager, Integritätsschlüssel-Rückfallebenen, Mindestversion und Flusher-Einstellungen für asynchrone Schreibvorgänge.
- Impl-Block (Trait `std::fmt::Debug`) `crates/contextra-store/src/wal/config.rs:22`
- Impl-Block (Trait `Default`) `crates/contextra-store/src/wal/config.rs:37`
- Impl-Block `crates/contextra-store/src/wal/config.rs:49`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `key_manager` | `Option<Arc<KeyManager>>` | `crates/contextra-store/src/wal/config.rs:15` |
| `allow_legacy_integrity_key_fallback` | `bool` | `crates/contextra-store/src/wal/config.rs:16` |
| `min_wal_version` | `WalVersion` | `crates/contextra-store/src/wal/config.rs:17` |
| `flusher_config` | `WalFlusherConfig` | `crates/contextra-store/src/wal/config.rs:18` |
| `id_gen` | `Option<Arc<dyn contextra_env::IdGen>>` | `crates/contextra-store/src/wal/config.rs:19` |

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn with_legacy_fallback(mut self, allow: bool) -> Self` | `crates/contextra-store/src/wal/config.rs:50` |  |
| `pub fn with_id_gen(mut self, id_gen: Arc<dyn contextra_env::IdGen>) -> Self` | `crates/contextra-store/src/wal/config.rs:55` |  |

### Modul `wal::core`

#### `struct Wal`
- Ort: `crates/contextra-store/src/wal/core.rs:45`
```rust
pub struct Wal
```
Dokumentation: Hauptstruktur fuer das Write-Ahead-Log mit Append- und Replay-Operationen.
- Impl-Block (Trait `Drop`) `crates/contextra-store/src/wal/core.rs:62`
- Impl-Block (Trait `std::fmt::Debug`) `crates/contextra-store/src/wal/core.rs:68`
- Impl-Block `crates/contextra-store/src/wal/core.rs:77`
- Impl-Block `crates/contextra-store/src/wal/core.rs:636`
- Impl-Block `crates/contextra-store/src/wal/flusher.rs:97`
- Impl-Block `crates/contextra-store/src/wal/hmac.rs:150`
- Impl-Block `crates/contextra-store/src/wal/io/append.rs:19`
- Impl-Block `crates/contextra-store/src/wal/io/truncate.rs:19`
- Impl-Block `crates/contextra-store/src/wal/replay.rs:30`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `path` | `PathBuf` | `crates/contextra-store/src/wal/core.rs:46` |
| `size` | `Arc<std::sync::atomic::AtomicU64>` | `crates/contextra-store/src/wal/core.rs:47` |
| `header_written` | `Arc<std::sync::atomic::AtomicBool>` | `crates/contextra-store/src/wal/core.rs:48` |
| `key_manager` | `Option<Arc<KeyManager>>` | `crates/contextra-store/src/wal/core.rs:49` |
| `fallback_integrity_key` | `Option<[u8; 32]>` | `crates/contextra-store/src/wal/core.rs:50` |
| `allow_legacy_integrity_key_fallback` | `Arc<std::sync::atomic::AtomicBool>` | `crates/contextra-store/src/wal/core.rs:51` |
| `legacy_key_used` | `Arc<std::sync::atomic::AtomicBool>` | `crates/contextra-store/src/wal/core.rs:52` |
| `was_legacy_rekeyed` | `Arc<std::sync::atomic::AtomicBool>` | `crates/contextra-store/src/wal/core.rs:53` |
| `last_hmac` | `Arc<tokio::sync::Mutex<[u8; 32]>>` | `crates/contextra-store/src/wal/core.rs:54` |
| `flusher_tx` | `std::sync::RwLock<Option<tokio::sync::mpsc::Sender<WalCommand>>>` | `crates/contextra-store/src/wal/core.rs:55` |
| `flusher_task` | `std::sync::Mutex<Option<tokio::task::JoinHandle<()>>>` | `crates/contextra-store/src/wal/core.rs:56` |
| `sealed` | `Arc<std::sync::atomic::AtomicBool>` | `crates/contextra-store/src/wal/core.rs:57` |
| `truncate_lock` | `Arc<tokio::sync::Mutex<()>>` | `crates/contextra-store/src/wal/core.rs:58` |
| `poison_state` | `Arc<std::sync::atomic::AtomicU8>` | `crates/contextra-store/src/wal/core.rs:59` |

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub async fn open(path: impl AsRef<Path>) -> Result<Self>` | `crates/contextra-store/src/wal/core.rs:78` |  |
| `pub async fn migrate_legacy_wal( path: impl AsRef<Path>, key_manager: Option<Arc<KeyManager>>, ) -> Result<bool>` | `crates/contextra-store/src/wal/core.rs:95` | Explicitly migrates a legacy WAL segment to the V3 format signed with a secure integrity key. **INV-WAL-LEGACY-KEY-1**: Legacy migration requires explicit opera |
| `pub async fn open_for_legacy_migration( path: impl AsRef<Path>, key_manager: Option<Arc<KeyManager>>, ) -> Result<Self>` | `crates/contextra-store/src/wal/core.rs:139` | Explicitly opens a legacy WAL file requiring migration using the legacy integrity key fallback. **INV-WAL-LEGACY-KEY-1**: This is the ONLY entry point permitted |
| `pub async fn open_read_only( path: impl AsRef<Path>, key_manager: Option<Arc<KeyManager>>, ) -> Result<Self>` | `crates/contextra-store/src/wal/core.rs:214` | Opens an existing WAL file solely for read-only replay, without spawning a background write flusher actor or keeping write handles open. |
| `pub async fn open_read_only_with_config( path: impl AsRef<Path>, config: WalConfig, ) -> Result<Self>` | `crates/contextra-store/src/wal/core.rs:229` | Opens an existing WAL file solely for read-only replay with explicit configuration. |
| `pub async fn open_with_key_manager( path: impl AsRef<Path>, key_manager: Option<Arc<KeyManager>>, ) -> Result<Self>` | `crates/contextra-store/src/wal/core.rs:276` | Opens or creates a WAL file with an optional KeyManager. |
| `pub async fn open_with_config(path: impl AsRef<Path>, config: WalConfig) -> Result<Self>` | `crates/contextra-store/src/wal/core.rs:291` | Opens or creates a WAL file with explicit configuration options. |
| `pub fn size(&self) -> u64` | `crates/contextra-store/src/wal/core.rs:468` | Helper to expose integrity key for tests |
| `pub fn path(&self) -> &Path` | `crates/contextra-store/src/wal/core.rs:472` |  |
| `pub fn is_sealed(&self) -> bool` | `crates/contextra-store/src/wal/core.rs:476` |  |
| `pub fn is_poisoned(&self) -> bool` | `crates/contextra-store/src/wal/core.rs:481` | Gibt an, ob das Handle vergiftet ist. |
| `pub fn poison_cause(&self) -> Option<PoisonCause>` | `crates/contextra-store/src/wal/core.rs:486` | Gibt die typisierte Ursache des Poisoning zurück, sofern vorhanden. |
| `pub fn poison_with_cause(&self, cause: PoisonCause)` | `crates/contextra-store/src/wal/core.rs:494` | Vergiftet das Handle mit der angegebenen typisierten Ursache. Das erste Setzen gewinnt (`compare_exchange`), um die primäre Ursache deterministisch zu bewahren. |
| `pub fn poison_for_test(&self)` | `crates/contextra-store/src/wal/core.rs:504` |  |
| `pub fn allow_legacy_fallback_for_test(&self) -> bool` | `crates/contextra-store/src/wal/core.rs:509` |  |
| `pub fn legacy_key_used_for_test(&self) -> bool` | `crates/contextra-store/src/wal/core.rs:514` |  |
| `pub fn was_legacy_rekeyed(&self) -> bool` | `crates/contextra-store/src/wal/core.rs:519` |  |
| `pub async fn truncate_uncommitted_tail(&self) -> Result<u64>` | `crates/contextra-store/src/wal/core.rs:532` | Truncates any uncommitted entries appearing after the last committed `TxEnd` marker in the WAL. Returns the number of bytes truncated. **Policy Note**: `truncat |
| `pub async fn recover_from_poison(&self) -> Result<()>` | `crates/contextra-store/src/wal/core.rs:574` | Stellt ein mit `PoisonCause::TornWrite` vergiftetes `Wal`-Handle nach einem partiellen Schreibabbruch wieder her. Re-verifiziert den physischen Dateizustand per |
| `pub fn open_loom() -> Self` | `crates/contextra-store/src/wal/core.rs:637` |  |
| `pub async fn prepare_batch(&self, ops: Vec<(WalOp, u64)>) -> Result<(PreparedBatch, [u8` | `crates/contextra-store/src/wal/hmac.rs:152` | Prepares a batch of WAL operations with sequential sequence numbers and HMAC hash-chaining. |
| `pub async fn rollback_hmac(&self, prev_hmac: [u8` | `crates/contextra-store/src/wal/hmac.rs:178` | Rolls back the in-memory HMAC state to `prev_hmac` asynchronously. |
| `pub fn rollback_hmac_sync(&self, prev_hmac: [u8` | `crates/contextra-store/src/wal/hmac.rs:185` | Synchronously rolls back the in-memory HMAC state to `prev_hmac` using non-blocking `try_lock`. Suitable for use within `Drop` implementations. |
| `pub fn integrity_key_for_test(&self) -> Result<[u8` | `crates/contextra-store/src/wal/hmac.rs:209` |  |
| `pub fn last_hmac_lock_for_test(&self) -> std::sync::Arc<tokio::sync::Mutex<[u8` | `crates/contextra-store/src/wal/hmac.rs:215` |  |
| `pub fn legacy_integrity_key_for_test() -> Result<[u8` | `crates/contextra-store/src/wal/hmac.rs:220` |  |
| `pub async fn append_batch(&self, batch: PreparedBatch) -> Result<()>` | `crates/contextra-store/src/wal/io/append.rs:184` |  |
| `pub async fn enqueue_append_batch_locked( &self, batch: PreparedBatch, _guard: &tokio::sync::MutexGuard<'_, ()>, ) -> Result<tokio::sync::oneshot::Receiver<Result<()>>>` | `crates/contextra-store/src/wal/io/append.rs:196` |  |
| `pub async fn append_batch_locked( &self, batch: PreparedBatch, guard: &tokio::sync::MutexGuard<'_, ()>, ) -> Result<()>` | `crates/contextra-store/src/wal/io/append.rs:235` |  |
| `pub async fn try_append_batch(&self, batch: PreparedBatch) -> Result<()>` | `crates/contextra-store/src/wal/io/append.rs:249` | Non-blocking attempt to append a prepared batch. Returns `Err(StoreError::Storage("WAL queue full (backpressure)"))` if the flusher channel buffer is full. |
| `pub async fn try_enqueue_append_batch_locked( &self, batch: PreparedBatch, _guard: &tokio::sync::MutexGuard<'_, ()>, ) -> Result<tokio::sync::oneshot::Receiver<Result<()>>>` | `crates/contextra-store/src/wal/io/append.rs:261` |  |
| `pub async fn try_append_batch_locked( &self, batch: PreparedBatch, guard: &tokio::sync::MutexGuard<'_, ()>, ) -> Result<()>` | `crates/contextra-store/src/wal/io/append.rs:306` |  |
| `pub async fn scan_entries_with_callback<F>( &self, file_size: u64, mut callback: F, ) -> Result<WalVersion> where F: FnMut(u64, WalEntry, u64) -> bool,` | `crates/contextra-store/src/wal/io/truncate.rs:25` | Scans the WAL entry by entry, executing full HMAC chain validation, CRC checks, and key manager decryption. Invokes `callback(seq_no, entry, end_offset)` for ea |
| `pub async fn truncate(&self, offset: u64, new_last_hmac: [u8` | `crates/contextra-store/src/wal/io/truncate.rs:97` |  |
| `pub async fn rotate_and_seal(&self) -> Result<PathBuf>` | `crates/contextra-store/src/wal/io/truncate.rs:130` |  |
| `pub async fn find_tx_offset(&self, target_tx_id: TxId) -> Result<(u64, [u8` | `crates/contextra-store/src/wal/io/truncate.rs:156` |  |
| `pub async fn last_hmac_snapshot(&self) -> [u8` | `crates/contextra-store/src/wal/io/truncate.rs:189` | Returns a snapshot of the last HMAC written to the log. |
| `pub fn arm_truncate_failure_for_test(&self)` | `crates/contextra-store/src/wal/io/truncate.rs:199` | Aktiviert eine einmalige simulierte Truncate-I/O-Failure für Tests. Der Flusher gibt beim nächsten `WalCommand::Truncate` einen Fehler zurück, ohne `set_len()`  |
| `pub async fn replay(&self) -> Result<Vec<(u64, WalEntry, u64)>>` | `crates/contextra-store/src/wal/replay.rs:67` | Replays the WAL using memory mapping (`memmap2`) for zero-copy entry decoding. Falls back to stream replay if mmap or parsing fails. |
| `pub async fn replay_with_sink<S: ReplayProgressSink>( &self, sink: &S, ) -> Result<Vec<(u64, WalEntry, u64)>>` | `crates/contextra-store/src/wal/replay.rs:72` | Replays the WAL while notifying the provided `ReplayProgressSink` of progress. |
| `pub async fn replay_stream(&self) -> Result<Vec<(u64, WalEntry, u64)>>` | `crates/contextra-store/src/wal/replay.rs:81` | Replays the WAL using the stream reader (`BufReader`). |
| `pub async fn replay_from(&self, since_seq_no: u64) -> Result<Vec<WalEntry>>` | `crates/contextra-store/src/wal/replay.rs:97` | Replays the WAL and returns all entries with seq_no > since_seq_no. |
| `pub async fn replay_mmap(&self) -> Result<(Vec<(u64, WalEntry, u64)>, WalVersion)>` | `crates/contextra-store/src/wal/replay.rs:142` | Replays the WAL using zero-copy memory mapping (`mmap`). Returns all valid entries with sequence numbers, entries, end offsets, and detected WAL version. |
| `pub async fn replay_mmap_with_sink<S: ReplayProgressSink>( &self, sink: &S, ) -> Result<(Vec<(u64, WalEntry, u64)>, WalVersion)>` | `crates/contextra-store/src/wal/replay.rs:148` | Replays the WAL using zero-copy memory mapping (`mmap`) with progress callback. |
| `pub async fn scan_entries_mmap<F>(&self, file_size: u64, mut callback: F) -> Result<WalVersion> where F: FnMut(u64, WalEntry, u64) -> bool,` | `crates/contextra-store/src/wal/replay.rs:202` | Scans the WAL using read-only memory mapping (`memmap2`) for zero-copy entry parsing. If mmap mapping or slice scanning encounters any error, it logs a warning  |

#### `const DEFAULT_WAL_QUEUE_CAPACITY`
- Ort: `crates/contextra-store/src/wal/core.rs:25`
```rust
pub const DEFAULT_WAL_QUEUE_CAPACITY: usize = 1_024
```
Dokumentation: Default capacity for the bounded WAL flusher command channel.

#### `const MAX_WAL_ENTRY_SIZE`
- Ort: `crates/contextra-store/src/wal/core.rs:22`
```rust
pub const MAX_WAL_ENTRY_SIZE: u32 = 64 * 1024 * 1024
```
Dokumentation: Maximum size for a single WAL entry payload (64MB).

#### `const MAX_WAL_SIZE`
- Ort: `crates/contextra-store/src/wal/core.rs:19`
```rust
pub const MAX_WAL_SIZE: u64 = 128 * 1024 * 1024
```

#### `static DELAY_APPEND_FOR_TX`
- Ort: `crates/contextra-store/src/wal/core.rs:28`
```rust
pub static DELAY_APPEND_FOR_TX: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0)
```

#### `static DELAY_APPEND_MS`
- Ort: `crates/contextra-store/src/wal/core.rs:29`
```rust
pub static DELAY_APPEND_MS: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0)
```

#### `static FAIL_APPEND_AFTER_PARTIAL_BYTES`
- Ort: `crates/contextra-store/src/wal/core.rs:36`
```rust
pub static FAIL_APPEND_AFTER_PARTIAL_BYTES: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0)
```
- Gate: `#[cfg(feature = "fault-injection")]`

#### `static FAIL_APPEND_FOR_TX`
- Ort: `crates/contextra-store/src/wal/core.rs:27`
```rust
pub static FAIL_APPEND_FOR_TX: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0)
```

#### `static FAIL_APPEND_PARTIAL_ONCE`
- Ort: `crates/contextra-store/src/wal/core.rs:40`
```rust
pub static FAIL_APPEND_PARTIAL_ONCE: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false)
```
- Gate: `#[cfg(feature = "fault-injection")]`

#### `static FAIL_TRUNCATE_ONCE`
- Ort: `crates/contextra-store/src/wal/core.rs:32`
```rust
pub static FAIL_TRUNCATE_ONCE: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false)
```
- Gate: `#[cfg(feature = "fault-injection")]`

### Modul `wal::encode`

#### `struct PreparedBatch`
- Ort: `crates/contextra-store/src/wal/encode.rs:83`
```rust
pub struct PreparedBatch(pub(crate) Vec<WalEntry>)
```
Dokumentation: A batch of prepared WAL entries bound to a specific HMAC chain. `PreparedBatch` has a private inner field and cannot be constructed outside `wal.rs`. It can only be created by calling [`crate::wal::Wal::prepare_batch`].
- Impl-Block `crates/contextra-store/src/wal/encode.rs:85`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn is_empty(&self) -> bool` | `crates/contextra-store/src/wal/encode.rs:87` | Returns `true` if the batch contains no entries. |
| `pub fn len(&self) -> usize` | `crates/contextra-store/src/wal/encode.rs:92` | Returns the number of entries in the batch. |
| `pub fn entries(&self) -> &[WalEntry]` | `crates/contextra-store/src/wal/encode.rs:97` | Returns a slice of the inner WAL entries. |
| `pub fn into_inner(self) -> Vec<WalEntry>` | `crates/contextra-store/src/wal/encode.rs:102` | Consumes the `PreparedBatch` and returns the inner vector of entries. |
| `pub fn extend(&mut self, other: PreparedBatch)` | `crates/contextra-store/src/wal/encode.rs:107` | Extends this prepared batch with entries from another prepared batch. |

#### `struct WalEntry`
- Ort: `crates/contextra-store/src/wal/encode.rs:114`
```rust
pub struct WalEntry
```
Dokumentation: A single entry in the Write-Ahead Log.
- Impl-Block `crates/contextra-store/src/wal/encode.rs:125`
- Impl-Block `crates/contextra-store/src/wal/encode.rs:131`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `op` | `WalOp` | `crates/contextra-store/src/wal/encode.rs:116` |
| `seq_no` | `u64` | `crates/contextra-store/src/wal/encode.rs:118` |
| `checksum` | `[u8; 32]` | `crates/contextra-store/src/wal/encode.rs:120` |
| `prev_hmac` | `[u8; 32]` | `crates/contextra-store/src/wal/encode.rs:122` |

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn tx_id(&self) -> TxId` | `crates/contextra-store/src/wal/encode.rs:126` |  |
| `pub fn try_new( op: WalOp, seq_no: u64, integrity_key: &[u8], prev_hmac: [u8` | `crates/contextra-store/src/wal/encode.rs:133` | Creates a new WAL entry with HMAC-SHA256 checksum and chaining. |
| `pub fn compute_checksum_v3( op: &WalOp, seq_no: u64, integrity_key: &[u8], prev_hmac: [u8` | `crates/contextra-store/src/wal/encode.rs:149` | Computes V3 checksum (includes tx_id and length prefixes for key/value). |
| `pub fn compute_checksum_v2( op: &WalOp, seq_no: u64, integrity_key: &[u8], prev_hmac: [u8` | `crates/contextra-store/src/wal/encode.rs:187` | Legacy V2 checksum calculation (without tx_id and length-prefixes in HMAC). |
| `pub fn compute_checksum( op: &WalOp, seq_no: u64, integrity_key: &[u8], prev_hmac: [u8` | `crates/contextra-store/src/wal/encode.rs:215` |  |
| `pub fn to_bytes(&self) -> StoreResult<Vec<u8>>` | `crates/contextra-store/src/wal/encode.rs:225` | Serializes the entry to bytes. |
| `pub fn from_bytes_classified(data: &[u8]) -> std::result::Result<Self, WalParseError>` | `crates/contextra-store/src/wal/encode.rs:290` | Deserializes a WAL entry from bytes, returning a classified `WalParseError` on failure. |
| `pub fn from_bytes(data: &[u8]) -> StoreResult<Self>` | `crates/contextra-store/src/wal/encode.rs:444` | Deserializes a WAL entry from bytes, verifying CRC32. |

#### `enum WalOp`
- Ort: `crates/contextra-store/src/wal/encode.rs:42`
```rust
pub enum WalOp
```
- Impl-Block `crates/contextra-store/src/wal/encode.rs:55`

Varianten:

- `Put{` — `crates/contextra-store/src/wal/encode.rs:44`
- `Delete{ tx_id: TxId, key: Vec<u8> }` — `crates/contextra-store/src/wal/encode.rs:50`
- `TxEnd{ tx_id: TxId, committed: bool }` — `crates/contextra-store/src/wal/encode.rs:52`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn tx_id(&self) -> TxId` | `crates/contextra-store/src/wal/encode.rs:56` |  |

#### `enum WalParseError`
- Ort: `crates/contextra-store/src/wal/encode.rs:20`
```rust
pub enum WalParseError
```
Dokumentation: Typed parsing errors encountered when decoding WAL entries from raw bytes.

Varianten:

- `HeaderTooShort` — `crates/contextra-store/src/wal/encode.rs:22`
- `CrcMismatch{ stored: u32, computed: u32 }` — `crates/contextra-store/src/wal/encode.rs:24`
- `Truncated` — `crates/contextra-store/src/wal/encode.rs:26`
- `OversizedKey` — `crates/contextra-store/src/wal/encode.rs:28`
- `OversizedValue{ limit: usize }` — `crates/contextra-store/src/wal/encode.rs:30`
- `UnknownOp(u8)` — `crates/contextra-store/src/wal/encode.rs:32`
- `InvalidCommittedByte(u8)` — `crates/contextra-store/src/wal/encode.rs:34`
- `TrailingBytes` — `crates/contextra-store/src/wal/encode.rs:36`
- `Malformed(&'static str)` — `crates/contextra-store/src/wal/encode.rs:38`

#### `enum WalVersion`
- Ort: `crates/contextra-store/src/wal/encode.rs:72`
```rust
pub enum WalVersion
```

Varianten:

- `V1` — `crates/contextra-store/src/wal/encode.rs:73`
- `V2` — `crates/contextra-store/src/wal/encode.rs:74`
- `V3` — `crates/contextra-store/src/wal/encode.rs:75`

#### `const WAL_V2_HEADER`
- Ort: `crates/contextra-store/src/wal/encode.rs:66`
```rust
pub const WAL_V2_HEADER: [u8
```
Dokumentation: Magic header for V2 batch-encrypted WAL files (`b"MFW2"`).

#### `const WAL_V3_HEADER`
- Ort: `crates/contextra-store/src/wal/encode.rs:69`
```rust
pub const WAL_V3_HEADER: [u8
```
Dokumentation: Magic header for V3 WAL files (`b"MFW3"`).

### Modul `wal::flusher`

#### `struct WalFlusherConfig`
- Ort: `crates/contextra-store/src/wal/flusher.rs:27`
```rust
pub struct WalFlusherConfig
```
Dokumentation: Configuration for the WAL background flusher actor. Controls how aggressively the flusher coalesces concurrent writes into a single `sync_all()` call to reduce fsync overhead under load, and sets bounded backpressure queue capacity.
- Impl-Block (Trait `Default`) `crates/contextra-store/src/wal/flusher.rs:40`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `batch_window_micros` | `u64` | `crates/contextra-store/src/wal/flusher.rs:33` |
| `queue_capacity` | `usize` | `crates/contextra-store/src/wal/flusher.rs:37` |

#### `static FAIL_SYNC_ONCE`
- Ort: `crates/contextra-store/src/wal/flusher.rs:15`
```rust
pub static FAIL_SYNC_ONCE: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false)
```
- Gate: `#[cfg(feature = "fault-injection")]`

### Modul `wal::fs`

#### `mod loom_fs`
- Ort: `crates/contextra-store/src/wal/fs.rs:10`
```rust
pub mod loom_fs
```

### Modul `wal::fs::loom_fs`

#### `struct LoomFile`
- Ort: `crates/contextra-store/src/wal/fs.rs:153`
```rust
pub struct LoomFile
```
- Impl-Block `crates/contextra-store/src/wal/fs.rs:158`
- Impl-Block (Trait `tokio::io::AsyncRead`) `crates/contextra-store/src/wal/fs.rs:199`
- Impl-Block (Trait `tokio::io::AsyncWrite`) `crates/contextra-store/src/wal/fs.rs:216`
- Impl-Block (Trait `tokio::io::AsyncSeek`) `crates/contextra-store/src/wal/fs.rs:239`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `pos` | `usize` | `crates/contextra-store/src/wal/fs.rs:154` |
| `buf` | `Vec<u8>` | `crates/contextra-store/src/wal/fs.rs:155` |

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new() -> Self` | `crates/contextra-store/src/wal/fs.rs:159` |  |
| `pub async fn open<P: AsRef<Path>>(_path: P) -> std::io::Result<Self>` | `crates/contextra-store/src/wal/fs.rs:165` |  |
| `pub async fn write_all(&mut self, buf: &[u8]) -> std::io::Result<()>` | `crates/contextra-store/src/wal/fs.rs:168` |  |
| `pub async fn flush(&self) -> std::io::Result<()>` | `crates/contextra-store/src/wal/fs.rs:172` |  |
| `pub async fn sync_all(&self) -> std::io::Result<()>` | `crates/contextra-store/src/wal/fs.rs:175` |  |
| `pub async fn set_len(&mut self, len: u64) -> std::io::Result<()>` | `crates/contextra-store/src/wal/fs.rs:178` |  |
| `pub async fn seek(&mut self, pos: std::io::SeekFrom) -> std::io::Result<u64>` | `crates/contextra-store/src/wal/fs.rs:182` |  |
| `pub async fn metadata(&self) -> std::io::Result<LoomMetadata>` | `crates/contextra-store/src/wal/fs.rs:194` |  |

#### `struct LoomMetadata`
- Ort: `crates/contextra-store/src/wal/fs.rs:59`
```rust
pub struct LoomMetadata
```
- Impl-Block `crates/contextra-store/src/wal/fs.rs:60`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn len(&self) -> u64` | `crates/contextra-store/src/wal/fs.rs:61` |  |
| `pub fn permissions(&self) -> std::fs::Permissions` | `crates/contextra-store/src/wal/fs.rs:64` |  |

#### `struct OpenOptions`
- Ort: `crates/contextra-store/src/wal/fs.rs:124`
```rust
pub struct OpenOptions
```
- Impl-Block `crates/contextra-store/src/wal/fs.rs:125`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new() -> Self` | `crates/contextra-store/src/wal/fs.rs:126` |  |
| `pub fn read(&mut self, _read: bool) -> &mut Self` | `crates/contextra-store/src/wal/fs.rs:129` |  |
| `pub fn write(&mut self, _write: bool) -> &mut Self` | `crates/contextra-store/src/wal/fs.rs:132` |  |
| `pub fn create(&mut self, _create: bool) -> &mut Self` | `crates/contextra-store/src/wal/fs.rs:135` |  |
| `pub fn create_new(&mut self, _create_new: bool) -> &mut Self` | `crates/contextra-store/src/wal/fs.rs:138` |  |
| `pub fn append(&mut self, _append: bool) -> &mut Self` | `crates/contextra-store/src/wal/fs.rs:141` |  |
| `pub fn mode(&mut self, _mode: u32) -> &mut Self` | `crates/contextra-store/src/wal/fs.rs:144` |  |
| `pub async fn open<P: AsRef<Path>>(&self, _path: P) -> std::io::Result<LoomFile>` | `crates/contextra-store/src/wal/fs.rs:147` |  |

#### `fn copy`
- Ort: `crates/contextra-store/src/wal/fs.rs:34`
```rust
pub async fn copy<P: AsRef<Path>, Q: AsRef<Path>>(from: P, to: Q) -> std::io::Result<u64>
```

#### `fn create_dir_all`
- Ort: `crates/contextra-store/src/wal/fs.rs:13`
```rust
pub async fn create_dir_all<P: AsRef<Path>>(path: P) -> std::io::Result<()>
```

#### `fn hard_link`
- Ort: `crates/contextra-store/src/wal/fs.rs:38`
```rust
pub async fn hard_link<P: AsRef<Path>, Q: AsRef<Path>>(from: P, to: Q) -> std::io::Result<()>
```

#### `fn metadata`
- Ort: `crates/contextra-store/src/wal/fs.rs:53`
```rust
pub async fn metadata<P: AsRef<Path>>(path: P) -> std::io::Result<LoomMetadata>
```

#### `fn read`
- Ort: `crates/contextra-store/src/wal/fs.rs:17`
```rust
pub async fn read<P: AsRef<Path>>(path: P) -> std::io::Result<Vec<u8>>
```

#### `fn remove_file`
- Ort: `crates/contextra-store/src/wal/fs.rs:26`
```rust
pub async fn remove_file<P: AsRef<Path>>(path: P) -> std::io::Result<()>
```

#### `fn rename`
- Ort: `crates/contextra-store/src/wal/fs.rs:30`
```rust
pub async fn rename<P: AsRef<Path>, Q: AsRef<Path>>(from: P, to: Q) -> std::io::Result<()>
```

#### `fn set_permissions`
- Ort: `crates/contextra-store/src/wal/fs.rs:46`
```rust
pub async fn set_permissions<P: AsRef<Path>>( path: P, perm: std::fs::Permissions, ) -> std::io::Result<()>
```

#### `fn try_exists`
- Ort: `crates/contextra-store/src/wal/fs.rs:42`
```rust
pub async fn try_exists<P: AsRef<Path>>(path: P) -> std::io::Result<bool>
```

#### `fn write`
- Ort: `crates/contextra-store/src/wal/fs.rs:20`
```rust
pub async fn write<P: AsRef<Path>, C: AsRef<[u8]>>( path: P, contents: C, ) -> std::io::Result<()>
```

#### `type File`
- Ort: `crates/contextra-store/src/wal/fs.rs:263`
```rust
pub type File = LoomFile
```

### Modul `wal::hmac`

#### `enum LegacyKeyStatus`
- Ort: `crates/contextra-store/src/wal/hmac.rs:55`
```rust
pub enum LegacyKeyStatus
```
Dokumentation: Status indicating whether a WAL segment or key configuration uses the obfuscated legacy key.
- Impl-Block `crates/contextra-store/src/wal/hmac.rs:62`

Varianten:

- `Standard` — `crates/contextra-store/src/wal/hmac.rs:57`
- `LegacyActive` — `crates/contextra-store/src/wal/hmac.rs:59`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub const fn is_legacy(self) -> bool` | `crates/contextra-store/src/wal/hmac.rs:64` | Returns `true` if the status is [`LegacyKeyStatus::LegacyActive`]. |
| `pub const fn is_standard(self) -> bool` | `crates/contextra-store/src/wal/hmac.rs:69` | Returns `true` if the status is [`LegacyKeyStatus::Standard`]. |

#### `fn migrate_legacy_key`
- Ort: `crates/contextra-store/src/wal/hmac.rs:115`
```rust
pub fn migrate_legacy_key(legacy: &[u8], new_key_material: &[u8]) -> Result<[u8
```
Dokumentation: Migrates a legacy integrity key to a new 32-byte key material state. # Chain Compatibility When migrating a legacy WAL segment to a new integrity key format, HMAC chain validation during replay must verify historical entries using the legacy key before signing new entries or rewriting segments using `new_key_material`. # Security & Zeroize Note `contextra-store` does not use a `zeroize` dependency. To avoid non-volatile manual memory wipes, key material in memory is handled via standard Rust slice operations without manual zeroing. # Errors Returns [`StoreError::Storage`] if `legacy` does not 
- Gate: `#[cfg(feature = "legacy-wal-key")]`

#### `fn migrate_legacy_key`
- Ort: `crates/contextra-store/src/wal/hmac.rs:144`
```rust
pub fn migrate_legacy_key(_legacy: &[u8], _new_key_material: &[u8]) -> Result<[u8
```

### Modul `wal::io`

#### `mod acl`
- Ort: `crates/contextra-store/src/wal/io/mod.rs:5`
```rust
pub mod acl
```

#### `mod append`
- Ort: `crates/contextra-store/src/wal/io/mod.rs:6`
```rust
pub mod append
```

#### `mod scan`
- Ort: `crates/contextra-store/src/wal/io/mod.rs:7`
```rust
pub mod scan
```

#### `mod truncate`
- Ort: `crates/contextra-store/src/wal/io/mod.rs:8`
```rust
pub mod truncate
```

### Modul `wal::poison`

#### `enum PoisonCause`
- Ort: `crates/contextra-store/src/wal/poison.rs:15`
```rust
pub enum PoisonCause
```
Dokumentation: Typisierte Ursache für das Vergiften eines WAL-Handles (SPEC §3.5, §12.1, INV-FSYNC-1). Nach einem Fsync-, Write-, Truncate- oder HMAC-Kettendivergenz-Fehler wird das WAL-Handle irreversibel vergiftet. Einzig bei einem partiellen Schreibabbruch (`TornWrite`) darf das Handle über `recover_from_poison()` repariert werden.
- Impl-Block `crates/contextra-store/src/wal/poison.rs:28`

Varianten:

- `FsyncFailed= 1` — `crates/contextra-store/src/wal/poison.rs:17`
- `WriteFailed= 2` — `crates/contextra-store/src/wal/poison.rs:19`
- `TruncateFailed= 3` — `crates/contextra-store/src/wal/poison.rs:21`
- `HmacChainDiverged= 4` — `crates/contextra-store/src/wal/poison.rs:23`
- `TornWrite= 5` — `crates/contextra-store/src/wal/poison.rs:25`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub const fn to_u8(self) -> u8` | `crates/contextra-store/src/wal/poison.rs:30` |  |
| `pub const fn from_u8(val: u8) -> Option<Self>` | `crates/contextra-store/src/wal/poison.rs:35` |  |

### Modul `wal::replay`

#### `struct NoopReplayProgressSink`
- Ort: `crates/contextra-store/src/wal/replay.rs:25`
```rust
pub struct NoopReplayProgressSink
```
- Impl-Block (Trait `ReplayProgressSink`) `crates/contextra-store/src/wal/replay.rs:26`

#### `trait ReplayProgressSink`
- Ort: `crates/contextra-store/src/wal/replay.rs:21`
```rust
pub trait ReplayProgressSink: Send + Sync
```

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `fn on_entry_replayed(&self, seq: WalSeq, entries_total_estimate: Option<u64>)` | `crates/contextra-store/src/wal/replay.rs:22` |  |

#### `type WalSeq`
- Ort: `crates/contextra-store/src/wal/replay.rs:19`
```rust
pub type WalSeq = u64
```

## 9. Impl-Blöcke ohne Typ im selben Crate

- `impl<S: StorageEngine> StorageEngineHandle for Arc<S>` — `crates/contextra-store/src/compat/engine/tenant_storage.rs:70` (0 pub Methoden)
- `impl From<StoreError> for ContextraError` — `crates/contextra-store/src/compat/error.rs:9` (0 pub Methoden)
- `impl<T: StoreFs + ?Sized> StoreFs for Arc<T>` — `crates/contextra-store/src/traits.rs:108` (0 pub Methoden)
