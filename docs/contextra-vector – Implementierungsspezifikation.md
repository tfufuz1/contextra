# contextra-vector – Implementierungsspezifikation

*Principal-Engineer-Review und Umsetzungsplan, Stand 27.11.2026, Repo `tfufuz1/contextra`, Crate `crates/contextra-vector` (ca. 16,9k LoC src, ca. 29k LoC inkl. Tests).*

## 0. Methodik und Grenzen

- **Durchgeführt:** Klon des Repos, statische Durchsicht von `Cargo.toml`, `lib.rs`, `persistence/{atomic,mmap,header}.rs`, `hnsw/types.rs` (Save-Pfad, Lock-Felder), `diskann/{build,persistence}.rs` (Schreibpfade), Grep-Audits für `unsafe`, `allow(...)`, Lock-Nutzung, fsync/rename.
- **Nicht durchgeführt:** Es ist kein Rust-Toolchain im Container (`cargo` fehlt). Es wurde nichts kompiliert und kein Test ausgeführt. Befunde sind daher als **\[belegt\]** (Code gelesen, Zeile genannt) oder **\[zu verifizieren\]** (aus Grep/Struktur abgeleitet) markiert.
- **Nicht gelesen:** Der Großteil von `hnsw/core_*.rs`, `quantize*`, `acorn/*` und der Tests. Die Algorithmik (Insert, Search, Heuristik) ist **nicht** Gegenstand dieser Spezifikation, sondern Hülle, Durability, Concurrency und Wartbarkeit.

## 1. Zusammenfassung der Kernbefunde

Der Crate hat eine gute Basis: `forbid(unsafe_code)`, deny-Lints für `unwrap`/`indexing`, durchgängig checked arithmetic im mmap-Reader und viele Tests. Die Schwächen liegen in Architektur und Durability, nicht im Algorithmus:

| ID | Schwere | Befund |
| --- | --- | --- |
| F-01 | Hoch | Feature-Modell widerspricht der Klasse-A-Behauptung (optionale Deps werden ungegatet genutzt) |
| F-02 | Hoch | Vier divergente Atomic-Write-Implementierungen, davon eine mit stillschweigend ignorierten Fehlern |
| F-03 | Hoch | HNSW-Datei ohne Integritätsprüfsumme, Formatvarianten werden über Längen-Heuristik erkannt |
| F-04 | Mittel-Hoch | mmap-Lebenszyklus: SIGBUS-Risiko, öffentliche Felder, Schein-`async` |
| F-05 | Hoch | Concurrency: Save hält den Schreib-Mutex während I/O, Lock-Hierarchie nur als Kommentar, Tests maskieren `await_holding_lock` |
| F-06 | Mittel | Panic-Safety des Index-Zustands unter `panic = "unwind"` nicht strukturell abgesichert |
| F-07 | Mittel | Stringly-typed Fehler (`ContextraError::Storage(String)`) |
| F-08 | Mittel | Persistierte Snapshots widersprechen dem Erasure-Versprechen (Blind Spot) |
| F-09 | Mittel | Testlandschaft: Prozess-Artefakt-Namen, Dubletten, keine Crash-Konsistenz-Tests für HNSW |
| F-10 | Niedrig | Doku/Kommentare veraltet und teils falsch |

## 2. Befunde im Detail

### F-01 Feature- und Abhängigkeitsmodell \[belegt\]

- `Cargo.toml` deklariert `contextra-types`, `-ports`, `-mvcc`, `-sys` als `optional = true` hinter `ports-compat`. Der Code nutzt `contextra_types`/`contextra_sys` aber in rund 40 Dateien ohne Gate (z. B. `persistence/mmap.rs`, `error.rs`, `hnsw/types.rs`); nur 3 Stellen sind mit `cfg(feature = "ports-compat")` versehen. \[zu verifizieren per Build:\] Das README-Beispiel `default-features = false, features = ["std"]` kompiliert damit voraussichtlich nicht. Die Aussage „vollständig entkoppelter autarker Baustein, keine internen Pflichtabhängigkeiten“ ist so nicht haltbar.
- Leere Features ohne Wirkung: `serde`, `tracing`, `inspect`, `mmap`, `graph`. `memmap2`, `serde`, `ahash` sind dagegen unbedingte Dependencies.
- README nennt ein Feature `simd`, das nicht existiert. `lib.rs` behauptet Runtime-Dispatch AVX-512 > AVX2 > Skalar, obwohl `contextra-simd` keine Dependency ist und `forbid(unsafe_code)` gilt.
- `lib.rs` behauptet „HNSW-Graphen liegen exklusiv im RAM“, während `persistence/` und `diskann/` Dateien schreiben und mappen.

