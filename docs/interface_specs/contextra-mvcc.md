# Schnittstellenspezifikation: `contextra-mvcc`

> Erzeugt durch `tools/extract_interfaces.py` aus dem Quellcode. Keine Handpflege. Jede Angabe trägt Datei:Zeile. Maßgeblich bleibt der Quellcode.

## 1. Kopf

| Feld | Wert |
|---|---|
| Name | `contextra-mvcc` |
| Version | workspace |
| Pfad | `crates/contextra-mvcc` |
| Rolle | MVCC-Hilfen |
| Bibliothekspfad | src/lib.rs (Standard) |

**Crate-Dokumentation (//!):**

> # `Contextra` MVCC — Multi-Version Concurrency Control, Sequence Log, Transaction Staging und SSI-Validierung.
>
> Multi-Version Concurrency Control (MVCC, Mehrversions-Nebenläufigkeitssteuerung) ermöglicht es
> Lese- und Schreiboperationen gleichzeitig ohne gegenseitiges Blockieren auszuführen.
> Leser sehen konsistente Snapshots (Schnappschüsse) zu einem logischen Zeitpunkt,
> während Schreiber isoliert neue Versionen im Staging-Puffer vorbereiten.
>
> ## Architektur-Rolle (Ring 0 / Ebene E1a — Kernkomponente)
>
> `contextra-mvcc` gehört zur innersten Systemschicht (Ring 0 / Ebene E1a). Es bietet fundamentale
> Isolationsgarantien für relationale und vektorielle Indexstrukturen und besitzt keinerlei
> Kenntnisse über Festplatten-Layouts oder Netzwerkprotokolle.

## 2. Abhängigkeiten

| Name | Art | Quelle | optional | verwendet von (normal/dev) |
|---|---|---|---|---|
| `ahash` | normal | workspace | nein |  |
| `bincode` | dev | workspace | nein |  |
| `bytes` | normal | workspace | nein |  |
| `contextra-env` | dev | path | nein | contextra-checkpoint, contextra-crypto, contextra-graph, contextra-infer-candle, contextra-kvcache, contextra-mvcc, contextra-ports, contextra-store, contextra-text, contextra-vector |
| `contextra-ids` | normal | path | nein | contextra-checkpoint, contextra-crypto, contextra-graph, contextra-mvcc, contextra-store, contextra-text, contextra-types, contextra-vector |
| `contextra-ports` | dev | path | nein | contextra, contextra-adapt, contextra-agent, contextra-checkpoint, contextra-cognition, contextra-crypto, contextra-crypto-fuzz, contextra-db, contextra-engine, contextra-graph, contextra-graph-fuzz, contextra-infer-candle, contextra-infer-ollama, contextra-infer-onnx, contextra-kvcache, contextra-mcp, contextra-mvcc, contextra-rank, contextra-router, contextra-store, contextra-store-fuzz, contextra-testkit, contextra-text, contextra-vector, contextra-vector-fuzz |
| `contextra-types` | dev | path | nein | contextra, contextra-adapt, contextra-agent, contextra-bench, contextra-checkpoint, contextra-cognition, contextra-crypto, contextra-crypto-fuzz, contextra-db, contextra-engine, contextra-graph, contextra-graph-fuzz, contextra-infer-candle, contextra-infer-ollama, contextra-infer-onnx, contextra-ingest, contextra-kvcache, contextra-mcp, contextra-mvcc, contextra-ports, contextra-py, contextra-rank, contextra-router, contextra-simd, contextra-store, contextra-store-fuzz, contextra-testkit, contextra-text, contextra-vault, contextra-vector, contextra-vector-fuzz |
| `criterion` | dev | workspace | nein |  |
| `loom` | dev | registry | nein |  |
| `parking_lot` | normal | workspace | nein |  |
| `proptest` | dev | registry | nein |  |
| `serde` | normal | workspace | nein |  |
| `serde_json` | dev | workspace | nein |  |
| `thiserror` | normal | workspace | nein |  |
| `tokio` | dev | workspace | nein |  |
| `tracing` | normal | workspace | nein |  |

**Verwendet von (Crate-Ebene):** `contextra-db`, `contextra-store`, `contextra-vector`, `contextra-vector-fuzz`

## 3. Features

| Feature | Aktiviert |
|---|---|
| `default` | – |
| `docid-128` | `contextra-ids/docid-128` |
| `ports-compat` | `workspace` |
| `types-compat` | `workspace` |
| `workspace` | `dep:contextra-types`, `dep:contextra-ports` |

## 4. Metriken (Quellcode)

| Kennzahl | Wert |
|---|---|
| Quelldateien | 10 |
| Quellzeilen | 4858 |
| Öffentliche Items (Modul-Ebene) | 35 |
| davon `struct` | 11 |
| davon `enum` | 3 |
| davon `trait` | 3 |
| davon `type` | 1 |
| davon `const` | 8 |
| davon `mod` | 9 |
| Öffentliche Makros (`#[macro_export]`) | 0 |
| Öffentliche Re-Exports (`pub use`) | 14 |
| `unsafe`-Vorkommen (Produktionscode) | 0 |
| `.unwrap()` (Produktionscode) | 0 |
| `.expect(` (Produktionscode) | 0 |
| `panic!` (Produktionscode) | 0 |
| `todo!/unimplemented!/unreachable!` | 0 |
| Indexierungen `x[…]` (grob, Produktionscode) | 0 |
| `#[deprecated]` | 0 |
| `#[cfg(feature…)]`-Gates | 2 |
| Parser-Abdeckung (geparste `pub`-Deklarationen / Grep-Treffer) | 172 / 172 = 100 % |

## 5. Modulbaum und Dateien

| Modul | Datei | Zeilen |
|---|---|---|
| `compat::error` | `crates/contextra-mvcc/src/compat/error.rs` | 70 |
| `compat` | `crates/contextra-mvcc/src/compat/mod.rs` | 12 |
| `contract` | `crates/contextra-mvcc/src/contract.rs` | 11 |
| `crate` | `crates/contextra-mvcc/src/lib.rs` | 112 |
| `mvcc_error` | `crates/contextra-mvcc/src/mvcc_error.rs` | 64 |
| `seq_log` | `crates/contextra-mvcc/src/seq_log.rs` | 858 |
| `snapshot` | `crates/contextra-mvcc/src/snapshot.rs` | 965 |
| `ssi` | `crates/contextra-mvcc/src/ssi.rs` | 1237 |
| `tombstone` | `crates/contextra-mvcc/src/tombstone.rs` | 55 |
| `tx_buffer` | `crates/contextra-mvcc/src/tx_buffer.rs` | 1474 |

## 6. Re-Exports (`pub use`)

- `pub use contextra_types::error as workspace_error` — `crates/contextra-mvcc/src/compat/mod.rs:10` (Modul `compat`)
- `pub use contextra_types::error::ContextraError` — `crates/contextra-mvcc/src/compat/mod.rs:11` (Modul `compat`)
- `pub use contextra_types::error::Result` — `crates/contextra-mvcc/src/compat/mod.rs:12` (Modul `compat`)
- `pub use contextra_env::` — `crates/contextra-mvcc/src/contract.rs:11` (Modul `contract`)
- `pub use crate::mvcc_error::` — `crates/contextra-mvcc/src/lib.rs:81` (Modul `crate`)
- `pub use contextra_ids::` — `crates/contextra-mvcc/src/lib.rs:82` (Modul `crate`)
- `pub use contextra_ids::` — `crates/contextra-mvcc/src/lib.rs:86` (Modul `types`)
- `pub use crate::compat::` — `crates/contextra-mvcc/src/lib.rs:94` (Modul `crate`)
- `pub use crate::mvcc_error::Result` — `crates/contextra-mvcc/src/lib.rs:96` (Modul `crate`)
- `pub use seq_log::` — `crates/contextra-mvcc/src/lib.rs:107` (Modul `crate`)
- `pub use snapshot::` — `crates/contextra-mvcc/src/lib.rs:108` (Modul `crate`)
- `pub use ssi::` — `crates/contextra-mvcc/src/lib.rs:109` (Modul `crate`)
- `pub use tombstone::` — `crates/contextra-mvcc/src/lib.rs:110` (Modul `crate`)
- `pub use tx_buffer::` — `crates/contextra-mvcc/src/lib.rs:111` (Modul `crate`)

## 8. Öffentliche Schnittstelle je Item

### Modul `compat`

#### `mod error`
- Ort: `crates/contextra-mvcc/src/compat/mod.rs:8`
```rust
pub mod error
```
Dokumentation: Brücke zur Konvertierung lokaler `MvccError`-Fehler in `contextra_types::ContextraError`.

### Modul `crate`

#### `mod compat`
- Ort: `crates/contextra-mvcc/src/lib.rs:91`
```rust
pub mod compat
```
- Gate: `#[cfg(feature = "workspace")]`

#### `mod mvcc_error`
- Ort: `crates/contextra-mvcc/src/lib.rs:100`
```rust
pub mod mvcc_error
```

#### `mod seq_log`
- Ort: `crates/contextra-mvcc/src/lib.rs:101`
```rust
pub mod seq_log
```

#### `mod snapshot`
- Ort: `crates/contextra-mvcc/src/lib.rs:102`
```rust
pub mod snapshot
```

#### `mod ssi`
- Ort: `crates/contextra-mvcc/src/lib.rs:103`
```rust
pub mod ssi
```

#### `mod tombstone`
- Ort: `crates/contextra-mvcc/src/lib.rs:104`
```rust
pub mod tombstone
```

#### `mod tx_buffer`
- Ort: `crates/contextra-mvcc/src/lib.rs:105`
```rust
pub mod tx_buffer
```

#### `mod types`
- Ort: `crates/contextra-mvcc/src/lib.rs:85`
```rust
pub mod types
```
Dokumentation: Kanonische Typen und Identifikatoren für MVCC aus `contextra-ids`.

### Modul `mvcc_error`

#### `enum MvccError`
- Ort: `crates/contextra-mvcc/src/mvcc_error.rs:18`
```rust
pub enum MvccError
```
Dokumentation: Lokaler Fehlertyp für Multi-Version Concurrency Control und Transaktionsverwaltung.
- Impl-Block `crates/contextra-mvcc/src/mvcc_error.rs:49`

Varianten:

- `Conflict(String)` — `crates/contextra-mvcc/src/mvcc_error.rs:21`
- `LimitExceeded{` — `crates/contextra-mvcc/src/mvcc_error.rs:25`
- `Transaction(String)` — `crates/contextra-mvcc/src/mvcc_error.rs:34`
- `Storage(String)` — `crates/contextra-mvcc/src/mvcc_error.rs:38`
- `InvalidInput(String)` — `crates/contextra-mvcc/src/mvcc_error.rs:42`
- `Internal(String)` — `crates/contextra-mvcc/src/mvcc_error.rs:46`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn limit_exceeded(limit: usize, context: impl Into<String>) -> Self` | `crates/contextra-mvcc/src/mvcc_error.rs:53` | Hilfskonstruktor zur Erzeugung eines [`MvccError::LimitExceeded`]. |

#### `type Result`
- Ort: `crates/contextra-mvcc/src/mvcc_error.rs:63`
```rust
pub type Result<T, E = MvccError> = std::result::Result<T, E>
```
Dokumentation: Lokaler Result-Typ für MVCC-Operationen.

### Modul `seq_log`

#### `struct SeqLogEntry`
- Ort: `crates/contextra-mvcc/src/seq_log.rs:58`
```rust
pub struct SeqLogEntry
```
Dokumentation: Ein versionierter Protokolleintrag für die Snapshot-Isolation (`_at`-Familie). Jeder Eintrag repräsentiert die Lebensspanne einer Dokumentversion: von der Einfügesequenznummer `insert_seq` bis zur Löschsequenznummer `delete_seq`. ### Speicherverbrauch Jeder Eintrag belegt 24 Bytes: - 8 Bytes für `DocId` (Dokument-Kennung) - 8 Bytes für `insert_seq` (Sequenznummer der Erstellung) - 8 Bytes für `delete_seq: Option<u64>` (Sequenznummer der Entfernung/Deletion) Einträge, deren `delete_seq` unterhalb der minimal aktiven Sequenznummer (`min_active_seqno`) aller aktiven Leser liegt, können via [`Sequ
- Impl-Block `crates/contextra-mvcc/src/seq_log.rs:67`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `doc_id` | `DocId` | `crates/contextra-mvcc/src/seq_log.rs:60` |
| `insert_seq` | `u64` | `crates/contextra-mvcc/src/seq_log.rs:62` |
| `delete_seq` | `Option<u64>` | `crates/contextra-mvcc/src/seq_log.rs:64` |

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn is_visible(&self, as_of: u64) -> bool` | `crates/contextra-mvcc/src/seq_log.rs:89` | Prüft in O(1), ob dieser Protokolleintrag zu einem bestimmten Snapshot-Stand `as_of` sichtbar ist. ### Formale Sichtbarkeitsinvariante (INV-INDEX-SNAPSHOT-1): E |

#### `struct SequenceLog`
- Ort: `crates/contextra-mvcc/src/seq_log.rs:151`
```rust
pub struct SequenceLog
```
Dokumentation: Verwaltet das chronologische Sequenz-Protokoll und Sichtbarkeitsfilter für Indexstrukturen. ### Funktionsweise: `SequenceLog` speichert alle Einfüge- und Löschvorgänge chronologisch in einem Vektor (`entries`). Zudem merkt es sich aktive Snapshot-Pins (`pinned_snapshots`), um zu verhindern, dass alte Datenbereiche in Indizes vorzeitig freigegeben (geprunt) werden, solange noch Leser darauf zugreifen. ### Port-Nutzung: Verwendet [`Clock`] zur Erfassung von Zeitstempeln für gepinnte Snapshots, um verwaiste Pins diagnostizieren und automatisch bereinigen zu können.
- Impl-Block (Trait `std::fmt::Debug`) `crates/contextra-mvcc/src/seq_log.rs:166`
- Impl-Block (Trait `Default`) `crates/contextra-mvcc/src/seq_log.rs:178`
- Impl-Block `crates/contextra-mvcc/src/seq_log.rs:184`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new() -> Self` | `crates/contextra-mvcc/src/seq_log.rs:188` | Erstellt ein neues, leeres `SequenceLog` mit der Standard-Systemuhr [`SystemClock`] und einer maximalen Pin-Dauer von 5 Minuten. |
| `pub fn with_clock(clock: Arc<dyn Clock>) -> Self` | `crates/contextra-mvcc/src/seq_log.rs:194` | Erstellt ein neues, leeres `SequenceLog` mit einer benutzerdefinierten Implementierung des [`Clock`]-Ports (z. B. `ManualClock` in deterministischen Tests). |
| `pub fn with_time_fn<F>(time_fn: F) -> Self where F: Fn() -> u64 + Send + Sync + 'static,` | `crates/contextra-mvcc/src/seq_log.rs:206` | Erstellt ein `SequenceLog` mit einer anpassbaren Zeitstempel-Closure (State-Passing per `Fn() -> u64`). |
| `pub fn clock(&self) -> &Arc<dyn Clock>` | `crates/contextra-mvcc/src/seq_log.rs:223` | Gibt eine Referenz auf den aktiven [`Clock`]-Port zurück. |
| `pub fn set_clock(&mut self, clock: Arc<dyn Clock>)` | `crates/contextra-mvcc/src/seq_log.rs:228` | Ersetzt die aktive [`Clock`]-Instanz für dieses Log. |
| `pub fn with_max_pin_duration(mut self, duration: Duration) -> Self` | `crates/contextra-mvcc/src/seq_log.rs:233` | Konfiguriert die maximale Pin-Dauer als Fluent-Builder und gibt `self` zurück. |
| `pub fn set_max_pin_duration(&mut self, duration: Duration)` | `crates/contextra-mvcc/src/seq_log.rs:239` | Setzt die maximale Pin-Dauer für Diagnosezwecke. |
| `pub fn max_pin_duration(&self) -> Duration` | `crates/contextra-mvcc/src/seq_log.rs:244` | Liefert die aktuell konfigurierte Maximaldauer für Pins. |
| `pub fn pin_snapshot(&mut self, seq_no: u64)` | `crates/contextra-mvcc/src/seq_log.rs:255` | Pinnt eine historische Sequenznummer, um zu verhindern, dass die Hintergrund-Bereinigung (Rebuild/Compaction) gelöschte Knoten entfernt, die für diesen Stand no |
| `pub fn pin_snapshot_at(&mut self, seq_no: u64, at_nanos: u64)` | `crates/contextra-mvcc/src/seq_log.rs:261` | Pinnt eine Sequenznummer zu einem expliziten Nanosekunden-Zeitstempel (für deterministische Tests). |
| `pub fn unpin_snapshot(&mut self, seq_no: u64)` | `crates/contextra-mvcc/src/seq_log.rs:275` | Gibt einen gepinnten Snapshot wieder frei. ### Ablauf: 1. Tombstone-Bit der Sequenznummer ausmaskieren. 2. Den letzten Zeitstempel vom Stack dieses Eintrags ent |
| `pub fn expired_pins_at(&self, now_nanos: u64) -> Vec<u64>` | `crates/contextra-mvcc/src/seq_log.rs:292` | Findet alle gepinnten Sequenznummern, deren Haltezeit `max_pin_duration` überschreitet, bezogen auf den übergebenen Monotonie-Zeitstempel `now_nanos`. Rückgabe  |
| `pub fn expired_pins(&self) -> Vec<u64>` | `crates/contextra-mvcc/src/seq_log.rs:308` | Liefert alle abgelaufenen Snapshot-Pins bezogen auf die aktuelle Systemuhr. |
| `pub fn reap_expired_pins_at(&mut self, now_nanos: u64) -> Vec<u64>` | `crates/contextra-mvcc/src/seq_log.rs:318` | Bereinigt (entpinnt) alle Pins, die die Maximaldauer bezüglich `now_nanos` überschritten haben. ### Ablauf: 1. Abgelaufene Pins via [`SequenceLog::expired_pins_ |
| `pub fn reap_expired_pins(&mut self) -> Vec<u64>` | `crates/contextra-mvcc/src/seq_log.rs:327` | Bereinigt alle abgelaufenen Snapshot-Pins bezogen auf die aktuelle Uhrzeit. |
| `pub fn min_retention_seq_at(&self, now_nanos: u64) -> Option<u64>` | `crates/contextra-mvcc/src/seq_log.rs:337` | Berechnet die kleinste aktuell gepinnte Sequenznummer für Aufbewahrungs- und Pruning-Zwecke. Gibt `None` zurück, falls aktuell kein einziger Snapshot gepinnt is |
| `pub fn min_retention_seq(&self) -> Option<u64>` | `crates/contextra-mvcc/src/seq_log.rs:360` | Berechnet die kleinste aktuell gepinnte Sequenznummer bezogen auf die aktuelle Systemuhr. |
| `pub fn deletion_seq(&self, doc_id: DocId) -> Option<u64>` | `crates/contextra-mvcc/src/seq_log.rs:365` | Liefert die Lösch-Sequenznummer des Dokuments, falls es aktuell als gelöscht verzeichnet ist. |
| `pub fn record_insert(&mut self, doc_id: DocId, seq: u64)` | `crates/contextra-mvcc/src/seq_log.rs:378` | Zeichnet eine Einfügeoperation für ein Dokument zu einer Sequenznummer auf. ### Ablauf: 1. Tombstone-Bit der Sequenznummer ausmaskieren. 2. Falls für `doc_id` b |
| `pub fn record_delete(&mut self, doc_id: DocId, seq: u64)` | `crates/contextra-mvcc/src/seq_log.rs:401` | Zeichnet eine Löschoperation für ein Dokument zu einer Sequenznummer auf. ### Ablauf: 1. Tombstone-Bit der Sequenznummer ausmaskieren. 2. Den letzten noch aktiv |
| `pub fn is_visible(&self, doc_id: DocId, seq_no: u64) -> bool` | `crates/contextra-mvcc/src/seq_log.rs:418` | Prüft, ob ein Dokument zu einer bestimmten Sequenznummer `seq_no` im Index sichtbar ist. Ein Dokument ist sichtbar, wenn es vor oder bei `seq_no` eingefügt wurd |
| `pub fn compact(&mut self, min_active_seqno: u64)` | `crates/contextra-mvcc/src/seq_log.rs:450` | Verdichtet (kompaktiert) das Sequenz-Log und entfernt historische Versionen, deren Entfernung strikt kleiner als `min_active_seqno` ist. ### Invariante: Version |
| `pub fn len(&self) -> usize` | `crates/contextra-mvcc/src/seq_log.rs:469` | Gibt die Anzahl der aktuell im Log vorgehaltenen Einträge zurück. |
| `pub fn is_empty(&self) -> bool` | `crates/contextra-mvcc/src/seq_log.rs:474` | Gibt `true` zurück, wenn das Sequenz-Log keine Einträge enthält. |
| `pub fn changes_since(&self, snapshot_seq: u64) -> Vec<SeqLogChange>` | `crates/contextra-mvcc/src/seq_log.rs:484` | Liefert alle Änderungen (Einfügungen und Entfernungen), die strikt nach `snapshot_seq` stattgefunden haben, aufsteigend sortiert nach Sequenznummer. Wird verwen |

#### `enum SeqLogChange`
- Ort: `crates/contextra-mvcc/src/seq_log.rs:104`
```rust
pub enum SeqLogChange
```
Dokumentation: Repräsentiert eine historische Änderung (Einfügen oder Löschen) im Sequenz-Log für Delta-Replays. Wird von Abfragen wie [`SequenceLog::changes_since`] erzeugt, um Indizes inkrementell auf den neuesten Stand nachzuziehen.
- Impl-Block `crates/contextra-mvcc/src/seq_log.rs:121`

Varianten:

- `Insert{` — `crates/contextra-mvcc/src/seq_log.rs:106`
- `Delete{` — `crates/contextra-mvcc/src/seq_log.rs:113`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn seq(&self) -> u64` | `crates/contextra-mvcc/src/seq_log.rs:123` | Liefert die logische Sequenznummer dieser Änderung. |
| `pub fn doc_id(&self) -> DocId` | `crates/contextra-mvcc/src/seq_log.rs:130` | Liefert die Kennung des betroffenen Dokuments. |

#### `const DEFAULT_MAX_PIN_DURATION`
- Ort: `crates/contextra-mvcc/src/seq_log.rs:38`
```rust
pub const DEFAULT_MAX_PIN_DURATION: Duration = Duration::from_secs(300)
```
Dokumentation: Standard-Lebensdauer für Snapshot-Pins (5 Minuten), bevor Diagnose-Warnungen für blockierende oder verwaiste Pins ausgelöst werden.

### Modul `snapshot`

#### `struct SnapshotGuard`
- Ort: `crates/contextra-mvcc/src/snapshot.rs:527`
```rust
pub struct SnapshotGuard
```
Dokumentation: RAII-Guard (Resource Acquisition Is Initialization) für einen aktiven Lese-Snapshot. Hält einen Snapshot in der [`SnapshotRegistry`] gepinnt, solange die Instanz existiert. Beim Verlassen des Gültigkeitsbereichs (`Drop`) wird der Snapshot automatisch deregistriert.
- Impl-Block `crates/contextra-mvcc/src/snapshot.rs:534`
- Impl-Block (Trait `Drop`) `crates/contextra-mvcc/src/snapshot.rs:546`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn seq_no(&self) -> u64` | `crates/contextra-mvcc/src/snapshot.rs:536` | Liefert die Sequenznummer, die durch diesen Guard geschützt wird. |
| `pub fn tenant(&self) -> Option<TenantId>` | `crates/contextra-mvcc/src/snapshot.rs:541` | Liefert den zugeordneten Mandanten, falls angegeben. |

#### `struct SnapshotRegistry`
- Ort: `crates/contextra-mvcc/src/snapshot.rs:84`
```rust
pub struct SnapshotRegistry
```
Dokumentation: Zentrale Registrierung für aktive Lese-Snapshots im System. ### Synchronisationsstrategie: - `active`: Ein sortierter `BTreeMap`, geschützt durch einen `parking_lot::Mutex`. Schlüssel ist die Snapshot-Sequenznummer; Wert ist eine Liste von Metadaten aller Guards, die diesen Snapshot halten (Referenzzählung). - `min_active_seqno`: Atomarer 64-Bit-Ganzzahlwert für lock-freies Auslesen durch Hintergrund-Threads. - `clock`: Zeitport zur Erfassung der Haltedauer aktiver Snapshots. - `config`: Konfigurierte Kapazitäts- und Altersgrenzen.
- Impl-Block (Trait `std::fmt::Debug`) `crates/contextra-mvcc/src/snapshot.rs:91`
- Impl-Block (Trait `Default`) `crates/contextra-mvcc/src/snapshot.rs:100`
- Impl-Block `crates/contextra-mvcc/src/snapshot.rs:106`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new() -> Self` | `crates/contextra-mvcc/src/snapshot.rs:109` | Erstellt eine neue, leere Snapshot-Registry mit Standard-[`SystemClock`] und Standardkonfiguration. |
| `pub fn with_clock(clock: Arc<dyn Clock>) -> Self` | `crates/contextra-mvcc/src/snapshot.rs:114` | Erstellt eine neue Snapshot-Registry mit benutzerdefiniertem [`Clock`]-Port (für Tests). |
| `pub fn with_clock_and_config(clock: Arc<dyn Clock>, config: SnapshotRegistryConfig) -> Self` | `crates/contextra-mvcc/src/snapshot.rs:119` | Erstellt eine neue Snapshot-Registry mit benutzerdefiniertem [`Clock`]-Port und Konfiguration. |
| `pub fn with_time_fn<F>(time_fn: F) -> Self where F: Fn() -> u64 + Send + Sync + 'static,` | `crates/contextra-mvcc/src/snapshot.rs:129` | Erstellt eine neue Snapshot-Registry mit einer Zeitstempel-Closure (State-Passing per `Fn() -> u64`). |
| `pub fn clock(&self) -> &Arc<dyn Clock>` | `crates/contextra-mvcc/src/snapshot.rs:146` | Liefert eine Referenz auf den aktiven Zeitport ([`Clock`]). |
| `pub fn config(&self) -> SnapshotRegistryConfig` | `crates/contextra-mvcc/src/snapshot.rs:151` | Liefert die aktuelle Konfiguration der Registry. |
| `pub fn set_config(&mut self, config: SnapshotRegistryConfig)` | `crates/contextra-mvcc/src/snapshot.rs:156` | Setzt eine neue Konfiguration für die Registry. |
| `pub fn oldest_active_snapshot(&self) -> Option<u64>` | `crates/contextra-mvcc/src/snapshot.rs:164` | Liefert die Untergrenze für Garbage Collection (älteste aktive Snapshot-Sequenz). Entspricht dem Vertrag für Store-Kompaktierung und ist äquivalent zu `min_read |
| `pub fn try_register(self: &Arc<Self>, seq_no: u64) -> Result<SnapshotGuard>` | `crates/contextra-mvcc/src/snapshot.rs:173` | Versucht, einen Lese-Snapshot zu registrieren. Im Gegensatz zu `register` schlägt diese Methode fail-closed mit `MvccError::LimitExceeded` fehl, wenn die konfig |
| `pub fn try_register_at(self: &Arc<Self>, seq_no: u64, at_nanos: u64) -> Result<SnapshotGuard>` | `crates/contextra-mvcc/src/snapshot.rs:179` | Versucht, einen Lese-Snapshot zu einem expliziten Nanosekunden-Zeitstempel zu registrieren. |
| `pub fn try_register_for_tenant( self: &Arc<Self>, tenant: TenantId, seq_no: u64, ) -> Result<SnapshotGuard>` | `crates/contextra-mvcc/src/snapshot.rs:184` | Versucht, einen Lese-Snapshot für einen bestimmten Mandanten zu registrieren. |
| `pub fn try_register_for_tenant_at( self: &Arc<Self>, tenant: Option<TenantId>, seq_no: u64, at_nanos: u64, ) -> Result<SnapshotGuard>` | `crates/contextra-mvcc/src/snapshot.rs:194` | Versucht, einen Lese-Snapshot für einen Mandanten zu einem expliziten Zeitstempel zu registrieren. |
| `pub fn register(self: &Arc<Self>, seq_no: u64) -> SnapshotGuard` | `crates/contextra-mvcc/src/snapshot.rs:236` | Registriert einen Lese-Snapshot und liefert einen RAII-Guard ([`SnapshotGuard`]) zurück. Sobald der Guard den Gültigkeitsbereich verlässt (Drop), wird der Snaps |
| `pub fn register_for_tenant(self: &Arc<Self>, tenant: TenantId, seq_no: u64) -> SnapshotGuard` | `crates/contextra-mvcc/src/snapshot.rs:242` | Registriert einen Lese-Snapshot für einen bestimmten Mandanten. |
| `pub fn register_for_tenant_at( self: &Arc<Self>, tenant: TenantId, seq_no: u64, at_nanos: u64, ) -> SnapshotGuard` | `crates/contextra-mvcc/src/snapshot.rs:248` | Registriert einen Lese-Snapshot für einen bestimmten Mandanten mit explizitem Zeitstempel. |
| `pub fn register_at(self: &Arc<Self>, seq_no: u64, at_nanos: u64) -> SnapshotGuard` | `crates/contextra-mvcc/src/snapshot.rs:270` | Registriert einen Lese-Snapshot mit explizitem Zeitstempel ohne Mandantenzuordnung. |
| `pub fn min_active_seqno(&self) -> u64` | `crates/contextra-mvcc/src/snapshot.rs:289` | Liefert die kleinste Sequenznummer aller derzeit aktiven Snapshots (`u64::MAX`, falls keine aktiv). |
| `pub fn min_read_snapshot(&self) -> Option<u64>` | `crates/contextra-mvcc/src/snapshot.rs:295` | Liefert die kleinste Snapshot-Sequenznummer als `Option<u64>` (ohne Sentinel). |
| `pub fn min_for_compaction(&self, current_seq: u64) -> u64` | `crates/contextra-mvcc/src/snapshot.rs:306` | Berechnet die minimale Sequenznummer für die Kompaktierung (GC-Grenze) ohne Sentinel. |
| `pub fn revoke_covering( &self, tenant: TenantId, range: std::ops::Range<u64>, gated_seq: u64, ) -> usize` | `crates/contextra-mvcc/src/snapshot.rs:312` | Widerruft Snapshots im Bereich für einen Mandanten (Tenant-Isolation). |
| `pub fn pin(&self, seq_no: u64)` | `crates/contextra-mvcc/src/snapshot.rs:346` | Dauerhaftes Pinnen einer Sequenznummer (z. B. für SAOS Checkpoints oder Backups), ohne einen RAII-Guard zu halten. Muss manuell über [`SnapshotRegistry::unpin`] |
| `pub fn pin_for_tenant(&self, tenant: TenantId, seq_no: u64)` | `crates/contextra-mvcc/src/snapshot.rs:352` | Pinnt eine Sequenznummer für einen Mandanten bezogen auf die aktuelle Systemuhr. |
| `pub fn pin_for_tenant_at(&self, tenant: TenantId, seq_no: u64, at_nanos: u64)` | `crates/contextra-mvcc/src/snapshot.rs:358` | Pinnt eine Sequenznummer für einen Mandanten zu einem expliziten Zeitstempel. |
| `pub fn pin_at(&self, seq_no: u64, at_nanos: u64)` | `crates/contextra-mvcc/src/snapshot.rs:369` | Pinnt eine Sequenznummer zu einem expliziten Zeitstempel. |
| `pub fn unpin(&self, seq_no: u64)` | `crates/contextra-mvcc/src/snapshot.rs:380` | Hebt einen dauerhaften Pin auf und bereinigt den minimal aktiven Wasserstand. |
| `pub fn longest_active_pin_at(&self, now_nanos: u64) -> Option<(u64, Duration)>` | `crates/contextra-mvcc/src/snapshot.rs:440` | Findet den am längsten aktiven Snapshot bezüglich des Nanosekunden-Zeitstempels `now_nanos`. Liefert `Some((seq_no, duration))` oder `None`, falls aktuell keine |
| `pub fn longest_active_pin(&self) -> Option<(u64, Duration)>` | `crates/contextra-mvcc/src/snapshot.rs:461` | Findet den am längsten aktiven Snapshot bezogen auf die aktuelle Systemuhr. |
| `pub fn reap_expired_pins_at(&self, max_pin_duration: Duration, now_nanos: u64) -> Vec<u64>` | `crates/contextra-mvcc/src/snapshot.rs:473` | Bereinigt und entfernt alle aktiven Snapshot-Pins, die älter als `max_pin_duration` sind. ### Ablauf: 1. Berechne Schwellenwert in Nanosekunden. 2. Iteriere übe |
| `pub fn reap_expired_pins(&self, max_pin_duration: Duration) -> Vec<u64>` | `crates/contextra-mvcc/src/snapshot.rs:504` | Bereinigt alle überalterten Snapshot-Pins bezogen auf die aktuelle Uhrzeit. |

#### `struct SnapshotRegistryConfig`
- Ort: `crates/contextra-mvcc/src/snapshot.rs:46`
```rust
pub struct SnapshotRegistryConfig
```
Dokumentation: Konfigurationsoptionen für die [`SnapshotRegistry`].
- Impl-Block (Trait `Default`) `crates/contextra-mvcc/src/snapshot.rs:53`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `max_active_snapshots` | `usize` | `crates/contextra-mvcc/src/snapshot.rs:48` |
| `max_snapshot_age` | `Duration` | `crates/contextra-mvcc/src/snapshot.rs:50` |

### Modul `ssi`

#### `struct PruningBlockerInfo`
- Ort: `crates/contextra-mvcc/src/ssi.rs:228`
```rust
pub struct PruningBlockerInfo
```
Dokumentation: Diagnoseinformationen über aktive Snapshots oder Sequenznummern, die das Pruning im Validator blockieren. Wird erzeugt, wenn der Speicher für Commit-Schlüssel zu mehr als 80 % gefüllt ist, um im Tracing und Monitoring exakt zu erkennen, welcher langlaufende Leser das Aufräumen blockiert.

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `min_unpruned_seq` | `u64` | `crates/contextra-mvcc/src/ssi.rs:230` |
| `pruned_through_seq` | `u64` | `crates/contextra-mvcc/src/ssi.rs:232` |
| `tracked_commit_keys` | `usize` | `crates/contextra-mvcc/src/ssi.rs:234` |
| `total_seq_buckets` | `usize` | `crates/contextra-mvcc/src/ssi.rs:236` |
| `coarsened_seq_buckets` | `usize` | `crates/contextra-mvcc/src/ssi.rs:238` |
| `max_tracked_keys` | `usize` | `crates/contextra-mvcc/src/ssi.rs:240` |
| `longest_active_snapshot_seq` | `Option<u64>` | `crates/contextra-mvcc/src/ssi.rs:242` |
| `longest_pin_duration` | `Option<Duration>` | `crates/contextra-mvcc/src/ssi.rs:244` |
| `is_pin_expired` | `bool` | `crates/contextra-mvcc/src/ssi.rs:246` |

#### `struct ReadSet`
- Ort: `crates/contextra-mvcc/src/ssi.rs:67`
```rust
pub struct ReadSet
```
Dokumentation: Protokolliert gelesene Einzelschlüssel, Präfixe und deren jeweilige Snapshot-Sequenznummer für SSI. ### Speicher- und Risikoanalyse: Der Speicherbedarf pro Transaktion wächst linear $O(R + P)$ mit der Anzahl der gelesenen Schlüssel $R$ und Präfixe $P$. Bei massiven Lese-Transaktionen muss die Obergrenze über [`crate::tx_buffer::TxBuffer::try_register_read`] überwacht werden, um Speichererschöpfung zu verhindern. ### Phantom-Schutz: Normale Einzelschlüssel-Leser registrieren Keys via [`ReadSet::record_read`]. Bereichsscans (Range Scans) müssen [`ReadSet::record_prefix`] aufrufen, um neu eingefü
- Impl-Block `crates/contextra-mvcc/src/ssi.rs:76`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new() -> Self` | `crates/contextra-mvcc/src/ssi.rs:78` | Erstellt ein neues, leeres [`ReadSet`]. |
| `pub fn record_read(&mut self, key: impl Into<Vec<u8>>, snapshot_seq: u64)` | `crates/contextra-mvcc/src/ssi.rs:94` | Zeichnet einen Lesezugriff auf `key` zu einer bestimmten Snapshot-Sequenznummer `snapshot_seq` auf. ### Ablauf: 1. Maskiere das Tombstone-Bit der Sequenznummer  |
| `pub fn record_prefix(&mut self, prefix: impl Into<Vec<u8>>, snapshot_seq: u64)` | `crates/contextra-mvcc/src/ssi.rs:117` | Zeichnet einen Bereichsscan-Präfix zu `snapshot_seq` auf, um Phantom-Reads zu erkennen. Wenn nach diesem Lesezugriff eine andere Transaktion einen neuen Schlüss |
| `pub fn register_read(&mut self, key: impl Into<Vec<u8>>, snapshot_seq: u64)` | `crates/contextra-mvcc/src/ssi.rs:136` | Alias-Funktion für [`ReadSet::record_read`]. |
| `pub fn get(&self, key: &[u8]) -> Option<u64>` | `crates/contextra-mvcc/src/ssi.rs:141` | Liefert die registrierte Snapshot-Sequenznummer für einen Schlüssel, falls vorhanden. |
| `pub fn is_empty(&self) -> bool` | `crates/contextra-mvcc/src/ssi.rs:146` | Liefert `true`, falls weder Einzelschlüssel noch Präfixe im ReadSet enthalten sind. |
| `pub fn len(&self) -> usize` | `crates/contextra-mvcc/src/ssi.rs:151` | Liefert die Gesamtzahl aller getrackten Schlüssel und Präfixe. |
| `pub fn clear(&mut self)` | `crates/contextra-mvcc/src/ssi.rs:156` | Leert das ReadSet vollständig und setzt die kleinste Sequenznummer zurück. |
| `pub fn min_snapshot_seq(&self) -> Option<u64>` | `crates/contextra-mvcc/src/ssi.rs:163` | Liefert die kleinste Snapshot-Sequenznummer aller gelesenen Schlüssel und Präfixe. |
| `pub fn iter(&self) -> impl Iterator<Item = (&Vec<u8>, &u64)>` | `crates/contextra-mvcc/src/ssi.rs:168` | Liefert einen Iterator über alle gelesenen Schlüssel und deren Snapshot-Sequenzen. |
| `pub fn prefixes_iter(&self) -> impl Iterator<Item = (&Vec<u8>, &u64)>` | `crates/contextra-mvcc/src/ssi.rs:173` | Liefert einen Iterator über alle Bereichs-Präfixe und deren Snapshot-Sequenzen. |
| `pub fn keys_map(&self) -> &AHashMap<Vec<u8>, u64>` | `crates/contextra-mvcc/src/ssi.rs:178` | Liefert die interne Schlüssel-Map als Referenz. |
| `pub fn prefixes_map(&self) -> &AHashMap<Vec<u8>, u64>` | `crates/contextra-mvcc/src/ssi.rs:183` | Liefert die interne Präfix-Map als Referenz. |

#### `struct SequenceLogSsiValidator`
- Ort: `crates/contextra-mvcc/src/ssi.rs:528`
```rust
pub struct SequenceLogSsiValidator
```
Dokumentation: Referenz-Implementierung von [`SsiValidator`] auf Basis von [`SequenceLog`] und In-Memory-Commit-Tracking. Arbeitet vollständig ohne Festplatten-I/O im Arbeitsspeicher und bietet deterministische Konflikterkennung für Transaktionen. ### Deterministische Invariante (INV-MVCC-SSI-1): Die Konfliktauflösung basiert rein auf den Sequenznummern der Lese- und Schreiboperationen. Gleiche Eingabereihenfolgen führen stets zu exakt identischen Validierungsergebnissen.
- Impl-Block (Trait `std::fmt::Debug`) `crates/contextra-mvcc/src/ssi.rs:539`
- Impl-Block (Trait `Default`) `crates/contextra-mvcc/src/ssi.rs:553`
- Impl-Block `crates/contextra-mvcc/src/ssi.rs:559`
- Impl-Block (Trait `SsiValidator`) `crates/contextra-mvcc/src/ssi.rs:1104`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new() -> Self` | `crates/contextra-mvcc/src/ssi.rs:563` | Erstellt einen neuen [`SequenceLogSsiValidator`] mit Standard-Obergrenze (1.000.000 Schlüssel) und Standard-[`SystemClock`]. |
| `pub fn new_with_bounds(max_tracked_keys: usize) -> Self` | `crates/contextra-mvcc/src/ssi.rs:568` | Erstellt einen neuen Validator mit benutzerdefinierter Kapazitätsgrenze für Schlüssel. |
| `pub fn new_with_clock_and_bounds(clock: Arc<dyn Clock>, max_tracked_keys: usize) -> Self` | `crates/contextra-mvcc/src/ssi.rs:573` | Erstellt einen neuen Validator mit benutzerdefiniertem [`Clock`]-Port und Kapazitätsgrenze. |
| `pub fn new_with_time_fn_and_bounds<F>(time_fn: F, max_tracked_keys: usize) -> Self where F: Fn() -> u64 + Send + Sync + 'static,` | `crates/contextra-mvcc/src/ssi.rs:587` | Erstellt einen neuen Validator mit einer Zeitstempel-Closure (State-Passing per `Fn() -> u64`) und Kapazitätsgrenze. |
| `pub fn with_clock(mut self, clock: Arc<dyn Clock>) -> Self` | `crates/contextra-mvcc/src/ssi.rs:604` | Bindet eine benutzerdefinierte Implementierung des [`Clock`]-Ports an den Validator. |
| `pub fn with_time_fn<F>(self, time_fn: F) -> Self where F: Fn() -> u64 + Send + Sync + 'static,` | `crates/contextra-mvcc/src/ssi.rs:610` | Bindet eine Zeitstempel-Closure (State-Passing per `Fn() -> u64`) an den Validator. |
| `pub fn set_clock(&mut self, clock: Arc<dyn Clock>)` | `crates/contextra-mvcc/src/ssi.rs:627` | Ersetzt die aktive [`Clock`]-Instanz. |
| `pub fn clock(&self) -> &Arc<dyn Clock>` | `crates/contextra-mvcc/src/ssi.rs:632` | Liefert eine Referenz auf den aktiven [`Clock`]-Port. |
| `pub fn with_sequence_log(seq_log: Arc<RwLock<SequenceLog>>) -> Self` | `crates/contextra-mvcc/src/ssi.rs:637` | Erstellt einen Validator, der an ein bestehendes [`SequenceLog`] angebunden ist. |
| `pub fn with_snapshot_registry(mut self, registry: Arc<SnapshotRegistry>) -> Self` | `crates/contextra-mvcc/src/ssi.rs:651` | Verknüpft eine [`SnapshotRegistry`] zur Überwachung aktiver Snapshot-Pins. |
| `pub fn set_snapshot_registry(&mut self, registry: Arc<SnapshotRegistry>)` | `crates/contextra-mvcc/src/ssi.rs:657` | Setzt die verknüpfte [`SnapshotRegistry`]. |
| `pub fn with_metrics_sink( mut self, sink: Arc<parking_lot::RwLock<Arc<dyn MetricsSink>>>, ) -> Self` | `crates/contextra-mvcc/src/ssi.rs:662` | Bindet einen Metrik-Empfänger ([`MetricsSink`]) für Monitoring an. |
| `pub fn set_metrics_sink(&mut self, sink: Arc<parking_lot::RwLock<Arc<dyn MetricsSink>>>)` | `crates/contextra-mvcc/src/ssi.rs:671` | Aktualisiert den Metrik-Empfänger. |
| `pub fn with_max_pin_duration(mut self, duration: Duration) -> Self` | `crates/contextra-mvcc/src/ssi.rs:676` | Konfiguriert die maximale Pin-Dauer für Diagnosealarme. |
| `pub fn prune_through(&self, bound_seq: u64) -> usize` | `crates/contextra-mvcc/src/ssi.rs:693` | Bereinigt (prunt) bestätigte Schreib-Einträge mit `commit_seq <= bound_seq`. Aktualisiert den atomaren `pruned_through`-Wasserstand. ### Fail-Closed-Sicherheit  |
| `pub fn prune_through_checked(&self, bound: Option<u64>) -> Result<usize>` | `crates/contextra-mvcc/src/ssi.rs:711` | Bereinigt bestätigte Schreib-Einträge bis zu einem optionalen Grenzwert mit expliziter Fehlerbehandlung. ### Fehlerfälle: Liefert `Err(MvccError::Internal(...)) |
| `pub fn forget_from(&self, first_seq: u64) -> usize` | `crates/contextra-mvcc/src/ssi.rs:736` | Entfernt alle Commit-Registrierungen mit `commit_seq >= first_seq` (z. B. nach Transaktionsabbruch). |
| `pub fn pruned_through(&self) -> u64` | `crates/contextra-mvcc/src/ssi.rs:743` | Liefert den aktuellen `pruned_through`-Wasserstand. |
| `pub fn tracked_commit_keys(&self) -> usize` | `crates/contextra-mvcc/src/ssi.rs:748` | Liefert die Anzahl aktuell gehaltener Commit-Schlüssel. |
| `pub fn max_tracked_keys(&self) -> usize` | `crates/contextra-mvcc/src/ssi.rs:753` | Liefert die konfigurierte Obergrenze für Schlüssel. |
| `pub fn diagnose_pruning_blocker(&self) -> Option<PruningBlockerInfo>` | `crates/contextra-mvcc/src/ssi.rs:820` | Diagnostiziert, welcher unbereinigte Stand oder aktiver Snapshot das Pruning im Validator blockiert. |
| `pub fn diagnose_pruning_blocker_at(&self, now_nanos: u64) -> Option<PruningBlockerInfo>` | `crates/contextra-mvcc/src/ssi.rs:825` | Diagnostiziert Blockaden bezüglich eines expliziten Zeitstempels `now_nanos`. |
| `pub fn record_commit_key(&self, key: &[u8], commit_seq: u64)` | `crates/contextra-mvcc/src/ssi.rs:873` | Zeichnet einen committeten Schlüssel zu einer Sequenznummer auf. ### Ablauf: 1. Tombstone-Bit der Commit-Sequenznummer ausmaskieren. 2. Schlüssel im Schreibregi |
| `pub fn record_commit_keys<'a>( &self, keys: impl IntoIterator<Item = &'a [u8]>, commit_seq: u64, )` | `crates/contextra-mvcc/src/ssi.rs:881` | Zeichnet eine Menge committeter Schlüssel stapelweise (Batch) zu einer Commit-Sequenznummer auf. |
| `pub fn validate_and_record<'a>( &self, tx_id: TxId, read_set: &ReadSet, write_keys: impl IntoIterator<Item = &'a [u8]>, commit_seq: u64, ) -> Result<()>` | `crates/contextra-mvcc/src/ssi.rs:1085` | Atomically validates `read_set` and records `write_keys` at `commit_seq` under a single write lock. Führt die Validierung und das Eintragen der Commit-Schlüssel |

#### `trait SsiValidator`
- Ort: `crates/contextra-mvcc/src/ssi.rs:195`
```rust
pub trait SsiValidator
```
Dokumentation: Schnittstelle (Trait) für die Validierung von Transaktionen unter Serializable Snapshot Isolation (SSI). Prüft vor dem Festschreiben (Commit) einer Transaktion, ob nebenläufige Transaktionen Schlüssel oder Präfixe verändert haben, die im [`ReadSet`] dieser Transaktion enthalten sind.

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `fn validate(&self, tx_id: TxId, read_set: &ReadSet) -> Result<()>` | `crates/contextra-mvcc/src/ssi.rs:207` | Validiert das [`ReadSet`] der Transaktion `tx_id`. ### Fehlerfälle: Gibt `Err(MvccError::Conflict(...))` zurück, falls: - Ein Schlüssel im ReadSet nach dem Lese |

#### `const DEFAULT_MAX_TRACKED_COMMIT_KEYS`
- Ort: `crates/contextra-mvcc/src/ssi.rs:51`
```rust
pub const DEFAULT_MAX_TRACKED_COMMIT_KEYS: usize = 1_000_000
```
Dokumentation: Standard-Höchstgrenze für im Speicher gehaltene bestätigte Schreib-Schlüssel (1.000.000 Schlüssel).

### Modul `tombstone`

#### `struct SeqBitTombstone`
- Ort: `crates/contextra-mvcc/src/tombstone.rs:38`
```rust
pub struct SeqBitTombstone
```
Dokumentation: Standard-Referenzimplementierung für [`TombstoneSemanticsCheck`]. Nutzt bitweise ODER-Verknüpfung mit [`TOMBSTONE_BIT`] sowie Flag-Prüfung auf Bit 0.
- Impl-Block (Trait `TombstoneSemanticsCheck`) `crates/contextra-mvcc/src/tombstone.rs:40`

#### `trait TombstoneSemanticsCheck`
- Ort: `crates/contextra-mvcc/src/tombstone.rs:23`
```rust
pub trait TombstoneSemanticsCheck
```
Dokumentation: Trait zur Bestimmung und Erzeugung von Tombstones in Sequenzprotokollen und Indizes. NOTE: In Contextra wird das höchstwertige Bit (`TOMBSTONE_BIT` = 1 << 63) als Indikator für gelöschte Versionen verwendet. Zusätzlich können Low-Level-Flags (z. B. Bit 0x01) signalisieren, dass ein Key-Value-Record ein Tombstone ist.

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `fn is_tombstone(seq: u64, flags: u8) -> bool` | `crates/contextra-mvcc/src/tombstone.rs:25` | Gibt `true` zurück, falls die Sequenznummer oder die Flags eine gelöschte Version (Tombstone) anzeigen. |
| `fn make_tombstone(seq: u64) -> Vec<u8>` | `crates/contextra-mvcc/src/tombstone.rs:28` | Kodiert eine logische Sequenznummer als 8-Byte Little-Endian Bytevektor mit gesetztem `TOMBSTONE_BIT`. |

### Modul `tx_buffer`

#### `struct TxBuffer`
- Ort: `crates/contextra-mvcc/src/tx_buffer.rs:236`
```rust
pub struct TxBuffer<T: Clone>
```
Dokumentation: Verwaltet gepufferte Transaktionen, deren Staging-Operationen und ReadSets. ### Synchronisations- und Sperrstrategie: Jeder Shard besitzt ein eigenes `parking_lot::RwLock`. Leseoperationen erfordern ein Read-Lock, Mutationen ein Write-Lock. Es wird niemals mehr als ein Shard-Lock gleichzeitig gehalten. Multi-Shard-Scans durchlaufen die Shards strikt aufsteigend (0 bis $N-1$).
- Impl-Block (Trait `std::fmt::Debug`) `crates/contextra-mvcc/src/tx_buffer.rs:245`
- Impl-Block `crates/contextra-mvcc/src/tx_buffer.rs:256`
- Impl-Block `crates/contextra-mvcc/src/tx_buffer.rs:766`
- Impl-Block (Trait `Default`) `crates/contextra-mvcc/src/tx_buffer.rs:899`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new() -> Self` | `crates/contextra-mvcc/src/tx_buffer.rs:259` | Erstellt einen neuen Puffer mit 64 Shards, 30 Sekunden Timeout und [`SystemClock`]. |
| `pub fn new_with_config(shard_count: usize, tx_timeout: Duration) -> Self` | `crates/contextra-mvcc/src/tx_buffer.rs:264` | Erstellt einen neuen Puffer mit konfigurierter Shard-Anzahl und Transaktions-Timeout. |
| `pub fn new_with_config_ext( shard_count: usize, _tx_timeout: Duration, config: TxBufferConfig, ) -> Self` | `crates/contextra-mvcc/src/tx_buffer.rs:273` | Erstellt einen neuen Puffer mit expliziter Konfiguration ([`TxBufferConfig`]). |
| `pub fn new_with_clock_and_config( clock: Arc<dyn Clock>, shard_count: usize, config: TxBufferConfig, ) -> Self` | `crates/contextra-mvcc/src/tx_buffer.rs:282` | Erstellt einen Puffer mit benutzerdefinierter [`Clock`], Shard-Anzahl und Konfiguration. |
| `pub fn new_with_time_fn_and_config<F>( time_fn: F, shard_count: usize, config: TxBufferConfig, ) -> Self where F: Fn() -> u64 + Send + Sync + 'static,` | `crates/contextra-mvcc/src/tx_buffer.rs:305` | Erstellt einen Puffer mit einer Zeitstempel-Closure (State-Passing per `Fn() -> u64`), Shard-Anzahl und Konfiguration. |
| `pub fn with_clock(mut self, clock: Arc<dyn Clock>) -> Self` | `crates/contextra-mvcc/src/tx_buffer.rs:326` | Bindet eine benutzerdefinierte [`Clock`]-Instanz an diesen Puffer an. |
| `pub fn with_time_fn<F>(self, time_fn: F) -> Self where F: Fn() -> u64 + Send + Sync + 'static,` | `crates/contextra-mvcc/src/tx_buffer.rs:332` | Bindet eine Zeitstempel-Closure (State-Passing per `Fn() -> u64`) an diesen Puffer an. |
| `pub fn clock(&self) -> &Arc<dyn Clock>` | `crates/contextra-mvcc/src/tx_buffer.rs:349` | Liefert eine Referenz auf den aktiven [`Clock`]-Port. |
| `pub fn staged_bytes(&self) -> usize` | `crates/contextra-mvcc/src/tx_buffer.rs:354` | Liefert den aktuell global über alle Transaktionen verbrauchten Staging-Speicher in Bytes. |
| `pub fn has_tx(&self, tx: TxId) -> bool` | `crates/contextra-mvcc/src/tx_buffer.rs:383` | Prüft, ob die gegebene Transaktion im Puffer registriert ist. |
| `pub fn begin_at(&self, tx: TxId, at_nanos: u64)` | `crates/contextra-mvcc/src/tx_buffer.rs:399` | Registriert eine neue Transaktion im Puffer zu einem bestimmten Nanosekunden-Zeitstempel. ### Ablauf: 1. Shard-Index für `tx` per Hash berechnen. 2. Shard exklu |
| `pub fn begin(&self, tx: TxId)` | `crates/contextra-mvcc/src/tx_buffer.rs:413` | Registriert eine neue Transaktion mit der aktuellen Uhrzeit. |
| `pub fn register_read(&self, tx: TxId, key: impl Into<Vec<u8>>, snapshot_seq: u64)` | `crates/contextra-mvcc/src/tx_buffer.rs:420` | Registriert einen Lesezugriff im ReadSet der Transaktion `tx`. |
| `pub fn try_register_read( &self, tx: TxId, key: impl Into<Vec<u8>>, snapshot_seq: u64, max_keys: usize, ) -> Result<()>` | `crates/contextra-mvcc/src/tx_buffer.rs:439` | Registriert einen gelesenen Schlüssel für Transaktion `tx` mit Obergrenzen-Prüfung (`max_keys`). Verhindert unbegrenztes Speicherwachstum bei massiven Lese-Scan |
| `pub fn record_read(&self, tx: TxId, key: impl Into<Vec<u8>>, snapshot_seq: u64)` | `crates/contextra-mvcc/src/tx_buffer.rs:467` | Alias-Funktion für [`TxBuffer::register_read`]. |
| `pub fn min_read_snapshot(&self) -> Option<u64>` | `crates/contextra-mvcc/src/tx_buffer.rs:476` | Findet die kleinste Snapshot-Sequenznummer über alle aktiven ReadSets aller Shards hinweg. Dient der Bestimmung des globalen GC-Schwellenwerts (Tombstone-Compac |
| `pub fn get_read_set(&self, tx: TxId) -> Option<ReadSet>` | `crates/contextra-mvcc/src/tx_buffer.rs:490` | Liefert eine Kopie des angesammelten [`ReadSet`] für Transaktion `tx`. |
| `pub fn read_set(&self, tx: TxId) -> Option<ReadSet>` | `crates/contextra-mvcc/src/tx_buffer.rs:498` | Alias-Funktion für [`TxBuffer::get_read_set`]. |
| `pub fn stage(&self, tx: TxId, op: IndexOp<T>) -> Result<()> where T: StagedOpSize,` | `crates/contextra-mvcc/src/tx_buffer.rs:504` | Puffert eine Indexoperation für die Transaktion `tx` unter Prüfung aller Speichergrenzen. |
| `pub fn stage_bounded(&self, tx: TxId, op: IndexOp<T>) -> Result<()> where T: StagedOpSize,` | `crates/contextra-mvcc/src/tx_buffer.rs:520` | Puffert eine Operation und erzwingt Obergrenzen für Operationsanzahl und Byte-Budgets. ### Ablauf: 1. Berechne geschätzten Speicherbedarf der Operation in Bytes |
| `pub fn stage_many(&self, tx: TxId, ops: impl IntoIterator<Item = IndexOp<T>>) -> Result<()> where T: StagedOpSize,` | `crates/contextra-mvcc/src/tx_buffer.rs:570` | Puffert mehrere Operationen stapelweise mit einer einzigen Sperrung des Shards. |
| `pub fn validate_pending_ops(&self, tx: TxId) -> Result<()>` | `crates/contextra-mvcc/src/tx_buffer.rs:625` | Prüft, ob für eine registrierte Transaktion tatsächlich anhängige Operationen existieren. |
| `pub fn drain(&self, tx: TxId) -> Vec<IndexOp<T>>` | `crates/contextra-mvcc/src/tx_buffer.rs:644` | Entnimmt alle gepufferten Operationen der Transaktion `tx` für den anschließenden Commit und gibt den reservierten Speicherplatz frei. |
| `pub fn discard(&self, tx: TxId)` | `crates/contextra-mvcc/src/tx_buffer.rs:665` | Verwirft alle gepufferten Operationen und das ReadSet einer Transaktion (Rollback). |
| `pub fn len(&self) -> usize` | `crates/contextra-mvcc/src/tx_buffer.rs:680` | Liefert die Gesamtzahl aller aktuell gepufferten Transaktionen über alle Shards. |
| `pub fn is_empty(&self) -> bool` | `crates/contextra-mvcc/src/tx_buffer.rs:685` | Gibt `true` zurück, wenn in keinem einzigen Shard gepufferte Transaktionen liegen. |
| `pub fn reap_orphans_at(&self, now_nanos: u64) -> Vec<TxId>` | `crates/contextra-mvcc/src/tx_buffer.rs:691` | Räumt abgelaufene (verwaiste) Transaktionen bezüglich `now_nanos` über alle Shards ab. |
| `pub fn reap_orphans(&self) -> Vec<TxId>` | `crates/contextra-mvcc/src/tx_buffer.rs:696` | Räumt verwaiste Transaktionen unter Verwendung der aktuellen Uhrzeit ab. |
| `pub fn reap_orphans_bounded_at(&self, max: usize, now_nanos: u64) -> Vec<TxId>` | `crates/contextra-mvcc/src/tx_buffer.rs:709` | Räumt maximal `max` verwaiste Transaktionen ab, um Latenzspitzen zu vermeiden. ### Ablauf: 1. Iteriere aufsteigend über die Shards (0 bis $N-1$). 2. Versuche Sp |
| `pub fn reap_orphans_bounded(&self, max: usize) -> Vec<TxId>` | `crates/contextra-mvcc/src/tx_buffer.rs:745` | Räumt bis zu `max` verwaiste Transaktionen bezogen auf die aktuelle Uhrzeit ab. |
| `pub fn get_ops(&self, tx: TxId) -> Option<Vec<IndexOp<T>>>` | `crates/contextra-mvcc/src/tx_buffer.rs:751` | Liefert eine Kopie der gepufferten Operationen einer Transaktion. |
| `pub fn tx_timeout(&self) -> Duration` | `crates/contextra-mvcc/src/tx_buffer.rs:759` | Liefert die konfigurierte Transaktions-Timeout-Dauer. |
| `pub fn stage_kv(&self, tx: TxId, op: IndexOp<(Vec<u8>, Vec<u8>)>) -> Result<()>` | `crates/contextra-mvcc/src/tx_buffer.rs:804` | Puffert eine Schlüssel-Wert-Operation und aktualisiert die Key-Staging-Map. Bei Fehlschlag von `stage()` (z. B. Budget-Überlauf) wird das Key-Tracking atomar zu |
| `pub fn drain_kv(&self, tx: TxId) -> Vec<IndexOp<(Vec<u8>, Vec<u8>)>>` | `crates/contextra-mvcc/src/tx_buffer.rs:834` | Entnimmt alle gepufferten KV-Operationen einer Transaktion und bereinigt das Key-Staging. |
| `pub fn discard_kv(&self, tx: TxId)` | `crates/contextra-mvcc/src/tx_buffer.rs:843` | Verwirft alle KV-Operationen einer Transaktion und bereinigt das Key-Staging. |
| `pub fn is_key_staged_for_tx(&self, tx_id: TxId, key: &[u8]) -> bool` | `crates/contextra-mvcc/src/tx_buffer.rs:851` | Prüft, ob ein Schlüssel `key` innerhalb der Transaktion `tx_id` als Einfügung vorgemerkt ist. |
| `pub fn staged_status(&self, key: &[u8]) -> Option<bool>` | `crates/contextra-mvcc/src/tx_buffer.rs:871` | Ermittelt atomar den Staging-Status eines Schlüssels über alle uncommitted Transaktionen. Liefert `Some(true)` bei Einfügung, `Some(false)` bei Entfernung oder  |

#### `struct TxBufferConfig`
- Ort: `crates/contextra-mvcc/src/tx_buffer.rs:75`
```rust
pub struct TxBufferConfig
```
Dokumentation: Konfigurationsoptionen für den [`TxBuffer`].
- Impl-Block (Trait `Default`) `crates/contextra-mvcc/src/tx_buffer.rs:92`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `tx_timeout` | `Duration` | `crates/contextra-mvcc/src/tx_buffer.rs:77` |
| `max_active_tx` | `usize` | `crates/contextra-mvcc/src/tx_buffer.rs:79` |
| `max_ops_per_tx` | `usize` | `crates/contextra-mvcc/src/tx_buffer.rs:83` |
| `max_tx_staged_bytes` | `usize` | `crates/contextra-mvcc/src/tx_buffer.rs:86` |
| `max_total_staged_bytes` | `usize` | `crates/contextra-mvcc/src/tx_buffer.rs:89` |

#### `enum IndexOp`
- Ort: `crates/contextra-mvcc/src/tx_buffer.rs:110`
```rust
pub enum IndexOp<T: Clone>
```
Dokumentation: Eine im Puffer vorgemerkte Operation auf einem Index.
- Impl-Block `crates/contextra-mvcc/src/tx_buffer.rs:127`
- Impl-Block `crates/contextra-mvcc/src/tx_buffer.rs:182`

Varianten:

- `Insert{` — `crates/contextra-mvcc/src/tx_buffer.rs:112`
- `Delete{` — `crates/contextra-mvcc/src/tx_buffer.rs:119`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn doc_id(&self) -> DocId` | `crates/contextra-mvcc/src/tx_buffer.rs:129` | Liefert die Kennung des Dokuments (`DocId`), das von dieser Operation betroffen ist. |
| `pub fn estimated_bytes(&self) -> usize` | `crates/contextra-mvcc/src/tx_buffer.rs:184` | Schätzt den gesamten Speicherbedarf der Operation in Bytes (Overhead + Nutzdaten). |

#### `trait StagedOpSize`
- Ort: `crates/contextra-mvcc/src/tx_buffer.rs:141`
```rust
pub trait StagedOpSize
```
Dokumentation: Trait zur Schätzung des Speicherverbrauchs in Bytes für gepufferte Nutzdaten. Ermöglicht genaue Buchführung des verbrauchten Hauptspeichers pro Transaktion und global.

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `fn staged_bytes(&self) -> usize` | `crates/contextra-mvcc/src/tx_buffer.rs:143` | Liefert den geschätzten Speicherbedarf der Nutzdaten in Bytes. |

#### `const DEFAULT_MAX_OPS_PER_TX`
- Ort: `crates/contextra-mvcc/src/tx_buffer.rs:59`
```rust
pub const DEFAULT_MAX_OPS_PER_TX: usize = 10_000
```
Dokumentation: Empfohlene Maximalanzahl an Operationen pro Einzelschreibtransaktion (10.000 Operationen).

#### `const DEFAULT_MAX_READ_SET_KEYS`
- Ort: `crates/contextra-mvcc/src/tx_buffer.rs:62`
```rust
pub const DEFAULT_MAX_READ_SET_KEYS: usize = 100_000
```
Dokumentation: Standard-Maximalanzahl an getrackten Lese-Schlüsseln pro Transaktion (100.000 Schlüssel).

#### `const DEFAULT_MAX_TOTAL_STAGED_BYTES`
- Ort: `crates/contextra-mvcc/src/tx_buffer.rs:68`
```rust
pub const DEFAULT_MAX_TOTAL_STAGED_BYTES: usize = 256 * 1024 * 1024
```
Dokumentation: Standard-Maximalgröße für den globalen Puffer über alle Transaktionen hinweg (256 MiB).

#### `const DEFAULT_MAX_TX_STAGED_BYTES`
- Ort: `crates/contextra-mvcc/src/tx_buffer.rs:65`
```rust
pub const DEFAULT_MAX_TX_STAGED_BYTES: usize = 16 * 1024 * 1024
```
Dokumentation: Standard-Maximalgröße für gepufferte Rohdaten pro Einzelschreibtransaktion (16 MiB).

#### `const DEFAULT_SHARD_COUNT`
- Ort: `crates/contextra-mvcc/src/tx_buffer.rs:56`
```rust
pub const DEFAULT_SHARD_COUNT: usize = 64
```
Dokumentation: Standardanzahl an Partitionen (Shards) im Transaktionspuffer (64 Shards).

#### `const STAGING_ENTRY_OVERHEAD_BYTES`
- Ort: `crates/contextra-mvcc/src/tx_buffer.rs:53`
```rust
pub const STAGING_ENTRY_OVERHEAD_BYTES: usize = 32
```
Dokumentation: Fester Speicher-Overhead pro gepufferter Operation (32 Bytes).

## 9. Impl-Blöcke ohne Typ im selben Crate

- `impl From<MvccError> for ContextraError` — `crates/contextra-mvcc/src/compat/error.rs:15` (0 pub Methoden)
- `impl<T> StagedOpSize for Vec<T>` — `crates/contextra-mvcc/src/tx_buffer.rs:146` (0 pub Methoden)
- `impl StagedOpSize for (Vec<u8>, Vec<u8>)` — `crates/contextra-mvcc/src/tx_buffer.rs:152` (0 pub Methoden)
- `impl StagedOpSize for String` — `crates/contextra-mvcc/src/tx_buffer.rs:158` (0 pub Methoden)
- `impl StagedOpSize for usize` — `crates/contextra-mvcc/src/tx_buffer.rs:164` (0 pub Methoden)
- `impl StagedOpSize for u64` — `crates/contextra-mvcc/src/tx_buffer.rs:170` (0 pub Methoden)
- `impl StagedOpSize for u8` — `crates/contextra-mvcc/src/tx_buffer.rs:176` (0 pub Methoden)
