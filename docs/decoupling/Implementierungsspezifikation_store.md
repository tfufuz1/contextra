# Implementierungsspezifikation: `contextra-store` → entkoppelte Storage-Bausteine

| | |
|---|---|
| **Crate** | `crates/contextra-store` (LSM-Tree: WAL, MemTable, SSTable, Manifest, Compaction) |
| **Zielzustand** | Sammlung isolierter, einzeln entnehmbarer Bausteine; kein Produkt-Ökosystem mehr |
| **Methode** | Rein statische Analyse (Quelltext, `Cargo.toml`, `AGENTS.md`, `grep`-Kopplungsmatrix). **Nichts wurde kompiliert oder ausgeführt.** Aussagen über Build-Fähigkeit sind Schlussfolgerungen aus dem Code und vor der Umsetzung per `cargo` zu bestätigen. |
| **Umfang Ist** | 19.576 Zeilen Nicht-Test-Quelltext in `src/`, 134 Integrationstests, 10 Fuzz-Targets, 6 Benchmarks |

---

## 1. Ziel und Nicht-Ziele

**Ziel.** Jeder Baustein (WAL, MemTable, SSTable-Format/-Reader, Block-Cache, Manifest, Compaction, Tenant-Codec, System-Pressure) lässt sich per `cp -r` in ein fremdes Projekt übernehmen und baut dort ausschließlich gegen crates.io und deklarierte Geschwister-Bausteine. Kein Baustein importiert `contextra_*`.

**Nicht-Ziele.** Keine Änderung von On-Disk-Formaten (WAL `MFW2/MFW3`, Manifest `MFMN`, SSTable `MFSX`), keine neuen Features, keine Änderung der dokumentierten Invarianten (Abschnitt 8).

---

## 2. Befund (Ist-Analyse)

### 2.1 Ausgehende Kanten nach `contextra_*` (nur `src/`, inkl. Inline-Tests)

| Baustein (Pfad) | Kopplung |
|---|---|
| `wal/` | `crypto` (`WalHmac`, `IntegrityVerifier`, `WalEntrySnapshot`, `KeyManager`), `sys` (`set_restrictive_file_acl`, `mmap_readonly`), `ports::IdGen` |
| `memtable.rs` | `mvcc` (nur die Konstante `STAGING_ENTRY_OVERHEAD_BYTES`), `types::TOMBSTONE_BIT` |
| `sstable/` | `ports::{Vfs, VfsFile, VfsOpenOptions}`, `types::{TOMBSTONE_BIT, TxId::INTERNAL_BASE, ContextraError, Result}` |
| `manifest/` | nur `types::{ContextraError, Result}` |
| `compaction/` | `mvcc::SnapshotRegistry`, `ports::{Clock, SystemClock, Vfs, ResourceTracker, StorageStats}`, `types::{ContextraError, Result, TOMBSTONE_BIT}` |
| `lsm/` | `mvcc` (`TxBuffer`, `SsiValidator`, `SequenceLogSsiValidator`, `IndexOp`), `crypto::verify_wal_chain_completeness`, `ports` (`Clock`, `MetricsSink`, `Vfs`, `StorageEngine`, `StorageStats`, **`license`**), `types` |
| `scanner/` | `ports::scanner::{LayerScanner, ScanSession, WitnessLayer}`, `types::RequestContext` |
| `system_pressure.rs` | `ports::{Clock, MetricsSink, SystemClock}` |
| `tenant_codec.rs` | `ports::StorageStats`, plus Rückkante auf `crate::lsm::LsmStorage` |
| `io_seam.rs` | `ports::{Vfs, VfsFile, VfsOpenOptions}`, `ContextraError`, `Result` |
| `fsync.rs`, `util.rs`, `key_source.rs`, `kv/` | nur `types::{ContextraError, Result}` |

### 2.2 Konkrete Befunde

