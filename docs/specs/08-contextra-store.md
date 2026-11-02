# `contextra-store` – Implementierungsspezifikation

|Feld|Wert|
|---|---|
|Zweck-Satz (D-97)|LSM-Speicher: WAL, Memtable, SSTables, Manifest, Kompaktion, Purge-Primitive.|
|Ring|2|
|Erlaubte Workspace-Abhängigkeiten|`types`, `ports`, `sys`, `crypto`, `mvcc` (SPEC 5.3). **Nicht:** `text`, `vector`, `erasure` (AR-2); `RealVfs` nur als Trait aus `ports`.|
|Stufe / Gates|S0; G-A (Build), G-C, **G0** (Speicherentscheid), G-D|
|Arbeitspakete|WP-0.3, WP-1.4, WP-3.3, WP-3.4 (G0-Prototyp), WP-4.2 (Kompaktions-Kooperation)|
|SPEC-Anker|8.8, 9.1–9.5, 10.3–10.4, 11.4, 11.9, D-02, D-05, D-31, D-32, D-55, D-57, D-61, D-65, D-73, D-76, D-78, D-85, D-94, F.4–F.6, I-1..I-3, I-9, I-16..I-18|

## 1. Ist-Zustand (HEAD `1454bbec0`) [V]

|Punkt|Befund|
|---|---|
|Umfang|26.996 Zeilen `src`, 126 Testdateien in `tests/` (Orakel und Regressionsschutz, SPEC 3.1)|
|Module|`wal/` (`encode`, `flusher`, `hmac`, `io`, `replay`), `lsm/` (`commit`, `group_commit`, `engine`, `flush`, `recovery`, `scan`, `validate`, `guard`, `observer`, `ops/`, `config`), `memtable.rs`, `sstable/` (`builder`, `reader`, `block_cache`, `bloom`, `stream`, …), `manifest/` (`core`, `entry`, `recovery`, `rollover`), `compaction/` (`adaptive`, `engine`, `merge_operator`, `retention`, `config`), `engine/` (`memory_only.rs`), `tenant_codec.rs` (`TenantKeyCodec`, `TenantScopedStorage`), `fsync.rs`, `system_pressure.rs`, `util.rs`|
|Schlüsselraum|Textpräfixe/Klartext-Tenant: `TenantKeyCodec::encode_chunk_key(collection, doc_id)`, `encode_graph_key` (Graph-Altlast); `DocId(u64)` unkeyed|
|Verschlüsselung optional|`key_manager: Option<Arc<KeyManager>>` in `wal/mod.rs:323,353,401,445,520,581` und `compaction/engine.rs:27,44`; Plaintext-Fallbacks (F-09)|
|Integritäts-Downgrade|`allow_legacy_integrity_key_fallback` in 11 Dateien (`wal/io.rs`, `wal/flusher.rs`, `wal/replay.rs`, `lsm/engine.rs`, Tests, Fuzz-Ziele), Feature `legacy-wal-key` (F-21); `DurabilityMode::{WalNoHmac, MemoryOnly}` in `lsm/config.rs`|
|Lizenz-Rest|`lsm/config.rs:44,58`: `contextra_ports::license::FeatureRing` (2× E0433) – Ursache für das Nichtbauen (F-32)|
|`mmap`|`wal/replay.rs:158,210`: Aufruf `unsafe fn contextra_sys::mmap_readonly` aus einem `forbid(unsafe_code)`-Crate (2× E0133); Abhängigkeit `memmap2`|
|Direkter Dateizugriff|171 Treffer (`std::fs`, `tokio::fs`, `File::open/create`, `OpenOptions`, `memmap2`) statt `Vfs` (F-29)|
|Features|`default`, `test-hooks`, `test-utils`, `memory-only-storage`, `fault-injection`, `block-cache-v2`, `sieve-cache`, `docid-128`, `legacy-wal-key`|
|Caches|`quick_cache` **und** `lru` (zwei Implementierungen)|
|Weitere Abhängigkeiten|`uuid`, `rand`, `crc32fast`, `tokio-util`, `memmap2` (prüfen/entfernen)|
|Build|bricht ab (F-32), `Cargo.lock` veraltet|

## 2. Verantwortung und Abgrenzung

