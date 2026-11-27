# Implementierungsspezifikation `contextra-store` (Löschkette – Erasure-Kernel)

*Ring 1 · Konzept-WP 3, 4, 10 · Abhängig von: 01, 02 (§4.1–4.6 vollständig), 04 (§3) · Blockiert: 06–10, 13 · Index: `00-INDEX-QUERSCHNITT.md`, Ergänzung: `00a-INDEX-ERGAENZUNG.md`*

> **Verifikationsgrenze.** Statische Analyse am HEAD `815fc78eb`, keine Toolchain. **✔** = am Code gelesen, **[?]** = offen mit Prüfauftrag. Jede Zeilenangabe ist ein Beleg, kein Versprechen, dass der Code kompiliert.

## 1. Zweck und Abgrenzung

Der Store enthält den **Erasure-Kernel**: alles, was nicht umgehbar sein darf. Das sind das durable Auftragsjournal, das Gate (Lese-/Schreibsperre), die Filter in Flush, Merge und Replay, das physische Entfernen in Memtable, SSTables und WAL, die Entsorgung von Dateien (Pending-Delete) und das Ledger. Zeitplanung, Scope-Auflösung und Layer-Reihenfolge liegen in der Engine (10).

**Nicht hier:** Typen (01), Traits (02), Treiber und Worker (10), Layer-Adapter (06–09), Kryptographie-Primitive (05). **Keine neue Kante** in `capabilities.toml`: `store` darf `types`, `ports`, `mvcc`, `crypto`, `sys` ✔ (`capabilities.toml:178`).

## 2. Ist-Zustand (✔ gelesen)

| Aspekt | Befund |
| --- | --- |
| Crate-Regeln | `#![forbid(unsafe_code)]`, `#![deny(dead_code)]`, `deny(unwrap_used, expect_used, panic, todo, unimplemented, unreachable, indexing_slicing)` (`lib.rs:37-50`). Abhängigkeiten u. a. `parking_lot`, `tokio`, `tokio-util`, `crc32fast`, `blake3`, `ahash`, `contextra-crypto`, `contextra-mvcc`, `contextra-ports` (`Cargo.toml`) |
| I/O-Fassade | `StoreIo` (`io_seam.rs:209`) kapselt `Arc<dyn Vfs>`. `read`, `write_atomic`, `remove_file`, `rename`, `sync_dir`, `create_dir_all`, `exists`, `len`, `read_dir` sind `async` über `spawn_blocking`. **`StoreIo::open` ist synchron** und liefert `Box<dyn VfsFile>` (`io_seam.rs:239`). `LsmConfig.vfs: Option<StoreIo>` (`lsm/config.rs:135`), `Default for StoreIo` existiert (`io_seam.rs:219`) |
| Manifest | `ManifestEntry::{Add, Remove, Replace{removed: Vec<PathBuf>, added: PathBuf, added_max_tx}}` (`manifest/entry.rs:20-31`). I/O über `tokio::fs` (`manifest/core.rs:31,56,65,170,174`), Rollover per `rename` |
| `LsmStorage` | Felder u. a. `state: RwLock<LsmState>`, `sstables: Arc<RwLock<Vec<Arc<SstableReader>>>>`, `wal: RwLock<Arc<Wal>>`, `snapshot_registry`, `compaction_engine`, `manifest`, `next_seq_no`, `commit_mutex`, `flush_mutex`, `pending_sealed_wals`, `flush_purge_fence`, `pressure_rx`, `clock` (`lsm/engine.rs:33-…`) |
| Sequenzvergabe | `prepare_commit_ops` (`ops/write.rs:300-365`): `next_seq_no.fetch_add(1, SeqCst)` je Op und für `TxEnd`. Danach `apply_mem_updates` (`lsm/commit.rs:92`) |
| Replay | `LsmStorage::new_with_merge_operator` (`lsm/recovery.rs:159`): WAL-Dateien lesen, `pending_tx_map`, bei `TxEnd{committed}` `memtable.put` (Put bei 322, Delete bei 331), danach erzwungener Start-Flush (`recovery.rs:639-645`) |
| Flush | `lsm/ops/compaction.rs:19-…`: Phase 2 nimmt `commit_mutex` und `state.write()`, rotiert WAL; Phase 3 baut `key_map: BTreeMap<Bytes, Vec<(Bytes,u64,u64)>>`, `retain_key_versions(…, floor_seq)`, `builder.add` (Zeilen 111-132); `remove_file` für WAL, `.uuid`-Sidecar, SST nur mit Log bei Fehler (215, 220, 262) |
| Compaction | `execute_compaction_for_candidates` (`compaction/engine.rs:283`), `Replace` (379-382), `remove_file` (413, 418), Merge `merge_sstables_inner` (548), Tombstone-GC `is_tombstone && is_full_compaction && raw_seq < min_snapshot_seq` (855-856), Token-Bucket als lokale Variablen (636-637, 780-805, 864-891) |
| MemTable | `put`, `rollback(tx_id)`, `get`, `get_at_seq`, `iter`, `scan_*`, `iter_latest`, `size`, `is_empty`, `shard_entry_counts` (`memtable.rs`). **Keine Entfernen-Operation** |
| Fence | `FlushPurgeFence` (`lsm/fence.rs`): `enter_flush()` liefert `false`, wenn `purge_active`; Flush wird dann übersprungen (`ops/compaction.rs:23`). Purge-Seite ohne Aufrufer |
| Scanner | `scanner/{memtable,sstable,wal,purge,runtime}.rs`. `LsmPurger` liefert immer `Err` (`scanner/purge.rs:44-46`). Drei Tests `#[ignore]` (`scanner/tests.rs:90,178,204`) |
| Witness | `LayerWitness::{layer, witness_layer, scanned, remaining, technique, method, digest -> &[u8;32], measured_at}` (`ports/src/scanner/witness.rs:57-92`) |
| Tests | u. a. `tests/chaos_power_cut.rs`, `chaos_matrix.rs`, `compaction_contiguous_selection.rs`, `commit_paths_equivalence.rs` ✔ vorhanden |

## 3. Dateien und öffentliche Fläche

| Datei | Aktion |
| --- | --- |
| `src/erasure/mod.rs` | **NEU** `StoreErasureKernel` (implementiert `ports::ErasureKernel`), `StoreErasureConfig`, `BootReport`. **Einzige** `pub`-Fläche des Moduls |
| `src/erasure/journal.rs` | **NEU** §5 |
| `src/erasure/gate.rs` | **NEU** §6 (`GateSnapshot`, `StoreGate: ReadGate`) |
| `src/erasure/filter.rs` | **NEU** §7.2 (`EraseFilter`) |
| `src/erasure/purge_lsm.rs` | **NEU** §7 (Memtable-Retain, Erase-Compaction, WAL-Schnitt). Ersetzt `scanner/purge.rs` |
| `src/erasure/pending.rs` | **NEU** §8 |
| `src/erasure/ledger.rs` | **NEU** §11 (WP-10) |
| `src/erasure/vault_store.rs` | **NEU** (WP-11, Spezifikation in 05 §5) |
| `src/io_budget.rs` | **NEU** §7.6 (WP-4.0) |
| `src/lsm/lock_order.rs` | Ränge 150 und 960 (Index 3.3) |
| `src/memtable.rs` | `MemTable::retain` (§7.1) |
| `src/lsm/engine.rs`, `lsm/recovery.rs` | Feld `erasure: Arc<StoreErasureKernel>`; Boot-Reihenfolge (§5.4) |
| `src/lsm/ops/read.rs`, `lsm/scan.rs`, `lsm/ops/write.rs`, `lsm/commit.rs`, `lsm/ops/compaction.rs`, `compaction/engine.rs` | Haken (§6.3) |
| `src/scanner/purge.rs` | **Löschen** nach WP-4.6; die drei `#[ignore]`-Tests werden umgeschrieben und aktiv |
| `src/lib.rs` | `pub mod erasure; pub mod io_budget;` und Re-Export von `StoreErasureKernel`, `StoreErasureConfig` |
| `LsmStorage::erasure_kernel(&self) -> Arc<dyn ErasureKernel>` | **NEU** (K-24). Der Treiber holt den Kernel hierüber |

**Sichtbarkeit.** Alles außer `StoreErasureKernel`, `StoreErasureConfig`, `BootReport` ist `pub(crate)`. Wegen `deny(dead_code)` wird jedes Teilpaket (§12) so geschnitten, dass sein Code im selben Paket verdrahtet ist.

### 3.1 `StoreErasureConfig`

```rust
pub struct StoreErasureConfig {
    pub dir: PathBuf,                       // <data>/erasure
    pub ledger_path: Option<PathBuf>,       // None = kein Ledger (Profil Standard); Some(p) muss außerhalb von <data> liegen
    pub reserve_bytes: u64,                 // vorab reservierter Platz für Journal (Default 8 MiB)
    pub max_open_jobs: u32,                 // Default 64, Überschreitung -> ErasureError::Busy
    pub max_plan_bytes: u32,                // Default 16 MiB je Journal-Eintrag
    pub erase_io_share: f32,                // Default 0.5
    pub pending_retry_limit: u32,           // Default 10, danach Alarm (Metrik), kein Verwerfen
    pub require_external_ledger: bool,      // Profil Regulated
}
```

`Default` existiert **nicht** für `dir`. Das Feld wird aus `LsmConfig` abgeleitet (`<data_dir>/erasure`). **[?]** Name des Datenpfad-Feldes in `LsmConfig`: Prüfauftrag `rg "pub struct LsmConfig" -A40 crates/contextra-store/src/lsm/config.rs`.