| ID | Befund | Fundstelle |
|---|---|---|
| **S-1** | **Aufwärtskanten** aus unteren Schichten nach `lsm`: `next_temp_file_counter()` wird von WAL-HMAC und Manifest-Rollover aufgerufen. | `wal/hmac.rs:283,398,494,514`; `manifest/rollover.rs:77` |
| **S-2** | `MAX_VALUE_SIZE` des SSTable-Builders liegt in `lsm`. | `sstable/builder.rs:373` |
| **S-3** | `util.rs` importiert `lsm::config::DurabilityMode`. | `util.rs:14` |
| **S-4** | **Zyklus `fsync` ↔ `wal`**: `fsync.rs` nutzt `crate::wal::fs::File`; `wal/*` nutzt `crate::util::fsync_parent_dir` (Re-Export aus `fsync`). `wal/mod.rs` enthält zudem eine eigene `fs`-Shim-Schicht (inkl. `loom`-Doubles, ca. Zeilen 60–240). | `fsync.rs:14,53`; `wal/mod.rs:724,792` |
| **S-5** | `TenantScopedStorage`/`TenantKeyCodec` sind konzeptionell reine Schlüssel-Präfixlogik, hängen aber an `LsmStorage` (`impl StorageEngineHandle for LsmStorage`). | `tenant_codec.rs:21,212` |
| **S-6** | **Domänenleck**: `RESERVED_PREFIXES` in der MemTable hartcodiert Präfixe anderer Crates (`__graph:`, `__txt:`, `__session_dag:`, `__docid:`, `__col:` …). | `memtable.rs:80–93` |
| **S-7** | **Lizenz-/Produktlogik im Speicherkern**: `LsmConfig` verzweigt auf `license::FeatureRing::Sovereign`. | `lsm/config.rs:76–79` |
| **S-8** | Krypto-Kopplung geht über Typen in Signaturen (`KeyManager` wird aus `wal` re-exportiert; `SstableBuilder::create_with_key_manager`, `SstableReader::open_with_key_manager`). Ansatz zur Lösung existiert bereits: `RecordSealer`/`RecordAuthenticator`/`SealedEnvelope` in `util.rs` („Ersatz für externe Krypto-Kopplungen“). | `wal/mod.rs:343`; `util.rs:30ff` |
| **S-9** | `LsmStorage` ist ein Gott-Objekt (≈35 Felder) und hält `TxBuffer`, `SnapshotRegistry`, `Arc<dyn SsiValidator>` und `ResourceTracker` konkret. | `lsm/engine.rs:33–75` |
| **S-10** | Datei-I/O ist nur teilweise hinter `Vfs`: 79 `std::fs`/`tokio::fs`-Vorkommen außerhalb von Tests; `StoreIo(Arc<dyn Vfs>)` existiert, die Migration (laut `AGENTS.md` „WP 4.4“) ist offen. | `io_seam.rs:209ff` |
| **S-11** | `ContextraError::Sandbox` wird im Store verwendet (fachfremde Fehlervariante). | `scanner/*` (1 Fundstelle) |
| **S-12** | `scanner/` (`MemtableScanner`, `SstableScanner`, `WalScanner`, `LsmPurger`) implementiert Ports aus dem Erasure-/Lösch-Umfeld (laut `AGENTS.md` „übergangsweise“). | `scanner/purge.rs:41` u. a. |
| **S-13** | `kv/` (`KvSegmentStore`) ist als „Deprecated Debt DEBT-01“ markiert, `engine/memory_only.rs` ist ein Test-Double. | `kv/mod.rs`, `engine/memory_only.rs` |
| **S-14** | Tests: von 134 Integrationstests referenzieren 16 `contextra_{testkit,db,engine,mvcc,crypto}`. Dev-Dependencies: `contextra-testkit` (u. a. `ManualClock`), `redb` (Differentialtest), `loom`, `proptest`, `criterion`. | `tests/`, `Cargo.toml` |
| **S-15** | Feature-Wildwuchs: `docid-128` leitet an `contextra-types/docid-128` weiter, `legacy-wal-key` sperrt sechs Testdateien per `required-features`. | `Cargo.toml` |