`store` speichert **opake Werte** (Record-Envelopes v2) unter **blinden Tag-Schlüsseln** in einem LSM mit WAL, Memtable, SSTables, Manifest und Kompaktion, und stellt die **Purge-Primitive** bereit. Der Store entschlüsselt nie Record-Inhalte: die Record-Verschlüsselung (DEK) liegt über dem Store (`RecordSealer`, `engine`); der Store verschlüsselt **Dateiblöcke** (Block-AEAD, Dateischlüssel aus `FileKeyring`).

Aufgaben:

1. WAL (HMAC-Kette, verschlüsselt, **nie umgeschrieben**), Gruppen-Commit mit `ReadGate`-Prüfung,
2. Memtable, Flush, SSTables mit Block-AEAD, Bloomfilter auf dem Präfix `tenant‖coll‖cluster‖doc_tag`,
3. Manifest-Log (Datei-Mengen, Schlüsselbereiche, Sequenzen, `WalFloor`),
4. Kompaktion (kooperativ abbrechend; verwirft `Erased`-Bereiche, nie `Restricted`),
5. Purge-Primitive und `ErasureLayer`-Implementierung für die Schicht `doc_store`,
6. `Poisoned`-Zustand (D-55), Plattenreserve (D-65),
7. `TenantScopedStorage` als **einziger** Zugang zum rohen Store (AR-6).

Nicht im Store: Schlüsseltresor, Journal, Saga, Gate-Verwaltung (`erasure`), Mandanten-/Collection-Logik, Suche (`engine`, `text`, `vector`).

## 3. Zielstruktur (Module) [E]

|Modul|Inhalt|Quelle|
|---|---|---|
|`wal`|Segmente `wal/<file_id>.wal`, Frames (F.4), `WalFloor`, Flusher, Replay (Torn-Write-Regel)|10.4, D-73|
|`memtable`|sortierte Map über `KeyBuf`; Drain nach `Matcher`|11.4|
|`sstable`|Builder, Reader, Blockcache, Bloom, Footer (F.5); byteidentische Envelope-Übernahme|F.5, I-3|
|`manifest`|Log (F.4): `AddFile`, `RemoveFile`, `WalFloor`, `SeqMark`, `Collection`|F.4|
|`commit`|Gruppen-Commit-Leader, Gate-Prüfung, Seq-Vergabe|D-57, 14.2|
|`compaction`|Auswahl, Merge, kooperativer Abbruch, Gate-Verwurf|D-78, I-16, I-18|
|`purge`|`drain_range`, `rewrite_without`, `seal_wal_at`, `drop_wal_until`, `unlink_verified`|8.8, 11.4|
|`erase`|`DocStoreLayer: ErasureLayer` (Budget 5.8: `store/src/erase`)|P12, D-34|
|`tenant_scoped`|`TenantScopedStorage<S>` (statt `tenant_codec`)|AR-6|
|`cache`|**eine** Blockcache-Implementierung|D-84|
|`health`|`Poisoned`-Flag, freier Platz, `fsync`-Statistik|D-55, D-64|
|`error`|`StoreError` → `types::Error`|7.5|

Entfallen: `engine/memory_only.rs`, `util.rs`, `fsync.rs` (läuft über `Vfs`), `system_pressure.rs` (prüfen, P8), `compaction/merge_operator.rs` (Wasm-Merge entfällt; Merge-Logik ist fest), `tenant_codec.rs` (`TenantKeyCodec`, Graph-Schlüssel).

## 4. Schnittstellen [E]

```rust
// statisch dispatcht (D-03); Engine ist generisch über S: StorageEngine
pub trait StorageEngine: Send + Sync + 'static {
    async fn get(&self, k: &KeyBuf, at: Option<Seq>) -> Result<Option<Bytes>>;        // nach Gate-Prüfung
    async fn scan(&self, r: &KeyRange, at: Option<Seq>) -> Result<ScanStream>;        // gegatete Elemente übersprungen (T-SCAN-1)
    async fn commit(&self, b: WriteBatch) -> Result<CommitInfo>;                      // Gruppen-Commit, Seq in CommitInfo
    async fn snapshot(&self, tenant: TenantTag) -> Result<SnapshotHandle>;
    async fn flush_memtable(&self) -> Result<Seq>;                                    // liefert Flush-Punkt F_seq
}
pub struct WriteBatch { ops: Vec<(KeyBuf, Option<Envelope>)> }                        // None = Tombstone (vlen = 0)

// Purge-Primitive (nur über ErasureLayer erreichbar; Reihenfolge fest, jeder Schritt idempotent)
pub trait PurgePrimitives {
    async fn drain_range(&self, m: &Matcher) -> Result<DrainStats>;                   // Memtable-Einträge im Umfang verwerfen
    async fn seal_wal_at(&self) -> Result<SealPoint>;                                 // aktives Segment an Seq-Grenze versiegeln
    async fn drop_wal_until(&self, f_seq: Seq) -> Result<WalFloor>;                   // Segmente ≤ F_seq als Ganzes entfernen
    async fn rewrite_without(&self, f: FileId, m: &Matcher, reserve: DiskReserve) -> Result<RewriteOutcome>;
    async fn unlink_verified(&self, f: FileId) -> Result<()>;                         // Manifest-Swap dauerhaft → F-Slot nullen → unlink
}
```