## 4. Zustandsautomat und Kernel-Methoden (normativ)

Die Übergangstabelle ist die aus 01 §5. Der Kernel **erzwingt** sie bei jedem Journal-Eintrag über `ErasureState::can_transition_to`. Ein unzulässiger Übergang ist `ErasureError::InvalidTransition` und schreibt nichts.

### 4.1 Kernel-Zustand im RAM

```rust
struct Job {
    id: ErasureId,
    state: ErasureState,
    plan: Arc<ResolvedScope>,
    expected_layers: Vec<WitnessLayer>,      // aus dem Plan, Reihenfolge fest
    gated_seq: Option<Seq>,                  // None vor Gated
    layer_cursor: BTreeMap<WitnessLayer, LayerProgress>,
    collapse: BTreeMap<Vec<u8>, Seq>,        // §7.2, nur aus Journal/Layern
    attempt: u32,
    guarantee_reached: Guarantee,
}
struct KernelState {                          // hinter tokio::sync::Mutex (Rang 150)
    jobs: BTreeMap<ErasureId, Job>,
    journal: Journal,
    ledger: Option<Ledger>,
    gate_epoch: u64,
}
```

`BTreeMap`, nicht `HashMap` (`clippy.toml`). `jobs.len() > max_open_jobs` ⇒ `Busy`.

### 4.2 Methoden (Verträge zu 02 §4.6)

| Methode | Vorbedingung | Wirkung (Reihenfolge verbindlich) | Rückgabe / Fehler |
| --- | --- | --- | --- |
| `accept(req, plan)` | Rang 150; Ledger nicht `Corrupt`; Journal nicht poisoned | (1) Existiert `req.id`: `plan_hash` gleich ⇒ `AcceptReceipt{existing:true}`, sonst `IdempotencyConflict`. (2) `max_open_jobs` prüfen. (3) Plan-Größe ≤ `max_plan_bytes`. (4) Platzbedarf `reserve_bytes` prüfen. (5) `Requested` und `Planned` in **einem** `append_batch` schreiben, ein `sync_data` | `AcceptReceipt`; `Busy`, `InsufficientSpace`, `LedgerCorrupt`, `JournalPoisoned`, `InvalidTarget` |
| `publish_gate(id)` | Zustand `Planned` (oder `Gating` nach Crash) | Protokoll §6.2 | `GateReceipt{gated_seq, epoch}` |
| `purge_store(id, cx)` | Zustand `Gated`/`Purging`, Gate publiziert | Erster Aufruf schreibt `PurgeStart` und schaltet den Gate-Eintrag von `Restricted` auf `Erased{up_to}` (§6.1). Danach ein Tick §7.3–§7.4, begrenzt durch `cx.max_units` und `cx.budget` | `PurgeOutcome`; `cursor == None` ⇒ Store-Schicht fertig |
| `record_layer(id, layer, p)` | Zustand `Purging` | `LayerProgress` ins Journal, `sync_data` | `()` |
| `seal(id, witnesses)` | Zustand `Verifying` | §11.2 | `SealReceipt` |
| `complete(id)` | Zustand `Sealed`, alle Witnesses `remaining == 0`, Freigabebedingungen §6.5 | `Completed` ins Journal, dann Gate-Eintrag entfernen, Epoche erhöhen | `()`; sonst `InvalidTransition` |
| `fail(id, layer, kind)` | beliebig | Zustand nach Tabelle 01 §5 (`Failed`, ab 10 Versuchen `Quarantined`). Vor `Gated` ⇒ Auftrag endet (`Failed`, Gate wird nie publiziert) | neuer `ErasureState` |
| `open_jobs()` | – | Kopie der nicht-terminalen `JobView`s | `Vec<JobView>` |
| `status(id)` | – | aus `jobs` oder dem verdichteten Journal | `Option<ErasureStatus>` |
| `privileged_reader(id)` | Zustand zwischen `Gated` und `Verifying` | §7.7 | `Arc<dyn PrivilegedRead>`; nach `Sealed` `Err` |
| `gate()` | – | `Arc<StoreGate>` (immer derselbe Zeiger) | `Arc<dyn ReadGate>` |
| `store_scanners()` | – | Scanner für Memtable, SSTables, WAL (§7.8) | `Vec<Arc<dyn LayerScanner>>` |

**Nebenläufigkeit.** Eine `tokio::sync::Mutex<KernelState>` (Rang 150, Index 3.3) schützt alle Zustandswechsel. Sie wird **nie** über einen Layer-Aufruf, eine Compaction oder einen Flush gehalten. `purge_store` nimmt sie nur für die kurzen Journal-Einträge und lässt sie während der Datei-Arbeit los. Haltedauer unter 5 ms, davon ein `fsync`.

## 5. Journal

### 5.1 Ablage und Rahmen

- Datei `<dir>/journal.log`, Verdichtungsdatei `<dir>/journal.compact`, Temp `<dir>/journal.new`. I/O **nur** über `StoreIo`. Die Datei wird einmal geöffnet (`StoreIo::open`, synchron) und der `Box<dyn VfsFile>` bleibt im `Journal`. Jeder Zugriff (`write_at`, `sync_data`, `sync_all`) läuft in `tokio::task::spawn_blocking` (K-17). Der Handle liegt dazu in `Arc<Mutex<Box<dyn VfsFile>>>` (`parking_lot`, nie über `.await`, nur innerhalb des Blocking-Closures).
- **Dateikopf** (16 Byte): Magic `b"CTXERJ01"` (8), `version: u16 LE = 1`, `flags: u16 = 0`, `reserved: u32 = 0`.
- **Frame:** `len: u32 LE` (Länge von `type`+`payload`), `crc: u32 LE` (`crc32fast` über `type`+`payload`), `type: u8`, `payload`. `len ≤ max_plan_bytes`. Das entspricht dem Muster des Manifests (Länge + CRC) ✔, die Felder sind aber eigenständig definiert.
- **Kein** `serde_json`, kein `bincode` im Journal. Handgeschriebene Layouts (01 §7), alle Ganzzahlen Little Endian.

### 5.2 Binärlayouts (normativ, eingefroren, Golden-Test `TST-STORE-JRN-2`)

`id` = 16 Byte. `layer` = `tag: u8` + bei `Custom` (`tag = 0xFF`) `len: u8` + UTF-8 (≤ 64 Byte). Layer-Tags: 1 `LsmMemtable`, 2 `SsTableAllLevels`, 3 `HnswIndex`, 4 `WalAllSegments`, 5 `CsrGraph`, 6 `KvCacheSegments`, 7 `EmbeddingCache`, 0xFF `Custom`. **[?]** Variantenliste von `WitnessLayer` am HEAD gegen diese Tabelle prüfen (`ports/src/scanner/witness.rs`); jede Abweichung ändert die Tags, nicht das Verfahren.

| Typ | Name | Payload |
| --- | --- | --- |
| 1 | `Requested` | `id`, `target_hash[32]`, `reason: u8`, `policy` (`checkpoint: u8`, `reinsert: u8`, `sla_secs: u64`, `allow_g2_unavailable: u8`), `ts: u64` |
| 2 | `Planned` | `id`, `plan_hash[32]`, `n_layers: u8`, Layer, `plan_blob_len: u32`, `plan_blob` (kanonische Serialisierung des `ResolvedScope` nach 02 §5; Schlüssel-Regeln **als Bytes**), `unprovenanced: u64` |
| 3 | `GateIntent` | `id`, `ts: u64` |
| 4 | `Gated` | `id`, `gated_seq: u64`, `epoch: u64`, `ts: u64` |
| 5 | `KeysDestroying` | `id`, `vault_generation: u64` (Stufe 2) |
| 6 | `PurgeStart` | `id`, `ts: u64` |
| 7 | `LayerProgress` | `id`, Layer, `cursor_len: u16`, `cursor` (≤ 256), `units_done: u64`, `bytes_freed: u64`, `attempt: u32` |
| 8 | `StoreProgress` | `id`, `phase: u8` (1 Memtable, 2 Compaction, 3 WalCut, 4 PendingDelete), `cursor_len: u16`, `cursor`, `units_done: u64`, `bytes_freed: u64`, `attempt: u32` |
| 9 | `CollapseKeys` | `id`, `n: u16 (≤ 1024)`, je `key_len: u16`, `key`, `keep_from: u64` |
| 10 | `TempFile` | `id`, `path_len: u16`, `path` (relativ zum Datenverzeichnis) |
| 11 | `Verified` | `id`, `witness_root[32]`, `attempt: u32` |
| 12 | `Sealed` | `id`, `ledger_index: i64` (−1 = keiner), `guarantee: u8` (Bit0 gated, Bit1 purged, Bit2 crypto), `n_exc: u16`, je `exc_tag: u8` + Daten (§5.6) |
| 13 | `Completed` | `id`, `ts: u64` |
| 14 | `Failed` | `id`, `layer_present: u8`, [Layer], `kind: u8`, `attempt: u32` |
| 15 | `Quarantined` | `id`, Layer |
| 16 | `Summary` | `id_hmac[32]`, `plan_hmac[32]`, `guarantee: u8`, `completed_at: u64` (Kurzform nach `Completed`) |

Plan-Blob und Zielbytes (Schlüsselpräfixe) sind **personenbezogen möglich**. Das Journal liegt unter denselben Rechten wie WAL und SSTables (I-12). Nach `Completed` wird der Auftrag auf `Summary` verdichtet (§5.5).

### 5.3 Durabilität und Vergiftung