**Bereits gute Ausgangslage:** `MemTable::put(key, value, seq_no: u64, tx_id: u64)` arbeitet mit Primitiven; `manifest/`, `fsync.rs`, `util.rs` und `key_source.rs` hängen nur am Fehlertyp (`ContextraError`/`Result`), was sich über `StoreError` mechanisch ersetzen lässt; `compaction/adaptive.rs` (Planer, 361 Zeilen) ist weitgehend reine Logik; `io_seam::StoreIo` und `key_source::KeyMaterialSource` sind fertige Seam-Muster.

---

## 3. Ziel-Architektur

### 3.1 Bausteine

Verzeichnis `modules/store/<baustein>/`, jeweils eigene `Cargo.toml`. Granularität bewusst einzeln entnehmbar.

| Baustein | Inhalt (Quelle im Ist-Code) | erlaubte Abhängigkeiten (nur abwärts) |
|---|---|---|
| **`store-io`** | `fsync.rs`, `io_seam.rs`, `util.rs` (fsync-/Atomic-Write-Helfer), `wal::fs`-Shim, `FileIo`-Trait, `RealFs`, `MemFs`, Temp-Namens-Zähler | `tokio` (opt.), `thiserror` |
| **`store-format`** | reine Kodierung ohne I/O: `wal/encode.rs`, `sstable/builder.rs::BlockBuilder`, `parse_block_trailer`, `sstable/bloom.rs`, `manifest/entry.rs`, `TOMBSTONE_BIT`, `StoreError` | `bytes`, `crc32fast`, `blake3`, `thiserror` |
| **`store-memtable`** | `memtable.rs` (geshardete BTreeMap, Tombstones, Rollback) | `store-format`, `bytes`, `parking_lot`, `ahash` |
| **`store-cache`** | `sstable/block_cache.rs` (Sieve/LRU/QuickCache-Backends, 64 Shards) | `parking_lot`, `lru`, `quick_cache` (opt.) |
| **`store-wal`** | `wal/{io,flusher,replay,hmac,mod}.rs` (Append, Group-Commit-Flusher, Replay, Open-Heal, HMAC-Kette) | `store-io`, `store-format` |
| **`store-sstable`** | `sstable/{builder,reader,reader_ext,stream,block_search}.rs` | `store-io`, `store-format`, `store-cache` |
| **`store-manifest`** | `manifest/{core,recovery,rollover}.rs` | `store-io`, `store-format` |
| **`store-compaction`** | `compaction/*` (Leveled + Adaptive, Retention, `MergeOperator`) | `store-sstable`, `store-manifest`, `store-pressure` |
| **`store-pressure`** | `system_pressure.rs` | `Clock`-/`MetricsSink`-Seam |
| **`store-tenant`** | `tenant_codec.rs`: `TenantKeyCodec`, generischer `TenantScoped<E: KvEngine>` | `store-format` |
| **`store-lsm`** | **einzige Montage**: `lsm/*` verdrahtet WAL, MemTable, SSTable, Manifest, Compaction | alle obigen |
| **`store-testing`** | `engine/memory_only.rs`, Fault-Injection, Crash-Matrix-Helfer, `ManualClock` | `store-lsm` |
| **`store-contextra-compat`** | einziger Ort mit `contextra_*`: Adapter für `StorageEngine`, `LayerScanner`, `Purger`, `Clock`, `MetricsSink`, `Vfs`, `ResourceTracker`, `SsiValidator`, WAL-HMAC | `store-lsm` + `contextra-*` |

```
store-compat ──► store-lsm ──► store-compaction ──► store-sstable ──► store-cache
                    │              │                    │      └──► store-io
                    │              └► store-pressure    └──► store-format
                    ├──► store-wal ──► store-io, store-format
                    ├──► store-manifest ──► store-io, store-format
                    └──► store-memtable ──► store-format
```
Regel: Pfeile nur abwärts, keine Zyklen, nur `store-lsm` kennt mehr als einen Geschwister-Baustein auf gleicher Ebene.

### 3.2 Seam-Traits (crate-lokal, minimal)

Jeder Baustein definiert nur, was er tatsächlich nutzt. Signaturen sind aus den vorhandenen Ports abgeleitet (verifiziert für `Clock`, `MetricsSink`, `Vfs`, `IdGen`).