`TenantScopedStorage<S>` bindet `TenantTag` fest und baut Schlüssel nur aus typisierten Tags (`KeyBuf::new`); roher Zugriff auf `S` bleibt crate-privat (T-ARCH-2).

## 5. Implementierungsvorgaben

|ID|Vorgabe|Quelle|Test|
|---|---|---|---|
|ST-01|Schlüsselraum `region ‖ tenant_tag ‖ coll_tag ‖ cluster_tag ‖ doc_tag ‖ family ‖ sub(u32) ‖ !commit_seq(u64)` (78 B). Keine Textpräfixe, keine Klartext-Mandanten-/Collection-/Dokumentkennung, keine Region `LOC`.|9.1, D-31, D-09, D-70|T-KEY-1, T-RAW-4|
|ST-02|Pflichtverschlüsselung: kein `Option<KeyManager>`, kein Klartextmodus, kein `DurabilityMode::{WalNoHmac, MemoryOnly}`. Block-AEAD jeder Datei mit **zufälligem Dateischlüssel** (`FileKeyring`), Steuerdateien mit abgeleitetem `K_file`.|D-05, D-32, D-60, D-85, HR-13|T-DEF-1, T-ENC-1..3|
|ST-03|WAL: Frame `len ‖ seq ‖ ct ‖ aead_tag ‖ mac`; HMAC-Kette `K_wal`; Nonce `0x00000000 ‖ u64(Framezähler)`; Recovery verwirft ab dem ersten fehlerhaften Frame **nur**, wenn der Rest der Datei Nullen ist (Torn Write); sonst `Corrupt` und Start verweigert. Kein Downgrade-Parameter.|10.4, F.4, D-05|T-WAL-1|
|ST-04|Der WAL wird **nie** umgeschrieben. Purge: Memtable-Einträge verwerfen, Memtable flushen, aktives Segment an Sequenzgrenze versiegeln, alle Segmente bis zum Flush-Punkt als Ganzes entfernen (Dateischlüssel vernichten, `unlink`); `WalFloor { seq, tail_mac, removed_file_ids }` im Manifest hält die Kettenfortsetzung; I-3 gilt auf **Record-Ebene**.|D-73, 11.4, F.4|T-WAL-2, T-ISO-1|
|ST-05|Gruppen-Commit (zwei `fsync` je Gruppe): Leader sammelt Batches, prüft `ReadGate` für **alle** Schlüssel jedes Batches und weist Schreibzugriffe in gegatete Bereiche mit `Erased`/`Restricted` ab; ruft `RecordSealer::make_slots_durable()` (Tresor-`sync_data`) **vor** den WAL-Frames; dann WAL + `sync_data`; danach Memtable und Sichtbarkeit. Commit hält die Commit-Barriere **shared**.|D-57, I-1, I-9, 5.5, 14.2|T-GATE-1, T-VAULT-3, T-CONC-1|
|ST-06|SSTable nach F.5: Datenblöcke (nominal 32 KiB), Indexblock, Bloomblock (10 Bit/Schlüssel über das Präfix `tenant‖coll‖cluster‖doc_tag`), Statistik, 64-Byte-Footer. Zielgröße Level ≥ 1: 32 MiB. Tombstone = `vlen = 0`.|F.5, D-76|T-FMT-1, BM-ERASE-WA|
|ST-07|`rewrite_without` kopiert Nicht-Ziel-Records als **byteidentische Envelopes** in eine neue Datei mit **neuem Dateischlüssel**; Schreiben, `sync_data`, Manifest-Eintrag (`fsync`), `F`-Slot der Altdatei nullen (`FileKeyring::destroy`, `fsync`), dann `unlink` und Verzeichnis-`fsync`. Fehler im Rewrite lässt die Altdatei unverändert.|D-85, I-3, I-17, 11.4|T-CRASH-2, T-CRASH-6, T-FK-2|
|ST-08|`F`-Slot einer Datei dauerhaft **vor** dem ersten dauerhaften Block (I-9b); beim Ausscheiden nach dauerhaftem Manifest-Swap, **vor** `unlink` (I-17). Recovery: Datei ohne Manifest-Eintrag ⇒ Slot nullen, Datei entfernen; `F`-Slot ohne Datei ⇒ nullen.|D-85, FM-14|T-FK-1, T-CRASH-6|
|ST-09|Verifikation vor `unlink`: neue Datei lesen und gegen den Matcher prüfen (MK1-Teil); `Pending(LayerFailed)` bei Treffer.|11.4|T-RAW-1|
|ST-10|Kompaktion prüft pro Block `purge_waiting` und **bricht kooperativ ab** (Teilergebnis verwerfen, nicht committen); keine Hintergrundoperation hält eine wartende Löschung länger als 1 s auf. Kompaktion verwirft Bereiche der Variante `Erased` beim Schreiben der Ausgabe, **nie** `Restricted`.|D-78, I-16, I-18|T-GATEGROUP-1, T-COMPACT-RESTRICT-1, BM-PURGE-WAIT|
|ST-11|Kompaktions-Retention: Versionen oberhalb `min_read_snapshot` **und** älter als `history_horizon` (Standard 24 h) entfernen; ohne offene Snapshots gilt `current_seq` (kein Sentinel). Das ist **keine** Löschaussage.|9.5, D-51, I-12|T-VER-1, T-SNAP-1|
|ST-12|Ein Fehler aus `sync_data`/`sync_all`/`sync_dir` setzt `Poisoned` für den Store: alle mutierenden Operationen liefern `Io`; Neuöffnen = Recovery; nie Wiederholung auf demselben Handle.|D-55, I-10, FM-5|T-FSYNC-1|
|ST-13|Plattenreserve: bei freiem Platz `< max(1 GiB, 2 × größte Kandidatendatei)` Schreibzugriffe mit `Limit::DiskReserve` abweisen; `erase` bis `Gated` läuft immer; der Rewrite reserviert vorab.|D-65, FM-3|T-CRASH-3, T-OPS-2|
|ST-14|Dateizugriff **ausschließlich** über `Vfs`: kein `std::fs`, `tokio::fs`, `memmap2`; Lesen über `read_at`. Kein `mmap_readonly`.|D-04, D-94|T-LINT-2|
|ST-15|`TenantScopedStorage` ist der einzige Zugang zum rohen Store (AR-6); Mandantenisolation: fremde Mandantenbereiche sind per Konstruktion nicht adressierbar.|AR-6, T-ISO-1|T-ARCH-2|
|ST-16|`ErasureLayer` `doc_store`: `resolve` liefert Bereiche (Dokument: bis `doc_tag`; Subject: ein Bereich je Collection; Collection: bis `coll_tag`; Mandant: bis `tenant_tag`); `key_coverage = full`; `witness` führt MK1 (Struktur: Manifest, SSTable-Indexblöcke, Bloom) und MK2 über `tmp/`, WAL, SSTables aus.|8.8, 9.1, D-69, 12.1|T-ISO-1, T-RAW-1, T-KEYCOV-1|
|ST-17|Mandanten-/Collection-Ziele: Dateien, die **ausschließlich** Zielbereiche enthalten, werden ohne Rewrite entfernt (Dateischlüssel vernichten, `unlink`); Randdateien werden umgeschrieben.|11.4, 11.10|T-TENANT-1, BM-ERASE-WA|
|ST-18|Start: Manifest-Log lesen, WAL-Recovery (`WalFloor`-Gleichheit `prev_mac == tail_mac` gegen den **letzten** Eintrag prüfen), Waisen in `tmp/` entfernen, Sequenz `next_seq = max(current_seq, ledger_floor) + 1` setzen. `open` bei 10 GiB ≤ 2 s ohne Argon2.|14.1, I-20, BM-OPEN|T-WAL-2, BM-OPEN|
|ST-19|Alle Hintergrundtasks (Flusher, Flush, Kompaktion) besitzen Stop-Flag und `JoinHandle`; `close()` beendet sie geordnet; `Drop` blockiert nie.|5.4, 14.3|T-CANCEL-1|
|ST-20|Keine Features außer Dev-Hooks; `test-hooks` nur als Dev-Feature, kein Selbst-Dev-Eintrag zum Aktivieren (AR-14); `fault-injection` ersetzt durch `testkit::FaultVfs`.|D-84, D-97|T-CFG-1, T-ARCH-6|