### F-02 Vier Atomic-Write-Pfade \[belegt\]

Stellen: `persistence/atomic.rs::write_atomic`, `hnsw/types.rs::save_internal` (ca. Z. 930–950), `diskann/build.rs` (ca. Z. 722–727), `diskann/persistence.rs` (ca. Z. 77–84). Zusätzlich existiert `contextra-durable-fs` im Workspace, wird aber nicht genutzt.

Abweichungen:

- `write_atomic`: `let _ = dir.sync_all()` und `if let Ok(dir) = File::open(parent)` verschlucken Fehler. Das verletzt das eigene Prinzip INV-FSYNC-1 („fail-closed“).
- `save_internal`: Open-Fehler des Elternverzeichnisses wird per `if let Ok` verschluckt, ein Rename-Fehler lässt die Temp-Datei liegen.
- Temp-Name `.{file}.{pid}.tmp` ist nur prozess-, nicht threadeindeutig. Zwei parallele Writer im selben Prozess auf dasselbe Ziel überschreiben sich gegenseitig die Temp-Datei.
- Nach einem fehlgeschlagenen `fsync` gibt es kein Poisoning. Retry nach `fsync`-Fehler ist unsicher (Linux kann Dirty-Pages verwerfen und beim zweiten `fsync` Erfolg melden).
- Keine Aufräumroutine für verwaiste `.tmp`-Dateien nach Crash.

### F-03 Dateiformat ohne Integrität \[belegt/zu verifizieren\]

- Prüfsummen (blake3, keyed) gibt es nur im DiskANN-Pfad (`diskann/integrity.rs`). Für das HNSW-Format sind im Grep von `persistence/` und `hnsw/types.rs` keine CRC/Hash-Treffer vorhanden. Ein Bitflip im Vektorbereich wird von `MmapIndex::open` nicht bemerkt, geprüft werden nur Größen und Record 0.
- `MmapIndex::open` rät die Headergröße (`HnswHeader::SIZE`, sonst 76, sonst 64) und `try_from_bytes` entscheidet über optionale Felder per `bytes.len() >= SIZE`. Das ist implizite Versionierung über Längen und bricht bei jeder Erweiterung.
- Die DocId-Breite (64/128 Bit) ist ein Compile-Time-Feature und steht nicht im Header. Ein Mismatch wird nur indirekt über Record-0-`vector_offset` erkannt.
- Es gibt keinen Commit-Marker/Trailer: eine abgeschnittene Datei mit intaktem Header ist nur über Offsetprüfungen erkennbar.

### F-04 mmap-Lebenszyklus \[belegt\]

- Truncation der Datei durch Dritte während des Mappings führt zu SIGBUS (Prozessabbruch, unabhängig von `panic = unwind`). Der Kommentar „mmap-Reads überleben POSIX file replace“ gilt für Rename, nicht für In-Place-Truncate.
- `MmapIndex` hat `pub`-Felder (`mmap`, `header`, `path`) und bricht die Kapselung.
- `open_async` ist synchron (blockiert den Executor) und trägt nur den Namen.
- `get_connections` läuft pro Aufruf O(Layer) über das Längenpräfix und allokiert pro Hop einen `Vec<u32>`. Nachbar-IDs werden nicht gegen `node_count` validiert.

### F-05 Concurrency \[belegt/zu verifizieren\]

- `save_sync` hält `write_mutex` über den gesamten Schreibvorgang inklusive `sync_all` und Rename. Alle Writer stehen währenddessen still.
- Es gibt mindestens acht RwLocks im Cold-State (`quantizer`, `sq8_bias`, `mmap_index`, `seq_log`, `deleted_nodes`, `erased_docs`, `clock`, `metrics_sink`) plus `nodes`, `doc_to_node`. Die Hierarchie (`write_mutex -> entry_point -> nodes/doc_to_node/deleted_nodes`) steht nur in Kommentaren (`hnsw/mod.rs`, `deletion.rs`). Das Verhältnis von `rebuild_mutex` (`core_rebuild.rs:42`) zu `write_mutex` (`:642`, `:861`) ist dort nicht dokumentiert. \[zu verifizieren\]
- `hnsw/tests.rs` und `persistence/tests.rs` erlauben `clippy::await_holding_lock`; der Lint wird in Tests also bewusst abgeschaltet.
- Search hält `nodes.read()` plus `deleted_nodes.read()` plus `mmap_index.read()` während der Traversierung (`vector_index_impl.rs` Z. 88–90, 148–153). Lange Suchen blockieren Writer; mit parking\_lot-Fairness entstehen Latenzspitzen. \[zu verifizieren per Benchmark\]
- Loom deckt nur den Quantizer ab (`loom_quantizer_*`), nicht Insert/Search/Rebuild/Save.

