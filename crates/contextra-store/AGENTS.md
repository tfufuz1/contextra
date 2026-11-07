# AGENTS.md — `contextra-store`

> **Ring 1** · Klasse `[K]` · Reifegrad `beta` · Unsafe: nein · Rolle: LSM-Storage: WAL, Memtable, SSTable, Manifest, Kompaktion, Block-Cache. Implementiert `StorageEngine`.
> Gemeinsame Regeln stehen in der Root-`AGENTS.md`; diese Datei nennt nur Crate-Spezifisches. Stand 2026-11-07, statisch gelesen, **nicht kompiliert**; Zahlen sind grep-Heuristiken.

## 1. Baukasten-Profil

| Aspekt | Wert |
|---|---|
| Standalone-Build | `cargo build -p contextra-store --no-default-features --locked` |
| Contextra-Abhängigkeiten **Ist** (Cargo) | `crypto`, `mvcc`, `ports`, `sys`, `types` |
| Contextra-Abhängigkeiten **Soll** (PLAN §4) | `ports`, `types`, `mvcc`, `crypto`, `sys`, `durable-fs` |
| Features | `block-cache-v2`, `sieve-cache`, `docid-128`, `fault-injection`, `memory-only-storage`, `legacy-wal-key`, `test-hooks` |
| Lints | `#![forbid(unsafe_code)]`, `deny(dead_code)` |

## 2. Abhängigkeiten & Schulden

- **Dev-Kanten auf höhere/gleiche Schichten** (M6): `graph`, `sandbox`, `vector` (+ Selbstreferenz). Nach `testkit`/`tests/` verlagern.
- **Kein Dep auf `durable-fs`**; direkte Datei-I/O (~77 `fs`-Treffer, ~176 `tokio::`-Treffer): Umstellung auf `Vfs` (D4, WP 4.4).
- **Löschreste:** `kv/segment.rs`, `kv/delete_mode.rs` (Shredding-Bezug, 28 Treffer), `compaction/engine.rs`, `compaction/adaptive.rs` (WP 3.2).
- Implementierte Ports sind vorhanden: `scanner/` (`LsmPurger`, `MemtableScanner`, `SstableScanner`, `WalScanner`).

## 3. Modul-Karte (am Code verifiziert)

| Pfad | Verantwortung |
|---|---|
| `wal/` (`encode`, `flusher`, `hmac`, `io`, `replay`) | Append-only WAL, HMAC-Kette, Flusher-Actor, Replay |
| `lsm/` (`engine`, `commit`, `flush`, `group_commit`, `guard`, `recovery`, `scan`, `ops/*`, `validate`) | Orchestrierung, Commit, Recovery |
| `memtable.rs` | Memtable mit Sequenznummern/Tombstones |
| `sstable/` (`builder`, `reader`, `block_cache`, `bloom`, `stream`, …) | Sortierte Segmente, Block-Cache, Bloom |
| `manifest/` | transaktionales Manifest |
| `compaction/` (`engine`, `adaptive`, `config`, `merge_operator`, `retention`) | Level-Merge, `MergeOperator` |
| `scanner/` (`memtable`, `sstable`, `wal`, `purge`) | **Port-Implementierungen** `LayerScanner`/`Purger` |
| `kv/` | KV-Segmente, Löschmodus (Schuld) |
| `fsync.rs`, `util.rs`, `system_pressure.rs`, `tenant_codec.rs`, `engine/` | Sync-Stelle, Hilfen, Druck, Tenant-Präfix, In-Memory-Engine |

## 4. Verträge & Invarianten