## 6. Rückbau und Migration

|Position|Maßnahme|WP|Nachweis|
|---|---|---|---|
|`lsm/config.rs:44,58` (`contextra_ports::license::FeatureRing`)|entfernen (Lizenzschicht D-23); `DurabilityMode::validate_against_features` entfällt mit den Modi|WP-0.3|`check --locked` grün|
|`wal/replay.rs:158,210`, `memmap2`|auf `Vfs::read_at` umstellen|WP-0.3|G-A|
|`Option<Arc<KeyManager>>`, `encryption_passphrase: None`, Klartext-Fallbacks|durch Pflicht-`FileKeyring`/`EncKey` ersetzen|WP-3.3|Grep `Option<.*KeyManager>` = 0|
|`allow_legacy_integrity_key_fallback`, Feature `legacy-wal-key`, `DurabilityMode::{WalNoHmac, MemoryOnly}`|entfernen (28 Treffer in 11 Dateien inkl. Fuzz-Ziele)|WP-1.2, WP-3.3|T-DEF-1, T-CFG-1|
|`encryption-at-rest`-`cfg` (11 Treffer), `#[allow(unexpected_cfgs)]`|entfernen|WP-1.2|T-CFG-1|
|`tenant_codec.rs` (`TenantKeyCodec`, `encode_graph_key`, `DocId`)|durch `tenant_scoped` + `types::key` ersetzen|WP-3.3|T-KEY-1, T-ISO-1|
|171 direkte Dateizugriffe|auf `Vfs` (async) umstellen; Aufteilung nach `wal`, `sstable`, `manifest`, `compaction`|WP-1.4|T-LINT-2|
|`memory-only-storage`, `engine/memory_only.rs`|entfernen; Tests nutzen `testkit` (`FaultVfs`, in-memory `Vfs`)|WP-1.2|T-CFG-1|
|`block-cache-v2`, `sieve-cache`, `lru` oder `quick_cache`|**eine** Cache-Implementierung behalten|WP-1.2|`cargo tree`|
|`merge_operator.rs` (`MergeOperator`-Trait), `WasmMergeOperator`-Bezüge|entfernen (D-24, D-84); Merge fest verdrahtet|WP-1.2|Grep|
|`uuid`, `rand`|IDs über `IdGen`, Zufall über `Rng`/OS-CSPRNG (P7)|WP-3.3|`cargo tree`|
|`util.rs`, `system_pressure.rs`, `fsync.rs`|auflösen/entfernen (D-90, P8)|WP-1.5|T-ARCH-7|
|Compaction-Retention `u64::MAX`-Sentinel (F-07, ältere Stelle `compaction.rs:228`)|bereits im HEAD der SPEC beseitigt; im `mvcc`-Fix mitprüfen|WP-3.3|Grep|
|Vorhandene 126 Testdateien|auf neues Schlüsselformat und `ctx_test`-Attribute migrieren; als Orakel behalten (SPEC 3.1)|WP-3.3|T-TRACE-1|