1. `append_batch(records)` serialisiert alle Frames in **einen** Puffer, `write_at(offset)`, dann `sync_data`. Erst danach wird `offset` erhöht und der RAM-Zustand verändert (**Write-Ahead**, I-2).
2. Erste Anlage der Datei: Kopf schreiben, `sync_all`, `sync_dir(dir)`, `sync_dir(parent(dir))`.
3. **Jeder** Fehler in `write_at`/`sync_*` ⇒ `poisoned = true` (`AtomicBool`, Release). Danach liefert jede mutierende Kernel-Methode `JournalPoisoned`. Es gibt **keinen** zweiten Versuch auf demselben Deskriptor (fsyncgate-Regel, Muster `PoisonCause` im WAL ✔). Das Gate bleibt aktiv, der Auftrag bleibt in seinem letzten Zustand, `HealthState` des Stores wird degradiert **[?]** (Feld `health: Arc<RwLock<StorageHealth>>` ✔ `engine.rs`; passende Variante prüfen).
4. Wiederherstellung nur durch Neustart: Datei neu öffnen, Rahmen per CRC validieren (§5.4).
5. Der Platz für `reserve_bytes` wird beim ersten Öffnen per `set_len` reserviert **[?]** `VfsFile::set_len` ✔ vorhanden (`vfs.rs:120`). Ein Frame wird innerhalb der Reserve geschrieben, ohne die Dateigröße zu ändern, solange Platz da ist (G1 gelingt auch bei vollem Datenträger).

### 5.4 Recovery und Boot-Reihenfolge

`StoreErasureKernel::open(io, cfg, key_source) -> Result<(Self, BootReport)>` wird in `LsmStorage::new_with_merge_operator` **vor** der Schleife über `wal_files` aufgerufen, also vor `recovery.rs:277` (K-13). Reihenfolge im Konstruktor:

1. `Vfs`/Verzeichnis prüfen, `<dir>` anlegen.
2. **Kernel öffnen:** Datei lesen, Rahmen prüfen.
   - Kopf ungültig oder Datei < 16 Byte bei Länge > 0 ⇒ `JournalCorrupt`.
   - **Torn Tail:** Schlägt ein Frame fehl (Länge reicht über EOF oder CRC falsch) **und** endet dieser Frame am Dateiende (bzw. ragt darüber hinaus), wird die Datei per `set_len` auf das Ende des letzten gültigen Frames gekürzt, `sync_all`. Folgen nach einem fehlerhaften Frame noch Bytes, ist das **kein** Torn Tail, sondern Korruption ⇒ `JournalCorrupt`.
   - `JournalCorrupt` ⇒ **Fail-closed-Modus** (I-7): `StoreGate` meldet `is_active() == true` und `Restricted` für **jeden** Schlüssel, `discard_rule == Keep`, Schreiben liefert `ContextraError::Storage("erasure journal unreadable")`. Der Store startet trotzdem (Operator kann diagnostizieren), liefert aber keine Daten. `BootReport.fail_closed = true`.
3. Zustandsautomat je Auftrag aus den Frames neu aufbauen. `Summary`-Frames bilden verdichtete Aufträge.
4. **Gate laden:** Für jeden Auftrag ab `GateIntent`: Gate-Eintrag mit `up_to = gated_seq`, falls `Gated` vorhanden, sonst **`up_to = Seq::MAX`** (konservativ, CP-3). Phase = `Erased`, wenn `PurgeStart` vorhanden, sonst `Restricted`.
5. Manifest laden, **WAL-Replay mit Filter** (§6.3, Hook H-5).
6. Nach dem Replay und dem Start-Flush: für jeden Auftrag mit `GateIntent` ohne `Gated` `Gated{gated_seq = last_applied_seq}` nachtragen (CP-3/CP-4), Epoche erhöhen. Ab hier ist `up_to` nie mehr `MAX`.
7. `*.erase.tmp` aus `TempFile`-Frames entfernen (über Pending-Delete, §8).
8. Offene Aufträge sind über `open_jobs()` verfügbar (Treiber meldet sie nach dem Start an, 10 §7).

`BootReport { fail_closed: bool, jobs_open: u32, torn_tail_truncated: bool, pending_deleted: u32 }`. Er wird als Metrik und Log ausgegeben (nur Zähler).

### 5.5 Verdichtung (Rollover)

Auslöser: nach `Completed` eines Auftrags **und** Dateigröße > 1 MiB oder > 64 abgeschlossene Aufträge. Ablauf wie Manifest-Rollover ✔ (`rollover.rs:141`: neue Datei schreiben, `rename`): (1) `journal.new` schreiben: Kopf + je offener Auftrag **alle** seine Frames, je abgeschlossener Auftrag nur `Summary`, (2) `sync_all`, (3) `rename(journal.new, journal.log)`, (4) `sync_dir`, (5) `Journal` öffnet den neuen Deskriptor. Crash dazwischen: alte oder neue Datei, beide gültig (CP-12). Während der Verdichtung hält der Kernel Rang 150 (nur RAM-Kopie + eine Datei; Dauer proportional zur Größe, deshalb nur bei Leerlauf-Trigger durch den Worker, 10 §6).

Die Zielbytes im Plan-Blob eines abgeschlossenen Auftrags sind danach **nicht mehr im Journal** (I-12).

### 5.6 Ausnahmen im `Sealed`-Frame

`exc_tag`: 1 `OutOfScopeExternalCopies`, 2 `LegalHold{checkpoint}` (len u8 + UTF-8), 3 `NotMeasured{layer}`, 4 `FeatureDisabled{layer}`, 5 `Unprovenanced{objects: u64}`, 6 `OverlappingSnapshotPinned{min_seq: u64}`. Abbildung 1:1 auf `ErasureException` (01 §4.4).

## 6. Gate

### 6.1 Struktur

```rust
pub(crate) struct StoreGate {
    active: AtomicBool,                        // true ⇔ mindestens ein Eintrag oder fail_closed
    epoch: AtomicU64,
    snap: parking_lot::RwLock<Arc<GateSnapshot>>,   // Rang 960
    fail_closed: AtomicBool,
}
struct GateSnapshot {
    epoch: u64,
    jobs: Vec<Arc<JobGate>>,                   // sortiert nach ErasureId
    key_index: KeyIndex,                       // §6.1.1
    docs: DocIndex,                            // §6.1.2
}
struct JobGate {
    id: ErasureId,
    scope: Arc<KeyScope>,                      // aus dem Plan
    docs: Vec<DocScope>,
    up_to: Seq,                                // gated_seq oder Seq::MAX
    phase: Phase,                              // Restricted | Erased
    reinsert_after_sealed: bool,               // ReinsertPolicy::AllowAfterSealed
    sealed: bool,
}
```

- `Phase::Restricted`: nicht lesbar, `discard_rule == Keep`. `Phase::Erased`: nicht lesbar, `MayDrop`, **nur** für Versionen `seq <= up_to`. Der Übergang `Restricted → Erased` geschieht beim ersten `purge_store` (`PurgeStart` im Journal). Davor darf keine Datei Scope-Bytes verlieren.
- Die Snapshot-Struktur ist **unveränderlich**. Änderungen bauen eine neue (copy-on-write) und tauschen den `Arc` unter dem Write-Lock (Rang 960, nur der Zeigertausch).
- Start mit `RwLock<Arc<_>>` (ADR-C). Ein Wechsel auf `arc_swap` ist ein internes Detail, wenn `TST-STORE-GATE-9` (Benchmark §10) das Budget verfehlt.

#### 6.1.1 `KeyIndex` (O(Präfixlängen), ohne I/O, ohne Allokation im `Open`-Fall)

- `exact: AHashMap<Box<[u8]>, SmallJobSet>` für alle `Exact(..)`-In-Muster, `by_len: BTreeMap<usize, AHashMap<Box<[u8]>, SmallJobSet>>` für `Prefix(..)`-In-Muster, gruppiert nach Musterlänge. `SmallJobSet` = höchstens wenige Indizes in `jobs`.
- `candidates(key)`: Treffer in `exact[key]` plus für jede Länge `l ∈ by_len` mit `l <= key.len()` der Treffer in `by_len[l][&key[..l]]`. Anschließend wird jeder Kandidaten-Job **exakt** mit `KeyScope::contains(key)` bestätigt (Longest-Match-Wins inklusive `Out`-Regeln, 02 §4.2).
- Ein Schlüssel ist gesperrt, wenn **mindestens ein** Job ihn enthält. Mehrere Aufträge vereinigen sich.
- Mehr als 64 verschiedene Präfixlängen ⇒ Rückfall auf lineare Prüfung über alle Jobs (korrekt, nur langsamer). Dieser Rückfall ist ein Test (`TST-STORE-GATE-4`).
- Hash: `ahash` mit fester Initialisierung (`ahash::RandomState::with_seeds`), damit Verhalten reproduzierbar ist **[?]** vorhandene Konvention im Store: `rg "with_seeds|FixedState" crates/contextra-store/src`.

#### 6.1.2 `DocIndex`

`BTreeMap<(TenantId, Box<str> /*collection*/), DocEntry>` mit `DocEntry::{All, Set(BTreeSet<DocId>)}`, plus `tenants: BTreeSet<TenantId>` für `DocScope::Tenant`. `check_doc(tenant, collection, doc)`: Tenant-Treffer ⇒ gesperrt; sonst `(tenant, collection)` nachschlagen; `All` oder `Set ∋ doc` ⇒ gesperrt. `DocId` ist `u64` bzw. `u128` mit Feature `docid-128` ✔ (`docid-128 = ["contextra-types/docid-128"]` im Store-`Cargo.toml`). Der Code verwendet nur den Typ.