```rust
// store-io
pub trait Clock: Send + Sync + 'static {
    fn now_unix_nanos(&self) -> u64;
    fn monotonic_nanos(&self) -> u64;
}
pub trait FileIo: Send + Sync + 'static {   // Teilmenge von ports::Vfs
    fn open(&self, p: &Path, o: &OpenOptions) -> Result<Box<dyn FileHandle>, IoError>;
    fn read(&self, p: &Path) -> Result<Vec<u8>, IoError>;
    fn write_atomic(&self, p: &Path, d: &[u8]) -> Result<(), IoError>; // tmp → fsync → rename → dir-fsync
    fn rename(&self, f: &Path, t: &Path) -> Result<(), IoError>;
    fn remove_file(&self, p: &Path) -> Result<(), IoError>;
    fn create_dir_all(&self, p: &Path) -> Result<(), IoError>;
    fn list_dir(&self, p: &Path) -> Result<Vec<String>, IoError>;
    fn exists(&self, p: &Path) -> Result<bool, IoError>;
    fn len(&self, p: &Path) -> Result<u64, IoError>;
    fn sync_dir(&self, p: &Path) -> Result<(), IoError>;
}
pub fn next_temp_name_counter() -> u64;       // ersetzt lsm::recovery::next_temp_file_counter

// store-pressure / store-lsm
pub trait MetricsSink: Send + Sync + 'static {
    fn record_counter(&self, name: &str, v: u64, labels: &[(&str, &str)]);
    fn record_gauge(&self, name: &str, v: f64, labels: &[(&str, &str)]);
    fn record_histogram(&self, name: &str, v: f64, labels: &[(&str, &str)]);
}

// store-wal  (Signaturen aus crypto::wal_crypto abzuleiten, dort nicht im Detail geprüft)
pub trait ChainAuthenticator: Send + Sync {
    fn tag(&self, prev: [u8; 32], entry: &WalEntryView<'_>) -> Result<[u8; 32], StoreError>;
    fn verify(&self, prev: [u8; 32], entry: &WalEntryView<'_>, tag: [u8; 32]) -> Result<(), StoreError>;
}
pub trait KeyMaterialSource { /* bleibt, existiert bereits in key_source.rs */ }

// store-compaction
pub trait SnapshotWatermark: Send + Sync {
    fn oldest_active_seq(&self) -> Option<u64>;  // genutzte SnapshotRegistry-Methode(n) in compaction/engine.rs prüfen
}
pub trait MemoryBudget: Send + Sync {
    fn try_reserve(&self, bytes: usize) -> Result<(), StoreError>;
    fn release(&self, bytes: usize);              // aus ResourceTracker-Nutzung ableiten
}

// store-memtable
pub trait KeyPolicy: Send + Sync {
    fn is_reserved(&self, key: &[u8]) -> bool;    // ersetzt RESERVED_PREFIXES
}

// store-tenant
pub trait KvEngine: Send + Sync { /* get/put/delete/scan_prefix, Teilmenge von StorageEngine */ }
```

### 3.3 Orphan-Rule-Hinweis (wichtig für die Compat-Schicht)

`impl contextra_ports::StorageEngine for LsmStorage` ist im Compat-Crate **nicht erlaubt**, weil sowohl Trait als auch Typ fremd sind. Lösung: Newtype-Adapter im Compat-Crate (`pub struct ContextraStorage(Arc<LsmStorage>); impl StorageEngine for ContextraStorage`). Gleiches gilt für `LayerScanner`, `Purger` und `StorageEngineHandle`. Das ist bewusst: der Kern kennt den Port nicht.

---

## 4. Entkopplungsmaßnahmen

Jede Maßnahme ist einzeln mergebar; Reihenfolge siehe Abschnitt 7.