## 7. Arbeitspakete (Reihenfolge)

1. **WP-0.3 (G-A):** Lizenz-Rest, `mmap_readonly`-Aufrufer ablösen, `Cargo.lock` erneuern.
2. **WP-1.2 / WP-1.4:** Features und Parallelpfade entfernen, auf `Vfs` umstellen.
3. **WP-3.3 (rot):** Tests T-ISO-1, T-WAL-1/2, T-ENC-1..3, T-DEF-1 gegen das neue Format. **(grün):** Schlüsselraum, Envelope-Durchreichung, Block-AEAD, WAL ohne Rewrite mit `WalFloor`, Pflichtverschlüsselung, Delete-Race-Fix (siehe Engine, F-08).
4. **WP-3.4 (G0-Prototyp):** Schlüsselraum mit D-70/D-71, `rewrite_without`; Messung BM-ERASE-WA/-WA-TEXT auf 1/10 GiB für Dokument, Subject (10³ Dokumente), Collection. Schwellen D-76: `bytes_written ≤ 3 × Zielbytes + L_t × 32 MiB`, absolut ≤ 512 MiB je Einzeldokument-Erase, Steigung `bytes(10 GiB)/bytes(1 GiB) ≤ 1,25`. Bei Verfehlung Fallbackleiter 11.9 (Stufe 1: Dateigrenzen an Clustergrenzen; Stufe 2: partitionierter LSM je Mandant; Stufe 3: Schlüsselvernichtung als Primärmittel; Stufe 4: Speicheradapter). **Die Fassung dieser Datei gilt für Stufe 0.**
5. **WP-4.2:** Kompaktions-Kooperation (`purge_waiting`), Barrieren aus `ports` (L-7), Gate-Verwurf.