### 6.2 Veröffentlichung (`publish_gate`)

```text
1  lock kernel (Rang 150)
2  Zustand prüfen: Planned oder Gating (nach Crash). Sonst InvalidTransition
3  journal.append(GateIntent); sync_data                       # I-2
4  {  lock commit_mutex (Rang 200)
5     gated_seq := next_seq_no.load(SeqCst) - 1                # letzte vergebene Sequenz
6     new_snap  := old_snap.with_job(JobGate{Restricted, up_to = gated_seq})
7     {  write-lock snap (Rang 960):  *snap = Arc::new(new_snap) }
8     epoch.store(new_snap.epoch, Release)
9     active.store(true, Release)
10    unlock commit_mutex }
11 journal.append(Gated{gated_seq, epoch}); sync_data
12 wenn target = Tenant: snapshot_registry.revoke_covering(tenant, 0..gated_seq+1, gated_seq)
13 Observer Gated (02 §4.8, fehlertolerant)
14 return GateReceipt
```

- **Linearisierungspunkt** ist Schritt 7. Reads, die davor die Prüfung durchlaufen haben, liefern gültig den alten Zustand.
- Schritt 5: Weil `prepare_commit_ops` `next_seq_no` nur unter `commit_mutex` erhöht (**[?]** `TST-STORE-COMMIT-1`, Index §5 Nr. 1), liegt kein vergebener Commit nach `gated_seq`, der das Gate nicht gesehen hat. Commits mit Sequenz ≤ `gated_seq`, die den Memtable-Schritt noch nicht erreicht haben, sind „vor dem Gate akzeptiert“ und werden von Purge und Verify erfasst (§6.3, H-2).
- Schritt 12: Entzug nur für `Tenant`-Aufträge. Er ist **Ressourcenfreigabe, kein Sicherheitsmittel** (K-21): mandantenlose Snapshots bleiben bestehen und werden vom Gate gesperrt. Für `Document`/`Collection` wird nicht entzogen, weil `DropKeys` den Floor ignoriert (§7.3) und fremde Leser nicht gestört werden dürfen.
- **`is_active()` nutzt `Acquire`**, nicht `Relaxed` (Präzisierung zu Index 3.2, siehe `00a` K-28): Auf x86 kostenlos, auf ARM wenige Nanosekunden, und es verhindert, dass ein Leser nach Rückkehr von `publish_gate` den alten Zustand sieht (I-5).
- Eine Epoche steigt bei **jeder** Veröffentlichung (Gate setzen, `Restricted → Erased`, Gate entfernen).

### 6.3 Durchsetzungshaken (jeder mit Regressionstest)

| Haken | Ort (✔ belegt) | Maßnahme |
| --- | --- | --- |
| **H-1 Commit-Ablehnung** | Beginn von `prepare_commit_ops` (`ops/write.rs:300`), **vor** dem ersten `next_seq_no.fetch_add` | `if gate.is_active()`: für jede Op `gate.check(key).is_blocked()` ⇒ `cleanup_intent_locks_for_tx(tx_id)` (wie der vorhandene Fehlerpfad bei 354) und `Err(ContextraError::Erased)`. Es wird **kein** Teil der Transaktion angewendet (Atomarität). `Delete` auf gesperrten Schlüssel wird ebenso abgewiesen |
| **H-2 Late Apply** | `apply_mem_updates` (`lsm/commit.rs:92`) | **Kein Verwerfen.** Siehe Begründung unten. Zähler `erasure_late_apply_total` (nur Anzahl) |
| **H-3 Punkt-Lesen** | `get_at_seq_tracked` (`ops/read.rs:37`) und `get_at_seq` (`:80`) | **Nach** dem Auflösen der neuesten sichtbaren Version `(key, seq)`: `check_version(key, seq).is_blocked()` ⇒ `Ok(None)`. Kein Durchfallen auf ältere Versionen (alle Versionen ≤ `up_to` sind ebenfalls gesperrt) |
| **H-4 Scan/Range/Prefix** | `lsm/scan.rs` und die `scan_*`-Pfade (`memtable.rs:446-582`, `sstable/stream.rs`) | Vorprüfung `check_prefix(prefix) == EntirelyBlocked` ⇒ leeres Ergebnis. Sonst jeden Ergebnissatz `(key, seq)` per `check_version` filtern. Für Bereichsscans (`scan_range_into`) ebenso je Eintrag |
| **H-5 Replay** | `recovery.rs:322` (Put) und `:331` (Delete) | Vor `memtable.put`: `discard_rule(key, raw_lsn) == MayDrop` ⇒ überspringen (nur `max_seq`/`max_tx` fortschreiben). Bei `Keep` normal anwenden (Gate maskiert) |
| **H-6 Flush** | Phase 3 (`ops/compaction.rs:111-132`) | Vor `retain_key_versions`: Versionen mit `discard_rule == MayDrop` entfernen. Leere Versionsliste ⇒ Schlüssel überspringen |
| **H-7 Merge** | `merge_sstables_inner` (`compaction/engine.rs:548`), vor `builder.add` (859) | `EraseFilter::decide` (§7.2). Jede Compaction filtert mit, auch die reguläre |
| **H-8 `delete_prefix`** | `ops/write.rs:190` | Gesperrte Schlüssel überspringen (ohne Fehler). Sie werden durch den Auftrag entfernt |
| **H-9 Zeitreise/Snapshot** | `get_at_seq*`, `SnapshotGuard`-Lesen | Durch H-3/H-4 abgedeckt (Sequenz-unabhängig) |
| **H-10 Export/Import/Backfill** | `engine/src/{export,import}.rs`, Rebuild-Pfade | Nutzen den Store-Lesepfad (H-3/H-4). **[?]** `rg "iter\(\)|scan_prefix_into" crates/*/src` auf Umgehungen (z. B. direkter `MemTable::iter`) |

**Begründung zu H-2 (Präzisierung K-1, siehe `00a` K-29).** Der Index schreibt „Verwerfen“. Das ist unzulässig: Eine Transaktion kann Schlüssel im Scope **und** außerhalb enthalten. Würde `apply_mem_updates` nur die gesperrten Ops verwerfen, wäre die Transaktion zur Hälfte angewendet, obwohl das WAL sie vollständig enthält (der Leader hat `commit_mutex` vor `wal.append_batch` freigegeben ✔ `lsm/mod.rs`, Abschnitt „`commit_mutex` Role“). Ablehnen ist nur in H-1 möglich, solange die Sequenz noch nicht vergeben ist. Eine Transaktion, die H-1 **vor** Schritt 7 passierte, ist „vor dem Erase akzeptiert“. Ihre Scope-Einträge landen (spät) in der Memtable und werden durch (a) Gate-Maskierung beim Lesen, (b) Flush-Filter H-6, (c) das erneute `MemTable::retain` in jedem `Verifying`-Durchlauf (§7.1) und (d) den Replay-Filter H-5 entfernt. Ein Durchlauf, in dem der `MemTable`-Scanner `remaining > 0` meldet, führt in `Purging` zurück (01 §5). Das ist konvergent, weil nach Schritt 7 keine **neue** Scope-Sequenz mehr entsteht.

### 6.4 Epochen-Protokoll für Mehrschritt-Lesepfade

`StoreGate::epoch()` liefert `Acquire`. Die Engine (10 §9) und die Layer verwenden `e1 = epoch(); … lesen …; e2 = epoch(); e1 != e2 ⇒ wiederholen`. Der Store selbst liest innerhalb **einer** Operation genau **einen** Snapshot-`Arc` (ein `read()` mit sofortigem Klonen), nie zwei.

### 6.5 Gate-Ende (I-9)

`complete(id)` entfernt den Eintrag nur, wenn **alle** gelten:

1. `Sealed` ist im Journal, und der Seal-Witness meldet je erwarteter Schicht `remaining == 0`.
2. Kein Checkpoint mit `LegalHold`/Taint-Markierung für diesen Auftrag (Auskunft über den Treiber, 09 §6; der Kernel erhält ein Flag in `complete`-Vorbedingung über `Job.holds: u32`, gesetzt durch `record_layer` des Checkpoint-Layers).
3. `snapshot_registry.min_read_snapshot()` ist `None` oder `> gated_seq`, **oder** der Auftrag hat `OverlappingSnapshotPinned` als Ausnahme ausgewiesen.

Sonst bleibt der Eintrag als `Erased{up_to}` stehen (`sealed = true`) und `complete` liefert `InvalidTransition`. Der Worker wiederholt `complete` bei jedem Snapshot-Ablauf-Ereignis (10 §6). Bei `reinsert_after_sealed` werden Versionen `seq > up_to` nach `sealed` wieder lesbar (`check_version`).

## 7. Physisches Entfernen

### 7.1 `MemTable::retain` (K-4)

```rust
impl MemTable {
    /// Entfernt alle Versionen, für die `drop` true liefert. Gibt (Anzahl, Bytes) zurück.
    /// Nimmt je Shard kurz den Schreib-Lock (Rang 700), nie über .await.
    pub fn retain<F: FnMut(&[u8], u64 /*raw_seq*/, bool /*tombstone*/) -> bool>(&self, drop: F) -> (u64, u64);
}
```