### F-06 Panic-Safety des Zustands \[zu verifizieren\]

Das Release-Profil nutzt `panic = "unwind"`, parking\_lot vergiftet Locks nicht. Ein Panic (z. B. OOM-nahe Allokation, Bug in einem Dependency) mitten in einer mehrstufigen Graphmutation hinterlässt halb verlinkte Nachbarn ohne Marker. Es gibt `tests/poisoning.rs`; ob es diesen Fall strukturell abdeckt, ist zu prüfen.

### F-07 Fehlermodell \[belegt\]

Die Persistenz meldet überwiegend `ContextraError::Storage(String)`. Aufrufer können Korruption, I/O, Formatversion und Konfigurationsfehler nicht typisiert unterscheiden (nötig für die Fallback-Entscheidung „neu aufbauen vs. abbrechen“).

### F-08 Erasure vs. Persistenz (Blind Spot)

RAM-Zeroize ist implementiert (INV-VEC-ZEROIZE-1). Ein bereits geschriebener Snapshot enthält gelöschte Vektoren aber bis zum nächsten Save weiter, und die alte Generation wird nach dem Rename nur entlinkt. Page-Cache und SSD-Remanenz sind nicht adressiert.

### F-09 Tests \[belegt/zu verifizieren\]

- Prozess-Artefakt-Namen: `campaign_j23_vector_index.rs`, `campaign_j24_test.rs`, `campaign_j41_vector.rs`, `k02_snapshot_rebuild_determinism.rs`.
- Wahrscheinliche Dubletten: `rabitq_recall_calibration.rs` / `rabitq_recall_calibration_test.rs`, `distance_determinism.rs` / `distance_determinism_test.rs`, `nan_validation_policy.rs` / `nan_validation_test.rs`, `loom_quantizer_test.rs` / `loom_quantizer_race_test.rs`, `hnsw_rebuild_connectivity_test.rs` / `connectivity_regression.rs`.
- Crash-Fault-Injection existiert nur für DiskANN (`diskann_fault_injection_test.rs`), nicht für den HNSW-Save-Pfad.

### F-10 Dokumentation

`lib.rs`-Kommentare (siehe F-01), `#[cfg_attr(test, allow(...))]` auf einem `pub use` in `lib.rs` ist wirkungslos, der Header-Kommentar „SIMD-Dispatch“ ist veraltet, `#[allow(deprecated)]` auf mehreren Re-Exports (`hnsw_scanner`, `hnsw_purger`, …) ohne Entfernungsplan. Einige `#[allow(clippy::unwrap_used, …)]` stehen auf Items außerhalb klar erkennbarer Test-Module (`partial_rebuild.rs:172`, `quantize/mod.rs:525/529`, `compute_pool.rs:161`, `diskann/types.rs:59`) \[zu verifizieren, ob `#[cfg(test)]`\].

## 3. Zielinvarianten

| Invariante | Aussage |
| --- | --- |
| INV-VEC-ATOMIC-1 | Nach jedem Crash existiert am Zielpfad entweder die vollständig alte oder die vollständig neue, valide Generation, nie ein Mischzustand. |
| INV-VEC-FSYNC-1 | Jeder Fehler aus `fsync`, `rename` oder Verzeichnis-Sync wird propagiert. Nach einem fsync-Fehler verweigert der Index weitere Saves (Poisoned) bis zum Neuladen. |
| INV-VEC-INTEGRITY-1 | Jedes Byte einer persistierten Datei ist durch eine Prüfsumme abgedeckt. Jeder Einzelbit-Flip wird im Modus `Verify::Full` erkannt. |
| INV-VEC-NOPANIC-1 | Kein Pfad aus Eingabedaten (Datei, API-Argumente) kann panicken oder abort auslösen (außer externer SIGBUS-Fall, siehe WP-4, der dokumentiert und per Modus vermeidbar ist). |
| INV-VEC-LOCK-1 | Lock-Reihenfolge ist im Code kodiert und in Debug-Builds geprüft, nicht nur kommentiert. |
| INV-VEC-NOAWAIT-1 | Kein Lock-Guard lebt über einen `.await`, auch nicht in Tests. |
| INV-VEC-AUTARK-1 | `cargo check -p contextra-vector --no-default-features --features std` baut ohne Workspace-Crates. |
| INV-VEC-REPAIR-1 | Eine unterbrochene Graphmutation markiert den Index als `needs_repair`; Lesen/Schreiben liefern `VecError::Poisoned`, bis `repair()` oder Neuladen erfolgt. |