## 8. Tests und Benchmarks

|ID|Prüft|
|---|---|
|T-ISO-1, T-DIFF-1|Nicht-Ziel-Records bitgleich (Lab-Differenz bis auf Manifest-/Seq-/Hüllenänderungen)|
|T-KEY-1, T-ISO-2|Schlüsselkodierung, Matcher-Bereiche|
|T-GATE-1, T-GATEGROUP-1, T-COMPACT-RESTRICT-1|Gate/Commit-Abweisung, Kooperation, `Restricted` unangetastet|
|T-WAL-1, T-WAL-2|korrupter Frame; WAL nie umgeschrieben, Fortsetzung nach `WalFloor` (auch mit mehreren Floors, L-12)|
|T-ENC-1..3, T-DEF-1, T-FK-1..3|Pflichtverschlüsselung, Dateischlüssel (Anlage vor erstem Block, Vernichtung nach Swap vor `unlink`)|
|T-CRASH-1..6|Absturz an jedem `write`/`fsync`/`rename`/`unlink`; Waisen; Platte voll|
|T-FSYNC-1, T-CANCEL-1, T-LOCK-1|`fsync`-Fehler fatal; Cancel-Safety; Lock|
|T-SCAN-1/2, T-VER-1, T-SNAP-1|Scan überspringt gegatete Elemente; Zeitreise; Snapshot-Widerruf|
|T-RAW-1..6, T-KEYCOV-1|MK1/MK2 je Struktur (WAL, SSTable, `tmp/`)|
|T-TENANT-1, T-SUBJ-1|Offboarding, Subject-Erase|
|T-DIFF-1 (Differenz), T-DIFF-Referenz|Differential-Tests gegen `ReferenceModel`|
|T-CONC-1|`loom`: Commit-Barriere, Gate, Purge-Barrier|
|BM-WRITE, BM-GET|≥ 20 k Dok./s (1 KiB), `get` p99 ≤ 200 µs; Regression ≤ 10 %|
|BM-ERASE-WA, BM-ERASE-BATCH|Gate G0; Dateien/Bytes je Purge; 1000 Erases im Fenster|
|BM-WRITE-DURING-ERASE, BM-PURGE-WAIT|Durchsatz ≥ 70 % der Ruhe; Wartezeit p99 ≤ 1 s|
|BM-OPEN|`open` ≤ 2 s bei 10 GiB|
|Abdeckung|Zeilen ≥ 90 % (Ratchet)|

## 9. Abnahme

Definition of Done (Index Abschnitt 8) und: `cargo check --workspace --all-targets --locked` grün (G-A); Grep `std::fs|tokio::fs|memmap2|File::open|File::create|OpenOptions` in `src` = 0; Grep `Option<.*KeyManager>|legacy|WalNoHmac|MemoryOnly` = 0; Produktzeilen gegenüber der Basislinie gesunken (Beitrag zum 30-%-Budget 5.8); G0-Messprotokoll liegt vor und legt die Fallbackstufe fest; T-CRASH-Matrix grün.

## 10. Offene Punkte

- **L-13 (neu):** AAD des Records bindet `key_bytes` **inklusive** `!commit_seq` (F.6). Die Commit-Sequenz wird aber erst vom Gruppen-Commit-Leader vergeben (5.5: „Envelope sealen“ vor der Store-Transaktion). Vorschlag [E]: Der Store stellt `reserve_seq()` bereit; `engine` versiegelt mit der reservierten Sequenz; der Leader committet in Reservierungsreihenfolge, nicht belegte Reservierungen werden als Lücke mit `SeqMark` behandelt. Alternative: Sealing im Leader (serieller Engpass). Klärung vor WP-3.3.
- L-7 (Barrieren-Ports), L-9 (`make_slots_durable`), L-12 (mehrere `WalFloor`), L-8 (`META`-Schlüssel).
- `crc32fast`: Sinn neben AEAD-Tags prüfen (P8); vermutlich nur für Torn-Write-Heuristik (nicht tragend, da Rest-Nullen-Regel ST-03).
- `system_pressure.rs` (Systemdruck-Monitor) gehört nicht zur SPEC; Entscheidung im Rückbau-PR.