- Aufrufer hält `state.read()` (Rang 400), **nicht** `state.write()` (Korrektur zum Konzept 5.5: `MemTable` mutiert intern, `put` tut es ebenso, `commit.rs:84-90`). Das gilt für die aktive und alle `immutable_memtables`.
- Nach dem Entfernen werden die internen Zähler (`size`, `tx_range`) angepasst **[?]** welche Felder `put` pflegt (`memtable.rs:235-294`); `retain` muss exakt dieselben Invarianten wahren (Test `TST-STORE-MEM-1`).
- Speicherbudget: `ResourceTracker` muss die entfernten Bytes freigeben. **[?]** Es wurde keine `release_memory`-Methode gefunden (`rg "fn release_memory|fn consume_memory" crates/contextra-store/src`). Fehlt sie, ergänzt WP-3.3 eine Freigabe, sonst driftet `budget_tracking_drift_bytes`.
- Prädikat: `filter.decide(key, raw_seq, tomb) == Drop`.
- Der Memtable-Scanner (`scanner/memtable.rs`) ist unverändert und misst die Restmenge.

### 7.2 `EraseFilter`

```rust
pub(crate) struct EraseFilter { jobs: Arc<GateSnapshot>, collapse: Arc<CollapseMap> }
pub(crate) enum Decision { Keep, Drop }
impl EraseFilter {
    /// Reine Funktion. Kein I/O. `tombstone` informiert nur über die Art.
    pub fn decide(&self, key: &[u8], raw_seq: u64, tombstone: bool) -> Decision;
}
```

`decide` liefert `Drop`, wenn **eine** der beiden Regeln greift:

1. **DropKeys:** `gate.discard_rule(key, Seq(raw_seq)) == MayDrop` (Job in `Phase::Erased`, `key ∈ scope`, `raw_seq <= up_to`). Das ersetzt für Scope-Schlüssel die Regel „neueste Version unter dem Floor bleibt“ (B-04) und die Bedingung `is_full_compaction` für Tombstones (855-856): ein Scope-Schlüssel **und** sein Tombstone fallen in **jeder** Compaction.
2. **CollapseVersions:** `collapse.get(key) = Some(keep_from)` und `raw_seq < keep_from`. Die Einträge stammen aus `ProgressSink::register_collapse` (Port-Erweiterung A-1, siehe 02-Änderung in `00a` K-30; Text-Layer 06 §4). `keep_from` ist die Sequenz der vom Layer neu geschriebenen Version. Ältere Versionen enthalten die Dokument-Term-Zuordnung und müssen fallen.

Reihenfolge der Prüfung: DropKeys vor Collapse. Die Regeln sind zustandslos je Eintrag, deshalb ist ein **1:1-Datei-Rewrite** (K-2) korrekt, ohne Sicht auf andere Dateien.

**Reguläre Compaction und Flush rufen denselben Filter**, solange `gate.is_active()`. Ohne aktive Gates kostet der Haken einen `Acquire`-Load.

### 7.3 Erase-Compaction (ein Tick, wiederaufnehmbar)

Voraussetzung: Job in `Gated`/`Purging`, `PurgeStart` geschrieben, Memtable-Retain (§7.1) ausgeführt.

1. **Kandidaten (Hülle), über die flache SSTable-Liste (K-2).** `sstables.read()` (Rang 600) klonen (`Vec<Arc<SstableReader>>`), Lock sofort loslassen. Kandidat ist jede Datei mit `scope.could_overlap_range(min_key, max_key)` für **mindestens einen** Auftrag im Koaleszenzfenster oder mit Treffer in `collapse` (Bereichsprüfung für Exakt-Schlüssel). Für Exakt-Muster dient zusätzlich der Bloom-Filter als Vorfilter. Prefix-Muster haben keinen Bloom-Vorfilter. **[?]** Zugriff auf Footer-Min/Max-Schlüssel: `SstableReader` zeigt `min_tx_id`, `max_tx_id`, `file_path` (`reader.rs:255-263`); ein Schlüsselbereichs-Zugriff wurde nicht gefunden (`rg "min_key|max_key|first_key|last_key" crates/contextra-store/src/sstable`). Fehlt er, wird er aus dem Footer ergänzt (kein Format-Änderung, nur Lesen).
2. **Reihenfolge:** Kandidaten nach `FileId` (`.uuid`-Sidecar ✔ `compaction/engine.rs:418`) sortieren, die Liste beim Tick-Start **einfrieren**. Neue Ausgabedateien gehören nicht in diese Liste (Terminierung).
3. **Je Datei F (1:1):**
   1. `budget.acquire(IoClass::Erase, bytes(F))` (§7.6). `Critical`-Druck ⇒ Tick endet mit `cursor` und Wartehinweis.
   2. Journal: `TempFile{path}` (bevor die Datei entsteht), `sync_data`. Pfad `<sst-name>.erase.tmp` im selben Verzeichnis (gleiches Dateisystem für `rename`).
   3. `F.stream()` (`reader.rs:1000`) lesen, jeden Eintrag durch `EraseFilter::decide`. `Keep` ⇒ `SstableBuilder::add` (`builder.rs:364`) mit **demselben `key_manager`** (`create_with_key_manager`/`create_with_vfs`, `builder.rs:237,244`), damit verschlüsselte Speicher nicht übersehen werden.
   4. **Keine Änderung** (nichts verworfen) ⇒ Ausgabe verwerfen, `TempFile` freigeben, F gilt als sauber (kein Rewrite, keine Manifest-Änderung).
   5. **Leeres Ergebnis** ⇒ Ausgabe verwerfen, Manifest `Remove{F}` (K-2: `Replace.added` ist ein einzelner Pfad).
   6. Sonst: `finish()` (`builder.rs:475`), `sync_all` auf der Ausgabe, `rename(tmp → final)`, `sync_dir`.
   7. **Commit-Punkt:** Unter `sstables.write()` (Rang 600) prüfen, dass F **unverändert** in der Liste steht (`Arc::ptr_eq`). Nein ⇒ Ausgabe löschen, Datei überspringen (Konkurrenz mit Compaction, R3; kein Fehler, nächster Durchlauf). Ja ⇒ Manifest `Replace{removed:[F], added: F', added_max_tx}` anhängen (Rang 800; Muster `compaction/engine.rs:379-382`), dann **an derselben Listenposition** ersetzen, Block-Cache-Einträge von F räumen (**[?]** Methode wie in `compaction/engine.rs` nach 413).
   8. F, F.`.uuid` ⇒ Pending-Delete (§8). `TempFile` im Journal ist erledigt.
   9. `StoreProgress{phase=2, cursor = FileId(F)}` schreiben (nur **nach** dem Manifest-Commit; Zählung nur committeter Arbeit: `units_committed += 1`, `bytes_freed += size(F) − size(F')`).
4. **Positionstreue.** Die Liste ist nach `max_seq` sortiert und die Auswahl der regulären Compaction verlangt zusammenhängende Eingaben (`compaction/engine.rs:331-345`). F' **muss** die Position von F behalten. Der Builder leitet `max_seq` aus dem Inhalt ab; verwirft der Filter die höchste Sequenz, würde F' sonst verrutschen. **Anforderung:** `SstableBuilder` erhält `with_max_seq_override(u64)`, F' erbt `max_seq` und `max_tx` von F **[?]** ob der Footer dies speichert, ist in WP-0 zu prüfen (`TST-STORE-ORDER-1`: Liste vor/nach Erase-Compaction gleich geordnet, `compaction_contiguous_selection.rs` bleibt grün).
5. **Tick-Ende:** nach `cx.max_units` Dateien oder erschöpftem Budget ⇒ `PurgeOutcome{cursor: Some(..)}`. Alle Kandidaten erledigt ⇒ `cursor: None` für Phase 2, weiter mit §7.4.
6. **Platz-Preflight vor dem ersten Tick:** Summe der Größen aller Kandidaten + `reserve_bytes` > freier Platz (**[?]** `Vfs` hat keine `statvfs`; Platzprüfung über `contextra-sys` prüfen: `rg "available_space|statvfs" crates/contextra-sys`) ⇒ `InsufficientSpace`, der Auftrag bleibt bei G1.
7. **Crash:** `*.erase.tmp` stehen im Journal (`TempFile`) und werden beim Start entfernt. Der Manifest-`Replace` ist der Commit-Punkt. Wiederholung findet nichts mehr im Scope (No-op, CP-6).

### 7.4 WAL-Schnitt

Das WAL enthält Scope-Bytes in versiegelten und im aktiven Segment. Einzelne Records werden **nicht** entfernt (die HMAC-Kette bleibt intakt). Der Schnitt ist: Memtable-Retain, erzwungene Rotation, Flush mit Filter, Entfernen aller versiegelten Segmente.

1. `MemTable::retain` (§7.1) auf aktive und immutable Memtables.
2. `LsmStorage::erase_wal_cut()` (neu, `lsm/ops/compaction.rs`):
   - Memtable und Immutables **nicht leer** ⇒ regulärer `flush()` (H-6 filtert). Er rotiert das WAL und löscht die versiegelten Segmente.
   - Memtable und Immutables **leer** ⇒ der reguläre Flush kehrt früh zurück (`ops/compaction.rs:62-64`) und rotiert **nicht**. Das aktive Segment würde die entfernten Scope-Records behalten. Deshalb: unter `commit_mutex` (200) → `state.write()` (400) eine **erzwungene Rotation** ausführen (neues Segment anlegen, altes versiegeln), dann das versiegelte Segment entsorgen. Zulässig, weil nach dem Retain alle Einträge des alten Segments entweder bereits geflusht oder Scope-Einträge sind.