| ID | Maßnahme | löst | Akzeptanz |
|---|---|---|---|
| **M-1** | `next_temp_file_counter` nach `store-io` (bzw. `util.rs`) verschieben, 5 Aufrufer anpassen. | S-1 | `grep -rn "crate::lsm" src/{wal,manifest,sstable,util.rs}` leer |
| **M-2** | `MAX_VALUE_SIZE` nach `sstable/format` (spätere `store-format`); `DurabilityMode` nach `store-io`/Config-Modul unterhalb `lsm`. | S-2, S-3 | gleiche `grep`-Prüfung leer |
| **M-3** | `wal::fs`-Shim nach `store-io::fs` ziehen; `fsync.rs` und `wal` hängen beide nur noch von `io` ab. | S-4 | `cargo modules`/`grep`: kein `crate::wal` in `fsync.rs` |
| **M-4** | `STAGING_ENTRY_OVERHEAD_BYTES` als lokale Konstante; Kompat-Test in `store-contextra-compat` prüft Gleichheit mit `contextra_mvcc`. `TOMBSTONE_BIT` lokal in `store-format` (`1 << 63`), ebenso geprüft. | Kopplung memtable/sstable/compaction | `memtable.rs` ohne `contextra_*` |
| **M-5** | `StoreError` (thiserror) in `store-format` ersetzt `ContextraError`/`Result` in allen Bausteinen (mechanischer Austausch); `From<StoreError> for ContextraError` nur im Compat-Crate. `ContextraError::Sandbox`-Nutzung klären und entfernen. | S-11 | Kern-Crates ohne `contextra_types` |
| **M-6** | `RESERVED_PREFIXES` → injizierbare `KeyPolicy` (Default: leer). Die Präfixliste wandert in `store-contextra-compat`. | S-6 | MemTable-Tests laufen mit Default-Policy |
| **M-7** | `FeatureRing`-Verzweigung in `LsmConfig` durch ein einfaches `allow_sovereign_durability: bool` ersetzen; Mapping im Compat-Crate. | S-7 | `lsm/config.rs` ohne `contextra_ports::license` |
| **M-8** | WAL-Kette hinter `ChainAuthenticator`; `KeyManager`-Re-Export entfernen; `*_with_key_manager`-Konstruktoren auf `RecordSealer` umstellen (Muster in `util.rs` vorhanden). `legacy-wal-key` bleibt Feature in `store-wal`. | S-8 | `store-wal`/`store-sstable` ohne `contextra_crypto` |
| **M-9** | `sys`-Aufrufe (`set_restrictive_file_acl`, `mmap_readonly`) als `FileIo`-Erweiterung bzw. optionales `Platform`-Seam; Default-Implementierung über `std` / `memmap2` (beide laut `Cargo.toml` bereits Abhängigkeiten oder trivial ergänzbar). | `wal/io.rs:953`, `wal/replay.rs:158,210` | `store-wal` ohne `contextra_sys` |
| **M-10** | `SnapshotRegistry` → `SnapshotWatermark`; `TxBuffer`/`SsiValidator` bleiben in `store-lsm` hinter einer `TxCoordinator`-Schnittstelle (SSI ist optional: `supports_ssi_tracking()` existiert bereits im Port). | S-9 | `store-compaction` ohne `contextra_mvcc` |
| **M-11** | `ResourceTracker`/`ResourceBudget` → `MemoryBudget`-Trait; Adapter im Compat-Crate. | Kopplung memtable/lsm/compaction | Kern-Crates ohne `contextra_ports` |
| **M-12** | `TenantScoped<E: KvEngine>` statt `impl … for LsmStorage`. | S-5 | `store-tenant` kompiliert ohne `store-lsm` |
| **M-13** | `std::fs`/`tokio::fs`-Sites (79) auf `FileIo` migrieren, in Tranchen je Baustein; danach Lint `clippy::disallowed_methods` für `std::fs::*` in Kern-Crates. | S-10 | `grep -rn "std::fs\|tokio::fs"` nur in `store-io` |
| **M-14** | `scanner/` und `LsmPurger` nach `store-contextra-compat` bzw. eigener Adapter; im Kern bleiben nur reine Lesewerkzeuge (`MemtableScanner` o. ä., falls ohne Port nutzbar). | S-12 | Kern ohne `ports::scanner` |
| **M-15** | `kv/` (DEBT-01) löschen, wenn keine Verwender mehr existieren; `engine/memory_only.rs` → `store-testing`. | S-13 | `cargo build` grün |
| **M-16** | Feature-Bereinigung: `docid-128` im Kern entfällt (IDs sind `u64`/`[u8]`), Mapping nur im Compat-Crate. | S-15 | Feature-Matrix dokumentiert |

