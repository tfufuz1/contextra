# Schnittstellenspezifikation: `contextra-types`

> Erzeugt durch `tools/extract_interfaces.py` aus dem Quellcode. Keine Handpflege. Jede Angabe trägt Datei:Zeile. Maßgeblich bleibt der Quellcode.

## 1. Kopf

| Feld | Wert |
|---|---|
| Name | `contextra-types` |
| Version | workspace |
| Pfad | `crates/contextra-types` |
| Rolle | Gemeinsame Typen |
| Bibliothekspfad | src/lib.rs (Standard) |

**Crate-Dokumentation (//!):**

> `Contextra` Types — Kanonische Domänenmodelle, Identifikatoren, Budgets, Filter und Fehlertypen.
>
> # Architektur-Rolle: Gruppe contracts (Ebene 2, Typen-Fundament)
>
> Dieses Crate definiert das fundamentale Typfundament für das gesamte Contextra-System.
> Gemäß der Baukasten-Architektur (reiner Rechenkern):
> - Keine asynchronen Laufzeiten (kein `tokio`).
> - Keine direkten I/O- oder Dateisystemabhängigkeiten.
> - Kein `unsafe` (`#![forbid(unsafe_code)]`).
> - Vollständig deterministische Datenstrukturen und fehlerresistente DTO-Konvertierungen.
>
> # Einbindung im Workspace

## 2. Abhängigkeiten

| Name | Art | Quelle | optional | verwendet von (normal/dev) |
|---|---|---|---|---|
| `ahash` | normal | workspace | nein |  |
| `bincode` | normal | workspace | nein |  |
| `blake3` | normal | workspace | nein |  |
| `bytes` | normal | workspace | nein |  |
| `contextra-ids` | normal | workspace | nein | contextra-checkpoint, contextra-crypto, contextra-graph, contextra-mvcc, contextra-store, contextra-text, contextra-types, contextra-vector |
| `criterion` | dev | workspace | nein |  |
| `proptest` | dev | registry | nein |  |
| `serde` | normal | workspace | nein |  |
| `serde_json` | normal | workspace | nein |  |
| `sha2` | normal | registry | nein |  |
| `thiserror` | normal | workspace | nein |  |

**Verwendet von (Crate-Ebene):** `contextra`, `contextra-adapt`, `contextra-agent`, `contextra-bench`, `contextra-checkpoint`, `contextra-cognition`, `contextra-crypto`, `contextra-crypto-fuzz`, `contextra-db`, `contextra-engine`, `contextra-graph`, `contextra-graph-fuzz`, `contextra-infer-candle`, `contextra-infer-ollama`, `contextra-infer-onnx`, `contextra-ingest`, `contextra-kvcache`, `contextra-mcp`, `contextra-mvcc`, `contextra-ports`, `contextra-py`, `contextra-rank`, `contextra-router`, `contextra-simd`, `contextra-store`, `contextra-store-fuzz`, `contextra-testkit`, `contextra-text`, `contextra-vault`, `contextra-vector`, `contextra-vector-fuzz`

## 3. Features

| Feature | Aktiviert |
|---|---|
| `auto-extraction-opt-out` | – |
| `default` | – |
| `docid-128` | `contextra-ids/docid-128` |

## 4. Metriken (Quellcode)

| Kennzahl | Wert |
|---|---|
| Quelldateien | 25 |
| Quellzeilen | 7510 |
| Öffentliche Items (Modul-Ebene) | 100 |
| davon `fn` | 2 |
| davon `struct` | 42 |
| davon `enum` | 33 |
| davon `trait` | 1 |
| davon `type` | 3 |
| davon `const` | 2 |
| davon `mod` | 17 |
| Öffentliche Makros (`#[macro_export]`) | 0 |
| Öffentliche Re-Exports (`pub use`) | 25 |
| `unsafe`-Vorkommen (Produktionscode) | 0 |
| `.unwrap()` (Produktionscode) | 62 |
| `.expect(` (Produktionscode) | 22 |
| `panic!` (Produktionscode) | 1 |
| `todo!/unimplemented!/unreachable!` | 0 |
| Indexierungen `x[…]` (grob, Produktionscode) | 12 |
| `#[deprecated]` | 44 |
| `#[cfg(feature…)]`-Gates | 1 |
| Parser-Abdeckung (geparste `pub`-Deklarationen / Grep-Treffer) | 230 / 230 = 100 % |

## 5. Modulbaum und Dateien

| Modul | Datei | Zeilen |
|---|---|---|
| `erasure` | `crates/contextra-types/src/erasure.rs` | 617 |
| `error` | `crates/contextra-types/src/error.rs` | 1177 |
| `error_dto` | `crates/contextra-types/src/error_dto.rs` | 599 |
| `ids` | `crates/contextra-types/src/ids.rs` | 69 |
| `crate` | `crates/contextra-types/src/lib.rs` | 76 |
| `model_fingerprint` | `crates/contextra-types/src/model_fingerprint.rs` | 79 |
| `refs` | `crates/contextra-types/src/refs.rs` | 5 |
| `retrieval_strategy` | `crates/contextra-types/src/retrieval_strategy.rs` | 101 |
| `schema` | `crates/contextra-types/src/schema.rs` | 129 |
| `tenant_scope` | `crates/contextra-types/src/tenant_scope.rs` | 3 |
| `tombstone` | `crates/contextra-types/src/tombstone.rs` | 159 |
| `types::budget` | `crates/contextra-types/src/types/budget.rs` | 709 |
| `types::domain::document` | `crates/contextra-types/src/types/domain/document.rs` | 247 |
| `types::domain::ids` | `crates/contextra-types/src/types/domain/ids.rs` | 3 |
| `types::domain::misc` | `crates/contextra-types/src/types/domain/misc.rs` | 624 |
| `types::domain` | `crates/contextra-types/src/types/domain/mod.rs` | 43 |
| `types::domain::tests::doc_tests` | `crates/contextra-types/src/types/domain/tests/doc_tests.rs` | 293 |
| `types::domain::tests::ids_tests` | `crates/contextra-types/src/types/domain/tests/ids_tests.rs` | 473 |
| `types::domain::tests::misc_tests` | `crates/contextra-types/src/types/domain/tests/misc_tests.rs` | 265 |
| `types::domain::tests` | `crates/contextra-types/src/types/domain/tests/mod.rs` | 6 |
| `types::filter` | `crates/contextra-types/src/types/filter.rs` | 427 |
| `types::importance` | `crates/contextra-types/src/types/importance.rs` | 329 |
| `types::saos` | `crates/contextra-types/src/types/saos.rs` | 942 |
| `types` | `crates/contextra-types/src/types.rs` | 37 |
| `witness` | `crates/contextra-types/src/witness.rs` | 98 |

## 6. Re-Exports (`pub use`)

- `pub use contextra_ids::*` — `crates/contextra-types/src/ids.rs:7` (Modul `ids`)
- `pub use erasure::*` — `crates/contextra-types/src/lib.rs:61` (Modul `crate`)
- `pub use ids::` — `crates/contextra-types/src/lib.rs:63` (Modul `crate`)
- `pub use refs::` — `crates/contextra-types/src/lib.rs:64` (Modul `crate`)
- `pub use error::` — `crates/contextra-types/src/lib.rs:67` (Modul `crate`)
- `pub use error_dto::ContextraErrorDto` — `crates/contextra-types/src/lib.rs:68` (Modul `crate`)
- `pub use model_fingerprint::ModelFingerprint` — `crates/contextra-types/src/lib.rs:69` (Modul `crate`)
- `pub use retrieval_strategy::RetrievalStrategy` — `crates/contextra-types/src/lib.rs:70` (Modul `crate`)
- `pub use schema::` — `crates/contextra-types/src/lib.rs:71` (Modul `crate`)
- `pub use tenant_scope::` — `crates/contextra-types/src/lib.rs:72` (Modul `crate`)
- `pub use tombstone::` — `crates/contextra-types/src/lib.rs:73` (Modul `crate`)
- `pub use types::*` — `crates/contextra-types/src/lib.rs:74` (Modul `crate`)
- `pub use witness::` — `crates/contextra-types/src/lib.rs:76` (Modul `crate`)
- `pub use contextra_ids::` — `crates/contextra-types/src/refs.rs:3` (Modul `refs`)
- `pub use contextra_ids::` — `crates/contextra-types/src/tenant_scope.rs:3` (Modul `tenant_scope`)
- `pub use budget::*` — `crates/contextra-types/src/types.rs:33` (Modul `types`)
- `pub use domain::*` — `crates/contextra-types/src/types.rs:34` (Modul `types`)
- `pub use filter::*` — `crates/contextra-types/src/types.rs:35` (Modul `types`)
- `pub use importance::*` — `crates/contextra-types/src/types.rs:36` (Modul `types`)
- `pub use saos::*` — `crates/contextra-types/src/types.rs:37` (Modul `types`)
- `pub use contextra_ids::` — `crates/contextra-types/src/types/domain/ids.rs:3` (Modul `types::domain::ids`)
- `pub use contextra_ids::TOMBSTONE_BIT` — `crates/contextra-types/src/types/domain/misc.rs:80` (Modul `types::domain::misc`)
- `pub use document::*` — `crates/contextra-types/src/types/domain/mod.rs:38` (Modul `types::domain`)
- `pub use ids::*` — `crates/contextra-types/src/types/domain/mod.rs:39` (Modul `types::domain`)
- `pub use misc::*` — `crates/contextra-types/src/types/domain/mod.rs:40` (Modul `types::domain`)

## 8. Öffentliche Schnittstelle je Item

### Modul `crate`

#### `mod erasure`
- Ort: `crates/contextra-types/src/lib.rs:55`
```rust
pub mod erasure
```
Dokumentation: Audit-Trails, Zeugnisse (Witnesses) und Saga-Transaktionsaufzeichnungen.

#### `mod error`
- Ort: `crates/contextra-types/src/lib.rs:31`
```rust
pub mod error
```
Dokumentation: Fehlertypen und Fehler-Hierarchie für Contextra.

#### `mod error_dto`
- Ort: `crates/contextra-types/src/lib.rs:34`
```rust
pub mod error_dto
```
Dokumentation: Serialisierbare Data-Transfer-Objects (DTO) für fehlerresistente Schnittstellenübertragung.

#### `mod ids`
- Ort: `crates/contextra-types/src/lib.rs:56`
```rust
pub mod ids
```

#### `mod model_fingerprint`
- Ort: `crates/contextra-types/src/lib.rs:37`
```rust
pub mod model_fingerprint
```
Dokumentation: Kryptografische Modell-Fingerabdrücke zur Validierung von Embedding- und Inferenz-Konfigurationen.

#### `mod refs`
- Ort: `crates/contextra-types/src/lib.rs:57`
```rust
pub mod refs
```

#### `mod retrieval_strategy`
- Ort: `crates/contextra-types/src/lib.rs:40`
```rust
pub mod retrieval_strategy
```
Dokumentation: Retrieval-Strategien und Routing-Optionen für die hybride Suche.

#### `mod schema`
- Ort: `crates/contextra-types/src/lib.rs:43`
```rust
pub mod schema
```
Dokumentation: Manifest-Schemaversionen und Dokument-ID-Breitenkonfigurationen.

#### `mod tenant_scope`
- Ort: `crates/contextra-types/src/lib.rs:46`
```rust
pub mod tenant_scope
```
Dokumentation: Mandantenspezifische Wrapper-Typen zur Durchsetzung strikter Mandantenisolation.

#### `mod tombstone`
- Ort: `crates/contextra-types/src/lib.rs:49`
```rust
pub mod tombstone
```
Dokumentation: Bitmaskenbasierte Tombstones (Löschmarker) für Multi-Version-Concurrency-Control (MVCC).

#### `mod types`
- Ort: `crates/contextra-types/src/lib.rs:52`
```rust
pub mod types
```
Dokumentation: Zentrale Domänentypen, Filter, Budgets, Scoring-Funktionen und SAOS-Suchstrukturen.

#### `mod witness`
- Ort: `crates/contextra-types/src/lib.rs:58`
```rust
pub mod witness
```

### Modul `erasure`

#### `struct Armed`
- Ort: `crates/contextra-types/src/erasure.rs:438`
```rust
pub struct Armed(())
```

#### `struct ControlEvidence`
- Ort: `crates/contextra-types/src/erasure.rs:446`
```rust
pub struct ControlEvidence(())
```
- Impl-Block `crates/contextra-types/src/erasure.rs:448`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub const fn observed() -> Self` | `crates/contextra-types/src/erasure.rs:451` | Bezeugt das Auffinden des Kontrollmusters (nur in autorisierten Modulen aufgerufen). |

#### `struct EraseReceipt`
- Ort: `crates/contextra-types/src/erasure.rs:137`
```rust
pub struct EraseReceipt
```

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `id` | `ErasureId` | `crates/contextra-types/src/erasure.rs:139` |
| `status` | `ErasureStatus` | `crates/contextra-types/src/erasure.rs:141` |
| `gated_at` | `Timestamp` | `crates/contextra-types/src/erasure.rs:143` |

#### `struct EraseRequest`
- Ort: `crates/contextra-types/src/erasure.rs:120`
```rust
pub struct EraseRequest
```

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `target` | `EraseTarget` | `crates/contextra-types/src/erasure.rs:122` |
| `timing` | `Timing` | `crates/contextra-types/src/erasure.rs:124` |
| `retention` | `RetentionAttestation` | `crates/contextra-types/src/erasure.rs:126` |
| `mode` | `EraseMode` | `crates/contextra-types/src/erasure.rs:128` |

#### `struct LayerId`
- Ort: `crates/contextra-types/src/erasure.rs:152`
```rust
pub struct LayerId(pub [u8
```
- Impl-Block `crates/contextra-types/src/erasure.rs:154`
- Impl-Block (Trait `fmt::Debug`) `crates/contextra-types/src/erasure.rs:166`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub const fn new(bytes: [u8` | `crates/contextra-types/src/erasure.rs:156` | Erzeugt eine neue `LayerId`. |
| `pub const fn as_bytes(&self) -> &[u8` | `crates/contextra-types/src/erasure.rs:161` | Liefert die rohen Bytes der Schicht-ID. |

#### `struct LayerWitness`
- Ort: `crates/contextra-types/src/erasure.rs:559`
```rust
pub struct LayerWitness
```
- Impl-Block `crates/contextra-types/src/erasure.rs:569`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn layer(&self) -> LayerId` | `crates/contextra-types/src/erasure.rs:572` | Liefert die Schicht-ID. |
| `pub fn class(&self) -> MeasureClass` | `crates/contextra-types/src/erasure.rs:578` | Liefert die Messklasse (MK1-MK3). |
| `pub fn method_id(&self) -> u16` | `crates/contextra-types/src/erasure.rs:584` | Liefert die Methoden-ID. |
| `pub fn files(&self) -> u64` | `crates/contextra-types/src/erasure.rs:590` | Liefert die Anzahl geprüfter Dateien. |
| `pub fn bytes(&self) -> u64` | `crates/contextra-types/src/erasure.rs:596` | Liefert die Anzahl geprüfter Bytes. |
| `pub fn duration(&self) -> Duration` | `crates/contextra-types/src/erasure.rs:602` | Liefert die Scandauer. |
| `pub fn hash(&self) -> &[u8` | `crates/contextra-types/src/erasure.rs:608` | Liefert den kryptografischen Hash des Prüflaufs. |
| `pub const fn matches(&self) -> u64` | `crates/contextra-types/src/erasure.rs:614` | matches() ist per Konstruktion immer 0 (I-19). |

#### `struct Residue`
- Ort: `crates/contextra-types/src/erasure.rs:462`
```rust
pub struct Residue
```
- Impl-Block (Trait `fmt::Display`) `crates/contextra-types/src/erasure.rs:467`
- Impl-Block (Trait `std::error::Error`) `crates/contextra-types/src/erasure.rs:477`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `matches` | `u64` | `crates/contextra-types/src/erasure.rs:464` |

#### `struct RetentionAttestation`
- Ort: `crates/contextra-types/src/erasure.rs:105`
```rust
pub struct RetentionAttestation
```

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `code` | `RetentionCode` | `crates/contextra-types/src/erasure.rs:107` |
| `attested_by` | `OperatorRef` | `crates/contextra-types/src/erasure.rs:109` |
| `attested_at` | `Timestamp` | `crates/contextra-types/src/erasure.rs:111` |

#### `struct Scanner`
- Ort: `crates/contextra-types/src/erasure.rs:485`
```rust
pub struct Scanner<S>
```
- Impl-Block `crates/contextra-types/src/erasure.rs:494`
- Impl-Block `crates/contextra-types/src/erasure.rs:520`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new(layer: LayerId, class: MeasureClass, method_id: u16) -> Self` | `crates/contextra-types/src/erasure.rs:496` | Erzeugt einen neuen unbewaffneten Scanner. |
| `pub fn arm(self, _ev: ControlEvidence) -> Scanner<Armed>` | `crates/contextra-types/src/erasure.rs:508` | Einzige Brücke nach Armed; erfordert Positivkontrollbeleg (TY-10). |
| `pub fn record(&mut self, files: u64, bytes: u64)` | `crates/contextra-types/src/erasure.rs:522` | Zeichnet geprüfte Dateien und Bytes auf. |
| `pub fn finish( self, matches: u64, duration: Duration, hash: [u8` | `crates/contextra-types/src/erasure.rs:530` | Schließt die Messung ab. matches != 0 => Err(Residue). LayerWitness wird ausschließlich bei matches == 0 ausgegeben (TY-10, I-19, D-72). |

#### `struct Unarmed`
- Ort: `crates/contextra-types/src/erasure.rs:431`
```rust
pub struct Unarmed
```

#### `enum EraseMode`
- Ort: `crates/contextra-types/src/erasure.rs:71`
```rust
pub enum EraseMode
```

Varianten:

- `Erase` — `crates/contextra-types/src/erasure.rs:73`
- `Restrict` — `crates/contextra-types/src/erasure.rs:75`

#### `enum EraseTarget`
- Ort: `crates/contextra-types/src/erasure.rs:21`
```rust
pub enum EraseTarget
```

Varianten:

- `Document{` — `crates/contextra-types/src/erasure.rs:23`
- `Subject{` — `crates/contextra-types/src/erasure.rs:32`
- `Collection{` — `crates/contextra-types/src/erasure.rs:39`
- `Tenant{` — `crates/contextra-types/src/erasure.rs:46`

#### `enum ErasureStatus`
- Ort: `crates/contextra-types/src/erasure.rs:239`
```rust
pub enum ErasureStatus
```

Varianten:

- `Requested` — `crates/contextra-types/src/erasure.rs:241`
- `Gated` — `crates/contextra-types/src/erasure.rs:243`
- `KeyDestroyed` — `crates/contextra-types/src/erasure.rs:245`
- `Purging` — `crates/contextra-types/src/erasure.rs:247`
- `Verifying` — `crates/contextra-types/src/erasure.rs:249`
- `Completed` — `crates/contextra-types/src/erasure.rs:251`
- `Restricted` — `crates/contextra-types/src/erasure.rs:253`
- `Lifted` — `crates/contextra-types/src/erasure.rs:255`
- `Pending{` — `crates/contextra-types/src/erasure.rs:257`
- `Refused(RefusalReason)` — `crates/contextra-types/src/erasure.rs:264`

#### `enum GateVariant`
- Ort: `crates/contextra-types/src/erasure.rs:288`
```rust
pub enum GateVariant
```

Varianten:

- `Erased` — `crates/contextra-types/src/erasure.rs:290`
- `Restricted` — `crates/contextra-types/src/erasure.rs:292`

#### `enum KeyCoverage`
- Ort: `crates/contextra-types/src/erasure.rs:273`
```rust
pub enum KeyCoverage
```

Varianten:

- `Full` — `crates/contextra-types/src/erasure.rs:275`
- `Unlinked` — `crates/contextra-types/src/erasure.rs:277`
- `None` — `crates/contextra-types/src/erasure.rs:279`

#### `enum MeasureClass`
- Ort: `crates/contextra-types/src/erasure.rs:301`
```rust
pub enum MeasureClass
```

Varianten:

- `Mk1` — `crates/contextra-types/src/erasure.rs:303`
- `Mk2` — `crates/contextra-types/src/erasure.rs:305`
- `Mk3` — `crates/contextra-types/src/erasure.rs:307`

#### `enum PendingReason`
- Ort: `crates/contextra-types/src/erasure.rs:182`
```rust
pub enum PendingReason
```

Varianten:

- `Io` — `crates/contextra-types/src/erasure.rs:184`
- `SlaExceeded` — `crates/contextra-types/src/erasure.rs:186`
- `LayerFailed(LayerId)` — `crates/contextra-types/src/erasure.rs:188`
- `WitnessMissing` — `crates/contextra-types/src/erasure.rs:190`
- `DiskReserve` — `crates/contextra-types/src/erasure.rs:192`

#### `enum Phase`
- Ort: `crates/contextra-types/src/erasure.rs:214`
```rust
pub enum Phase
```

Varianten:

- `Intent` — `crates/contextra-types/src/erasure.rs:216`
- `Gate` — `crates/contextra-types/src/erasure.rs:218`
- `KeyDestroy` — `crates/contextra-types/src/erasure.rs:220`
- `Purge` — `crates/contextra-types/src/erasure.rs:222`
- `Verify` — `crates/contextra-types/src/erasure.rs:224`
- `Replace` — `crates/contextra-types/src/erasure.rs:226`
- `Supersede` — `crates/contextra-types/src/erasure.rs:228`
- `Restrict` — `crates/contextra-types/src/erasure.rs:230`

#### `enum RefusalReason`
- Ort: `crates/contextra-types/src/erasure.rs:201`
```rust
pub enum RefusalReason
```

Varianten:

- `RetentionObligation` — `crates/contextra-types/src/erasure.rs:203`
- `InvalidParameters` — `crates/contextra-types/src/erasure.rs:205`

#### `enum ResidualRisk`
- Ort: `crates/contextra-types/src/erasure.rs:357`
```rust
pub enum ResidualRisk
```
- Impl-Block `crates/contextra-types/src/erasure.rs:390`

Varianten:

- `FlashRemanence` — `crates/contextra-types/src/erasure.rs:359`
- `FsJournalOrCow` — `crates/contextra-types/src/erasure.rs:361`
- `WrappedKeyRemanence` — `crates/contextra-types/src/erasure.rs:363`
- `FreedBlockRemanence` — `crates/contextra-types/src/erasure.rs:365`
- `OldKeyslotRemanence` — `crates/contextra-types/src/erasure.rs:367`
- `ProcessMemory` — `crates/contextra-types/src/erasure.rs:369`
- `IndexTopologyInference` — `crates/contextra-types/src/erasure.rs:371`
- `TextIndexUntilPurge` — `crates/contextra-types/src/erasure.rs:373`
- `ReplicaWindow` — `crates/contextra-types/src/erasure.rs:375`
- `ExternalCopies` — `crates/contextra-types/src/erasure.rs:377`
- `ExternalProcessors` — `crates/contextra-types/src/erasure.rs:379`
- `PseudonymCommitment` — `crates/contextra-types/src/erasure.rs:381`
- `PurgeScalesWithDb` — `crates/contextra-types/src/erasure.rs:383`
- `PurgeDeferred` — `crates/contextra-types/src/erasure.rs:385`
- `KeyCompromise` — `crates/contextra-types/src/erasure.rs:387`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn id(&self) -> &'static str` | `crates/contextra-types/src/erasure.rs:392` | Feste Identifikator-Zeichenkette nach SPEC Anhang D. |
| `pub fn condition(&self) -> RiskCondition` | `crates/contextra-types/src/erasure.rs:413` | Statische Bedingung für Audit- und Verifikationswerkzeuge (AR-4). |

#### `enum RetentionCode`
- Ort: `crates/contextra-types/src/erasure.rs:84`
```rust
pub enum RetentionCode
```

Varianten:

- `NoObligation` — `crates/contextra-types/src/erasure.rs:86`
- `ObligationExpired` — `crates/contextra-types/src/erasure.rs:88`
- `ObligationActive` — `crates/contextra-types/src/erasure.rs:90`
- `LegalClaimsPending` — `crates/contextra-types/src/erasure.rs:92`
- `DataSubjectRequestedRestriction` — `crates/contextra-types/src/erasure.rs:94`
- `OperatorOverride` — `crates/contextra-types/src/erasure.rs:96`

#### `enum RiskCondition`
- Ort: `crates/contextra-types/src/erasure.rs:342`
```rust
pub enum RiskCondition
```

Varianten:

- `Always` — `crates/contextra-types/src/erasure.rs:344`
- `CowOrJournaling` — `crates/contextra-types/src/erasure.rs:346`
- `UntilPurgeComplete` — `crates/contextra-types/src/erasure.rs:348`

#### `enum ScanMethod`
- Ort: `crates/contextra-types/src/erasure.rs:316`
```rust
pub enum ScanMethod
```
- Impl-Block `crates/contextra-types/src/erasure.rs:325`

Varianten:

- `Mk1` — `crates/contextra-types/src/erasure.rs:318`
- `Mk2` — `crates/contextra-types/src/erasure.rs:320`
- `Mk3` — `crates/contextra-types/src/erasure.rs:322`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub const fn class(&self) -> MeasureClass` | `crates/contextra-types/src/erasure.rs:327` | Liefert die zugehörige Messklasse (`MeasureClass`). |

#### `enum Timing`
- Ort: `crates/contextra-types/src/erasure.rs:58`
```rust
pub enum Timing
```

Varianten:

- `Now` — `crates/contextra-types/src/erasure.rs:60`
- `Batched` — `crates/contextra-types/src/erasure.rs:62`

### Modul `error`

#### `struct StepId`
- Ort: `crates/contextra-types/src/error.rs:61`
```rust
pub struct StepId(pub u64)
```
- Impl-Block `crates/contextra-types/src/error.rs:63`
- Impl-Block (Trait `std::fmt::Display`) `crates/contextra-types/src/error.rs:75`
- Impl-Block (Trait `From<u64>`) `crates/contextra-types/src/error.rs:81`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub const fn new(id: u64) -> Self` | `crates/contextra-types/src/error.rs:65` | Creates a new `StepId` wrapping a `u64`. |
| `pub const fn inner(self) -> u64` | `crates/contextra-types/src/error.rs:70` | Returns the inner raw `u64` value. |

#### `enum CacheDirective`
- Ort: `crates/contextra-types/src/error.rs:90`
```rust
pub enum CacheDirective
```
- Impl-Block (Trait `Default`) `crates/contextra-types/src/error.rs:107`

Varianten:

- `Pin{` — `crates/contextra-types/src/error.rs:92`
- `NeverCache` — `crates/contextra-types/src/error.rs:97`
- `Auto` — `crates/contextra-types/src/error.rs:99`
- `ReleaseAfterStep{` — `crates/contextra-types/src/error.rs:101`

#### `enum ContextraError`
- Ort: `crates/contextra-types/src/error.rs:129`
```rust
pub enum ContextraError
```
- Impl-Block `crates/contextra-types/src/error.rs:389`
- Impl-Block (Trait `From<std::array::TryFromSliceError>`) `crates/contextra-types/src/error.rs:507`
- Impl-Block (Trait `From<contextra_ids::IdError>`) `crates/contextra-types/src/error.rs:514`

Varianten:

- `Internal(String)` — `crates/contextra-types/src/error.rs:133`
- `InvalidInput(String)` — `crates/contextra-types/src/error.rs:137`
- `NotFound(String)` — `crates/contextra-types/src/error.rs:141`
- `PolicyViolation(String)` — `crates/contextra-types/src/error.rs:145`
- `Storage(String)` — `crates/contextra-types/src/error.rs:150`
- `Io(#[from] std::io::Error)` — `crates/contextra-types/src/error.rs:154`
- `WalCorruption{` — `crates/contextra-types/src/error.rs:159`
- `ChecksumMismatch{` — `crates/contextra-types/src/error.rs:169`
- `Transaction(String)` — `crates/contextra-types/src/error.rs:179`
- `TransactionTimeout{` — `crates/contextra-types/src/error.rs:183`
- `Conflict(String)` — `crates/contextra-types/src/error.rs:192`
- `InvalidSequenceNumber(u64)` — `crates/contextra-types/src/error.rs:196`
- `Index(String)` — `crates/contextra-types/src/error.rs:201`
- `EmbeddingDimensionMismatch{` — `crates/contextra-types/src/error.rs:205`
- `HnswConnectivityDegraded{` — `crates/contextra-types/src/error.rs:218`
- `Text(String)` — `crates/contextra-types/src/error.rs:225`
- `MemoryBudgetExceeded{` — `crates/contextra-types/src/error.rs:230`
- `Sandbox(String)` — `crates/contextra-types/src/error.rs:239`
- `MemoryLimitExceeded(String)` — `crates/contextra-types/src/error.rs:243`
- `SandboxTimeout(String)` — `crates/contextra-types/src/error.rs:247`
- `Timeout{` — `crates/contextra-types/src/error.rs:251`
- `Serialization(String)` — `crates/contextra-types/src/error.rs:261`
- `Json(#[from] serde_json::Error)` — `crates/contextra-types/src/error.rs:265`
- `Crypto(String)` — `crates/contextra-types/src/error.rs:269`
- `CheckpointNotFound` — `crates/contextra-types/src/error.rs:273`
- `Cluster(String)` — `crates/contextra-types/src/error.rs:277`
- `ParseError(String)` — `crates/contextra-types/src/error.rs:281`
- `Bincode(#[from] bincode::Error)` — `crates/contextra-types/src/error.rs:285`
- `CapabilityUnsupported{` — `crates/contextra-types/src/error.rs:289`
- `SnapshotUnsupportedForSignal(String)` — `crates/contextra-types/src/error.rs:298`
- `StaleRead(String)` — `crates/contextra-types/src/error.rs:302`
- `LimitExceeded{` — `crates/contextra-types/src/error.rs:306`
- `ModelLoad{` — `crates/contextra-types/src/error.rs:316`
- `OrphanedVectorReference{` — `crates/contextra-types/src/error.rs:328`
- `CommitTimeout{` — `crates/contextra-types/src/error.rs:337`
- `KvDeleteModeConfig(&'static str)` — `crates/contextra-types/src/error.rs:344`
- `KvQuantization(String)` — `crates/contextra-types/src/error.rs:348`
- `DurabilityConfig(String)` — `crates/contextra-types/src/error.rs:352`
- `Plugin(String)` — `crates/contextra-types/src/error.rs:356`
- `PinBudgetExceeded(String)` — `crates/contextra-types/src/error.rs:360`
- `GraphRepairFailed(#[from] HnswDeletionError)` — `crates/contextra-types/src/error.rs:368`
- `WalTruncationDetected{` — `crates/contextra-types/src/error.rs:372`
- `CrossDeviceLink{` — `crates/contextra-types/src/error.rs:381`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn limit_exceeded(limit: usize, context: impl Into<String>) -> Self` | `crates/contextra-types/src/error.rs:392` |  |
| `pub fn model_load(path: impl Into<String>, reason: impl Into<String>) -> Self` | `crates/contextra-types/src/error.rs:401` |  |
| `pub fn snapshot_unsupported_for_signal(strategy: impl Into<String>) -> Self` | `crates/contextra-types/src/error.rs:410` |  |
| `pub fn orphaned_vector_reference( doc_id: impl Into<String>, index_id: impl Into<String>, ) -> Self` | `crates/contextra-types/src/error.rs:416` |  |
| `pub fn capability_unsupported( capability: impl Into<String>, reason: impl Into<String>, ) -> Self` | `crates/contextra-types/src/error.rs:428` |  |
| `pub fn invalid_input(msg: impl Into<String>) -> Self` | `crates/contextra-types/src/error.rs:439` |  |
| `pub fn wal_corruption(offset: u64, reason: impl Into<String>) -> Self` | `crates/contextra-types/src/error.rs:445` |  |
| `pub fn checksum_mismatch(path: impl Into<String>, block_id: u64) -> Self` | `crates/contextra-types/src/error.rs:454` |  |
| `pub fn kv_quantization(msg: impl Into<String>) -> Self` | `crates/contextra-types/src/error.rs:463` |  |
| `pub fn durability_config(msg: impl Into<String>) -> Self` | `crates/contextra-types/src/error.rs:469` |  |
| `pub fn pin_budget_exceeded(msg: impl Into<String>) -> Self` | `crates/contextra-types/src/error.rs:475` |  |
| `pub fn cross_device_link( source_path: impl Into<String>, target_path: impl Into<String>, ) -> Self` | `crates/contextra-types/src/error.rs:481` |  |
| `pub fn wal_truncation_detected(expected_hmac: [u8` | `crates/contextra-types/src/error.rs:493` |  |
| `pub fn is_occ_conflict(&self) -> bool` | `crates/contextra-types/src/error.rs:502` |  |

#### `enum HnswDeletionError`
- Ort: `crates/contextra-types/src/error.rs:38`
```rust
pub enum HnswDeletionError
```

Varianten:

- `NodeNotFound(DocId)` — `crates/contextra-types/src/error.rs:41`
- `DisconnectedComponent(DocId)` — `crates/contextra-types/src/error.rs:45`
- `VerificationFailed{` — `crates/contextra-types/src/error.rs:49`

#### `type Result`
- Ort: `crates/contextra-types/src/error.rs:115`
```rust
pub type Result<T> = std::result::Result<T, ContextraError>
```

### Modul `error_dto`

#### `struct ContextraErrorDto`
- Ort: `crates/contextra-types/src/error_dto.rs:32`
```rust
pub struct ContextraErrorDto
```
- Impl-Block `crates/contextra-types/src/error_dto.rs:41`
- Impl-Block (Trait `std::fmt::Display`) `crates/contextra-types/src/error_dto.rs:67`
- Impl-Block (Trait `std::error::Error`) `crates/contextra-types/src/error_dto.rs:73`
- Impl-Block (Trait `From<ContextraError>`) `crates/contextra-types/src/error_dto.rs:75`
- Impl-Block (Trait `From<String>`) `crates/contextra-types/src/error_dto.rs:81`
- Impl-Block (Trait `From<&str>`) `crates/contextra-types/src/error_dto.rs:91`
- Impl-Block (Trait `From<&ContextraError>`) `crates/contextra-types/src/error_dto.rs:106`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `kind` | `String` | `crates/contextra-types/src/error_dto.rs:34` |
| `message` | `String` | `crates/contextra-types/src/error_dto.rs:36` |
| `details` | `Option<serde_json::Value>` | `crates/contextra-types/src/error_dto.rs:38` |

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new(kind: impl Into<String>, message: impl Into<String>) -> Self` | `crates/contextra-types/src/error_dto.rs:44` |  |
| `pub fn with_details( kind: impl Into<String>, message: impl Into<String>, details: serde_json::Value, ) -> Self` | `crates/contextra-types/src/error_dto.rs:54` |  |

### Modul `ids`

#### `struct ErasureId`
- Ort: `crates/contextra-types/src/ids.rs:16`
```rust
pub struct ErasureId(pub [u8
```
- Impl-Block `crates/contextra-types/src/ids.rs:18`
- Impl-Block (Trait `fmt::Debug`) `crates/contextra-types/src/ids.rs:30`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub const fn new(bytes: [u8` | `crates/contextra-types/src/ids.rs:20` | Erzeugt eine `ErasureId` aus rohen 16 Bytes. |
| `pub const fn as_bytes(&self) -> &[u8` | `crates/contextra-types/src/ids.rs:25` | Liefert die 16 Bytes als Slice. |

#### `struct GateId`
- Ort: `crates/contextra-types/src/ids.rs:47`
```rust
pub struct GateId(pub [u8
```
- Impl-Block `crates/contextra-types/src/ids.rs:49`
- Impl-Block (Trait `fmt::Debug`) `crates/contextra-types/src/ids.rs:61`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub const fn new(bytes: [u8` | `crates/contextra-types/src/ids.rs:51` | Erzeugt eine `GateId` aus rohen 16 Bytes. |
| `pub const fn as_bytes(&self) -> &[u8` | `crates/contextra-types/src/ids.rs:56` | Liefert die 16 Bytes als Slice. |

### Modul `model_fingerprint`

#### `struct ModelFingerprint`
- Ort: `crates/contextra-types/src/model_fingerprint.rs:33`
```rust
pub struct ModelFingerprint
```
Dokumentation: Identifiziert eine Modelldatei und deren Quantisierungsstufe eindeutig.
- Impl-Block `crates/contextra-types/src/model_fingerprint.rs:42`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `hash` | `[u8; 32]` | `crates/contextra-types/src/model_fingerprint.rs:35` |
| `model_id` | `String` | `crates/contextra-types/src/model_fingerprint.rs:37` |
| `quantization` | `String` | `crates/contextra-types/src/model_fingerprint.rs:39` |

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new( hash: [u8` | `crates/contextra-types/src/model_fingerprint.rs:48` | Creates a new `ModelFingerprint`. # Ablauf Übernimmt Hash, Modellbezeichner und Quantisierungsstring ohne Nebenwirkungen. |

### Modul `retrieval_strategy`

#### `enum RetrievalStrategy`
- Ort: `crates/contextra-types/src/retrieval_strategy.rs:22`
```rust
pub enum RetrievalStrategy
```
Dokumentation: Semantische Retrieval-Strategie für Signal-Fusion (§8/§9).
- Impl-Block (Trait `fmt::Display`) `crates/contextra-types/src/retrieval_strategy.rs:41`

Varianten:

- `Vector` — `crates/contextra-types/src/retrieval_strategy.rs:24`
- `Text` — `crates/contextra-types/src/retrieval_strategy.rs:26`
- `Graph` — `crates/contextra-types/src/retrieval_strategy.rs:28`
- `Hybrid` — `crates/contextra-types/src/retrieval_strategy.rs:30`
- `Global{` — `crates/contextra-types/src/retrieval_strategy.rs:33`

### Modul `schema`

#### `enum DocIdWidth`
- Ort: `crates/contextra-types/src/schema.rs:19`
```rust
pub enum DocIdWidth
```
Dokumentation: Gibt die von einer Schemaversion erwartete oder erzeugte Bitbreite einer `DocId` an.
- Impl-Block `crates/contextra-types/src/schema.rs:26`

Varianten:

- `Bit64= 64` — `crates/contextra-types/src/schema.rs:21`
- `Bit128= 128` — `crates/contextra-types/src/schema.rs:23`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn current() -> Self` | `crates/contextra-types/src/schema.rs:32` | Returns the active compile-time `DocIdWidth`. # Ablauf Prüft zur Compile-Zeit via `cfg!(feature = "docid-128")`, welche Breite gewählt ist. |
| `pub fn bytes_len(self) -> usize` | `crates/contextra-types/src/schema.rs:42` | Returns the size in bytes of a single `DocId` value under this width. |

#### `enum ManifestSchemaVersion`
- Ort: `crates/contextra-types/src/schema.rs:58`
```rust
pub enum ManifestSchemaVersion
```
Dokumentation: Schema-Versionskennung für SSTable- und WAL-Manifeste.
- Impl-Block `crates/contextra-types/src/schema.rs:66`

Varianten:

- `V1= 1` — `crates/contextra-types/src/schema.rs:61`
- `V2= 2` — `crates/contextra-types/src/schema.rs:63`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn doc_id_width(self) -> DocIdWidth` | `crates/contextra-types/src/schema.rs:69` | Returns the corresponding `DocIdWidth` for this schema version. |
| `pub fn as_u8(self) -> u8` | `crates/contextra-types/src/schema.rs:78` | Returns the numeric version byte. |
| `pub fn from_u8(version_byte: u8) -> Result<Self>` | `crates/contextra-types/src/schema.rs:87` | Parses a version from a numeric byte indicator. # Fehler Liefert `ContextraError::Serialization`, falls das übergebene Byte keiner bekannten Version entspricht. |
| `pub fn is_compatible_with_current_build(self) -> bool` | `crates/contextra-types/src/schema.rs:99` | Returns `true` if this schema version is compatible with the compile-time DocId feature flags. |

### Modul `tombstone`

#### `struct SeqBitTombstone`
- Ort: `crates/contextra-types/src/tombstone.rs:53`
```rust
pub struct SeqBitTombstone
```
- Impl-Block (Trait `TombstoneSemanticsCheck`) `crates/contextra-types/src/tombstone.rs:55`

#### `trait TombstoneSemanticsCheck`
- Ort: `crates/contextra-types/src/tombstone.rs:38`
```rust
pub trait TombstoneSemanticsCheck
```

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `fn is_tombstone(seq: u64, flags: u8) -> bool` | `crates/contextra-types/src/tombstone.rs:40` | Returns `true` if the sequence number or flags indicate a soft-deleted tombstone. |
| `fn make_tombstone(seq: u64) -> Vec<u8>` | `crates/contextra-types/src/tombstone.rs:43` | Encodes a sequence number as an 8-byte little-endian tombstone payload with `TOMBSTONE_BIT` set. |

### Modul `types`

#### `mod budget`
- Ort: `crates/contextra-types/src/types.rs:15`
```rust
pub mod budget
```

#### `mod domain`
- Ort: `crates/contextra-types/src/types.rs:19`
```rust
pub mod domain
```

#### `mod filter`
- Ort: `crates/contextra-types/src/types.rs:23`
```rust
pub mod filter
```

#### `mod importance`
- Ort: `crates/contextra-types/src/types.rs:27`
```rust
pub mod importance
```

#### `mod saos`
- Ort: `crates/contextra-types/src/types.rs:31`
```rust
pub mod saos
```

### Modul `types::budget`

#### `struct Reservation`
- Ort: `crates/contextra-types/src/types/budget.rs:131`
```rust
pub struct Reservation<'a>
```
- Impl-Block `crates/contextra-types/src/types/budget.rs:295`
- Impl-Block (Trait `Drop`) `crates/contextra-types/src/types/budget.rs:306`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn settle(self)` | `crates/contextra-types/src/types/budget.rs:301` | Bestätigt den Verbrauch endgültig. Nach `settle()` erfolgt bei Drop KEINE Rückerstattung mehr. # Compile-time Duplicate Call Prevention `settle(self)` consumes  |

#### `struct ResourceBudget`
- Ort: `crates/contextra-types/src/types/budget.rs:341`
```rust
pub struct ResourceBudget
```
- Impl-Block (Trait `Default`) `crates/contextra-types/src/types/budget.rs:346`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `memory_limit` | `u64` | `crates/contextra-types/src/types/budget.rs:343` |

#### `struct ResourceTracker`
- Ort: `crates/contextra-types/src/types/budget.rs:357`
```rust
pub struct ResourceTracker
```
- Impl-Block (Trait `Default`) `crates/contextra-types/src/types/budget.rs:364`
- Impl-Block `crates/contextra-types/src/types/budget.rs:370`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new(budget: ResourceBudget) -> Self` | `crates/contextra-types/src/types/budget.rs:372` | Creates a new ResourceTracker with the given budget. |
| `pub fn consume_memory(&self, bytes: u64) -> Result<()>` | `crates/contextra-types/src/types/budget.rs:384` | Attempts to consume `bytes` of memory from the tracked budget. # Errors Returns `Err(ContextraError::MemoryBudgetExceeded)` if consuming `bytes` exceeds the con |
| `pub fn release_memory(&self, bytes: u64)` | `crates/contextra-types/src/types/budget.rs:413` | Releases memory from the budget. Uses saturating subtraction to prevent atomic underflow (wrapping to `u64::MAX`) which would permanently block all future alloc |
| `pub fn memory_used(&self) -> u64` | `crates/contextra-types/src/types/budget.rs:437` | Returns current memory usage in bytes. |
| `pub fn budget(&self) -> &ResourceBudget` | `crates/contextra-types/src/types/budget.rs:442` | Returns a reference to the configured resource budget. |
| `pub fn has_memory_capacity(&self) -> bool` | `crates/contextra-types/src/types/budget.rs:447` | Returns true if memory usage is below 95% of the limit. |

#### `struct TokenBudget`
- Ort: `crates/contextra-types/src/types/budget.rs:51`
```rust
pub struct TokenBudget
```
- Impl-Block (Trait `Clone`) `crates/contextra-types/src/types/budget.rs:62`
- Impl-Block (Trait `PartialEq`) `crates/contextra-types/src/types/budget.rs:73`
- Impl-Block (Trait `Serialize`) `crates/contextra-types/src/types/budget.rs:82`
- Impl-Block (Trait `Deserialize<'de>`) `crates/contextra-types/src/types/budget.rs:105`
- Impl-Block `crates/contextra-types/src/types/budget.rs:137`
- Impl-Block (Trait `Default`) `crates/contextra-types/src/types/budget.rs:327`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `limit` | `usize` | `crates/contextra-types/src/types/budget.rs:53` |
| `strategy` | `BudgetStrategy` | `crates/contextra-types/src/types/budget.rs:55` |
| `reserved` | `usize` | `crates/contextra-types/src/types/budget.rs:57` |

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new(max_tokens: usize, reserve_tokens: usize) -> Self` | `crates/contextra-types/src/types/budget.rs:139` | Creates a new token budget with an exact max limit and reserve tokens. |
| `pub fn for_model(model: &str) -> Self` | `crates/contextra-types/src/types/budget.rs:154` | Erstellt ein Budget für gängige Modelle. # Maintenance Note Model name patterns use string matching. Explicit model family patterns (e.g. `claude-4`, `claude-op |
| `pub fn with_reserved(mut self, system_tokens: usize, answer_tokens: usize) -> Self` | `crates/contextra-types/src/types/budget.rs:180` | Reserviert Token für System-Prompt und Antwort. |
| `pub fn with_strategy(mut self, strategy: BudgetStrategy) -> Self` | `crates/contextra-types/src/types/budget.rs:186` | Sets the budget strategy. |
| `pub fn effective_limit(&self) -> usize` | `crates/contextra-types/src/types/budget.rs:192` | Calculates the effective token limit based on strategy. |
| `pub fn available(&self) -> usize` | `crates/contextra-types/src/types/budget.rs:201` | Returns tokens still available after subtracting reserved and consumed tokens. |
| `pub fn consume(&self, tokens: usize)` | `crates/contextra-types/src/types/budget.rs:208` | Records `tokens` as consumed, reducing future availability. |
| `pub fn reserve(&self, amount: usize) -> Result<Reservation<'_>>` | `crates/contextra-types/src/types/budget.rs:224` | Reserviert `amount` Tokens atomar. Schlägt fehl, wenn das verfügbare Budget (limit - reserved - consumed) unterschritten würde. |
| `pub fn try_reserve(&self, tokens: usize) -> Result<()>` | `crates/contextra-types/src/types/budget.rs:262` | Tries to reserve `tokens` from the available budget before step execution. Returns `Err(ContextraError::Internal(...))` if available budget is 0 or less than `t |
| `pub fn refund(&self, tokens: usize)` | `crates/contextra-types/src/types/budget.rs:275` | Refunds `tokens` back to the available budget if execution fails or consumes less than estimated. |
| `pub fn consumed(&self) -> usize` | `crates/contextra-types/src/types/budget.rs:290` | Returns total tokens consumed so far. |

#### `enum BudgetStrategy`
- Ort: `crates/contextra-types/src/types/budget.rs:39`
```rust
pub enum BudgetStrategy
```

Varianten:

- `Conservative` — `crates/contextra-types/src/types/budget.rs:41`
- `Aggressive` — `crates/contextra-types/src/types/budget.rs:43`
- `Exact(usize)` — `crates/contextra-types/src/types/budget.rs:45`

### Modul `types::domain::document`

#### `struct Embedding`
- Ort: `crates/contextra-types/src/types/domain/document.rs:174`
```rust
pub struct Embedding
```
- Impl-Block `crates/contextra-types/src/types/domain/document.rs:179`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `data` | `Vec<f32>` | `crates/contextra-types/src/types/domain/document.rs:176` |

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new(data: Vec<f32>) -> Self` | `crates/contextra-types/src/types/domain/document.rs:181` | Creates a new vector embedding from `f32` slice or vector. |
| `pub fn dim(&self) -> usize` | `crates/contextra-types/src/types/domain/document.rs:187` | Returns the dimension (length) of the embedding vector. |
| `pub fn as_slice(&self) -> &[f32]` | `crates/contextra-types/src/types/domain/document.rs:193` | Returns the vector data as an `f32` slice. |
| `pub fn l2_norm(&self) -> f32` | `crates/contextra-types/src/types/domain/document.rs:198` | Computes the Euclidean L2 norm of the vector. |
| `pub fn normalize(&self) -> Self` | `crates/contextra-types/src/types/domain/document.rs:207` | Returns an L2-unit-normalized clone of this embedding. Guarded against zero and subnormal/near-zero vector norms (`norm < 1e-12`). Vectors with `norm < 1e-12` a |

#### `struct RerankResult`
- Ort: `crates/contextra-types/src/types/domain/document.rs:238`
```rust
pub struct RerankResult
```

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `index` | `usize` | `crates/contextra-types/src/types/domain/document.rs:240` |
| `original_index` | `usize` | `crates/contextra-types/src/types/domain/document.rs:242` |
| `score` | `f32` | `crates/contextra-types/src/types/domain/document.rs:244` |
| `calibrated_score` | `f32` | `crates/contextra-types/src/types/domain/document.rs:246` |

#### `struct ScoredDocument`
- Ort: `crates/contextra-types/src/types/domain/document.rs:221`
```rust
pub struct ScoredDocument
```
- Impl-Block `crates/contextra-types/src/types/domain/document.rs:228`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `doc_id` | `DocId` | `crates/contextra-types/src/types/domain/document.rs:223` |
| `score` | `f32` | `crates/contextra-types/src/types/domain/document.rs:225` |

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new(doc_id: DocId, score: f32) -> Self` | `crates/contextra-types/src/types/domain/document.rs:230` | Creates a new `ScoredDocument` with document ID and score. |

#### `enum DistanceMetric`
- Ort: `crates/contextra-types/src/types/domain/document.rs:20`
```rust
pub enum DistanceMetric
```
- Impl-Block `crates/contextra-types/src/types/domain/document.rs:30`

Varianten:

- `Cosine` — `crates/contextra-types/src/types/domain/document.rs:23`
- `Euclidean` — `crates/contextra-types/src/types/domain/document.rs:25`
- `DotProduct` — `crates/contextra-types/src/types/domain/document.rs:27`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn compute(&self, a: &[f32], b: &[f32]) -> Result<f32>` | `crates/contextra-types/src/types/domain/document.rs:32` | Computes the distance between two f32 vectors using this metric. |
| `pub fn compute_u8(&self, a: &[u8], b: &[u8]) -> Result<u32>` | `crates/contextra-types/src/types/domain/document.rs:114` |  |

### Modul `types::domain::misc`

#### `struct ConfigFingerprint`
- Ort: `crates/contextra-types/src/types/domain/misc.rs:529`
```rust
pub struct ConfigFingerprint
```
- Impl-Block `crates/contextra-types/src/types/domain/misc.rs:547`
- Impl-Block (Trait `Default`) `crates/contextra-types/src/types/domain/misc.rs:588`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `model_id` | `String` | `crates/contextra-types/src/types/domain/misc.rs:531` |
| `quantization` | `String` | `crates/contextra-types/src/types/domain/misc.rs:534` |
| `prompt_template_hash` | `[u8; 32]` | `crates/contextra-types/src/types/domain/misc.rs:537` |
| `temperature_bits` | `u32` | `crates/contextra-types/src/types/domain/misc.rs:540` |
| `threshold_bits` | `u32` | `crates/contextra-types/src/types/domain/misc.rs:544` |

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new( model_id: impl Into<String>, quantization: impl Into<String>, prompt_template: &str, temperature: f32, ) -> Self` | `crates/contextra-types/src/types/domain/misc.rs:549` | Erstellt einen neuen `ConfigFingerprint`. |
| `pub fn with_threshold(mut self, threshold: f32) -> Self` | `crates/contextra-types/src/types/domain/misc.rs:570` | Setzt die Schwellenwert-Komponente (Threshold) für diesen Fingerabdruck (Builder Pattern). |
| `pub fn threshold(&self) -> f32` | `crates/contextra-types/src/types/domain/misc.rs:577` | Extrahiert Schwellenwert/Threshold als f32 (verlustfrei da via to_bits gespeichert). |
| `pub fn temperature(&self) -> f32` | `crates/contextra-types/src/types/domain/misc.rs:583` | Extrahiert Temperatur als f32 (verlustfrei da via to_bits gespeichert). |

#### `struct Edge`
- Ort: `crates/contextra-types/src/types/domain/misc.rs:192`
```rust
pub struct Edge
```
- Impl-Block `crates/contextra-types/src/types/domain/misc.rs:218`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `from` | `EntityId` | `crates/contextra-types/src/types/domain/misc.rs:194` |
| `to` | `EntityId` | `crates/contextra-types/src/types/domain/misc.rs:196` |
| `label` | `Arc<str>` | `crates/contextra-types/src/types/domain/misc.rs:198` |
| `weight` | `f32` | `crates/contextra-types/src/types/domain/misc.rs:200` |
| `tx_valid_from` | `Option<TxId>` | `crates/contextra-types/src/types/domain/misc.rs:203` |
| `tx_valid_to` | `Option<TxId>` | `crates/contextra-types/src/types/domain/misc.rs:206` |
| `business_valid_from` | `Option<i64>` | `crates/contextra-types/src/types/domain/misc.rs:209` |
| `business_valid_to` | `Option<i64>` | `crates/contextra-types/src/types/domain/misc.rs:212` |
| `source_doc_id` | `Option<DocId>` | `crates/contextra-types/src/types/domain/misc.rs:215` |

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new(from: EntityId, to: EntityId, label: impl Into<String>) -> Self` | `crates/contextra-types/src/types/domain/misc.rs:220` | Creates a new `Edge` between source and target entities with a label. |
| `pub fn with_source_doc_id(mut self, doc_id: DocId) -> Self` | `crates/contextra-types/src/types/domain/misc.rs:235` | Sets the optional source document ID from which this edge was derived for provenance tracking. |
| `pub fn try_new( from: EntityId, to: EntityId, label: impl Into<String>, weight: f32, ) -> Result<Self>` | `crates/contextra-types/src/types/domain/misc.rs:244` | Creates a new `Edge`, validating non-empty label and finite non-negative weight. # Errors Returns `ContextraError::InvalidInput` if `label` is empty or `weight` |
| `pub fn with_weight(mut self, weight: f32) -> Self` | `crates/contextra-types/src/types/domain/misc.rs:275` | Sets a custom weight on the edge. |
| `pub fn with_tx_validity(mut self, from: Option<TxId>, to: Option<TxId>) -> Self` | `crates/contextra-types/src/types/domain/misc.rs:281` | Sets transaction validity window (system time / MVCC) on the edge. |
| `pub fn with_business_validity(mut self, from: Option<i64>, to: Option<i64>) -> Self` | `crates/contextra-types/src/types/domain/misc.rs:288` | Sets business validity window (business time in Unix ms) on the edge. |
| `pub fn with_validity(self, from: Option<TxId>, to: Option<TxId>) -> Self` | `crates/contextra-types/src/types/domain/misc.rs:295` | Sets transaction validity window on the edge (alias for `with_tx_validity` for backward compatibility). |

#### `struct Entity`
- Ort: `crates/contextra-types/src/types/domain/misc.rs:129`
```rust
pub struct Entity
```
- Impl-Block `crates/contextra-types/src/types/domain/misc.rs:141`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `id` | `EntityId` | `crates/contextra-types/src/types/domain/misc.rs:131` |
| `name` | `Arc<str>` | `crates/contextra-types/src/types/domain/misc.rs:133` |
| `entity_type` | `Arc<str>` | `crates/contextra-types/src/types/domain/misc.rs:135` |

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new(id: EntityId, name: impl Into<String>, entity_type: impl Into<String>) -> Self` | `crates/contextra-types/src/types/domain/misc.rs:143` | Creates a new `Entity` with id, name, and type. |
| `pub fn try_new( id: EntityId, name: impl Into<String>, entity_type: impl Into<String>, ) -> Result<Self>` | `crates/contextra-types/src/types/domain/misc.rs:156` | Creates a new `Entity`, validating that `name` and `entity_type` are non-empty. # Errors Returns `ContextraError::InvalidInput` if `name` or `entity_type` is em |

#### `struct MemoryLink`
- Ort: `crates/contextra-types/src/types/domain/misc.rs:117`
```rust
pub struct MemoryLink
```

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `target` | `DocId` | `crates/contextra-types/src/types/domain/misc.rs:119` |
| `relation` | `LinkRelation` | `crates/contextra-types/src/types/domain/misc.rs:121` |
| `created_at_tx` | `TxId` | `crates/contextra-types/src/types/domain/misc.rs:123` |

#### `struct PprConfig`
- Ort: `crates/contextra-types/src/types/domain/misc.rs:484`
```rust
pub struct PprConfig
```
- Impl-Block (Trait `Default`) `crates/contextra-types/src/types/domain/misc.rs:507`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `damping_factor` | `f32` | `crates/contextra-types/src/types/domain/misc.rs:486` |
| `max_iterations` | `u32` | `crates/contextra-types/src/types/domain/misc.rs:488` |
| `convergence_epsilon` | `f32` | `crates/contextra-types/src/types/domain/misc.rs:490` |
| `algorithm` | `PprAlgorithm` | `crates/contextra-types/src/types/domain/misc.rs:493` |
| `warn_on_non_convergence` | `bool` | `crates/contextra-types/src/types/domain/misc.rs:499` |

#### `struct TlHfdParams`
- Ort: `crates/contextra-types/src/types/domain/misc.rs:389`
```rust
pub struct TlHfdParams
```
- Impl-Block (Trait `Default`) `crates/contextra-types/src/types/domain/misc.rs:430`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `sigma` | `f32` | `crates/contextra-types/src/types/domain/misc.rs:392` |
| `delta` | `f32` | `crates/contextra-types/src/types/domain/misc.rs:395` |
| `gamma` | `f32` | `crates/contextra-types/src/types/domain/misc.rs:398` |
| `max_iterations` | `u32` | `crates/contextra-types/src/types/domain/misc.rs:401` |
| `max_top_k_expansion` | `usize` | `crates/contextra-types/src/types/domain/misc.rs:404` |
| `max_hyperedge_sort_size` | `usize` | `crates/contextra-types/src/types/domain/misc.rs:407` |

#### `struct WorkflowState`
- Ort: `crates/contextra-types/src/types/domain/misc.rs:61`
```rust
pub struct WorkflowState
```

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `tx` | `TxId` | `crates/contextra-types/src/types/domain/misc.rs:63` |
| `graph_hash` | `[u8; 32]` | `crates/contextra-types/src/types/domain/misc.rs:66` |

#### `enum AutoExtractionMode`
- Ort: `crates/contextra-types/src/types/domain/misc.rs:598`
```rust
pub enum AutoExtractionMode
```
- Impl-Block `crates/contextra-types/src/types/domain/misc.rs:605`
- Impl-Block (Trait `Default`) `crates/contextra-types/src/types/domain/misc.rs:613`

Varianten:

- `Enabled` — `crates/contextra-types/src/types/domain/misc.rs:600`
- `Disabled` — `crates/contextra-types/src/types/domain/misc.rs:602`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn is_enabled(self) -> bool` | `crates/contextra-types/src/types/domain/misc.rs:608` | Returns `true` if auto extraction mode is enabled. |

#### `enum LinkRelation`
- Ort: `crates/contextra-types/src/types/domain/misc.rs:103`
```rust
pub enum LinkRelation
```

Varianten:

- `Elaborates` — `crates/contextra-types/src/types/domain/misc.rs:105`
- `Contradicts` — `crates/contextra-types/src/types/domain/misc.rs:107`
- `Supersedes` — `crates/contextra-types/src/types/domain/misc.rs:109`
- `References` — `crates/contextra-types/src/types/domain/misc.rs:111`

#### `enum MemoryType`
- Ort: `crates/contextra-types/src/types/domain/misc.rs:315`
```rust
pub enum MemoryType
```
- Impl-Block `crates/contextra-types/src/types/domain/misc.rs:342`

Varianten:

- `Episodic` — `crates/contextra-types/src/types/domain/misc.rs:320`
- `Semantic` — `crates/contextra-types/src/types/domain/misc.rs:327`
- `Procedural` — `crates/contextra-types/src/types/domain/misc.rs:333`
- `Working` — `crates/contextra-types/src/types/domain/misc.rs:339`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn default_decay(&self) -> crate::types::importance::DecayFunction` | `crates/contextra-types/src/types/domain/misc.rs:344` | Gibt den Standard-Decay-Typ für diesen Gedächtnistyp zurück. |
| `pub fn default_ttl_tx(&self) -> Option<u64>` | `crates/contextra-types/src/types/domain/misc.rs:361` | Gibt die empfohlene TTL (in Transaktionen) für Session-scoped Working Memory zurück. None bedeutet kein automatisches Ablaufen. |
| `pub fn as_metadata_key(&self) -> &'static str` | `crates/contextra-types/src/types/domain/misc.rs:369` | Gibt den kanonischen Metadaten-Key zurück (für JSON-Serialisierung). |

#### `enum PprAlgorithm`
- Ort: `crates/contextra-types/src/types/domain/misc.rs:455`
```rust
pub enum PprAlgorithm
```

Varianten:

- `Auto` — `crates/contextra-types/src/types/domain/misc.rs:458`
- `DensePowerIteration` — `crates/contextra-types/src/types/domain/misc.rs:460`
- `ForwardPush` — `crates/contextra-types/src/types/domain/misc.rs:462`
- `ShadowMode` — `crates/contextra-types/src/types/domain/misc.rs:464`
- `TlHfd(TlHfdParams)` — `crates/contextra-types/src/types/domain/misc.rs:467`
- `ShadowModeTlHfd(TlHfdParams)` — `crates/contextra-types/src/types/domain/misc.rs:470`

#### `const EXPIRY_METADATA_KEY`
- Ort: `crates/contextra-types/src/types/domain/misc.rs:84`
```rust
pub const EXPIRY_METADATA_KEY: &str = ""
```

#### `const MAX_SEARCH_K`
- Ort: `crates/contextra-types/src/types/domain/misc.rs:98`
```rust
pub const MAX_SEARCH_K: usize = 1_000
```

### Modul `types::domain::misc::hex`

#### `fn deserialize`
- Ort: `crates/contextra-types/src/types/domain/misc.rs:34`
```rust
pub fn deserialize<'de, D>(deserializer: D) -> Result<[u8
```

#### `fn serialize`
- Ort: `crates/contextra-types/src/types/domain/misc.rs:20`
```rust
pub fn serialize<S>(bytes: &[u8
```

### Modul `types::filter`

#### `enum FilterExpr`
- Ort: `crates/contextra-types/src/types/filter.rs:26`
```rust
pub enum FilterExpr
```
- Impl-Block `crates/contextra-types/src/types/filter.rs:114`

Varianten:

- `Eq{` — `crates/contextra-types/src/types/filter.rs:28`
- `Ne{` — `crates/contextra-types/src/types/filter.rs:35`
- `Gt{` — `crates/contextra-types/src/types/filter.rs:42`
- `Gte{` — `crates/contextra-types/src/types/filter.rs:49`
- `Lt{` — `crates/contextra-types/src/types/filter.rs:56`
- `Lte{` — `crates/contextra-types/src/types/filter.rs:63`
- `In{` — `crates/contextra-types/src/types/filter.rs:70`
- `NotIn{` — `crates/contextra-types/src/types/filter.rs:77`
- `Exists{` — `crates/contextra-types/src/types/filter.rs:84`
- `And(Box<FilterExpr>, Box<FilterExpr>)` — `crates/contextra-types/src/types/filter.rs:91`
- `Or(Box<FilterExpr>, Box<FilterExpr>)` — `crates/contextra-types/src/types/filter.rs:93`
- `Not(Box<FilterExpr>)` — `crates/contextra-types/src/types/filter.rs:95`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn evaluate(&self, metadata: &serde_json::Value) -> bool` | `crates/contextra-types/src/types/filter.rs:116` | Evaluates the filter expression against a JSON metadata object. |

### Modul `types::importance`

#### `struct ImportanceScore`
- Ort: `crates/contextra-types/src/types/importance.rs:37`
```rust
pub struct ImportanceScore(f32)
```
- Impl-Block `crates/contextra-types/src/types/importance.rs:39`
- Impl-Block (Trait `Default`) `crates/contextra-types/src/types/importance.rs:58`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new(raw: f32) -> Self` | `crates/contextra-types/src/types/importance.rs:44` | Creates a new `ImportanceScore` clamped strictly within `[0.0, 1.0]`. # Zero-Panic Guarantee NaN inputs return `ImportanceScore(0.0)`. Positive/negative infinit |
| `pub fn value(&self) -> f32` | `crates/contextra-types/src/types/importance.rs:53` | Returns the raw `f32` importance score value. |

#### `struct MemoryImportance`
- Ort: `crates/contextra-types/src/types/importance.rs:186`
```rust
pub struct MemoryImportance
```
- Impl-Block `crates/contextra-types/src/types/importance.rs:195`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `base_score` | `ImportanceScore` | `crates/contextra-types/src/types/importance.rs:188` |
| `decay` | `DecayFunction` | `crates/contextra-types/src/types/importance.rs:190` |
| `created_at_tx` | `TxId` | `crates/contextra-types/src/types/importance.rs:192` |

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new(base_score: ImportanceScore, decay: DecayFunction, created_at_tx: TxId) -> Self` | `crates/contextra-types/src/types/importance.rs:197` | Creates a new `MemoryImportance` configuration. |
| `pub fn effective_score(&self, now_tx: TxId) -> f32` | `crates/contextra-types/src/types/importance.rs:209` | Computes the effective importance score: `base_score * decay_factor`. # Zero-Panic Guarantee Always returns a valid `f32` in `[0.0, 1.0]`. |

#### `enum DecayFunction`
- Ort: `crates/contextra-types/src/types/importance.rs:68`
```rust
pub enum DecayFunction
```
- Impl-Block `crates/contextra-types/src/types/importance.rs:89`

Varianten:

- `None` — `crates/contextra-types/src/types/importance.rs:71`
- `Exponential{` — `crates/contextra-types/src/types/importance.rs:73`
- `StepFloor{` — `crates/contextra-types/src/types/importance.rs:78`
- `WallClockExponential{` — `crates/contextra-types/src/types/importance.rs:83`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn decay_factor(&self, created_at_tx: TxId, now_tx: TxId) -> f32` | `crates/contextra-types/src/types/importance.rs:102` | Pure non-failing decay factor calculation. Returns a factor in `[0.0, 1.0]`. # Invarianten & Edge Cases - If `now_tx < created_at_tx` (e.g., due to snapshot rea |
| `pub fn decay_factor_wallclock( &self, created_at_epoch_secs: u64, now_epoch_secs: u64, created_at_tx: TxId, now_tx: TxId, ) -> f32` | `crates/contextra-types/src/types/importance.rs:155` | Wall-clock based decay factor calculation. Returns a factor in `[0.0, 1.0]` based on elapsed wall-clock seconds. For non-WallClockExponential variants, delegate |

### Modul `types::saos`

#### `struct ContextChunk`
- Ort: `crates/contextra-types/src/types/saos.rs:232`
```rust
pub struct ContextChunk
```
- Impl-Block `crates/contextra-types/src/types/saos.rs:252`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `doc_id` | `DocId` | `crates/contextra-types/src/types/saos.rs:234` |
| `content` | `String` | `crates/contextra-types/src/types/saos.rs:236` |
| `relevance` | `f32` | `crates/contextra-types/src/types/saos.rs:238` |
| `token_count` | `usize` | `crates/contextra-types/src/types/saos.rs:240` |
| `metadata` | `Option<serde_json::Value>` | `crates/contextra-types/src/types/saos.rs:242` |
| `contextual_prefix` | `Option<String>` | `crates/contextra-types/src/types/saos.rs:246` |
| `links` | `Vec<crate::types::domain::MemoryLink>` | `crates/contextra-types/src/types/saos.rs:249` |

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn combined_text_owned(&self) -> String` | `crates/contextra-types/src/types/saos.rs:258` | Kombinierten Text für Text-Indizierung und Embedding. Entspricht dem etablierten Contextual-Schema: prefix + "\n\n" + content. Allokiert nur wenn `contextual_pr |
| `pub fn combined_token_count(&self) -> usize` | `crates/contextra-types/src/types/saos.rs:273` | Kombinierter Token-Count (Heuristik: chars/4 ≈ Tokens). Verwendet für Budget-Management in `contextra-db/src/context.rs`. |
| `pub fn has_context_prefix(&self) -> bool` | `crates/contextra-types/src/types/saos.rs:281` | Gibt true zurück wenn contextual_prefix gesetzt und nicht leer. |

#### `struct ContextWindow`
- Ort: `crates/contextra-types/src/types/saos.rs:291`
```rust
pub struct ContextWindow
```
- Impl-Block (Trait `std::fmt::Display`) `crates/contextra-types/src/types/saos.rs:300`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `chunks` | `Vec<ContextChunk>` | `crates/contextra-types/src/types/saos.rs:293` |
| `total_tokens` | `usize` | `crates/contextra-types/src/types/saos.rs:295` |
| `truncated` | `bool` | `crates/contextra-types/src/types/saos.rs:297` |

#### `struct FusionWeights`
- Ort: `crates/contextra-types/src/types/saos.rs:154`
```rust
pub struct FusionWeights
```
- Impl-Block (Trait `Default`) `crates/contextra-types/src/types/saos.rs:162`
- Impl-Block `crates/contextra-types/src/types/saos.rs:173`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new(vector: f32, text: f32, graph: f32) -> Result<Self>` | `crates/contextra-types/src/types/saos.rs:175` | Creates 3-signal fusion weights (vector, text, graph) summing to 1.0. |
| `pub fn vector(&self) -> f32` | `crates/contextra-types/src/types/saos.rs:208` | Returns the vector signal weight component. |
| `pub fn text(&self) -> f32` | `crates/contextra-types/src/types/saos.rs:213` | Returns the text signal weight component. |
| `pub fn graph(&self) -> f32` | `crates/contextra-types/src/types/saos.rs:218` | Returns the graph signal weight component. |
| `pub fn metadata(&self) -> f32` | `crates/contextra-types/src/types/saos.rs:224` | Returns the metadata signal weight component. Reserved for future metadata-signal fusion; currently always 0.0. Do not expose a public constructor parameter for |

#### `struct HybridQuery`
- Ort: `crates/contextra-types/src/types/saos.rs:424`
```rust
pub struct HybridQuery
```
- Impl-Block `crates/contextra-types/src/types/saos.rs:476`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `text_query` | `Option<String>` | `crates/contextra-types/src/types/saos.rs:426` |
| `vector_query` | `Option<Vec<f32>>` | `crates/contextra-types/src/types/saos.rs:428` |
| `graph_start_node` | `Option<String>` | `crates/contextra-types/src/types/saos.rs:430` |
| `graph_strategy` | `GraphTraversalStrategy` | `crates/contextra-types/src/types/saos.rs:433` |
| `fusion_weights` | `FusionWeights` | `crates/contextra-types/src/types/saos.rs:435` |
| `fusion_strategy` | `SignalFusionStrategies` | `crates/contextra-types/src/types/saos.rs:438` |
| `filter` | `Option<FilterExpr>` | `crates/contextra-types/src/types/saos.rs:440` |
| `same_community_as` | `Option<EntityId>` | `crates/contextra-types/src/types/saos.rs:442` |
| `memory_type_filter` | `Option<Vec<MemoryType>>` | `crates/contextra-types/src/types/saos.rs:446` |
| `include_superseded` | `bool` | `crates/contextra-types/src/types/saos.rs:449` |
| `include_provenance` | `bool` | `crates/contextra-types/src/types/saos.rs:452` |
| `rerank_pool_multiplier` | `Option<usize>` | `crates/contextra-types/src/types/saos.rs:457` |
| `rerank_pool_max` | `Option<usize>` | `crates/contextra-types/src/types/saos.rs:461` |
| `has_reranker` | `bool` | `crates/contextra-types/src/types/saos.rs:465` |
| `on_signal_failure` | `OnSignalFailure` | `crates/contextra-types/src/types/saos.rs:468` |
| `k` | `usize` | `crates/contextra-types/src/types/saos.rs:470` |

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn builder() -> HybridQueryBuilder` | `crates/contextra-types/src/types/saos.rs:478` | Creates a new `HybridQueryBuilder`. |

#### `struct HybridQueryBuilder`
- Ort: `crates/contextra-types/src/types/saos.rs:493`
```rust
pub struct HybridQueryBuilder
```
- Impl-Block `crates/contextra-types/src/types/saos.rs:514`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new() -> Self` | `crates/contextra-types/src/types/saos.rs:516` | Creates a new `HybridQueryBuilder` with empty default options. |
| `pub fn with_text_query(mut self, q: impl Into<String>) -> Self` | `crates/contextra-types/src/types/saos.rs:521` | Sets the full-text search query string. |
| `pub fn with_vector_query(mut self, v: Vec<f32>) -> Self` | `crates/contextra-types/src/types/saos.rs:527` | Sets the vector query embedding. |
| `pub fn with_graph_start_node(mut self, start: impl Into<String>) -> Self` | `crates/contextra-types/src/types/saos.rs:533` | Sets the graph start node for graph traversal search. |
| `pub fn with_graph_strategy(mut self, strategy: GraphTraversalStrategy) -> Self` | `crates/contextra-types/src/types/saos.rs:539` | Sets the graph traversal strategy (Hops or PersonalizedPageRank). |
| `pub fn with_fusion_weights(mut self, weights: FusionWeights) -> Self` | `crates/contextra-types/src/types/saos.rs:545` | Sets custom signal fusion weights. |
| `pub fn with_fusion_strategy(mut self, strategy: impl Into<SignalFusionStrategies>) -> Self` | `crates/contextra-types/src/types/saos.rs:551` | Sets search fusion strategy per signal or globally (`FusionStrategy` or `SignalFusionStrategies`). |
| `pub fn with_filter(mut self, filter: FilterExpr) -> Self` | `crates/contextra-types/src/types/saos.rs:557` | Sets a metadata expression filter on the query. |
| `pub fn with_same_community_as(mut self, entity_id: EntityId) -> Self` | `crates/contextra-types/src/types/saos.rs:563` | Sets the community context entity filter/boost. |
| `pub fn with_memory_type_filter(mut self, types: Vec<MemoryType>) -> Self` | `crates/contextra-types/src/types/saos.rs:569` | Sets the memory_type pre-RRF filter. |
| `pub fn with_include_superseded(mut self, include: bool) -> Self` | `crates/contextra-types/src/types/saos.rs:575` | Sets whether to include superseded memory chunks (ADR-038). |
| `pub fn with_include_provenance(mut self, include: bool) -> Self` | `crates/contextra-types/src/types/saos.rs:581` | Sets whether to calculate and attach ProvenanceRecord to output results. |
| `pub fn with_rerank_pool_multiplier(mut self, multiplier: usize) -> Self` | `crates/contextra-types/src/types/saos.rs:587` | Sets the candidate pool multiplier for pre-reranking retrieval. |
| `pub fn with_rerank_pool_max(mut self, max: usize) -> Self` | `crates/contextra-types/src/types/saos.rs:593` | Sets the upper bound cap for pre-reranking candidate pool expansion. |
| `pub fn with_on_signal_failure(mut self, policy: OnSignalFailure) -> Self` | `crates/contextra-types/src/types/saos.rs:599` | Sets the signal failure policy ([`OnSignalFailure`]) for the query (Spec Teil 6.5). |
| `pub fn with_k(mut self, k: usize) -> Self` | `crates/contextra-types/src/types/saos.rs:605` | Sets the top-K limit for the query. |
| `pub fn build(self) -> Result<HybridQuery>` | `crates/contextra-types/src/types/saos.rs:614` | Builds the `HybridQuery` instance. # Errors Returns `Err` if query parameters or fusion weights fail validation. |

#### `struct QueryRewriteOutput`
- Ort: `crates/contextra-types/src/types/saos.rs:365`
```rust
pub struct QueryRewriteOutput
```
- Impl-Block `crates/contextra-types/src/types/saos.rs:375`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `text_query` | `Option<String>` | `crates/contextra-types/src/types/saos.rs:367` |
| `semantic_query` | `Option<String>` | `crates/contextra-types/src/types/saos.rs:369` |
| `anchor_entities` | `Vec<String>` | `crates/contextra-types/src/types/saos.rs:372` |

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new( text_query: Option<String>, semantic_query: Option<String>, anchor_entities: Vec<String>, ) -> Self` | `crates/contextra-types/src/types/saos.rs:377` | Creates a new `QueryRewriteOutput` with text, semantic, and anchor entity reformulations. |
| `pub fn text_only(text: impl Into<String>) -> Self` | `crates/contextra-types/src/types/saos.rs:390` | Creates a simple text-only `QueryRewriteOutput`. |
| `pub fn is_empty(&self) -> bool` | `crates/contextra-types/src/types/saos.rs:399` | Returns `true` if all reformulation components are empty or None. |

#### `struct ScoredEntry`
- Ort: `crates/contextra-types/src/types/saos.rs:412`
```rust
pub struct ScoredEntry
```

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `id` | `String` | `crates/contextra-types/src/types/saos.rs:414` |
| `final_score` | `f32` | `crates/contextra-types/src/types/saos.rs:416` |
| `metadata` | `Option<serde_json::Value>` | `crates/contextra-types/src/types/saos.rs:418` |

#### `struct SignalFusionStrategies`
- Ort: `crates/contextra-types/src/types/saos.rs:44`
```rust
pub struct SignalFusionStrategies
```
- Impl-Block (Trait `Default`) `crates/contextra-types/src/types/saos.rs:53`
- Impl-Block (Trait `From<FusionStrategy>`) `crates/contextra-types/src/types/saos.rs:63`
- Impl-Block `crates/contextra-types/src/types/saos.rs:73`
- Impl-Block (Trait `Deserialize<'de>`) `crates/contextra-types/src/types/saos.rs:90`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `vector` | `FusionStrategy` | `crates/contextra-types/src/types/saos.rs:46` |
| `text` | `FusionStrategy` | `crates/contextra-types/src/types/saos.rs:48` |
| `graph` | `FusionStrategy` | `crates/contextra-types/src/types/saos.rs:50` |

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn uniform(strategy: FusionStrategy) -> Self` | `crates/contextra-types/src/types/saos.rs:75` | Creates a uniform configuration where all signals use the specified strategy. |
| `pub fn strategy_for_signal(&self, signal_name: &str) -> FusionStrategy` | `crates/contextra-types/src/types/saos.rs:80` | Returns the fusion strategy configured for a specific signal name ("vector", "text", "graph"). |

#### `enum FusionStrategy`
- Ort: `crates/contextra-types/src/types/saos.rs:31`
```rust
pub enum FusionStrategy
```

Varianten:

- `Rrf` — `crates/contextra-types/src/types/saos.rs:35`
- `ScoreNormalized` — `crates/contextra-types/src/types/saos.rs:38`

#### `enum GraphTraversalStrategy`
- Ort: `crates/contextra-types/src/types/saos.rs:127`
```rust
pub enum GraphTraversalStrategy
```
- Impl-Block (Trait `Default`) `crates/contextra-types/src/types/saos.rs:145`

Varianten:

- `Hops{` — `crates/contextra-types/src/types/saos.rs:129`
- `PersonalizedPageRank(PprConfig)` — `crates/contextra-types/src/types/saos.rs:135`
- `PathRag{` — `crates/contextra-types/src/types/saos.rs:137`

#### `enum OnSignalFailure`
- Ort: `crates/contextra-types/src/types/saos.rs:354`
```rust
pub enum OnSignalFailure
```

Varianten:

- `Fail` — `crates/contextra-types/src/types/saos.rs:357`
- `Degrade` — `crates/contextra-types/src/types/saos.rs:359`

#### `enum Signal`
- Ort: `crates/contextra-types/src/types/saos.rs:321`
```rust
pub enum Signal
```
- Impl-Block (Trait `std::fmt::Display`) `crates/contextra-types/src/types/saos.rs:330`

Varianten:

- `Vector` — `crates/contextra-types/src/types/saos.rs:323`
- `Text` — `crates/contextra-types/src/types/saos.rs:325`
- `Graph` — `crates/contextra-types/src/types/saos.rs:327`

#### `type QueryOptions`
- Ort: `crates/contextra-types/src/types/saos.rs:474`
```rust
pub type QueryOptions = HybridQuery
```
Dokumentation: Alias for [`HybridQuery`] representing query options for hybrid retrieval operations (Spec Teil 6.5).

#### `type QueryOptionsBuilder`
- Ort: `crates/contextra-types/src/types/saos.rs:512`
```rust
pub type QueryOptionsBuilder = HybridQueryBuilder
```
Dokumentation: Alias for [`HybridQueryBuilder`] representing a query options builder for hybrid retrieval operations (Spec Teil 6.5).

### Modul `witness`

#### `struct RequestContext`
- Ort: `crates/contextra-types/src/witness.rs:57`
```rust
pub struct RequestContext
```
- Impl-Block `crates/contextra-types/src/witness.rs:64`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `tenant_id` | `crate::TenantId` | `crates/contextra-types/src/witness.rs:59` |
| `tx_id` | `crate::TxId` | `crates/contextra-types/src/witness.rs:61` |

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new(tenant_id: crate::TenantId, tx_id: crate::TxId) -> Self` | `crates/contextra-types/src/witness.rs:71` | Creates a new `RequestContext`. # Parameter - `tenant_id`: Mandantenidentifikator. - `tx_id`: Zugeordnete Transaktions-ID. |

#### `struct SagaRecord`
- Ort: `crates/contextra-types/src/witness.rs:84`
```rust
pub struct SagaRecord
```
- Impl-Block `crates/contextra-types/src/witness.rs:89`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `deleted_keys_hash` | `[u8; 32]` | `crates/contextra-types/src/witness.rs:86` |

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new(deleted_keys_hash: [u8` | `crates/contextra-types/src/witness.rs:95` | Creates a new `SagaRecord`. # Parameter - `deleted_keys_hash`: BLAKE3-Hash der zu löschenden Schlüssel. |

#### `enum ScanMethod`
- Ort: `crates/contextra-types/src/witness.rs:44`
```rust
pub enum ScanMethod
```

Varianten:

- `Structural` — `crates/contextra-types/src/witness.rs:46`
- `NotMeasured(String)` — `crates/contextra-types/src/witness.rs:48`
- `FeatureDisabled` — `crates/contextra-types/src/witness.rs:50`