3. Entsorgung über Pending-Delete (§8), nicht mehr nur Log bei Fehler (B-06: `ops/compaction.rs:215,220,262`).
4. **`FlushPurgeFence`:** Der Erase-Pfad setzt `purge_active` **nicht** (K-20), weil er selbst `flush()` aufruft und `enter_flush()` sonst `false` lieferte. Gleichzeitiger Flush und Erase-Compaction sind durch die Konkurrenzprüfung unter `sstables.write()` (§7.3 Schritt 7) und `flush_mutex` (300) sicher. **[?]** Ob reguläre Purge-Pfade `enter_purge()` aufrufen: `rg "enter_purge|purge_active" crates/contextra-store/src` (am HEAD nur in `fence.rs`).
5. **K-15 (Blocker).** Ein Leader mit **bereits geklontem** `Arc<Wal>` (`ops/write.rs:278`) kann nach der Rotation auf das versiegelte Segment schreiben. **Bis `TST-STORE-ROT-1` grün ist, ist der WAL-Schnitt gesperrt** (`erase_wal_cut` liefert `Ok(WalCutResult::Deferred)`, der Witness `WalAllSegments` meldet ehrlich `remaining > 0`, G3 wird nicht erreicht). Behebung falls bestätigt: Rotation unter `commit_mutex` **und** Leader prüfen nach `append_batch`, dass `Arc::ptr_eq(&wal_clone, &*storage.wal.read())`, sonst Batch im neuen Segment wiederholen.

### 7.5 `delete_prefix` und Retire-Pfad

`delete_prefix` (B-07) bleibt für den **Retire**-Pfad (kein Erase). Änderungen: H-8 (gesperrte Schlüssel überspringen). Der Erase-Pfad ruft `delete_prefix` **nie**. Eine Materialisierung aller Schlüssel entfällt für Erase, weil Gate und `DropKeys` das Entfernen übernehmen. Der Speicherbedarf des Retire-Pfads (`tx_buffer` hält alle Tombstones) ist **nicht** Gegenstand dieser Spezifikation. Offener Punkt O-S5.

### 7.6 `IoBudget` (WP-4.0, K-3)

- **Extraktion, verhaltensgleich:** `src/io_budget.rs` enthält `TokenBucket { bytes_written, last_reset_nanos, max_bps }` mit genau der Arithmetik aus `compaction/engine.rs:780-805, 864-891` (Token-Bucket nach `clock.monotonic_nanos()`). Die Merge-Schleife ruft `bucket.charge(bytes).await`. Test `TST-STORE-IOB-1`: mit `ManualClock` (testkit) identische Wartezeiten wie vor der Extraktion (bestehender Test `tests/compaction_io_rate_limit.rs` ✔ bleibt unverändert grün).
- **`StoreIoBudget: ports::IoBudget`** hält **einen** Bucket je Store (`Arc<Mutex<BucketState>>`, `parking_lot`, nie über `.await`). Klassen: `Interactive` nie gedrosselt. `Erase` und `Compaction` teilen `max_io_bytes_per_second`: `Erase` höchstens `erase_io_share × max`, hat beim Zuteilen **Vorrang**, darf ungenutzten Compaction-Anteil leihen, wenn kein Compaction-Waiter in der letzten Sekunde registriert war. Beide zusammen überschreiten die Grenze nie.
- `max_io_bytes_per_second == 0` bedeutet in der Compaction heute „unbegrenzt“ **[?]** (Prüfauftrag `compaction/config.rs`). Für `Erase` gilt dann: kein Limit, aber `pressure()` greift.
- `pressure()` liest `pressure_rx.borrow()` (`SystemPressure`, `system_pressure.rs` ✔). `Critical` ⇒ `acquire(Erase, _)` und `acquire(Compaction, _)` warten kooperativ (`clock`-basiert, mit Jitter aus `Rng`, cancel-safe). `Elevated` ⇒ halbes Erase-Budget.

### 7.7 `PrivilegedRead`

`StorePrivilegedReader` (pro Auftrag) liest **am Gate vorbei** über die interne Lesefunktion ohne H-3/H-4. Bei **jedem** Aufruf: `scope.keys.contains(raw_key)` ⇒ sonst `ErasureError::Layer{layer: "store", kind: ScopeViolation}` (I-8) und Sicherheitsmetrik. `scan_prefix(prefix, after, limit ≤ 1024)`: Ergebnisse werden einzeln gegen `contains` gefiltert; liegt der Präfix **nicht** vollständig im Scope (kein `In`-Muster ist Präfix von `prefix`), ist das ebenfalls `ScopeViolation`. Nach `Sealed` ⇒ `Err`. Gelesene `Bytes` werden nicht gepuffert, nicht geloggt.

### 7.8 Scanner und Witness

`store_scanners()` liefert die drei vorhandenen Scanner (`scanner/{memtable,sstable,wal}.rs` ✔) mit den Nadeln aus dem Scope (Schlüsselbytes, `DocId` als LE/BE/Dezimal/Hex, Konzept 5.10). **[?]** Aktuelle Signatur von `LayerScanner::scan` und Eingabetyp (heute `ScanTarget{target_id: String, target_hash}` laut 02 §2): ein Adapter `NeedleSet::from_scope(&ResolvedScope)` ist in WP-4.5 zu liefern, ohne die Mint-Schutz-Garantie (`ScanSession::begin/finish` bleiben `pub(crate)` in `ports`) anzutasten. Der Witness `SsTableAllLevels` durchsucht Dateiinhalte **und**, falls vorhanden, entschlüsselte Sicht (`DecryptedView`). Offene Datei-Handles ersetzter SSTables: Nach dem Manifest-Swap wartet der Kernel höchstens `reader_drain_timeout` (Default 30 s) auf `Arc::strong_count(F) == 1`; danach Ausnahme `OverlappingSnapshotPinned{min_seq}`.

## 8. Pending-Delete und Datei-Entsorgung

- Datei `<dir>/pending-delete.list` über `StoreIo::write_atomic`. Layout: Kopf `b"CTXPDL01"`, `n: u32`, je Eintrag `len: u16` + UTF-8-Pfad relativ zum Datenverzeichnis, `crc32: u32` über alles davor.
- **Ablauf je Dateigruppe** (`remove_after_commit(paths)`): (1) Pfade zur Liste hinzufügen, `write_atomic`; (2) je Pfad `remove_file` (`NotFound` zählt als erledigt); (3) `sync_dir(parent)`; (4) Pfade aus der Liste entfernen, `write_atomic`. Fehler in (2) oder (3) bleiben **in der Liste**, `retries += 1`, Metrik `erasure_pending_delete_remaining`. Nach `pending_retry_limit` Alarm, **nie** Verwerfen.
- **Aufrufer (WP-4.4):** Flush (`ops/compaction.rs:215,220,262`), Compaction (`compaction/engine.rs:413,418` und die Aufräumzweige 363, 534), Erase-Compaction (§7.3), WAL-Schnitt, `*.erase.tmp`. Diese Stellen ersetzen `remove_file` + `warn!` durch `pending.remove_after_commit`. Das Verhalten ohne aktive Aufträge ist unverändert (Datei wird entfernt), nur Fehler werden nicht mehr verschluckt.
- **Boot:** Liste abarbeiten (§5.4 Schritt 7). **Wiederholung** im Leerlauf durch den Worker (10 §6), ohne Polling: Der Kernel meldet über einen `Notify`, wenn die Liste nicht leer ist.
- **Hardlinks (Checkpoints):** Der Store entfernt nur **seinen** Namen. Das vollständige Entfernen aller Namen ist Sache von 09 §5.

## 9. Crash-Matrix (Testspezifikation)

CP-1 … CP-12 aus dem Konzept 8.1 gelten unverändert. Ergänzt (Index §6 Nr. 2):

| ID | Crash-Punkt | Zustand nach Neustart | Wiederaufnahme |
| --- | --- | --- | --- |
| CP-13 | Journal-Frame zur Hälfte geschrieben (Torn Write) am Dateiende | letzter gültiger Frame | Torn Tail kürzen (§5.4), Zustand davor |
| CP-14 | Fehlerhafter Frame **in der Mitte** (Bitflip) | `JournalCorrupt` | Fail-closed-Modus, Alarm, kein Öffnen des Gates |
| CP-15 | Nach `TempFile`-Frame, vor `rename` der Ausgabe | `*.erase.tmp` liegt da | beim Boot über Pending-Delete entfernt, Datei F unverändert |
| CP-16 | Nach `rename(tmp→final)`, vor Manifest-`Replace` | neue Datei F' liegt ohne Manifest-Eintrag | F' ist verwaist (nicht im Manifest): Boot-Aufräumen der Manifest-fremden `*.sst` wie bei der regulären Compaction **[?]** `recovery.rs` räumt Waisen? (Prüfauftrag `TST-STORE-ORPHAN-1`) |
| CP-17 | Nach `Manifest::Replace`, vor `sync_dir` des Entfernens von F | F existiert noch, ist aber nicht im Manifest | Pending-Delete-Liste enthält F ⇒ Boot entfernt F |
| CP-18 | Pending-Delete-Liste: Crash zwischen Schritt (1) und (2) bzw. (3) und (4) | Liste enthält Pfade, die teils fehlen | `NotFound` zählt als erledigt, Liste wird bereinigt |

**Voraussetzung für alle:** `CrashVfs` im Testkit (13 §4). `FaultVfs` reicht **nicht** (kein `open`/`rename`, `sync_dir` No-op ✔ `fault_vfs.rs:307`).

## 10. Tests

Bestehende Tests bleiben unverändert grün. Neu:

| ID | Inhalt |
| --- | --- |
| TST-STORE-JRN-1 | Round-Trip aller 16 Frame-Typen, Wiederaufbau des Zustandsautomaten |
| TST-STORE-JRN-2 | Golden-Test: feste Eingaben ⇒ feste Bytes je Typ (Layout eingefroren) |
| TST-STORE-JRN-3 | CP-13/CP-14 (Torn Tail vs. Korruption); Frame am Ende mit falscher CRC wird gekürzt, in der Mitte ⇒ `JournalCorrupt` |
| TST-STORE-JRN-4 | `fsync`-Fehler ⇒ `JournalPoisoned`, keine weitere Mutation, Gate bleibt aktiv (CP via `CrashVfs`) |
| TST-STORE-JRN-5 | Rollover: Crash an jeder Stelle (CP-12), Verdichtung entfernt Zielbytes (Canary im Plan-Blob nach `Completed` nicht mehr in `journal.log`) |
| TST-STORE-GATE-1 | `check`/`check_version`/`check_prefix`/`discard_rule`/`check_doc` gegen ein Brute-Force-Modell (proptest über Schlüsselmengen, `KeyScope` mit `Out`-Regeln, mehrere Aufträge) |
| TST-STORE-GATE-2 | `up_to`-Semantik: nach `Sealed` + `AllowAfterSealed` ist `seq > up_to` lesbar, `seq <= up_to` nicht |
| TST-STORE-GATE-3 | Fail-closed: kaputtes Journal ⇒ jeder Schlüssel `Restricted`, Schreiben abgewiesen |
| TST-STORE-GATE-4 | > 64 Präfixlängen ⇒ linearer Rückfall liefert dieselben Ergebnisse |
| TST-STORE-GATE-5 | `is_active()` ohne Gates ⇒ Open, Epoche unverändert; Veröffentlichung erhöht Epoche streng monoton |
| TST-STORE-GATE-6 (Loom) | R1: Commit gegen `publish_gate`: jede Sequenz `<= gated_seq` ist „akzeptiert“, jede `> gated_seq` wird abgewiesen. **Loom** nur für die Gate-Zustandsmaschine (der Store ist unter `cfg(loom)` nicht gebaut: `lib.rs` `#[cfg(not(loom))]` ✔), deshalb ein **Modell** der Protokollschritte 4–10 in `tests/loom_erasure_gate.rs` |
| TST-STORE-GATE-7 (Loom) | R4: Leser mit `e1`/`e2` gegen Veröffentlichung |
| TST-STORE-GATE-8 | H-1 … H-10 je ein Regressionstest (Schlüssel gesperrt ⇒ `get`, `get_at_seq`, `scan_prefix`, `scan_range`, `iter_latest`, Zeitreise, Replay, Flush, Merge, `delete_prefix`) |
| TST-STORE-GATE-9 | Benchmark (`benchmarks/contextra-bench`): ohne Gates 1 Load; ≤ 1 000 Einträge p99 ≤ 200 ns; entscheidet `RwLock` gegen `arc_swap` |
| TST-STORE-COMMIT-1 | **[?]-Auflösung:** `prepare_commit_ops` wird in `single_commit_path` und im Group-Commit-Leader nur unter `commit_mutex` aufgerufen; Test erzwingt Verschränkung mit `publish_gate` |
| TST-STORE-ROT-1 | **[?]-Auflösung K-15:** Leader mit geklontem `Arc<Wal>` schreibt nach Rotation; erwartetes Ergebnis laut §7.4 Punkt 5 |
| TST-STORE-INTENT-1 | **[?]-Auflösung Index §5 Nr. 5:** Lebensdauer der Intent-Dateien (`recovery.rs` ~449–725, 860–878), Entfernung über `StoreIo` |
| TST-STORE-MEM-1 | `MemTable::retain`: Invarianten (`size`, `tx_range`, Iteratoren), parallel zu `put`/`get` (proptest + Stress) |
| TST-STORE-ORDER-1 | Positionstreue (§7.3 Punkt 4); `compaction_contiguous_selection` bleibt grün |
| TST-STORE-ORPHAN-1 | CP-16 (Waisen-SST) |
| TST-STORE-IOB-1 | Verhaltensgleichheit der Extraktion (§7.6), Anteil `erase_io_share`, Vorrang, Druckkopplung |
| TST-STORE-E2E-1 | Die drei `#[ignore]`-Tests (`scanner/tests.rs:90,178,204`) umgeschrieben auf den Kernel und aktiv; `test_purge_clears_remaining_residuals` erwartet `remaining == 0` |
| TST-STORE-E2E-2 | Ende-zu-Ende ohne Engine: schreiben, `accept → publish_gate → purge_store (Ticks) → seal` mit Mock-Layern; Nachbar-Schlüssel bit-identisch (C3) |
| TST-STORE-ISO-1 | Mandanten: Auftrag für Tenant 7 berührt Schlüssel von Tenant 8 nicht (`t:7:` vs. `t:70:` — **Prefix-Falle**, siehe §12) |
| Mutationstest | `mutants-guarantees.toml` ✔ um `erasure/gate.rs`, `erasure/filter.rs` erweitern (Garantie-Mutanten müssen sterben) |

## 11. Ledger (WP-10)

### 11.1 Ablage und Format

- Datei `<ledger_path>/ledger.log` **außerhalb** des Datenverzeichnisses. Profil `Regulated` (`require_external_ledger`) verweigert den Start, wenn der Pfad unter `<data>` liegt (Pfadvergleich nach `canonicalize` **über `StoreIo`-lose Pfadlogik**: nur `Path::starts_with` auf bereits absolute Pfade, kein Dateisystemzugriff).
- Kopf `b"CTXLDG01"`, Frames wie das Journal (`len`, `crc`, `type=1`, Payload). Payload `LedgerEntry` (02 §4.7) handgeschrieben: `index u64`, `erasure_id_hmac[32]`, `guarantee u8`, `layers_root[32]`, `n_exc u16`+Ausnahmen (§5.6), `plan_hash[32]`, `policy_version u32`, `completed_at u64`, `prev_hash[32]`, `entry_mac[32]`.
- `entry_hash = BLAKE3(frame_payload_ohne_mac)`; `prev_hash` des ersten Eintrags = `[0;32]`. `entry_mac = HMAC-SHA256(ledger_key, alle Felder ‖ prev_hash)` (`hmac`/`sha2` sind `crypto`-Abhängigkeiten; der Store nutzt die Primitive **über** `contextra-crypto`: dort bereitzustellen `ledger_mac(key, bytes) -> [u8;32]`, 05 §6).
- `ledger_key` und `erasure_log_key` (für `erasure_id_hmac` und Log-HMACs) kommen aus `KeyMaterialSource` (`store/src/key_source.rs` ✔) per `hkdf_expand` (`crypto` Export ✔) mit Kontexten `b"ctx-ledger-mac-v1"` und `b"ctx-erasure-log-v1"`. **Nie** aus Konstanten.

### 11.2 `seal(id, witnesses)`

1. Zustand `Verifying`; für **jede** erwartete Schicht (`Job.expected_layers`) muss ein Witness mit passendem `witness_layer()` vorliegen. Fehlende ⇒ Ausnahme `NotMeasured{layer}`, `purged = false`.
2. `purged = true` ⇔ alle Witnesses `remaining() == 0` **und** keine `NotMeasured`/`FeatureDisabled`. `crypto_erased` nur mit Stufe-2-Beleg (`KeysDestroying` im Journal **und** Vault-Generation bestätigt). `gated = true` immer ab `Gated`. Invariante `purged ⇒ gated` (01 §4.4).
3. `layers_root = BLAKE3(b"ctx-layers-v1" ‖ je Layer in Tag-Reihenfolge: tag ‖ digest())`. Die Digests stammen aus `LayerWitness::digest()` ✔.
4. Ausnahmen: `OutOfScopeExternalCopies` **immer**; `Unprovenanced{n}` aus dem Plan; `LegalHold` aus `Job.holds`; `OverlappingSnapshotPinned`.
5. **Ledger-Append** (falls konfiguriert; `fsync` der Ledger-Datei), **dann** `Sealed` ins Journal (Reihenfolge Ledger → Journal; CP-10: Crash dazwischen ⇒ Wiederholung, `append` ist über `(erasure_id_hmac, plan_hash)` idempotent und liefert den vorhandenen Eintrag).
6. Danach `LedgerAnchor::publish(head)` **asynchron und fehlertolerant** (Fehler zählen, nie den Auftrag blockieren).

### 11.3 `verify`, Korruption

`verify()` rekonstruiert Kette und MACs über die ganze Datei (Streaming). Ein Fehler ⇒ Ledger `Corrupt`. Folge: `accept` lehnt neue Aufträge mit `LedgerCorrupt` ab, Lesen und laufende Aufträge bleiben möglich (Konzept 5.10). Kürzung am Ende erkennt nur der Anker (Konzept); dafür ist `LedgerAnchor` Pflicht im Profil `Regulated`.

## 12. Arbeitspakete (jeweils verdrahtet, `deny(dead_code)`)

