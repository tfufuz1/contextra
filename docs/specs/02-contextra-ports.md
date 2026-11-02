# `contextra-ports` – Implementierungsspezifikation

|Feld|Wert|
|---|---|
|Zweck-Satz (D-97)|Schnittstellen (Traits) zwischen den Schichten: Speicher, Index, Dateisystem, Zeit, Zufall, Löschschicht.|
|Ring|0|
|Erlaubte Workspace-Abhängigkeiten|`types` (SPEC 5.3)|
|Stufe / Gates|S0; G-B, G0|
|Arbeitspakete|WP-1.1 (Altports), WP-1.4 (ein `Vfs`), WP-3.1 (Löschports, D-96)|
|SPEC-Anker|8.2, D-03, D-04, D-34, D-53, D-88, D-96, 5.4, 11.3, 11.8, AR-10|

## 1. Ist-Zustand (HEAD `1454bbec0`) [V]

|Punkt|Befund|
|---|---|
|Umfang|2.668 Zeilen `src`; Dateien `clock.rs`, `id_gen.rs`, `metrics.rs`, `rng.rs`, `storage.rs`, `text_index.rs`, `vector_index.rs`, `vfs.rs` (Soll) sowie `embedding.rs`, `reranker.rs`, `query_rewriter.rs`, `contract_tests.rs` (nicht Soll)|
|Vorhandene Traits|`Clock`, `IdGen`, `MetricsSink`, `Rng`, `StorageEngine`, `TextIndex`, `VectorIndex`, `Vfs`, `VfsFile`; nicht Soll: `TextEmbeddingEngine`, `EmbeddingProvider`, `TextGenerator`, `LlmTextGenerator`, `LlmTextGeneratorStreaming`, `QueryRewriter`, `Reranker`, `SegmentSynthesizer`, `HybridSearchProvider` (prüfen)|
|Löschports|`ErasureLayer`, `ReadGate`, `RecordSealer`, `FileKeyring`, `LedgerSink` fehlen (F-33)|
|`Vfs`|**synchron** (`std::io::Result`; `read_at`, `write_at`, `set_len`, `len`, `sync_data`, `sync_all`; auf `Vfs`: `open`, `read`, `write_atomic`, `create_dir_all`, `remove_file`, `rename`, `read_dir`, `exists`, `len`, `list_dir`). Einziges `Vfs`-Trait im Repo (F-29).|
|`StorageEngine`|`BoxFuture`-basiert (dyn-fähig), Kommentar „retained for backward compatibility“; trägt `get_at_seq`, `get_tracked` (SSI-Read-Set)|
|Features|`default`, `contract-tests`, `testkit` (zu entfernen)|
|Re-Export|Glob-Re-Export `pub use contextra_types::*` (laut SPEC zu entfernen, D-88/AR-10)|

## 2. Verantwortung und Abgrenzung

`ports` definiert **ausschließlich Traits und die dafür nötigen Parameter-/Ergebnistypen**, die zwischen Schichten fließen. Es enthält keine Implementierung außer trivialen Hilfen (`BoxFuture`-Alias, Fehler-Konvertierung). Es re-exportiert Typen aus `types` nur **einzeln** und nur, wenn sie in Trait-Signaturen vorkommen (AR-10). Contract-Tests gehören ins `testkit` (F-36). `erasure` kennt Speicher, Text und Vektor nur über diese Traits (P12, AR-3).

## 3. Zielstruktur (Module) [E]

|Modul|Trait(s)|Dispatch (D-03)|
|---|---|---|
|`storage`|`StorageEngine`|statisch (`async fn`/`-> impl Future + Send`), Engine generisch|
|`text_index`|`TextIndex`|statisch|
|`vector_index`|`VectorIndex`|statisch|
|`vfs`|`Vfs`, `VfsFile`, `VfsOpenOptions`|dyn (`BoxFuture`)|
|`clock`, `rng`, `id_gen`|`Clock`, `Rng`, `IdGen`|dyn|
|`erasure`|`ErasureLayer`, `LayerScope`, `PurgeCx`, `PurgeOutcome`|dyn (`BoxFuture`)|
|`gate`|`ReadGate`, `GateDecision`|dyn, **synchron** (O(log n), nur Speicher)|
|`keys`|`RecordSealer`, `FileKeyring`, `FileKey`|dyn|
|`ledger`|`LedgerSink`, `LedgerEntry`|dyn|
|`metrics`|`Metrics`, `MetricName` (statische Namen)|dyn|
|`barrier`|`CommitBarrier`, `PurgeBarrier`|dyn (**Entwurf, L-7**)|
|`memory`|`ArenaGuard` (mlock/`MADV_DONTDUMP` für die Vektor-Arena)|dyn (**D-88 (2)**)|