1. **`fsync`-Fehler propagieren** (`?`), nie `let _ =`; nach Fehler Handle vergiften (INV-FSYNC-1). `fsync.rs` ist die einzige Stelle für Datei-/Verzeichnis-Sync.
2. **Atomares Schreiben:** tmp-Datei → `fsync` → `rename` → Parent-`fsync`.
3. **Tombstone-Disziplin:** Bit 63 maskieren (`seq & !TOMBSTONE_BIT`) vor jedem `max_seq`-Vergleich (verifiziert u. a. in `compaction/adaptive.rs`).
4. **Single-Load-Regel:** `last_committed_tx` in `get_at_seq`/`scan_prefix_at` **einmal** laden (Snapshot-Isolation). **Flush-before-Visible:** `last_committed_tx` vor dem Sichtbarmachen der SSTable aktualisieren (ADR-043, Altdokument: am Code in `lsm/flush.rs` prüfen).
5. **WAL-HMAC-Schlüssel** nur über `load_or_create_integrity_key` (`wal/hmac.rs`, `pub(crate)`), nie hartcodiert; `legacy-wal-key` nur für Migration.
6. **Tenant-Präfix in jedem Schlüssel** (INV-TENANT-1); Schlüssel/Dateinamen nur opake IDs (INV-KEYSPACE-1).
7. **Neu verboten:** DSGVO-/Proof-/Saga-/Shredding-Logik (M7). Purge-Kompaktion nur über `Purger` (`compact_range(scope)`).
8. Jede Datei: Magic + Formatversion (INV-FORMAT-1); Prüfsummen erkennen Bit-Rot (kein Korrekturversprechen).

## 5. Ports

**Implementiert:** `StorageEngine`, `Purger`, `LayerScanner`, `BlockCache`, `MergeOperator`. **Konsumiert:** `Clock`, `Vfs` (Soll), `MetricsSink`.

## 6. Nebenläufigkeit & Performance

- **Sperrenordnung** (Altdokument, vor Änderung verifizieren): `write_lock` → `memtable` → `sstables` → `snapshot_registry`. Nie umgekehrt, nie `parking_lot`-Guard über `.await` (`clippy::await_holding_lock`).
- Schreibpfad über Group-Commit und Flusher-Actor (Message-Passing statt geteiltem Zustand). `loom`: `just loom-store`.
- Lesepfad Ziel: allokationsfrei (`with_value`, `PinnedBlock`, WP 4.1). Blockierende Datei-I/O in `async` nur via `spawn_blocking`.
- Komplexität: Punktlesen `O(log n)`-Ebenen + Bloom; Compaction-Schreibverstärkung messen (`tests/amplification_benchmark.rs`).

## 7. Anti-Patterns

- `let _ = file.sync_all()`.
- `bincode::deserialize(..).unwrap_or_default()` auf WAL-Daten.
- `std::fs` direkt statt `Vfs`.
- Neue Abhängigkeit auf `vector`/`text`/`graph` (Peer-Import verboten).
- Löschlogik/Proof-Typen im Store.

## 8. Tests & Messung

```bash
cargo test -p contextra-store --locked
cargo clippy -p contextra-store --all-targets -- -D warnings
just loom-store
cargo bench -p contextra-store            # wal, memtable, sstable, block_cache, lsm
cargo mutants --config mutants-guarantees.toml -p contextra-store -f "crates/contextra-store/src/wal/**"
```

- Vorhanden: Chaos (`chaos_*`), Campaign-/Property-Tests, Fuzz (`fuzz_wal_replay`, `wal_roundtrip`, `fuzz_manifest_load`, `wal_mutation_chaos`), Benchmarks inkl. Vergleich `redb`.
- Ratchet: `tokio::`/`fs`-Treffer dürfen nicht steigen.

## 9. Offene Arbeit

| WP | Inhalt |
|---|---|
| 3.2 | Löschketten-/Shredding-Relikte entfernen, nur `Purger`/`LayerScanner` behalten |
| 4.1 | Compaction als Level-Merge, `PinnedBlock`, `with_value` |
| 4.4 / D4 | `Vfs` statt `fs`/`tokio::fs`; Dep auf `durable-fs` |
| M6 | Dev-Kanten `graph`, `sandbox`, `vector` entfernen |