## 4. Work Packages

Reihenfolge: WP-1 → WP-2 → WP-6 → WP-3 → WP-4 → WP-5 → WP-7 → WP-8 (WP-8 läuft begleitend, die CI-Gates aus 4.1 gelten ab Tag 1).

### WP-1 Feature- und Abhängigkeitsmodell (F-01, F-10)

**Ziel:** Der Crate ist wirklich autark. Die Workspace-Integration ist ein Adapter.

**Spezifikation**

1. Neuer eigener Fehlertyp `VecError` (siehe WP-6) und eigene Kern-Typen (`DocId`, `TxId`, `Clock`, `MetricsSink`-Traits liegen bereits in `types.rs`/`contract.rs`, prüfen, dass sie nicht aus `contextra_*` re-exportiert werden).
2. Alle `use contextra_*` verschwinden aus den Kernmodulen. Sie wandern nach `src/compat/` (`#[cfg(feature = "ports-compat")]`): `impl From<VecError> for ContextraError`, Adapter für `contextra_ports::{Clock, MetricsSink, VectorIndex}`, Anbindung an `contextra_mvcc::SequenceLog` (Der Index nutzt `SequenceLog` in `ColdState`. Entweder ersetzt durch Trait `SequenceSource` im Kern oder das Feld wird in `compat` gekapselt.)
3. Features:
   - `std` (default), `mmap` (aktiviert `memmap2` als optionale Dep und den mmap-Pfad), `serde` (optional Dep), `tracing` (optionale Dep, sonst No-op-Makro-Shim), `ports-compat`, `docid-128`, `experimental-*`, `partial-index-rebuild`.
   - Leere Features (`inspect`, `graph`) entfernen oder implementieren; `simd` in der Doku entfernen oder real an `contextra-simd` anbinden (Adapter `DistanceFn`, siehe README-Muster).
4. `default = ["std"]`. Der Workspace aktiviert `ports-compat` explizit in den abhängigen Crates (`contextra-db`, `contextra-engine`, …).
5. `#![doc = include_str!("../README.md")]`, damit README-Beispiele als Doctests laufen. `lib.rs`-Kopfkommentare korrigieren (RAM/mmap, SIMD).

**Akzeptanzkriterien**

- `cargo hack check -p contextra-vector --each-feature` und `--no-default-features --features std` sind grün.
- `grep -r "contextra_" src/ | grep -v compat/` liefert nichts (CI-Check).
- README-Doctest läuft mit `default-features = false`.

### WP-2 Einheitliche Durable-Write-Primitive (F-02)

**Ziel:** Genau eine Stelle führt `create → write → fsync → rename → fsync(dir)` aus.

**Spezifikation** (`src/persistence/durable.rs`, intern):

```rust
pub struct AtomicFileWriter { /* tmp: PathBuf, file: Option<File>, dest: PathBuf, committed: bool */ }

impl AtomicFileWriter {
    /// create_new(true); Name: ".{stem}.{pid}.{seq}.{rand64}.tmp" (seq = process-wide AtomicU64)
    pub fn create(dest: &Path) -> Result<Self, VecError>;
    pub fn commit(self) -> Result<CommitReceipt, VecError>; // sync_all(file) -> rename -> open(parent)?.sync_all()?
}
impl std::io::Write for AtomicFileWriter { /* delegiert an file */ }
impl Drop for AtomicFileWriter { /* wenn !committed: remove tmp (best effort, Fehler nur loggen) */ }

pub fn sweep_stale_tmp(dir: &Path, stem: &str) -> Result<usize, VecError>; // beim Open aufrufen
```

Regeln:

- `File::open(parent)` und `sync_all()` des Verzeichnisses sind auf Unix **fatal**. Auf Windows ist der Verzeichnis-Sync ein dokumentiertes No-op (Plattform-Matrix in der Doku).
- Es gibt einen `DurabilityState` (`AtomicBool`) pro Index. Ein Fehler aus `sync_all`/Dir-Sync setzt ihn auf `Poisoned`, danach liefert `save` sofort `VecError::DurabilityPoisoned`. Kein automatischer Retry.
- Alle Schreiber (`hnsw::save_internal`, `diskann::build`, `diskann::persistence`, `write_atomic`) werden darauf umgestellt. `write_atomic` bleibt als dünner Wrapper `#[deprecated]`.
- Klassenregel: `clippy.toml` `disallowed-methods` für `std::fs::rename`, `File::sync_all`, `File::sync_data` außerhalb von `durable.rs` (per `#[allow]` nur dort).
- Optional (Entscheidung offen, siehe Abschnitt 7): Adapter-Feature `durable-fs`, das `AtomicFileWriter` auf `contextra-durable-fs` abbildet, statt die Logik zu duplizieren.

**Akzeptanzkriterien**

- Property-/Fault-Test (WP-8): Fehlerinjektion an jedem der Schritte (`write`, `sync_all`, `rename`, `dir open`, `dir sync`) liefert `Err` und hinterlässt entweder die alte Datei oder keine Temp-Datei.
- Zwei Threads schreiben parallel auf denselben Pfad: kein Fehler durch Temp-Kollision, Endzustand ist eine der beiden Generationen.

### WP-3 Dateiformat v3 mit Integrität (F-03)

**Ziel:** Explizit versioniertes, prüfsummengesichertes, abwärtskompatibel lesbares Format.

**Layout (little-endian, Offsets 64-Byte-aligned für Vektordaten):**

```
[Header 128 B]
   magic u32 | version u16 | min_reader_version u16 | flags u32 | required_flags u32
   dimension u32 | m u16 | metric u8 | quant u8 | node_count u64 | entry_point i64 | last_tx_id u64
   section_count u16 | section_table[offset,len,kind,hash16] (feste Obergrenze, z. B. 8)
   header_crc32c u32 (über alle vorherigen Header-Bytes)
[Section NODES] [Section VECTORS] [Section CONNECTIONS] [Section CALIBRATION]
[Trailer 48 B]  commit_magic u64 | file_len u64 | blake3_full[32]
```

flags: `bit0=quantized`, `bit1=docid128`, `bit2=has_sq8_bias`. Unbekannte Bits in `required_flags` führen zu `VecError::Format(UnsupportedFlags)`.

Regeln:

- **Writer** schreibt immer v3; **Reader** liest v1/v2 über getrennte, explizite Parser (feste Größen pro Version, **keine** Längenraterei wie `76`/`64`).
- `Verify::Header` (O(1)): Magic, Version, Header-CRC, Trailer-Magic und `file_len == tatsächliche Länge`, Section-Table konsistent (Bounds, keine Überlappung, Alignment), DocId-Breite gegen Compile-Feature. Default für mmap.
- `Verify::Full` (O(n), gestreamt): `blake3_full` plus Section-Hashes. Default beim Laden in den RAM, in `repair()` und im CLI/Inspect.
- Neighbor-IDs: `Verify::Full` validiert `id < node_count` und Layer-Zählung. Im Header-Modus wird beim Zugriff validiert (Fehler zählen, Nachbar überspringen, Metrik `corrupt_neighbor_skipped`).
- CONNECTIONS erhält eine **Layer-Offset-Tabelle pro Node** (`u32 × (layers+1)`), damit `get_connections(layer)` O(1) ist und ohne `Vec`-Allokation als Iterator über `&[u8]` arbeitet.
- Goldene Fixtures v1/v2/v3 (je Quantized/Unquantized, 64/128-Bit DocId) unter `tests/fixtures/` sind eingecheckt und werden bei jedem Lauf geladen.

**Akzeptanzkriterien**

- Property-Test: Für jede Fixture-Datei führt **jeder** einzelne Bitflip zu `Err` in `Verify::Full` (stichprobenartig 10k Positionen in CI, vollständig nightly).
- Truncation an jeder Byte-Grenze → `Err(Corrupt::Truncated)`, nie Panic.
- Fuzz-Target `fuzz_hnsw_persistence` mit Fixtures als Seeds, 0 Crashes über 10 Minuten in CI.