## 4. Schnittstellen

### 4.1 Aus der SPEC fest (8.2, wörtlich)

```rust
trait ErasureLayer: Send + Sync {
    fn id(&self) -> LayerId;
    fn resolve(&self, t: &EraseTarget) -> BoxFuture<Result<LayerScope>>;                    // Umfang bestimmen
    fn on_gate(&self, s: &LayerScope) -> BoxFuture<Result<()>>;                             // Sofortmaßnahmen vor Gated
    fn purge(&self, s: &LayerScope, cx: &PurgeCx) -> BoxFuture<Result<PurgeOutcome>>;
    fn witness(&self, s: &LayerScope, sc: &Scanner<Armed>) -> BoxFuture<Result<LayerWitness>>; // nur lesend
    fn key_coverage(&self) -> KeyCoverage;                                                  // full | unlinked | none
}
trait FileKeyring: Send + Sync {                                                            // D-85
    fn create(&self, kind: FileKind, id: FileId) -> BoxFuture<Result<FileKey>>;
    fn open(&self, id: FileId) -> BoxFuture<Result<FileKey>>;
    fn destroy(&self, id: FileId) -> BoxFuture<Result<()>>;                                 // Slot nullen, sync, Fehlen prüfen
}
```

Weitere Ports laut 8.2: `ReadGate`, `RecordSealer`, `LedgerSink`, `Metrics`, `Clock`, `Rng`, `IdGen`, `Vfs`, `StorageEngine`, `TextIndex`, `VectorIndex`.

### 4.2 Entwürfe [E]

**`Vfs` (löst L-1):**

```rust
pub trait VfsFile: Send + Sync {
    fn read_at<'a>(&'a self, buf: &'a mut [u8], off: u64) -> BoxFuture<'a, Result<usize>>;
    fn write_at<'a>(&'a self, buf: &'a [u8], off: u64) -> BoxFuture<'a, Result<usize>>;
    fn set_len(&self, len: u64) -> BoxFuture<'_, Result<()>>;
    fn len(&self) -> BoxFuture<'_, Result<u64>>;
    fn sync_data(&self) -> BoxFuture<'_, Result<(), SyncError>>;   // SyncError ⇒ Datei fatal (D-55)
    fn sync_all(&self) -> BoxFuture<'_, Result<(), SyncError>>;
}
pub trait Vfs: Send + Sync + 'static {
    fn open<'a>(&'a self, p: &'a Path, o: &'a VfsOpenOptions) -> BoxFuture<'a, Result<Box<dyn VfsFile>>>;
    fn write_atomic<'a>(&'a self, p: &'a Path, d: &'a [u8]) -> BoxFuture<'a, Result<()>>; // tmp, fsync, rename, Verzeichnis-fsync
    fn rename<'a>(&'a self, from: &'a Path, to: &'a Path) -> BoxFuture<'a, Result<()>>;
    fn remove_file<'a>(&'a self, p: &'a Path) -> BoxFuture<'a, Result<()>>;
    fn sync_dir<'a>(&'a self, p: &'a Path) -> BoxFuture<'a, Result<(), SyncError>>;
    fn create_dir_all<'a>(&'a self, p: &'a Path) -> BoxFuture<'a, Result<()>>;
    fn list_dir<'a>(&'a self, p: &'a Path) -> BoxFuture<'a, Result<Vec<String>>>;
    fn exists<'a>(&'a self, p: &'a Path) -> BoxFuture<'a, Result<bool>>;
    fn lock_exclusive<'a>(&'a self, p: &'a Path) -> BoxFuture<'a, Result<DirLock>>;        // D-50
    fn probe(&self, dir: &Path) -> BoxFuture<'_, Result<FsProbe>>;                          // D-56 (Startprobe)
    fn scrub<'a>(&'a self, p: &'a Path) -> BoxFuture<'a, Result<()>>;                       // nur nach D-83
}
```

`SyncError` ist ein eigener Typ, der **nicht** in `Ok` umgedeutet werden kann; ein Handle, das ihn geliefert hat, markiert sich selbst als verbraucht (D-55). Die synchrone Ist-Form (`std::io::Result`) entfällt; ein `block_on` im Bibliothekscode ist verboten (D-03).