Jedes Paket endet mit `cargo build/test --workspace --locked`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo deny check`, `tests/layering.rs` und den CI-Gates G-1 … G-6 (Index 3.7).

| WP | Inhalt | Verdrahtung |
| --- | --- | --- |
| **3.0** | Offene Punkte auflösen: TST-STORE-COMMIT-1, -INTENT-1, Footer-Min/Max, `ResourceTracker`-Freigabe, `LsmConfig`-Datenpfad, `WitnessLayer`-Tags, `StoreIo::open`-Nutzbarkeit | nur Tests/Lesezugriffe |
| **3.1** | `erasure/journal.rs` (+ Tests JRN-1…5, benötigt `CrashVfs` aus 13 §4) | im selben Paket von `StoreErasureKernel::open` aufgerufen; `open` wird in `LsmStorage::new…` vor Zeile 277 verdrahtet (leerer Kernel, kein Gate) |
| **3.2** | `erasure/gate.rs`, Ränge 150/960, `publish_gate`, `accept`, `status`, `open_jobs` | `LsmStorage.erasure`; H-1, H-3, H-4 |
| **3.3** | `MemTable::retain`, H-2, H-5, H-6, H-7, H-8, `EraseFilter` (zunächst mit DropKeys) | Flush, Merge, Replay |
| **3.4** | Boot-Reihenfolge (§5.4), `revoke_covering` für Tenant | Konstruktor |
| **3.5** | `LsmStorage::erasure_kernel()`, `privileged_reader` | Engine kann ab 10 anbinden |
| **3.6** | **I/O auf `StoreIo`:** `lsm/recovery.rs` (u. a. Zeilen 449, 462, 656, 725, 860, 878) und `manifest/rollover.rs:141,177` von `tokio::fs` auf `StoreIo`. Das **Manifest-Framing** bleibt unverändert, nur der Dateizugriff wechselt. Voraussetzung für `CrashVfs`-Tests von CP-7, CP-12, CP-16. Verhalten ohne Fehlerinjektion identisch (`chaos_power_cut.rs` ✔ bleibt grün) | Recovery, Manifest |
| **4.0** | `io_budget.rs`: verhaltensgleiche Extraktion (§7.6) | Compaction-Schleife |
| **4.1** | `pending.rs` (§8) und Umstellung der `remove_file`-Aufrufer | Flush, Compaction |
| **4.2** | Erase-Compaction (§7.3) inkl. `with_max_seq_override`, Platz-Preflight | `purge_store` |
| **4.3** | WAL-Schnitt (§7.4), Rotation erzwingen; **gesperrt**, bis TST-STORE-ROT-1 grün | `purge_store` |
| **4.4** | `CollapseKeys` (A-1), Filter-Regel 2 | Filter |
| **4.5** | Scanner-Adapter (§7.8), `seal`/`complete` (ohne Ledger), `PrivilegedRead` | Kernel |
| **4.6** | `scanner/purge.rs` löschen, `#[ignore]`-Tests aktiv, `Purger`/`ScanTarget` bleiben `#[deprecated]` in `ports` bis 06–08 | Aufräumen |
| **10** | `ledger.rs`, `verify`, Anker (§11) | `seal` |

## 13. Definition of Done

1. Alle Tests aus §10 grün, **auch unter `CrashVfs`** (13), plus `chaos_power_cut` und die Differential-Tests (`differential_redb`) unverändert grün.
2. Die vier Anforderungen aus Index §6: Invarianten I-1 … I-12 je ein Test; CP-1 … CP-18 laufen systematisch; Canary-Test `T-DEL-1` für die Store-Schicht (eindeutige Bytefolge ⇒ nach `Sealed` 0 Treffer in `<data>` **ohne** Ledger-/Journal-Treffer nach Verdichtung); Leerlauf-CPU 0.
3. `rg "block_on|let _ =|unwrap_or_default|\.ok\(\);" crates/contextra-store/src/erasure` leer (G-1, G-2). `rg "tokio::fs|std::fs" crates/contextra-store/src/erasure` leer (G-4).
4. `capabilities.toml` unverändert; `tests/layering.rs` grün.
5. Kein `pub` außer §3. `deny(dead_code)` grün ohne `#[allow]`.
6. Dokumentation mit „# Fehler / # Crash“ auf jedem öffentlichen Element (Repo-Konvention ✔ `ports`).

## 14. Offene Punkte

| Nr. | Frage | Vorschlag |
| --- | --- | --- |
| O-S1 | Prefix-Falle: `t:7:` ist Präfix von `t:70:`. Ein `Tenant`-Scope mit `Prefix(b"t:7:")` trifft `t:70:…` | Mandantenpräfixe **mit abschließendem `:`** verwenden (`t:7:` enthält den Doppelpunkt, `t:70:` beginnt mit `t:7`, aber nicht mit `t:7:`). Test `TST-STORE-ISO-1` belegt es. **[?]** `TenantKeyCodec::scan_prefix()` ✔ (`tenant_codec.rs`) liefert tatsächlich den Doppelpunkt |
| O-S2 | Collapse verwirft ältere Posting-Versionen auch für **andere** Dokumente (Zeitreise auf `i:`-Schlüssel) | Akzeptiert (Konzept 5.5). Dokumentiert als Ausnahme-Hinweis im Ledger? Entscheidung bei 06 |
| O-S3 | Fail-closed-Modus bei `JournalCorrupt` macht den Store unbenutzbar | Bewusst (I-7). Betriebsvorgabe im Runbook (13 §8): Backup des Journals, Operator-Werkzeug außerhalb dieser Spezifikation |
| O-S4 | `StoreIo::open` ist synchron; Journal nutzt `spawn_blocking` je Operation | Ausreichend für < 5 ms Haltedauer. Messung in `TST-STORE-GATE-9`-Umfeld (Journal-Append-Latenz) |
| O-S5 | `delete_prefix` hält alle Tombstones im `tx_buffer` | Außerhalb der Löschkette, eigener Fehlerbericht |
| O-S6 | Die Annahme „Reader-Handles halten Dateien nach `unlink`“ (Linux) gilt auf Windows nicht (Entfernen schlägt fehl) | Pending-Delete behandelt es generisch (Wiederholung); Plattform-Test in WP-12 |

## Revision 2 (28.11.2026, Audit gegen HEAD `e12873de6`; Details in 14)

**Stand.** `crates/contextra-store/src/erasure/`: `mod.rs` (1 747 LOC, Kernel), `journal.rs` (1 889), `ledger.rs` (523), `gate.rs` (721), `purge_lsm.rs` (818), `pending.rs` (301), `filter.rs` (86), Tests `erasure_kernel_e2e.rs`, `loom_erasure_gate.rs` ✔. WP-3.6 (`StoreIo` statt `tokio::fs`) ist umgesetzt (PR #6093).

**Korrekturen an §4.2 (verbindlich).**

| Methode | Änderung |
| --- | --- |
| `begin_purge(id)` **neu** | Logik aus `purge_store` (`mod.rs:1146-1190`) extrahiert; `Gated/KeysDestroying/Failed/Purging ⇒ Purging` mit `PurgeStart`, Gate `Restricted → Erased`; idempotent. `purge_store` ruft es intern |
| `record_layer` | Vorbedingung unverändert `Purging`; Aufruf nur nach `begin_purge` |
| `fail` | Schwelle aus `StoreErasureConfig.quarantine_after_attempts` (neu, Default 10; bisher hart `10` bei `mod.rs:1622`); `layer = None` ab Schwelle ⇒ `Quarantined{layer:"job"}`; `Some(l)` ⇒ `layer_cursor.remove(l)`, identisch im Replay (`apply_record_to_jobs`, Zweig `Failed`) |
| `resume(id)` **neu** | `Quarantined ⇒ Failed{attempt:0} ⇒ Purging`, nur durch Operator-Aufruf |
| `to_job_view` | `layers_done` aus `layer_cursor` mit `cursor == None` (unverändert; durch Cursor-Entfernung korrekt) |

**K-33 / A-2 (Auslieferungsstopp, weiterhin offen).** `StoreErasureKernel::open_early` leitet `ledger_key` und `erasure_log_key` aus `key_source.fill_key_bytes(&mut prk)` ab (`mod.rs:344`) und erzeugt damit bei **jedem Start** neue Schlüssel; `recovery.rs` übergibt `OsKeyMaterialSource`. Korrekt ist die Ableitung aus einer **persistenten** Wurzel (05 Revision 2): `open_early(.., root: &dyn ErasureRootProvider, ..)`, `ErasureRoot::derive(ErasureLabel::LedgerMac | ErasureLog, &[])`. Die HKDF-Labels sind identisch (`ctx-ledger-mac-v1`, `ctx-erasure-log-v1`), der Wechsel ändert keine Ableitungsvorschrift, wohl aber die Quelle. Journal oder Ledger vorhanden ohne Wurzel ⇒ Fail-closed, keine Neuerzeugung (`TST-STORE-ROOT-3`).

**K-50 (Encoder).** In `journal.rs:97-157` werden Wildcard-Fallbacks (`_ => 1`, `_ => 4`) entfernt; die Encoder liefern `Result` (`JournalPoisoned`/`InvalidTarget`). Golden-Test `TST-STORE-JRN-2` wird um „unbekannter Wert ⇒ Fehler“ ergänzt.

**K-51 (§7.8 Scanner).** `WalAllSegments` iteriert alle `wal-*.log` inklusive `pending_sealed_wals` und der Pending-Delete-Liste, Lesefehler ⇒ `Err`, bei Verschlüsselung Technik `Structural`. Zusätzlich neuer Zeuge `Manifest` (Min/Max-Schlüssel je Datei, alte Manifest-Generationen) — Datenort-Matrix Zeile 7.

**§9 Crash-Matrix.** CP-19 … CP-30 siehe 14 §5. **§10 Tests.** `TST-STORE-ROOT-1…5`, `TST-STORE-WIT-1/2`, `TST-STORE-MAN-2`, `TST-STORE-SST-3`, `TST-STORE-LOCK-1`, `TST-JRN-ENC-1` (Tabelle 14 §6).