### WP-4 mmap-Sicherheit und Reader-Kapselung (F-04)

1. `MmapIndex`-Felder privat. Zugriff über `header()`, `node(i)`, `vector(i) -> &[u8]`, `connections(i, layer) -> ConnIter<'_>`.
2. `open_async` entfällt (Deprecated-Shim, der synchron delegiert und das in der Doku sagt). Ein echtes Async-Open gehört in einen Adapter-Crate, der Tokio kennt.
3. `OpenMode::{Mmap, ReadToHeap}`: `Mmap` ist schnell, `ReadToHeap` vermeidet SIGBUS-Risiko (Netzwerk-FS, Dritte, die schreiben). Dokumentierter Vertrag: Dateien sind nach Commit immutable; Writer ändern nie in place (gewährleistet WP-2).
4. Optionaler Shared-Advisory-Lock beim Mapping über `contextra-sys` (Feature `mmap`), da der Crate selbst `forbid(unsafe_code)` hat. Beim Fehlen des Locks Warnung, kein Abbruch.
5. `MmapIndex::open` ruft `sweep_stale_tmp` nicht selbst auf (Reader schreiben nie); das macht `HnswIndex::open` explizit.

**Akzeptanz:** Test, der die gemappte Datei per Rename ersetzt: Reader liest weiter konsistente Alt-Generation. Doku-Test für den Truncate-Vertrag.

### WP-5 Concurrency (F-05)

**5.1 Lock-Rangordnung kodieren.** Verbindliche Ordnung (Soll, Ist ist per Audit zu bestätigen und ggf. zu korrigieren):

`L0 rebuild_mutex → L1 save_mutex → L2 write_mutex → L3 nodes → L4 doc_to_node → L5 deleted_nodes → L6 erased_docs → L7 quantizer/sq8_bias/mmap_index → L8 seq_log → L9 clock/metrics_sink`

`entry_point`/Zähler sind Atomics und stehen außerhalb der Ordnung (dokumentierte Orderings: `Acquire/Release`, `SeqCst` nur wo begründet; die aktuellen `SeqCst`-Loads wie `last_tx_id` sind einzeln zu begründen oder zu lockern).

Umsetzung: Wrapper `Ordered<const LEVEL: u8, T>` für `Mutex`/`RwLock`. In `debug_assertions` pflegt ein Thread-Local den höchsten gehaltenen Level und `debug_assert!`et monoton steigende Level. In Release null Overhead. Verstöße sind damit in allen Tests sofort sichtbar.

**5.2 Save außerhalb des Schreib-Mutex.** `save()` nimmt kurz `write_mutex`, erzeugt einen konsistenten **Snapshot** (`Arc<CoreSnapshot>`, wie ihn Rebuild bereits per Swap nutzt, plus `last_tx_id`), gibt den Lock frei und schreibt dann via `AtomicFileWriter`. Parallele `save`-Aufrufe serialisiert ein eigener `save_mutex`. Nach Erfolg wird `last_saved_tx` gesetzt.

**5.3 Reader ohne lange Locks.** Search klont den aktuellen `Arc<Core>` (RCU) und traversiert ohne `nodes`-Read-Guard über die ganze Suche. Alternativ (Entscheidung nach Benchmark): behalten, aber mit Time-Budget und Dokumentation. Benchmark `contention_bench`: p50/p99-Suchlatenz bei 1/4/8 Writer-Threads, vor/nach Änderung, Zielwert p99 nach Änderung maximal 1,5× des Einzel-Writer-Wertes.

**5.4 Kein Guard über `.await`.** Die Kernlogik ist synchron (`insert_sync`, `search_sync`, `delete_sync`). Die `async`-Methoden sind dünne Wrapper ohne eigene Locks. Die `allow(await_holding_lock)` in `hnsw/tests.rs` und `persistence/tests.rs` werden entfernt, die betroffenen Tests angepasst.

**5.5 Loom/Shuttle-Modelle** (klein, begrenzte Knotenzahl): `insert ∥ search`, `rebuild-swap ∥ search`, `save ∥ insert`, `delete ∥ search`. Invarianten: Suche liefert nie einen gelöschten/erased Treffer nach Rückkehr von `delete`, nie einen Index außerhalb `nodes.len()`, und `save` schreibt einen Zustand, der einem Präfix der Transaktionsfolge entspricht.