**`ReadGate` (11.3, sync):**

```rust
pub trait ReadGate: Send + Sync {
    fn check(&self, k: &KeyBuf) -> GateDecision;                       // Open | Erased | Restricted
    fn check_bound(&self, b: &KeyBound) -> GateDecision;               // Dateigrenzen für Kompaktion/Scan
    fn variant_for_range(&self, r: &KeyRange) -> RangeGate;            // Kompaktion: Erased verwerfen, Restricted nie (I-18)
    fn gated_seq(&self, k: &KeyBuf) -> Option<Seq>;                    // Restore-Guard, Commit-Abweisung
}
```

**`RecordSealer` (D-53):**

```rust
pub trait RecordSealer: Send + Sync {
    fn seal<'a>(&'a self, ctx: &'a SealCtx, key: &'a KeyBuf, plain: &'a [u8]) -> BoxFuture<'a, Result<Envelope>>;  // legt fehlende Slots an
    fn open<'a>(&'a self, key: &'a KeyBuf, env: &'a [u8]) -> BoxFuture<'a, Result<Zeroizing<Vec<u8>>>>;           // fehlt ein Slot: Erased
    fn evict(&self, scope: &KeyRange);                                  // DEK-Cache zeroisieren (D-79)
    fn make_slots_durable(&self) -> BoxFuture<'_, Result<()>>;          // sync_data des Tresors vor WAL-Frames (I-9, L-9)
}
```

**`CommitBarrier`, `PurgeBarrier` (L-7):**

```rust
pub trait CommitBarrier: Send + Sync {
    fn shared(&self)    -> BoxFuture<'_, BarrierShared>;                // Gruppen-Commit-Phase
    fn exclusive(&self) -> BoxFuture<'_, BarrierExclusive>;             // Gate-Gruppe (D-57, D-78)
}
pub trait PurgeBarrier: Send + Sync {
    fn compaction(&self) -> BoxFuture<'_, CompactionPermit>;            // shared
    fn purge(&self)      -> BoxFuture<'_, PurgePermit>;                 // exklusiv
    fn purge_waiting(&self) -> bool;                                    // Kompaktion prüft pro Block (I-16)
}
```

Die Guards sind Eigentümer-Typen (kein Lifetime-Borrow über `.await` einer fremden Task, `await_holding_lock = deny`).

**`LedgerSink` (11.8):** `append(&self, e: &LedgerEntry)`, `read_all()`, `prune(older_than: Timestamp)`; Eintrag `(commitment, range_set, gated_seq, completed_at)` mit Kettenhash und `ledger_mac` (F.9). Ort **außerhalb** des Datenverzeichnisses.

## 5. Implementierungsvorgaben

|ID|Vorgabe|Quelle|Test|
|---|---|---|---|
|PO-01|Genau **ein** `Vfs`-Trait im Repo (hier). Kein zweites Trait gleichen Namens oder gleicher Aufgabe.|D-04, F-13|T-LINT-2|
|PO-02|`StorageEngine`, `TextIndex`, `VectorIndex` sind statisch dispatcht (`async fn` im Trait oder `-> impl Future + Send`). Kalte Ports sind dyn-fähig (`BoxFuture`). Kein `block_on`.|D-03|T-LINT-2, BM-GET|
|PO-03|Kein `pub use contextra_types::*`; Re-Export nur einzeln und nur für Typen in Trait-Signaturen.|AR-10, D-88|T-ARCH-6|
|PO-04|Löschports (`ErasureLayer`, `ReadGate`, `RecordSealer`, `FileKeyring`, `LedgerSink`) und ihre Typen entstehen **vor** jeder Implementierung in `store`, `text`, `vector`.|D-96|T-ARCH-3|
|PO-05|`ErasureLayer::witness` ist rein lesend; `purge` ist idempotent und wiederholbar (jeder Schritt idempotent).|8.2, 11.2, I-15|T-CRASH-1|
|PO-06|`FileKeyring::destroy` nullt den Slot, führt `sync_data` aus und prüft das Fehlen lesend; Fehler ⇒ `Pending` mit Neuöffnen (D-55).|8.2, 11.5|T-FK-2, T-VAULT-1|
|PO-07|`ReadGate::check` ist synchron und führt keinen I/O aus; O(log n).|11.3|BM-GET|
|PO-08|Alle öffentlichen Typen tragen keine Nutzerinhalte in `Debug`/`Display`.|HR-4|T-LOG-1|
|PO-09|Das Trait `Metrics` kennt nur statische Metriknamen (`MetricName`-Enum), keine ID-Labels. (Ist: `MetricsSink`.)|D-64, 14.5|T-OPS-1|
|PO-10|`Rng` trennt „deterministisch (Tests)“ von „OS-CSPRNG für Schlüsselmaterial“; Schlüsselmaterial nimmt nie den injizierten `Rng`.|P7|T-NONCE-1|
|PO-11|Die Vektor-Arena sperrt Speicher über einen Port (`ArenaGuard`), den `engine` mit `sys` implementiert; `vector` hängt weder an `sys` noch an `mvcc`.|D-88 (2), D-63|T-ARCH-3, T-DUMP-1|
|PO-12|Contract-Tests (Ist `contract_tests.rs`, 494 Z.) wandern nach `testkit/conformance` und laufen gegen jede Implementierung.|F-36, D-53|Conformance-Suite|