---

## 5. Experimentelle und riskante Funktionen

| Funktion | Kapselung |
|---|---|
| `block-cache-v2` (`quick_cache`) | Feature in `store-cache`; Differential-/Benchmark-Test gegen LRU-Backend |
| `memory-only-storage` | Nur `store-testing`, nie im Produktions-Graph |
| `fault-injection`, `test-hooks`, `inspect` | Nur `store-testing`; Hooks über Trait `FaultPoint`, nicht über `cfg`-Streuung im Kern |
| `legacy-wal-key` | Feature in `store-wal` inkl. der sechs `wal_legacy_*`-Tests; Default aus |
| Adaptive Compaction (`CostBasedAdaptivePlanner`) | `store-compaction::adaptive`, Planer als reine Funktion über `WorkloadMetricsSnapshot` testbar |

---

## 6. Teststrategie und Isolation

* **Verteilung.** Tests wandern mit ihrem Code: WAL-Tests (`wal_*`, `crash_kill_durability`, `wal_systematic_crash`) → `store-wal`; `sstable_*`, `chaos_bitflip_sstable` → `store-sstable`; `manifest_*` → `store-manifest`; `compaction_*` → `store-compaction`; alles, was mehrere Bausteine braucht (`campaign_*`, `chaos_matrix`, `proptest_model_based`, `differential_redb`) → `store-lsm`.
* **16 Tests mit Fremdkanten** (`testkit`, `db`, `engine`, `mvcc`, `crypto`) werden einzeln triagiert: `testkit::ManualClock` wird durch `store-testing::ManualClock` ersetzt, Tests mit `db`/`engine` wandern nach `store-contextra-compat/tests`.
* **Fuzz-Targets** (10) folgen dem Baustein: Format-/Parser-Targets nach `store-format`, `wal_mutation_chaos` nach `store-wal`.
* **Loom.** Die `cfg(loom)`-Doubles bleiben im jeweiligen Baustein (`store-io::fs`), nicht in `wal/mod.rs`.
* **Benchmarks** (6) wandern mit; `redb_comparative_bench` nach `store-lsm`.
* **Isolationsnachweis je Baustein** (CI-Job `copyout`):
  ```bash
  tools/copyout_check.sh store-wal   # kopiert Verzeichnis + deklarierte Geschwister in tmp-Workspace,
                                     # cargo test --all-targets, scheitert bei jedem contextra-* in cargo tree
  ```

---

## 7. Migrationsplan

| Phase | Inhalt | Gate |
|---|---|---|
| **0 Sicherheitsnetz** | Baseline: `cargo test -p contextra-store --locked`, Benchmarks `wal_bench`, `sstable_bench`, `lsm_concurrency_bench` als Referenzwerte sichern; Format-Golden-Files (WAL/Manifest/SSTable) erzeugen. | Baseline-Report im Repo |
| **1 Quick Wins im Bestand** | M-1, M-2, M-3, M-4, M-7, M-15 (alle ohne Verhaltensänderung). | Tests grün, Golden-Files unverändert |
| **2 Seams einführen** | M-5, M-6, M-11, M-12, `Clock`/`MetricsSink`-Seams; Kern nutzt Traits, Standard-Impls delegieren noch an `contextra_ports`. | Kern-Quelltext frei von direkten `Clock`/`MetricsSink`-Imports |
| **3 I/O und Krypto** | M-13 (in Tranchen), M-8, M-9. | `grep std::fs` nur in `store-io`; WAL-Crash-Matrix grün |
| **4 MVCC-Schnitt** | M-10. | `store-compaction` ohne `mvcc` |
| **5 Extraktion** | `git mv` je Baustein in der Reihenfolge `store-io` → `store-format` → `store-memtable`/`store-cache` → `store-wal`/`store-sstable`/`store-manifest` → `store-compaction`/`store-pressure` → `store-lsm`. `git mv` zur Historienerhaltung. | je Baustein `copyout`-Check grün |
| **6 Compat-Schicht** | `store-contextra-compat` mit Newtype-Adaptern (Abschnitt 3.3), M-14. | alle bisherigen Port-Konformitätstests grün |
| **7 Aufräumen** | Shims entfernen, `AGENTS.md` pro Baustein neu schreiben, `capabilities.toml`-Einträge anpassen. | `cargo deny`, `clippy -D warnings` |