### WP-6 Fehlermodell und Panic-Safety (F-06, F-07)

```rust
#[derive(Debug, thiserror::Error)]
pub enum VecError {
    #[error("io: {0}")] Io(#[from] std::io::Error),
    #[error("format: {0}")] Format(FormatError),      // UnsupportedVersion, UnsupportedFlags, DocIdWidth, BadMagic
    #[error("corrupt: {section} @{offset}")] Corrupt { section: Section, offset: u64, kind: CorruptKind }, // Truncated, Checksum, OutOfBounds, BadNeighbor
    #[error("config: {0}")] Config(ConfigError),
    #[error("limit: {0}")] Limit(LimitError),
    #[error("durability poisoned")] DurabilityPoisoned,
    #[error("index needs repair")] Poisoned,
}
```

- `VecError::is_recoverable_by_rebuild()` entscheidet für DiskANN-Fallback (`DiskAnnFallbackPolicy`) typisiert statt per String-Match.
- `From<VecError> for ContextraError` liegt in `compat`.
- **Mutations-Guard:** Mehrstufige Graphmutationen laufen in `MutationGuard<'a>`. Wird er ohne `finish()` gedroppt (Panic/Unwind), setzt `Drop` das Flag `needs_repair`. Alle Folgeoperationen prüfen das Flag (einzelner relaxed Atomic-Load) und liefern `VecError::Poisoned`. `repair()` baut aus dem letzten validen Snapshot neu auf oder validiert/heilt Nachbarschaften.
- Lint-Hygiene: `#![cfg_attr(not(test), deny(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::indexing_slicing, clippy::todo, clippy::unimplemented, clippy::unreachable))]`; alle `allow(...)` in Nicht-Test-Code entfernen oder mit Begründungskommentar und Ticket versehen; wirkungsloses `cfg_attr(test, allow)` auf `pub use` in `lib.rs` entfernen.
- Deprecated-Re-Exports (`hnsw_scanner`, `hnsw_purger`, …): `#[deprecated(since = "0.1", note = "…")]`, Entfernung in 0.2, `#[allow(deprecated)]` nur an einer Stelle.

**Akzeptanz:** Test, der per Fault-Injection (`fault_injection_insert_target` existiert bereits) einen Panic mitten im Insert auslöst: danach `Err(Poisoned)` statt stiller Inkonsistenz; `repair()` stellt Recall-Gleichheit zum Referenz-Rebuild her.

### WP-7 Erasure und Persistenz (F-08)

1. `erase(doc)` markiert den Eintrag, nullt RAM-Daten (besteht) und setzt `dirty_since_save`.
2. `save()` schreibt **ohne** erased Einträge (Kompaktierung beim Schreiben). Nach erfolgreichem Commit wird die Vorgängergeneration entlinkt. Optional `secure_wipe_old_generation` (Überschreiben mit Nullen vor `unlink`), dokumentiert als Best Effort (SSD, COW-FS, Journal).
3. `erase_and_persist(doc)` kombiniert beides für Aufrufer, die ein hartes „nicht mehr auf Platte“-Versprechen brauchen, und liefert `ErasureReceipt { generation, ram_zeroized, on_disk_rewritten }`.
4. Der gemappte Altzustand wird erst nach Drop aller Reader freigegeben. Dokumentiere die Zeitspanne und biete `wait_readers_drained(timeout)` an.
5. Optional: keyed MAC (blake3 keyed) statt reinem Hash auch für HNSW, Schlüssel wie in `diskann/integrity.rs`.

**Akzeptanz:** Test sucht im Dateibytestrom nach dem Vektorbyte-Muster eines erased Dokuments (nach `erase_and_persist`): Muster nicht mehr vorhanden.

### WP-8 Teststruktur und CI-Gates (F-09)