## 6. Rückbau und Migration

|Position|Maßnahme|WP|Nachweis|
|---|---|---|---|
|`embedding.rs` (245 Z.), `reranker.rs`, `query_rewriter.rs`; `TextEmbeddingEngine`, `EmbeddingProvider`, `TextGenerator`, `LlmTextGenerator(Streaming)`, `QueryRewriter`, `Reranker`|entfernen (D-24)|WP-1.1|Grep = 0|
|`SegmentSynthesizer`, `HybridSearchProvider`|Verbraucher prüfen; ohne Kernkonsument entfernen (P8)|WP-1.1|Grep|
|`contract_tests.rs`, Features `contract-tests`, `testkit`, `default`|nach `testkit`; Features entfernen|WP-1.2|T-CFG-1|
|`pub use contextra_types::*`|durch Einzel-Re-Exporte ersetzen|WP-1.5|T-ARCH-6|
|`Vfs` synchron|auf `BoxFuture` umstellen (L-1); Konsumenten in `durable-fs`, `store`, `vector` folgen|WP-1.4|T-LINT-2|
|`StorageEngine` mit `BoxFuture`|auf `async fn` umstellen (L-2); `get_tracked`/SSI-Read-Set-Pfad erhalten|WP-3.3|BM-GET|
|`MetricsSink`|Umbenennung/Neufassung zu `Metrics` mit `MetricName`|WP-5.2|T-OPS-1|

## 7. Arbeitspakete

1. WP-1.1: Altports entfernen (Konsumenten gleichzeitig bereinigen, sonst bricht der Build).
2. WP-1.4: `Vfs` async; `durable-fs` und `testkit::FaultVfs` ziehen nach.
3. WP-3.1 (rot): Ports als **leere Traits** mit Conformance-Tests im `testkit` (rot, weil es keine Implementierung gibt), dann Signaturen (grün).
4. WP-3.3: `StorageEngine` auf statisch umstellen, zusammen mit `store`.

## 8. Tests und Benchmarks

|ID|Prüft|
|---|---|
|T-LINT-2|kein zweites `Vfs`, kein `block_on`, kein `std::fs`/`tokio::fs` im Store|
|T-ARCH-3, T-ARCH-6|Ring/Matrix; keine Glob-Re-Exporte; Dev-Abhängigkeiten laut Matrix|
|Conformance (`testkit`)|je Port eine Suite: `Vfs`, `StorageEngine`, `TextIndex`, `VectorIndex`, `RecordSealer`, `FileKeyring`, `LedgerSink`, `ReadGate` (D-53)|
|T-FSYNC-1|`SyncError` kann nicht wiederholt werden (Handle verbraucht) – Vertragstest|
|BM-GET|statische vs. dyn-Dispatch auf dem Heißpfad (Begründung D-03)|

## 9. Abnahme

Definition of Done (Index Abschnitt 8) und: `cargo tree -p contextra-ports` zeigt nur `types` und externe Kleinstabhängigkeiten (`futures-util`/`bytes`); Grep `Embedding|Reranker|QueryRewriter|LlmTextGenerator` = 0; ein einziges `trait Vfs` im Repo.

## 10. Offene Punkte

L-1 (async `Vfs`), L-2 (statische Ports), L-7 (Barrieren-Ports), L-9 (`make_slots_durable`). Die Form von `LayerScope`, `PurgeCx` und `PurgeOutcome` legt WP-3.1 mit `erasure` zusammen fest; SPEC 8.2 nennt nur die Namen.