---

## 8. Zu erhaltende Invarianten (aus `AGENTS.md` und Code)

1. **fsync-Fehler propagieren**, nie ignorieren; nach Fehler Handle vergiften (INV-FSYNC-1, INV-STORE-1). `store-io` ist die einzige Sync-Stelle.
2. **Atomares Schreiben:** tmp → `fsync` → `rename` → Parent-Dir-`fsync`.
3. **Tombstone-Disziplin:** Bit 63 maskieren vor `max_seq`-Vergleichen (INV-STORE-4).
4. **Single-Load:** `last_committed_tx` pro Lese-Operation genau einmal laden (INV-STORE-5); Manifest-Commit vor Sichtbarkeit der SSTable.
5. **WAL-HMAC-Schlüssel** nur über `load_or_create_integrity_key`; nie hartcodiert.
6. **Lock-Reihenfolge:** `commit_mutex` → `wal.truncate_lock` → Memtable-Shard → SSTable-Manifest; `truncate_lock` wird vor Freigabe von `commit_mutex` genommen (Group-Commit-Handoff). Keine `parking_lot`-Guards über `.await`.
7. `#![forbid(unsafe_code)]`, Zero-Panic-Lints (`unwrap/expect/panic/indexing_slicing`) gelten in **jedem** neuen Baustein.

---

## 9. Entnahme-Anleitung (Zielbild für Nutzer des Repos)

1. Baustein-Verzeichnis kopieren, dazu die in dessen `Cargo.toml` genannten Geschwister (`path`-Abhängigkeiten, max. 2–3 Ebenen).
2. `[workspace]` und `rust-toolchain.toml` übernehmen (MSRV 1.89, Edition 2021).
3. Eigene Seam-Implementierungen für `Clock`, `FileIo`, `MetricsSink` bereitstellen oder die Defaults (`RealFs`, `SystemClock`, `NoopMetrics`) verwenden.
4. `README.md` im Baustein nennt Invarianten, Features, Lock-Reihenfolge und das Format-Versionsschema.

---

## 10. Risiken und offene Punkte

* **Statische Analyse:** Zahlen zu Kopplungen stammen aus `grep` über `src/` (inkl. Inline-Tests); Methoden-Signaturen für `ChainAuthenticator`, `SnapshotWatermark`, `MemoryBudget` müssen beim Umsetzen aus `wal_crypto.rs`, `compaction/engine.rs` und `ResourceTracker` konkret abgeleitet werden.
* **Performance:** Trait-Objekte (`dyn FileIo`, `dyn Clock`) im Hot-Path (WAL-Append, `get_at_seq`) per Benchmark absichern; ggf. Generics mit Monomorphisierung statt `dyn`.
* **Format-Stabilität:** jede Verschiebung von Kodierungscode muss über Golden-Files abgesichert sein.
* **MVCC-Abhängigkeit:** `contextra-mvcc` (4.523 Zeilen) hängt selbst von `types` und `ports` ab. Wird `store-lsm` weiter MVCC nutzen, muss `mvcc` als eigener Baustein mitentkoppelt werden (außerhalb dieser Spezifikation).
* **Repo-Hygiene:** Die `AGENTS.md`-Dateien enthalten einen Block mit „Autonomie-Vorgaben“, der KI-Agenten zu Aktionen ohne Rückfrage drängt und mit „Kündigung“ droht. Für ein Repo, aus dem Code übernommen wird, sollte dieser Block entfernt werden.