- **Umbenennen/Konsolidieren:** `campaign_j23_*`, `campaign_j24_*`, `campaign_j41_*`, `k02_*` nach Verhalten (`hnsw_insert_delete_search.rs`, `hnsw_snapshot_determinism.rs`, …). Dubletten-Paare (siehe F-09) zusammenführen. Jede Datei bekommt einen Kommentar „Invariante: …“.
- **Crash-Harness:** Trait `FsOps` (create, write, sync, rename, dir\_sync) mit `RealFs` und `FaultFs`. `FaultFs` modelliert Stromausfall: Es hält pro Datei den zuletzt gesyncten Präfix vor und verwirft beim „Crash“ alles andere. Das Szenario läuft für alle Schreiber exhaustiv über jeden Fehlerpunkt und prüft INV-VEC-ATOMIC-1.
- **Format-Tests:** Bitflip- und Truncation-Matrix (bestehend: `persistence_mutation_matrix_test`, erweitern auf v3), Goldene Fixtures, Fuzz-Seeds.
- **Determinismus:** `hnsw_seed_determinism` und `k02_*` laufen mit festem Seed auf Linux/macOS, Ergebnis-Hashes als Golden-Files.
- **CI-Matrix (blockierend):**
  1. `cargo hack check --each-feature -p contextra-vector` und `--no-default-features --features std`
  2. `cargo clippy -p contextra-vector --all-features --all-targets -- -D warnings`
  3. `cargo test -p contextra-vector --all-features`
  4. `RUSTFLAGS="--cfg loom" cargo test -p contextra-vector --test loom_* --release`
  5. `cargo miri test` auf den reinen Parser-/Format-Modulen (ohne mmap)
  6. `cargo deny check`, `cargo mutants` für `persistence/` und `hnsw/deletion.rs` (Konfiguration `.cargo/mutants.toml` besteht)
  7. Fuzz Smoke-Run (60 s je Target)
  8. Doc-Tests inklusive README

## 5. Migration und Kompatibilität

- Reader behält v1/v2 bis 0.3 mit Warnung (wie bisher `tracing::warn!` bei v1). Writer schreibt nur v3. `HnswIndex::migrate(path)` liest v1/v2 und schreibt v3 über `AtomicFileWriter`.
- Public-API-Änderungen (private `MmapIndex`-Felder, `open_async`, `VecError`) laufen über ein Release mit Deprecation, `compat` hält die alten Signaturen bereit. API-Diff per `cargo semver-checks`.

## 6. Meilensteine

| Meilenstein | Inhalt | Exit-Kriterium |
| --- | --- | --- |
| M1 | WP-1, WP-2, CI-Gates 1–3 | `no-default-features` grün, genau ein Durable-Write-Pfad, Fault-Tests grün |
| M2 | WP-6, WP-3 (Writer + Reader) | Typisierte Fehler, v3 mit Fixtures, Bitflip-/Truncation-Matrix grün |
| M3 | WP-4, WP-5.1, 5.4 | Private mmap-API, Lock-Rangprüfung aktiv, keine Guards über `.await` |
| M4 | WP-5.2, 5.3, 5.5 | Save ohne Writer-Stall, Contention-Bench dokumentiert, Loom-Modelle grün |
| M5 | WP-7, WP-8 Abschluss | Erasure-Receipt, Tests umbenannt und konsolidiert, alle CI-Gates blockierend |

## 7. Offene Entscheidungen

1. **`contextra-durable-fs` wiederverwenden oder bewusst duplizieren?** Klasse A fordert Autarkie, spricht also für eine interne Primitive mit optionalem Adapter. Das ist die hier empfohlene Variante, bedarf aber einer Freigabe (ADR).
2. **Eigentümerschaft von `ContextraError`:** Soll `VecError` langfristig in `contextra-types` aufgehen oder bleibt der Crate bewusst eigenständig?
3. **Windows-Support:** Verzeichnis-Sync ist dort nicht verfügbar. Entscheidung, ob Windows als Tier-2 mit dokumentiertem Durability-Abstrich geführt wird.
4. **DocId-Breite zur Laufzeit statt Compile-Zeit?** Würde das Feature `docid-128` und die Fixture-Matrix vereinfachen, kostet aber Generics/Monomorphisierung im Hot Path.
5. **Reader-Strategie (RCU vs. Lock):** Entscheidung nach dem Contention-Benchmark aus WP-5.3.

## 8. Risiken

- Format-v3-Umstellung berührt Header, Offsets und Reader; Fixtures und Fuzzing vor dem Merge sind Pflicht.
- RCU-Reader erhöhen Speicherbedarf während Rebuild (zwei Cores gleichzeitig). Bestehende `ram_reduction`-Tests als Gate nutzen.
- Befunde mit **\[zu verifizieren\]** sind vor Beginn des jeweiligen WP durch einen Build/Test-Lauf zu bestätigen; die Toolchain (laut `CLAUDE.md` über conda-forge, Rust 1.89) stand in dieser Sitzung nicht zur Verfügung.
