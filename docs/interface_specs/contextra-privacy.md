# Schnittstellenspezifikation: `contextra-privacy`

> Erzeugt durch `tools/extract_interfaces.py` aus dem Quellcode. Keine Handpflege. Jede Angabe trägt Datei:Zeile. Maßgeblich bleibt der Quellcode.

## 1. Kopf

| Feld | Wert |
|---|---|
| Name | `contextra-privacy` |
| Version | workspace |
| Pfad | `crates/contextra-privacy` |
| Rolle | Datenschutz-Egress und Surrogate |
| Bibliothekspfad | src/lib.rs (Standard) |

**Crate-Dokumentation (//!):**

> `contextra-privacy` — Cloud Egress Security, DLP & Exfiltration Protection
>
> # Zweck und Architektur
> Dieses Crate implementiert die Sicherheits- und Datenschutzschicht für ausgehende Datenströme
> (Cloud Egress) in Contextra. Ziel ist es, Data Loss Prevention (DLP, Schutz vor Datenverlust),
> Erkennung von Exfiltrationsversuchen, automatisierte Pseudonymisierung über Surrogat-Token
> sowie unveränderbare Audit-Protokolle bereitzustellen.
>
> # Ebene und Rolle
> Ebene E0 (Autarker Baustein). Das Crate ist vollständig eigenständig kompilier- und testbar.
>
> # Einbindung und Abhängigkeiten

## 2. Abhängigkeiten

| Name | Art | Quelle | optional | verwendet von (normal/dev) |
|---|---|---|---|---|
| `bincode` | dev | registry | nein |  |
| `blake3` | normal | registry | nein |  |
| `criterion` | dev | registry | nein |  |
| `parking_lot` | normal | registry | nein |  |
| `rand` | normal | registry | nein |  |
| `regex` | normal | registry | nein |  |
| `scc` | normal | registry | nein |  |
| `serde` | normal | registry | nein |  |
| `serde_json` | normal | workspace | nein |  |
| `tempfile` | dev | registry | nein |  |
| `thiserror` | normal | workspace | nein |  |
| `tokio` | dev | registry | nein |  |
| `tracing` | normal | workspace | nein |  |
| `zeroize` | normal | registry | nein |  |

**Verwendet von (Crate-Ebene):** `contextra-mcp`

## 3. Features

| Feature | Aktiviert |
|---|---|
| `default` | `std` |
| `serde` | – |
| `std` | – |
| `tracing` | – |

## 4. Metriken (Quellcode)

| Kennzahl | Wert |
|---|---|
| Quelldateien | 13 |
| Quellzeilen | 5782 |
| Öffentliche Items (Modul-Ebene) | 96 |
| davon `fn` | 23 |
| davon `struct` | 30 |
| davon `enum` | 11 |
| davon `trait` | 8 |
| davon `type` | 6 |
| davon `const` | 6 |
| davon `mod` | 12 |
| Öffentliche Makros (`#[macro_export]`) | 0 |
| Öffentliche Re-Exports (`pub use`) | 16 |
| `unsafe`-Vorkommen (Produktionscode) | 0 |
| `.unwrap()` (Produktionscode) | 9 |
| `.expect(` (Produktionscode) | 18 |
| `panic!` (Produktionscode) | 4 |
| `todo!/unimplemented!/unreachable!` | 0 |
| Indexierungen `x[…]` (grob, Produktionscode) | 10 |
| `#[deprecated]` | 11 |
| `#[cfg(feature…)]`-Gates | 1 |
| Parser-Abdeckung (geparste `pub`-Deklarationen / Grep-Treffer) | 186 / 186 = 100 % |

## 5. Modulbaum und Dateien

| Modul | Datei | Zeilen |
|---|---|---|
| `audit_trace` | `crates/contextra-privacy/src/audit_trace.rs` | 123 |
| `bulk_exfiltration_detector` | `crates/contextra-privacy/src/bulk_exfiltration_detector.rs` | 458 |
| `context_edit_audit` | `crates/contextra-privacy/src/context_edit_audit.rs` | 380 |
| `contract` | `crates/contextra-privacy/src/contract.rs` | 16 |
| `egress_gateway` | `crates/contextra-privacy/src/egress_gateway.rs` | 870 |
| `egress_guard` | `crates/contextra-privacy/src/egress_guard.rs` | 544 |
| `egress_vault` | `crates/contextra-privacy/src/egress_vault.rs` | 1009 |
| `entity_recognizer` | `crates/contextra-privacy/src/entity_recognizer.rs` | 1391 |
| `error` | `crates/contextra-privacy/src/error.rs` | 190 |
| `guarded_payload` | `crates/contextra-privacy/src/guarded_payload.rs` | 128 |
| `crate` | `crates/contextra-privacy/src/lib.rs` | 80 |
| `processing_registry` | `crates/contextra-privacy/src/processing_registry.rs` | 422 |
| `types` | `crates/contextra-privacy/src/types.rs` | 171 |

## 6. Re-Exports (`pub use`)

- `pub use crate::egress_guard::TextSearchEngine` — `crates/contextra-privacy/src/contract.rs:14` (Modul `contract`)
- `pub use crate::egress_vault::EntityRecognizer` — `crates/contextra-privacy/src/contract.rs:15` (Modul `contract`)
- `pub use crate::types::` — `crates/contextra-privacy/src/contract.rs:16` (Modul `contract`)
- `pub use audit_trace::` — `crates/contextra-privacy/src/lib.rs:44` (Modul `crate`)
- `pub use bulk_exfiltration_detector::` — `crates/contextra-privacy/src/lib.rs:45` (Modul `crate`)
- `pub use context_edit_audit::` — `crates/contextra-privacy/src/lib.rs:48` (Modul `crate`)
- `pub use contract::` — `crates/contextra-privacy/src/lib.rs:52` (Modul `crate`)
- `pub use egress_gateway::` — `crates/contextra-privacy/src/lib.rs:54` (Modul `crate`)
- `pub use egress_guard::` — `crates/contextra-privacy/src/lib.rs:61` (Modul `crate`)
- `pub use egress_vault::NoOpRecognizer` — `crates/contextra-privacy/src/lib.rs:66` (Modul `crate`)
- `pub use egress_vault::` — `crates/contextra-privacy/src/lib.rs:67` (Modul `crate`)
- `pub use entity_recognizer::` — `crates/contextra-privacy/src/lib.rs:71` (Modul `crate`)
- `pub use error::` — `crates/contextra-privacy/src/lib.rs:74` (Modul `crate`)
- `pub use guarded_payload::` — `crates/contextra-privacy/src/lib.rs:75` (Modul `crate`)
- `pub use processing_registry::` — `crates/contextra-privacy/src/lib.rs:76` (Modul `crate`)
- `pub use types::` — `crates/contextra-privacy/src/lib.rs:80` (Modul `crate`)

## 8. Öffentliche Schnittstelle je Item

### Modul `audit_trace`

#### `trait EgressClassifierTrace`
- Ort: `crates/contextra-privacy/src/audit_trace.rs:87`
```rust
pub trait EgressClassifierTrace: EgressClassifier
```
Dokumentation: Erweiterter Trait für Egress-Klassifikatoren mit kryptografischem Audit-Trace. Kombiniert die reguläre Klassifikation mit der deterministischen Trace-Berechnung über einen Zeitstempel-Port.

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `fn classify_with_trace<'a>( &'a self, payload: &'a str, clock: &'a dyn Clock, ) -> BoxFuture<'a, (EgressClassification, [u8` | `crates/contextra-privacy/src/audit_trace.rs:100` | Klassifiziert einen Payload und liefert sowohl das Klassifikationsergebnis als auch einen 32-Byte-BLAKE3-Hash-Trace. # Ablauf 1. Liest den aktuellen Zeitstempel |

#### `fn compute_audit_trace`
- Ort: `crates/contextra-privacy/src/audit_trace.rs:62`
```rust
pub fn compute_audit_trace(payload: &str, rule_id: &str, nanos: u64) -> [u8
```
Dokumentation: Berechnet einen deterministischen 32-Byte-BLAKE3-Hash-Trace für eine Klassifikationsentscheidung. Segmente werden in fester Byte-Reihenfolge mit 8-Byte Little-Endian Längenpräfixen serialisiert: 1. Payload-Länge (u64 LE) + Payload-Bytes. 2. Regel-ID-Länge (u64 LE) + Regel-ID-Bytes. 3. Zeitstempel-Länge (8 Bytes, u64 LE) + Zeitstempel (Nanosekunden seit Unix-Epoche, u64 LE). # Vorbedingungen `nanos` muss aus einem standardkonformen Clock-Port stammen (Regel 3 aus CLAUDE.md). # Nachbedingungen Liefert reproduzierbar denselben 32-Byte BLAKE3-Hashwert für identische Eingaben.

#### `fn extract_rule_id`
- Ort: `crates/contextra-privacy/src/audit_trace.rs:36`
```rust
pub fn extract_rule_id(classification: &EgressClassification) -> String
```
Dokumentation: Extrahiert eine stabile, nicht-leere Regel-Kennung aus einer [`EgressClassification`]. # Ablauf 1. Bei `EgressClassification::Allow` wird die statische Kennung `"ALLOW"` zurückgegeben. 2. Bei `EgressClassification::Block(reason)` wird der Grund differenziert: - `SensitivePattern`: Gibt die konkrete Regel-ID (z. B. `"R-001"`) zurück. - `PolicyDenied`: Formatiert mit Präfix `"POLICY_DENIED:"`. - `EgressPolicyDenied`: Formatiert mit Präfix `"EGRESS_POLICY_DENIED:"`. - `ClassificationTimeout`: Gibt `"CLASSIFICATION_TIMEOUT"` zurück. - `InternalError`: Formatiert mit Präfix `"INTERNAL_ERROR:"`. # P

### Modul `bulk_exfiltration_detector`

#### `struct BulkExfiltrationDetector`
- Ort: `crates/contextra-privacy/src/bulk_exfiltration_detector.rs:159`
```rust
pub struct BulkExfiltrationDetector
```
Dokumentation: Volumen- und Zeitfenster-basierter Rate-Limiter für Cloud-Egress-Anfragen (Layer 4).
- Impl-Block (Trait `std::fmt::Debug`) `crates/contextra-privacy/src/bulk_exfiltration_detector.rs:168`
- Impl-Block `crates/contextra-privacy/src/bulk_exfiltration_detector.rs:177`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `max_bytes_per_window` | `usize` | `crates/contextra-privacy/src/bulk_exfiltration_detector.rs:161` |
| `window` | `Duration` | `crates/contextra-privacy/src/bulk_exfiltration_detector.rs:163` |

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new(max_bytes_per_window: usize, window: Duration) -> Self` | `crates/contextra-privacy/src/bulk_exfiltration_detector.rs:184` | Erstellt einen neuen `BulkExfiltrationDetector` mit Standard-Systemzeit-Clock. # Parameter * `max_bytes_per_window`: Maximale Byteanzahl im Fenster. * `window`: |
| `pub fn with_clock( max_bytes_per_window: usize, window: Duration, clock: Arc<dyn Clock>, ) -> Self` | `crates/contextra-privacy/src/bulk_exfiltration_detector.rs:195` | Erstellt einen neuen `BulkExfiltrationDetector` mit einem spezifischen [`Clock`]-Port. # Parameter * `max_bytes_per_window`: Maximale Byteanzahl im Fenster. * ` |
| `pub fn record_and_check( &self, session: SessionId, payload_len: usize, ) -> BulkExfiltrationOutcome` | `crates/contextra-privacy/src/bulk_exfiltration_detector.rs:216` | Registriert `payload_len` Bytes für `session` und bewertet, ob das Limit im gleitenden Fenster überschritten wurde. Thread-sicher für nebenläufige Anfragen über |
| `pub fn record_and_check_with_clock( &self, session: SessionId, payload_len: usize, clock: &dyn Clock, ) -> BulkExfiltrationOutcome` | `crates/contextra-privacy/src/bulk_exfiltration_detector.rs:226` | Registriert Bytes für eine Sitzung unter Verwendung einer explizit übergebenen [`Clock`]. |
| `pub fn record_and_check_at_nanos( &self, session: SessionId, payload_len: usize, now_nanos: u64, ) -> BulkExfiltrationOutcome` | `crates/contextra-privacy/src/bulk_exfiltration_detector.rs:243` | Interner Hilfsablauf zur Auswertung an einem expliziten Nanosekunden-Zeitstempel. # Ablauf 1. Sucht den Sitzungs-Eintrag in der `scc::HashMap` oder erzeugt ihn  |

#### `struct DeterministicBlake3Hasher`
- Ort: `crates/contextra-privacy/src/bulk_exfiltration_detector.rs:28`
```rust
pub struct DeterministicBlake3Hasher
```
Dokumentation: Deterministischer, P28-konformer Hasher auf Basis von BLAKE3.
- Impl-Block (Trait `Hasher`) `crates/contextra-privacy/src/bulk_exfiltration_detector.rs:32`

#### `struct DeterministicBuildHasher`
- Ort: `crates/contextra-privacy/src/bulk_exfiltration_detector.rs:48`
```rust
pub struct DeterministicBuildHasher
```
Dokumentation: Deterministischer BuildHasher für sessions in `BulkExfiltrationDetector`.
- Impl-Block (Trait `BuildHasher`) `crates/contextra-privacy/src/bulk_exfiltration_detector.rs:50`

#### `struct SessionId`
- Ort: `crates/contextra-privacy/src/bulk_exfiltration_detector.rs:61`
```rust
pub struct SessionId(pub String)
```
Dokumentation: Eindeutige Sitzungs-Kennung (SessionId) zur Gruppierung von Egress-Rate-Limiting-Buckets.
- Impl-Block (Trait `From<String>`) `crates/contextra-privacy/src/bulk_exfiltration_detector.rs:63`
- Impl-Block (Trait `From<&str>`) `crates/contextra-privacy/src/bulk_exfiltration_detector.rs:69`

#### `enum BulkExfiltrationOutcome`
- Ort: `crates/contextra-privacy/src/bulk_exfiltration_detector.rs:78`
```rust
pub enum BulkExfiltrationOutcome
```
Dokumentation: Ergebnis der Auswertung des ausgehenden Datenvolumens gegen die konfigurierten Sitzungslimits.

Varianten:

- `Allow` — `crates/contextra-privacy/src/bulk_exfiltration_detector.rs:80`
- `Block{` — `crates/contextra-privacy/src/bulk_exfiltration_detector.rs:82`

### Modul `context_edit_audit`

#### `struct ContextEditAuditRecord`
- Ort: `crates/contextra-privacy/src/context_edit_audit.rs:72`
```rust
pub struct ContextEditAuditRecord
```
Dokumentation: Nachvollziehbarer, kryptografisch verketteter Audit-Datensatz für eine Änderung am temporären Agenten-Kontext. **Sicherheitsgarantie:** Enthält keinen Klartext des Scratchpad-Inhalts (nur Längen, Hashes und Labels).

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `previous_hash` | `[u8; 32]` | `crates/contextra-privacy/src/context_edit_audit.rs:74` |
| `record_hash` | `[u8; 32]` | `crates/contextra-privacy/src/context_edit_audit.rs:76` |
| `task_id` | `String` | `crates/contextra-privacy/src/context_edit_audit.rs:78` |
| `tenant_id` | `TenantId` | `crates/contextra-privacy/src/context_edit_audit.rs:80` |
| `kind` | `ContextEditKind` | `crates/contextra-privacy/src/context_edit_audit.rs:82` |
| `chunk_label` | `Option<String>` | `crates/contextra-privacy/src/context_edit_audit.rs:84` |
| `bytes_affected` | `usize` | `crates/contextra-privacy/src/context_edit_audit.rs:86` |
| `recorded_at_tx` | `u64` | `crates/contextra-privacy/src/context_edit_audit.rs:88` |
| `subgoal_index` | `u64` | `crates/contextra-privacy/src/context_edit_audit.rs:90` |

#### `struct ContextEditInput`
- Ort: `crates/contextra-privacy/src/context_edit_audit.rs:52`
```rust
pub struct ContextEditInput
```
Dokumentation: Eingabe-Parameter zur Erstellung eines Audit-Datensatzes für eine Kontext-Änderung.

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `task_id` | `String` | `crates/contextra-privacy/src/context_edit_audit.rs:54` |
| `tenant_id` | `TenantId` | `crates/contextra-privacy/src/context_edit_audit.rs:56` |
| `kind` | `ContextEditKind` | `crates/contextra-privacy/src/context_edit_audit.rs:58` |
| `chunk_label` | `Option<String>` | `crates/contextra-privacy/src/context_edit_audit.rs:60` |
| `bytes_affected` | `usize` | `crates/contextra-privacy/src/context_edit_audit.rs:62` |
| `subgoal_index` | `u64` | `crates/contextra-privacy/src/context_edit_audit.rs:64` |

#### `enum ContextEditAuditError`
- Ort: `crates/contextra-privacy/src/context_edit_audit.rs:39`
```rust
pub enum ContextEditAuditError
```
Dokumentation: Lokaler Fehlertyp für Audit-Ketten-Verifizierungsfehler.

Varianten:

- `ChainTampered{` — `crates/contextra-privacy/src/context_edit_audit.rs:43`

#### `enum ContextEditKind`
- Ort: `crates/contextra-privacy/src/context_edit_audit.rs:25`
```rust
pub enum ContextEditKind
```
Dokumentation: Art der Kontext-Änderung am temporären Agenten-Scratchpad.

Varianten:

- `Append` — `crates/contextra-privacy/src/context_edit_audit.rs:27`
- `Replace` — `crates/contextra-privacy/src/context_edit_audit.rs:29`
- `Remove` — `crates/contextra-privacy/src/context_edit_audit.rs:31`
- `CheckpointReset` — `crates/contextra-privacy/src/context_edit_audit.rs:33`

#### `fn build_context_edit_audit_record`
- Ort: `crates/contextra-privacy/src/context_edit_audit.rs:175`
```rust
pub fn build_context_edit_audit_record( previous_hash: [u8
```
Dokumentation: Konstruiert deterministisch einen neuen [`ContextEditAuditRecord`] mit Hash-Verkettung. Der Zeitstempel wird ausschließlich über den injizierten `Clock`-Port bezogen (kein SystemTime::now()). # Ablauf 1. Bezieht aktuellen Zeitstempel über `clock.now_unix_nanos()`. 2. Berechnet den Hashwert via [`compute_record_hash`]. 3. Gibt den fertig zusammengestellten Datensatz zurück. # Nebenwirkungen Greift auf den injizierten `Clock`-Port zu.

#### `fn compute_record_hash`
- Ort: `crates/contextra-privacy/src/context_edit_audit.rs:114`
```rust
pub fn compute_record_hash( previous_hash: [u8
```
Dokumentation: Berechnet den deterministischen BLAKE3-Hash über alle Datenfelder eines Audit-Records. # Ablauf 1. Hash-Initialisierung mit `blake3::Hasher`. 2. Hinzufügen von `previous_hash` (32 Bytes). 3. Längenpräfix und Bytes der `task_id`. 4. `tenant_id` als 64-Bit Little-Endian. 5. Operationsart (`kind`) als einzelnes Byte (1..=4). 6. Optionale Kennung `chunk_label` mit Vorhandenseins-Flag und Längenpräfix. 7. Anzahl `bytes_affected` als 64-Bit Little-Endian. 8. Zeitstempel `recorded_at_tx` als 64-Bit Little-Endian. 9. `subgoal_index` als 64-Bit Little-Endian. # Parameter * `previous_hash`: BLAKE3-Hash 

#### `fn render_audit_line`
- Ort: `crates/contextra-privacy/src/context_edit_audit.rs:250`
```rust
pub fn render_audit_line(record: &ContextEditAuditRecord) -> String
```
Dokumentation: Formatiert einen Audit-Datensatz in eine einzeilige, menschenlesbare Log-Zeile. ### Format `"task={task_id} tenant={tenant_id} kind={kind:?} chunk_label={chunk_label} bytes={bytes_affected} recorded_at_tx={recorded_at_tx} subgoal={subgoal_index} prev_hash={prev_hash_hex} record_hash={record_hash_hex}"` # Ablauf 1. Wandelt optionale Labels um (Fallback auf `"none"`). 2. Kodiert Vorgängerhash und Record-Hash in Hexadezimaldarstellung. 3. Formatiert alle Attribute in einen standardisierten Log-String.

#### `fn verify_audit_chain`
- Ort: `crates/contextra-privacy/src/context_edit_audit.rs:210`
```rust
pub fn verify_audit_chain(chain: &[ContextEditAuditRecord]) -> Result<(), ContextEditAuditError>
```
Dokumentation: Verifiziert die Integrität und Hash-Verkettung einer Reihe von Audit-Records. # Ablauf 1. Initialisiert den erwarteten Vorgängerhash mit Nullen (`[0u8; 32]`). 2. Iteriert über alle Datensätze der Kette: a. Prüft, ob `record.previous_hash` dem erwarteten Hash entspricht. b. Rekonstruiert die Eingabedaten und berechnet den Soll-Hashwert neu. c. Prüft, ob `record.record_hash` mit dem Soll-Hashwert übereinstimmt. d. Setzt `expected_previous_hash` auf den aktuellen `record_hash`. 3. Bricht beim ersten Fehler mit `ContextEditAuditError::ChainTampered` ab (Fail-Closed). # Rückgabe `Ok(())` bei intakt

### Modul `crate`

#### `mod audit_trace`
- Ort: `crates/contextra-privacy/src/lib.rs:29`
```rust
pub mod audit_trace
```

#### `mod bulk_exfiltration_detector`
- Ort: `crates/contextra-privacy/src/lib.rs:30`
```rust
pub mod bulk_exfiltration_detector
```

#### `mod context_edit_audit`
- Ort: `crates/contextra-privacy/src/lib.rs:31`
```rust
pub mod context_edit_audit
```

#### `mod contract`
- Ort: `crates/contextra-privacy/src/lib.rs:32`
```rust
pub mod contract
```

#### `mod egress_gateway`
- Ort: `crates/contextra-privacy/src/lib.rs:33`
```rust
pub mod egress_gateway
```

#### `mod egress_guard`
- Ort: `crates/contextra-privacy/src/lib.rs:34`
```rust
pub mod egress_guard
```

#### `mod egress_vault`
- Ort: `crates/contextra-privacy/src/lib.rs:35`
```rust
pub mod egress_vault
```

#### `mod entity_recognizer`
- Ort: `crates/contextra-privacy/src/lib.rs:36`
```rust
pub mod entity_recognizer
```

#### `mod error`
- Ort: `crates/contextra-privacy/src/lib.rs:37`
```rust
pub mod error
```

#### `mod guarded_payload`
- Ort: `crates/contextra-privacy/src/lib.rs:38`
```rust
pub mod guarded_payload
```

#### `mod processing_registry`
- Ort: `crates/contextra-privacy/src/lib.rs:39`
```rust
pub mod processing_registry
```

#### `mod types`
- Ort: `crates/contextra-privacy/src/lib.rs:40`
```rust
pub mod types
```

### Modul `egress_gateway`

#### `struct CloudQueryRequest`
- Ort: `crates/contextra-privacy/src/egress_gateway.rs:451`
```rust
pub struct CloudQueryRequest
```
Dokumentation: Anfrage-Struktur für ausgehende Cloud-Queries.

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `query` | `String` | `crates/contextra-privacy/src/egress_gateway.rs:453` |
| `collection` | `Option<String>` | `crates/contextra-privacy/src/egress_gateway.rs:455` |
| `max_results` | `Option<usize>` | `crates/contextra-privacy/src/egress_gateway.rs:457` |

#### `struct CloudQueryResponse`
- Ort: `crates/contextra-privacy/src/egress_gateway.rs:463`
```rust
pub struct CloudQueryResponse
```
Dokumentation: Antwort-Struktur nach erfolgreicher Bereinigung und Durchlauf der Egress-Pipeline.

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `status` | `String` | `crates/contextra-privacy/src/egress_gateway.rs:465` |
| `query` | `String` | `crates/contextra-privacy/src/egress_gateway.rs:467` |
| `abstracted` | `bool` | `crates/contextra-privacy/src/egress_gateway.rs:469` |
| `results` | `Vec<Value>` | `crates/contextra-privacy/src/egress_gateway.rs:471` |
| `abstraction_notice` | `Option<String>` | `crates/contextra-privacy/src/egress_gateway.rs:473` |

#### `struct CloudResponseRehydrator`
- Ort: `crates/contextra-privacy/src/egress_gateway.rs:310`
```rust
pub struct CloudResponseRehydrator
```
Dokumentation: Rehydrierer zur sicheren Rückübersetzung von Surrogat-Tokens in Cloud-Antworten.
- Impl-Block `crates/contextra-privacy/src/egress_gateway.rs:314`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new(vault_map: impl IntoIterator<Item = (String, String)>) -> Self` | `crates/contextra-privacy/src/egress_gateway.rs:316` | Creates a new `CloudResponseRehydrator` with the provided surrogate-to-entity map. |
| `pub fn rehydrate(&self, cloud_response_text: &str) -> String` | `crates/contextra-privacy/src/egress_gateway.rs:326` | Rehydrates surrogate tokens matching `[USER_ENTITY_[0-9a-fA-F]{4}]` back to original entity values. Unknown or missing surrogate tokens remain unchanged without |

#### `struct DefaultEgressGuard`
- Ort: `crates/contextra-privacy/src/egress_gateway.rs:53`
```rust
pub struct DefaultEgressGuard
```
Dokumentation: Standardmäßiger Pass-Through Guard, falls kein separater L4-Vektor-Guard konfiguriert ist.
- Impl-Block (Trait `EgressGuardCheck`) `crates/contextra-privacy/src/egress_gateway.rs:55`

#### `struct EgressGateway`
- Ort: `crates/contextra-privacy/src/egress_gateway.rs:75`
```rust
pub struct EgressGateway
```
Dokumentation: Einheitliches Egress-Gateway zur Durchsetzung der 5-stufigen Schutzpipeline: 1. Policy/Guard check 2. Entity recognition 3. Surrogate replacement 4. Bulk volume detector check 5. Audit trace logging
- Impl-Block (Trait `std::fmt::Debug`) `crates/contextra-privacy/src/egress_gateway.rs:84`
- Impl-Block `crates/contextra-privacy/src/egress_gateway.rs:203`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn builder() -> EgressGatewayBuilder` | `crates/contextra-privacy/src/egress_gateway.rs:205` | Erzeugt einen neuen EgressGatewayBuilder. |
| `pub async fn process( &self, request: TenantScoped<CloudQueryRequest>, expected_tenant_id: &TenantId, ) -> Result<CloudQueryResponse, EgressError>` | `crates/contextra-privacy/src/egress_gateway.rs:210` | Executes the standard, mandatory 5-stage egress pipeline for a tenant-scoped request. |
| `pub async fn process_with_components( &self, request: TenantScoped<CloudQueryRequest>, expected_tenant_id: &TenantId, classifier: &dyn EgressClassifier, egress_guard: Option<&dyn EgressGuardCheck>, bulk_detector: Option<(&BulkExfiltrationDetector, SessionId)>, ) -> Result<CloudQueryResponse, EgressError>` | `crates/contextra-privacy/src/egress_gateway.rs:231` | Executes the standard 5-stage egress pipeline with optional custom classifier, guard, or bulk detector overrides: 1. Policy/Guard check -> block on violation/er |

#### `struct EgressGatewayBuilder`
- Ort: `crates/contextra-privacy/src/egress_gateway.rs:94`
```rust
pub struct EgressGatewayBuilder
```
Dokumentation: Builder zur typsicheren Konstruktion einer [`EgressGateway`]-Instanz.
- Impl-Block `crates/contextra-privacy/src/egress_gateway.rs:104`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new() -> Self` | `crates/contextra-privacy/src/egress_gateway.rs:106` | Erzeugt eine neue Builder-Instanz. |
| `pub fn with_classifier(mut self, classifier: Arc<dyn EgressClassifier>) -> Self` | `crates/contextra-privacy/src/egress_gateway.rs:111` | Setzt den Klassifikator. |
| `pub fn with_egress_guard(mut self, guard: Arc<dyn EgressGuardCheck>) -> Self` | `crates/contextra-privacy/src/egress_gateway.rs:117` | Setzt den Egress-Guard. |
| `pub fn with_recognizer(mut self, recognizer: Arc<dyn EntityRecognizer>) -> Self` | `crates/contextra-privacy/src/egress_gateway.rs:123` | Setzt den Entitätserkenner. |
| `pub fn with_egress_vault(mut self, vault: Arc<EgressVault>) -> Self` | `crates/contextra-privacy/src/egress_gateway.rs:129` | Setzt den Egress-Tresor. |
| `pub fn with_surrogate_vault(mut self, surrogate_vault: Arc<SurrogateVault>) -> Self` | `crates/contextra-privacy/src/egress_gateway.rs:135` | Setzt den Surrogat-Tresor. |
| `pub fn with_bulk_detector( mut self, detector: Arc<BulkExfiltrationDetector>, session_id: SessionId, ) -> Self` | `crates/contextra-privacy/src/egress_gateway.rs:141` | Setzt den Massenabfluss-Detektor. |
| `pub fn with_clock(mut self, clock: Arc<dyn Clock>) -> Self` | `crates/contextra-privacy/src/egress_gateway.rs:151` | Setzt die Uhr-Quelle. |
| `pub fn build(self) -> Result<EgressGateway, EgressError>` | `crates/contextra-privacy/src/egress_gateway.rs:157` | Builds the `EgressGateway`, strictly enforcing that mandatory components (including `egress_guard`) are present. |

#### `trait EgressGuardCheck`
- Ort: `crates/contextra-privacy/src/egress_gateway.rs:46`
```rust
pub trait EgressGuardCheck: Send + Sync
```
Dokumentation: Trait-Vertrag für Layer-4 k-NN Vektor-Ähnlichkeitsprüfungen gegen Bulk-Exfiltration.

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `fn check<'a>(&'a self, payload: &'a str) -> BoxFuture<'a, EgressClassification>` | `crates/contextra-privacy/src/egress_gateway.rs:48` | Evaluates the outbound payload against vector index / similarity models. |

#### `trait InjectionDetector`
- Ort: `crates/contextra-privacy/src/egress_gateway.rs:63`
```rust
pub trait InjectionDetector: Send + Sync
```
Dokumentation: Trait zur Prüfung auf Prompt-Injection in eingehenden Cloud-Antworten.

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `fn detect(&self, text: &str) -> Option<String>` | `crates/contextra-privacy/src/egress_gateway.rs:65` | Prüft den Text auf Muster. |

#### `fn check_bulk_exfiltration`
- Ort: `crates/contextra-privacy/src/egress_gateway.rs:583`
```rust
pub fn check_bulk_exfiltration( detector: &BulkExfiltrationDetector, session: SessionId, payload: &str, ) -> Result<(), EgressError>
```
Dokumentation: Überprüft eine Anfrage gegen den Bulk-Exfiltrations-Detektor.

#### `fn handle_cloud_query`
- Ort: `crates/contextra-privacy/src/egress_gateway.rs:478`
```rust
pub async fn handle_cloud_query( request: CloudQueryRequest, classifier: &dyn EgressClassifier, ) -> Result<CloudQueryResponse, EgressError>
```
Dokumentation: Verarbeitet eine ausgehende Cloud-Query über die Standard-Schutz-Pipeline.

#### `fn handle_cloud_query_scoped`
- Ort: `crates/contextra-privacy/src/egress_gateway.rs:491`
```rust
pub async fn handle_cloud_query_scoped( request: TenantScoped<CloudQueryRequest>, expected_tenant_id: &TenantId, classifier: &dyn EgressClassifier, ) -> Result<CloudQueryResponse, EgressError>
```
Dokumentation: Verarbeitet eine mandantengebundene Cloud-Query ([`TenantScoped`]).

#### `fn handle_cloud_query_scoped_with_bulk_detector`
- Ort: `crates/contextra-privacy/src/egress_gateway.rs:538`
```rust
pub async fn handle_cloud_query_scoped_with_bulk_detector( request: TenantScoped<CloudQueryRequest>, expected_tenant_id: &TenantId, classifier: &dyn EgressClassifier, egress_guard: Option<&dyn EgressGuardCheck>, bulk_detector: Option<(&BulkExfiltrationDetector, SessionId)>, ) -> Result<CloudQueryResponse, EgressError>
```
Dokumentation: Vollständige Prüfung inklusive Mandantenisolation, L4-Guard und Bulk-Exfiltrations-Detektor.

#### `fn handle_cloud_query_scoped_with_guard`
- Ort: `crates/contextra-privacy/src/egress_gateway.rs:514`
```rust
pub async fn handle_cloud_query_scoped_with_guard( request: TenantScoped<CloudQueryRequest>, expected_tenant_id: &TenantId, classifier: &dyn EgressClassifier, egress_guard: Option<&dyn EgressGuardCheck>, ) -> Result<CloudQueryResponse, EgressError>
```
Dokumentation: Verarbeitet eine mandantengebundene Cloud-Query mit zusätzlichem Layer-4 Vektor-Guard.

#### `fn handle_cloud_query_with_bulk_detector`
- Ort: `crates/contextra-privacy/src/egress_gateway.rs:605`
```rust
pub async fn handle_cloud_query_with_bulk_detector( request: CloudQueryRequest, classifier: &dyn EgressClassifier, egress_guard: Option<&dyn EgressGuardCheck>, bulk_detector: Option<(&BulkExfiltrationDetector, SessionId)>, ) -> Result<CloudQueryResponse, EgressError>
```
Dokumentation: Führt die Cloud-Query-Prüfung mit L4-Guard und Bulk-Exfiltrations-Detektor durch.

#### `fn handle_cloud_query_with_guard`
- Ort: `crates/contextra-privacy/src/egress_gateway.rs:568`
```rust
pub async fn handle_cloud_query_with_guard( request: CloudQueryRequest, classifier: &dyn EgressClassifier, egress_guard: Option<&dyn EgressGuardCheck>, ) -> Result<CloudQueryResponse, EgressError>
```
Dokumentation: Verarbeitet eine Cloud-Query mit zusätzlichem Layer-4 Vektor-Guard.

#### `fn pii_vault_forces_crypto_shred`
- Ort: `crates/contextra-privacy/src/egress_gateway.rs:398`
```rust
pub fn pii_vault_forces_crypto_shred(is_pii_match: bool, is_memory_only: bool) -> bool
```
Dokumentation: Entscheidet anhand von PII-Match und Speichermodus, ob kryptografisches Schreddern erzwungen wird. Hinweis: Veralteter Bezeichner; bitte zukünftig `pii_vault_requires_isolated_retention` verwenden.
- **deprecated**

#### `fn pii_vault_requires_isolated_retention`
- Ort: `crates/contextra-privacy/src/egress_gateway.rs:388`
```rust
pub fn pii_vault_requires_isolated_retention(is_pii_match: bool, is_memory_only: bool) -> bool
```
Dokumentation: Evaluates whether PII vault data requires isolated retention mode for an affected document (neutral API).

#### `fn process_cloud_response`
- Ort: `crates/contextra-privacy/src/egress_gateway.rs:629`
```rust
pub async fn process_cloud_response( raw_response: &str, injection_guard: &dyn InjectionDetector, rehydrator: &CloudResponseRehydrator, ) -> Result<String, EgressError>
```
Dokumentation: Verarbeitet die Antwort einer Cloud-Query (Rehydrierung und Prompt-Injection-Erkennung).

#### `fn resolve_effective_kv_delete_mode`
- Ort: `crates/contextra-privacy/src/egress_gateway.rs:430`
```rust
pub fn resolve_effective_kv_delete_mode<D, K>( is_pii_match: bool, durability_mode: D, preset_kv_delete_mode: K, crypto_shred_mode: K, ) -> K where D: std::fmt::Debug, K: Copy,
```
Dokumentation: Bestimmt den effektiven Löschmodus (Shredding vs. Tombstone) unter Berücksichtigung von PII-Vorgaben. Hinweis: Veralteter Bezeichner; bitte zukünftig `resolve_effective_retention_mode` verwenden.
- **deprecated**

#### `fn resolve_effective_retention_mode`
- Ort: `crates/contextra-privacy/src/egress_gateway.rs:404`
```rust
pub fn resolve_effective_retention_mode<D, K>( is_pii_match: bool, durability_mode: D, preset_mode: K, isolated_mode: K, ) -> K where D: std::fmt::Debug, K: Copy,
```
Dokumentation: Resolves the effective retention mode given PII status and storage configuration (neutral API).

#### `type DefaultEgressClassifier`
- Ort: `crates/contextra-privacy/src/egress_gateway.rs:38`
```rust
pub type DefaultEgressClassifier = EgressVault
```
Dokumentation: Standardmäßiger Klassifikator auf Basis von [`EgressVault`].

#### `const MAX_SEARCH_QUERY_BYTES`
- Ort: `crates/contextra-privacy/src/egress_gateway.rs:42`
```rust
pub const MAX_SEARCH_QUERY_BYTES: usize = 64 * 1024
```
Dokumentation: Maximale Suchanfragengröße in Bytes (64 KiB = 65.536 Bytes).

### Modul `egress_guard`

#### `struct EgressGuard`
- Ort: `crates/contextra-privacy/src/egress_guard.rs:63`
```rust
pub struct EgressGuard
```
Dokumentation: Layer-4 EgressGuard zur Erkennung und Blockierung von Bulk-Exfiltrationen. Baut auf einem Text-Sucher (`TextSearchEngine`) auf. Outbound-Payloads mit mindestens `min_bytes` werden über `search_text` abgefragt. Bei Cosine Similarity $\ge$ `threshold` wird die Anfrage blockiert. Strikte **Fail-Closed**-Semantik bei Index-Fehlern, Timeouts oder leeren Ergebnissen.
- Impl-Block `crates/contextra-privacy/src/egress_guard.rs:70`
- Impl-Block (Trait `EgressClassifier`) `crates/contextra-privacy/src/egress_guard.rs:189`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new(search_engine: Arc<dyn TextSearchEngine>, threshold: f32, min_bytes: usize) -> Self` | `crates/contextra-privacy/src/egress_guard.rs:73` | Erstellt eine neue `EgressGuard`-Instanz mit Standard-Timeout (200 ms). |
| `pub fn with_timeout(mut self, timeout: Duration) -> Self` | `crates/contextra-privacy/src/egress_guard.rs:84` | Setzt ein benutzerdefiniertes Timeout für die Index-Abfrage. |
| `pub fn threshold(&self) -> f32` | `crates/contextra-privacy/src/egress_guard.rs:91` | Gibt den gesetzten Schwellwert für Ähnlichkeitsprüfungen zurück. |
| `pub fn min_bytes(&self) -> usize` | `crates/contextra-privacy/src/egress_guard.rs:97` | Gibt die minimale Payload-Länge in Bytes zurück. |
| `pub fn timeout(&self) -> Duration` | `crates/contextra-privacy/src/egress_guard.rs:103` | Gibt das konfigurierte Abfrage-Timeout zurück. |
| `pub async fn check_scoped( &self, payload: crate::types::TenantScoped<&str>, expected_tenant_id: &crate::types::TenantId, ) -> EgressClassification` | `crates/contextra-privacy/src/egress_guard.rs:111` | Prüft einen an einen Mandanten gebundenen Payload ([`crate::types::TenantScoped`]) mit Mandantenisolationsprüfung. Liefert `Block(BlockReason::PolicyDenied(...) |
| `pub async fn check(&self, payload: &str) -> EgressClassification` | `crates/contextra-privacy/src/egress_guard.rs:136` | Prüft einen Egress-Payload auf mögliche Bulk-Exfiltration gegen die Collection. Gemäß **v17 Teil 10.3** (Priorität P1, "Fail-Closed-Randfall"), Invariante **INV |

#### `struct TextSearchResult`
- Ort: `crates/contextra-privacy/src/egress_guard.rs:27`
```rust
pub struct TextSearchResult
```
Dokumentation: Vereinfachtes Kandidaten-Ergebnis einer Text- bzw. Vektorähnlichkeitssuche.

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `id` | `String` | `crates/contextra-privacy/src/egress_guard.rs:29` |
| `score` | `f32` | `crates/contextra-privacy/src/egress_guard.rs:31` |

#### `trait TextSearchEngine`
- Ort: `crates/contextra-privacy/src/egress_guard.rs:36`
```rust
pub trait TextSearchEngine: Send + Sync
```
Dokumentation: Abstraktions-Trait für die Textähnlichkeitssuche zur Entkopplung von konkreten Datenbank-Engines.

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `fn search_text<'a>( &'a self, text: &'a str, limit: usize, ) -> BoxFuture<'a, Result<Vec<TextSearchResult>, String>>` | `crates/contextra-privacy/src/egress_guard.rs:38` | Führt eine Textsuche nach relevanten Dokumenten durch. |

#### `const DEFAULT_EGRESS_GUARD_MIN_BYTES`
- Ort: `crates/contextra-privacy/src/egress_guard.rs:53`
```rust
pub const DEFAULT_EGRESS_GUARD_MIN_BYTES: usize = 128
```
Dokumentation: Minimale Byte-Länge des Payloads, ab der die Ähnlichkeitsprüfung durchgeführt wird (128 Bytes).

#### `const DEFAULT_EGRESS_GUARD_THRESHOLD`
- Ort: `crates/contextra-privacy/src/egress_guard.rs:50`
```rust
pub const DEFAULT_EGRESS_GUARD_THRESHOLD: f32 = 0.85
```
Dokumentation: Standard-Schwellwert für Cosine-Similarity-Übereinstimmung (0.85 = 85 % Ähnlichkeit).

#### `const DEFAULT_EGRESS_GUARD_TIMEOUT`
- Ort: `crates/contextra-privacy/src/egress_guard.rs:47`
```rust
pub const DEFAULT_EGRESS_GUARD_TIMEOUT: Duration = Duration::from_millis(200)
```
Dokumentation: Standard-Timeout für EgressGuard Vector-Search (200 ms).

### Modul `egress_vault`

#### `struct CompiledPattern`
- Ort: `crates/contextra-privacy/src/egress_vault.rs:232`
```rust
pub struct CompiledPattern
```
Dokumentation: Ein vorkompiliertes Regex-Muster für die Egress-Klassifikation.
- Impl-Block `crates/contextra-privacy/src/egress_vault.rs:239`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `name` | `String` | `crates/contextra-privacy/src/egress_vault.rs:234` |
| `regex` | `regex::Regex` | `crates/contextra-privacy/src/egress_vault.rs:236` |

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new(name: impl Into<String>, pattern: &str) -> Result<Self, EgressVaultError>` | `crates/contextra-privacy/src/egress_vault.rs:241` | Erstellt ein neues `CompiledPattern`. |

#### `struct EgressVault`
- Ort: `crates/contextra-privacy/src/egress_vault.rs:373`
```rust
pub struct EgressVault
```
Dokumentation: Cloud Egress Vault zur Abhandlung und Bündelung von Layer-1-Klassifikationen.
- Impl-Block `crates/contextra-privacy/src/egress_vault.rs:380`
- Impl-Block (Trait `Default`) `crates/contextra-privacy/src/egress_vault.rs:561`
- Impl-Block (Trait `EgressClassifier`) `crates/contextra-privacy/src/egress_vault.rs:581`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn default_patterns() -> Vec<String>` | `crates/contextra-privacy/src/egress_vault.rs:385` | Standard-DLP-Muster für Egress-Klassifikationen (Secrets, API-Keys, PII, E-Mail). |
| `pub fn try_default() -> Result<Self, EgressVaultError>` | `crates/contextra-privacy/src/egress_vault.rs:396` | Versucht, eine `EgressVault`-Instanz mit den Standard-DLP-Mustern zu erstellen. |
| `pub fn new(patterns: Vec<String>) -> Result<Self, EgressVaultError>` | `crates/contextra-privacy/src/egress_vault.rs:402` | Erstellt eine neue `EgressVault`-Instanz aus einer Liste von Regex-Patterns. Jedem Pattern wird eine opake Regel-ID ("R-001", "R-002", ...) zugewiesen. |
| `pub fn with_timeout(mut self, timeout: Duration) -> Self` | `crates/contextra-privacy/src/egress_vault.rs:425` | Setzt ein benutzerdefiniertes Timeout für Klassifikationsabfragen. |
| `pub fn with_surrogate_vault(mut self, surrogate_vault: Arc<SurrogateVault>) -> Self` | `crates/contextra-privacy/src/egress_vault.rs:431` | Setzt einen benutzerdefinierten `SurrogateVault`. |
| `pub fn patterns(&self) -> &[CompiledPattern]` | `crates/contextra-privacy/src/egress_vault.rs:437` | Gibt die kompilierten Muster zurück. |
| `pub fn policy_category(&self) -> PolicyCategory` | `crates/contextra-privacy/src/egress_vault.rs:442` | Gibt die Policy-Kategorie dieser Vault-Instanz für Cloud-Egress-Klassifizierungen zurück. |
| `pub fn timeout(&self) -> Duration` | `crates/contextra-privacy/src/egress_vault.rs:447` | Gibt das gesetzte Timeout zurück. |
| `pub fn surrogate_vault(&self) -> &Arc<SurrogateVault>` | `crates/contextra-privacy/src/egress_vault.rs:452` | Gibt eine Referenz auf den verknüpften `SurrogateVault` zurück. |
| `pub async fn classify_scoped( &self, payload: crate::types::TenantScoped<&str>, expected_tenant_id: &crate::types::TenantId, ) -> EgressClassification` | `crates/contextra-privacy/src/egress_vault.rs:458` | Classifies a `TenantScoped` payload for egress export only if the bound `TenantId` matches `expected_tenant_id`. If a tenant scope violation occurs, returns `Bl |
| `pub fn sanitize_and_vault_scoped( &self, payload: crate::types::TenantScoped<&str>, expected_tenant_id: &crate::types::TenantId, recognizer: &dyn EntityRecognizer, ) -> Result<(crate::types::TenantScoped<String>, usize), EgressVaultError>` | `crates/contextra-privacy/src/egress_vault.rs:471` | Tokenizes and replaces recognized entities in a `TenantScoped` payload only if the bound `TenantId` matches `expected_tenant_id`. Re-wraps the sanitized output  |
| `pub fn sanitize_and_vault( &self, payload: &str, recognizer: &dyn EntityRecognizer, ) -> Result<(String, usize), EgressVaultError>` | `crates/contextra-privacy/src/egress_vault.rs:489` | Tokenisiert und ersetzt erkannte strukturierte Regex-Muster sowie unstrukturierte NER-Entitäten durch sitzungsstabile Surrogate und speichert die Zuordnungen im |

Assoziierte Konstanten/Typen:

- `pub const DEFAULT_TIMEOUT: Duration = Duration::from_millis(100)` — `crates/contextra-privacy/src/egress_vault.rs:382`

#### `struct NoOpRecognizer`
- Ort: `crates/contextra-privacy/src/egress_vault.rs:39`
```rust
pub struct NoOpRecognizer
```
Dokumentation: No-Op-Implementierung von [`EntityRecognizer`], die keine Entitäten erkennt (Null-Objekt-Muster).
- Impl-Block (Trait `EntityRecognizer`) `crates/contextra-privacy/src/egress_vault.rs:41`

#### `struct SurrogateVault`
- Ort: `crates/contextra-privacy/src/egress_vault.rs:51`
```rust
pub struct SurrogateVault
```
Dokumentation: Sitzungsgebundener Tresor zur bidirektionalen Abbildung zwischen Klartext-PII und Surrogat-Tokens. Alle kryptografischen Schlüssel, Salts und Zuordnungen werden bei Drop flüchtig genullt (Zeroization).
- Impl-Block (Trait `std::fmt::Debug`) `crates/contextra-privacy/src/egress_vault.rs:56`
- Impl-Block `crates/contextra-privacy/src/egress_vault.rs:65`
- Impl-Block (Trait `Drop`) `crates/contextra-privacy/src/egress_vault.rs:163`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new(session_salt: [u8` | `crates/contextra-privacy/src/egress_vault.rs:68` | Erstellt einen neuen `SurrogateVault` mit dem übergebenen 16-Byte Sitzungssalz. |
| `pub fn random_salt() -> Self` | `crates/contextra-privacy/src/egress_vault.rs:77` | Erstellt einen neuen `SurrogateVault` mit einem kryptografisch sicheren Zufallssalz. |
| `pub fn generate_surrogate(&self, entity_text: &str) -> Result<String, EgressVaultError>` | `crates/contextra-privacy/src/egress_vault.rs:97` | Generates a session-stable surrogate identifier for `entity_text` and stores the surrogate -> entity mapping. # Backward Compatibility & Token Format Note The s |
| `pub fn get_entity(&self, surrogate: &str) -> Option<String>` | `crates/contextra-privacy/src/egress_vault.rs:123` | Retrieves the original entity text for a given surrogate key if present. |
| `pub fn len(&self) -> usize` | `crates/contextra-privacy/src/egress_vault.rs:132` | Returns the current count of stored surrogate mappings. |
| `pub fn is_empty(&self) -> bool` | `crates/contextra-privacy/src/egress_vault.rs:137` | Returns true if the vault contains no surrogate mappings. |
| `pub fn clear(&self)` | `crates/contextra-privacy/src/egress_vault.rs:143` | Clears and zeroizes all cached surrogate mappings (neutral maintenance / B8 / SPEC §6). |
| `pub fn compact(&self)` | `crates/contextra-privacy/src/egress_vault.rs:155` | Compacts the internal mapping storage (neutral maintenance / SPEC §6). |

#### `enum BlockReason`
- Ort: `crates/contextra-privacy/src/egress_vault.rs:195`
```rust
pub enum BlockReason
```
Dokumentation: Grund für das Blockieren einer Egress-Anfrage.
- Impl-Block `crates/contextra-privacy/src/egress_vault.rs:208`

Varianten:

- `SensitivePattern(String)` — `crates/contextra-privacy/src/egress_vault.rs:197`
- `PolicyDenied(String)` — `crates/contextra-privacy/src/egress_vault.rs:199`
- `EgressPolicyDenied(String)` — `crates/contextra-privacy/src/egress_vault.rs:201`
- `ClassificationTimeout` — `crates/contextra-privacy/src/egress_vault.rs:203`
- `InternalError(String)` — `crates/contextra-privacy/src/egress_vault.rs:205`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn category(&self) -> PolicyCategory` | `crates/contextra-privacy/src/egress_vault.rs:210` | Gibt die zugehörige Policy-Kategorie des Blockierungsgrunds zurück. |

#### `enum EgressClassification`
- Ort: `crates/contextra-privacy/src/egress_vault.rs:222`
```rust
pub enum EgressClassification
```
Dokumentation: Klassifikationsergebnis der Egress-Prüfung.

Varianten:

- `Allow` — `crates/contextra-privacy/src/egress_vault.rs:224`
- `Block(BlockReason)` — `crates/contextra-privacy/src/egress_vault.rs:226`

#### `enum EgressVaultError`
- Ort: `crates/contextra-privacy/src/egress_vault.rs:258`
```rust
pub enum EgressVaultError
```
Dokumentation: Fehler beim Erstellen oder Laden des EgressVaults.

Varianten:

- `InvalidPattern{` — `crates/contextra-privacy/src/egress_vault.rs:261`
- `PayloadTooLarge{` — `crates/contextra-privacy/src/egress_vault.rs:269`
- `TenantScopeViolation(#[from] crate::types::TenantScopeViolation)` — `crates/contextra-privacy/src/egress_vault.rs:277`
- `SurrogateCollision{` — `crates/contextra-privacy/src/egress_vault.rs:280`

#### `enum PolicyCategory`
- Ort: `crates/contextra-privacy/src/egress_vault.rs:184`
```rust
pub enum PolicyCategory
```
Dokumentation: Policy-Kategorie für Audit-Logging und Zugriffskontrolle.

Varianten:

- `LocalAccess` — `crates/contextra-privacy/src/egress_vault.rs:186`
- `CloudEgress` — `crates/contextra-privacy/src/egress_vault.rs:188`

#### `trait EgressClassifier`
- Ort: `crates/contextra-privacy/src/egress_vault.rs:556`
```rust
pub trait EgressClassifier: Send + Sync
```
Dokumentation: Schnittstelle für Downstream-Crates (`contextra-mcp`, `contextra-router`).

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `fn classify<'a>(&'a self, payload: &'a str) -> BoxFuture<'a, EgressClassification>` | `crates/contextra-privacy/src/egress_vault.rs:558` | Klassifiziert einen Text-Payload für den Egress-Export. |

#### `trait EntityRecognizer`
- Ort: `crates/contextra-privacy/src/egress_vault.rs:31`
```rust
pub trait EntityRecognizer: Send + Sync
```
Dokumentation: Trait zur Erkennung benannter Entitäten (NER) im Text ohne Abhängigkeit von Machine-Learning-Crates.

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `fn recognize(&self, text: &str) -> Vec<(std::ops::Range<usize>, String)>` | `crates/contextra-privacy/src/egress_vault.rs:33` | Recognizes entities in the text and returns byte ranges and category labels. |

#### `fn classify_layer1`
- Ort: `crates/contextra-privacy/src/egress_vault.rs:297`
```rust
pub async fn classify_layer1( payload: &str, patterns: &[CompiledPattern], timeout: Duration, ) -> EgressClassification
```
Dokumentation: Führt eine Layer-1-Regex-Klassifikation auf dem Payload mit neuem Task und hartem Timeout aus. Invariante: **Fail-Closed**. Bei Timeout, Panic oder Fehler wird stets `Block(...)` zurückgegeben.

#### `fn classify_layer1_arc`
- Ort: `crates/contextra-privacy/src/egress_vault.rs:316`
```rust
pub async fn classify_layer1_arc( payload: &str, patterns: Arc<Vec<CompiledPattern>>, regex_set: Arc<regex::RegexSet>, timeout: Duration, ) -> EgressClassification
```
Dokumentation: Optimierter Ausführungspfad für `classify_layer1` unter Wiederverwendung von vorkompilierten `Arc`-Mengen.

#### `type BoxFuture`
- Ort: `crates/contextra-privacy/src/egress_vault.rs:178`
```rust
pub type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>
```
Dokumentation: Typalias für gepinnte, sende-fähige dynamische Zukunfts-Futures.

#### `const MAX_CLASSIFY_PAYLOAD_BYTES`
- Ort: `crates/contextra-privacy/src/egress_vault.rs:292`
```rust
pub const MAX_CLASSIFY_PAYLOAD_BYTES: usize = 65_536
```
Dokumentation: Maximale zulässige Payload-Länge in Bytes für Layer-1-Klassifikation (64 KiB = 65.536 Bytes).

### Modul `entity_recognizer`

#### `struct EntityMatch`
- Ort: `crates/contextra-privacy/src/entity_recognizer.rs:34`
```rust
pub struct EntityMatch
```
Dokumentation: Strukturierte Information über eine erkannte Entität.

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `range` | `Range<usize>` | `crates/contextra-privacy/src/entity_recognizer.rs:36` |
| `entity_type` | `String` | `crates/contextra-privacy/src/entity_recognizer.rs:38` |
| `rule_id` | `&'static str` | `crates/contextra-privacy/src/entity_recognizer.rs:40` |
| `confidence` | `f32` | `crates/contextra-privacy/src/entity_recognizer.rs:42` |

#### `struct Gazetteer`
- Ort: `crates/contextra-privacy/src/entity_recognizer.rs:48`
```rust
pub struct Gazetteer
```
Dokumentation: Erweiterungspunkt für benutzerdefinierte Begriff-, Namens- oder Unternehmenslisten (Wörterbuch-Abgleich).
- Impl-Block `crates/contextra-privacy/src/entity_recognizer.rs:52`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new() -> Self` | `crates/contextra-privacy/src/entity_recognizer.rs:55` | Erstellt eine neue leere [`Gazetteer`]-Instanz. |
| `pub fn add_term(&mut self, term: impl Into<String>)` | `crates/contextra-privacy/src/entity_recognizer.rs:63` | Fügt dem Verzeichnis einen Begriff hinzu (wird für case-insensitiven Abgleich in Kleinbuchstaben normalisiert). |
| `pub fn is_empty(&self) -> bool` | `crates/contextra-privacy/src/entity_recognizer.rs:72` | Prüft, ob das Begriffsverzeichnis leer ist. |
| `pub fn len(&self) -> usize` | `crates/contextra-privacy/src/entity_recognizer.rs:77` | Liefert die Anzahl der Begriffe im Gazetteer. |

#### `struct RuleBasedEntityRecognizer`
- Ort: `crates/contextra-privacy/src/entity_recognizer.rs:87`
```rust
pub struct RuleBasedEntityRecognizer
```
Dokumentation: Regelbasierter PII-Erkenner ohne externe Regex-Laufzeitabhängigkeit. Erkennt Kategorien personenbezogener Daten: EMAIL, IBAN, BIC, PHONE, CREDIT_CARD, STEUER_ID, IPV4, IPV6, PLZ, PLZ_ORT, LICENSE_PLATE, GAZETTEER.
- Impl-Block (Trait `Default`) `crates/contextra-privacy/src/entity_recognizer.rs:92`
- Impl-Block `crates/contextra-privacy/src/entity_recognizer.rs:101`
- Impl-Block (Trait `EntityRecognizer`) `crates/contextra-privacy/src/entity_recognizer.rs:228`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new() -> Self` | `crates/contextra-privacy/src/entity_recognizer.rs:104` | Erstellt eine neue Instanz von [`RuleBasedEntityRecognizer`] mit Standard-Längenlimit und leerem Gazetteer. |
| `pub fn with_gazetteer(gazetteer: Gazetteer) -> Self` | `crates/contextra-privacy/src/entity_recognizer.rs:110` | Erstellt einen Erkenner mit einem vorkonfigurierten Begriffsverzeichnis ([`Gazetteer`]). |
| `pub fn with_max_input_length(mut self, limit: usize) -> Self` | `crates/contextra-privacy/src/entity_recognizer.rs:119` | Konfiguriert das maximale Längenlimit in Bytes zur Begrenzung von DoS-Risiken. |
| `pub fn max_input_length(&self) -> usize` | `crates/contextra-privacy/src/entity_recognizer.rs:126` | Liefert die Obergrenze für Eingabetexte in Bytes. |
| `pub fn add_gazetteer_term(mut self, term: impl Into<String>) -> Self` | `crates/contextra-privacy/src/entity_recognizer.rs:132` | Ergänzt einen Begriff für das Verzeichnis über ein Builder-Muster. |
| `pub fn recognize(&self, text: &str) -> Vec<(Range<usize>, String)>` | `crates/contextra-privacy/src/entity_recognizer.rs:139` | Führt die Erkennung auf dem übergebenen Text aus und liefert Byte-Bereiche und Kategorielabels. |
| `pub fn recognize_detailed(&self, text: &str) -> Vec<EntityMatch>` | `crates/contextra-privacy/src/entity_recognizer.rs:144` | Führt die Erkennung aus und liefert detaillierte [`EntityMatch`]-Objekte inklusive Regel-ID und Konfidenz. |
| `pub fn recognize_bounded( &self, text: &str, ) -> Result<Vec<(Range<usize>, String)>, EgressError>` | `crates/contextra-privacy/src/entity_recognizer.rs:152` | Führt die Erkennung mit strikter Längenprüfung aus und liefert `Err(EgressError::CapacityExceeded)` bei Überschreitung. Gemäß `INV-PRIVACY-FAILCLOSED-1` wird be |
| `pub fn recognize_detailed_bounded(&self, text: &str) -> Result<Vec<EntityMatch>, EgressError>` | `crates/contextra-privacy/src/entity_recognizer.rs:167` | Führt die detaillierte Erkennung mit strikter Längenprüfung aus. |

#### `type PatternRecognizer`
- Ort: `crates/contextra-privacy/src/entity_recognizer.rs:29`
```rust
pub type PatternRecognizer = RuleBasedEntityRecognizer
```
Dokumentation: Alias für [`RuleBasedEntityRecognizer`] als standardmäßiger Mustererkenner.

#### `const DEFAULT_MAX_INPUT_LENGTH`
- Ort: `crates/contextra-privacy/src/entity_recognizer.rs:25`
```rust
pub const DEFAULT_MAX_INPUT_LENGTH: usize = MAX_CLASSIFY_PAYLOAD_BYTES
```
Dokumentation: Standardmäßige maximale Eingabelänge in Bytes für die Entitätserkennung (65.536 Bytes = 64 KiB).

### Modul `error`

#### `enum PrivacyError`
- Ort: `crates/contextra-privacy/src/error.rs:17`
```rust
pub enum PrivacyError
```
Dokumentation: Primärer Fehlertyp des Privacy- und Egress-Subsystems. Signalisiert spezifische Fehlerzustände bei der Analyse, Filterung und Weiterleitung von Daten an externe Dienste.
- Impl-Block (Trait `From<crate::egress_vault::EgressVaultError>`) `crates/contextra-privacy/src/error.rs:73`
- Impl-Block `crates/contextra-privacy/src/error.rs:98`

Varianten:

- `InvalidParams(String)` — `crates/contextra-privacy/src/error.rs:20`
- `InvalidInput(String)` — `crates/contextra-privacy/src/error.rs:24`
- `CapacityExceeded(String)` — `crates/contextra-privacy/src/error.rs:28`
- `PolicyViolation(String)` — `crates/contextra-privacy/src/error.rs:32`
- `InternalError(String)` — `crates/contextra-privacy/src/error.rs:36`
- `Io(String)` — `crates/contextra-privacy/src/error.rs:40`
- `StateConflict(String)` — `crates/contextra-privacy/src/error.rs:44`
- `Corrupt(String)` — `crates/contextra-privacy/src/error.rs:48`
- `SurrogateCollision{` — `crates/contextra-privacy/src/error.rs:52`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn invalid_params(msg: impl Into<String>) -> Self` | `crates/contextra-privacy/src/error.rs:101` | Konstruiert einen `PrivacyError::InvalidParams`. |
| `pub fn invalid_input(msg: impl Into<String>) -> Self` | `crates/contextra-privacy/src/error.rs:106` | Konstruiert einen `PrivacyError::InvalidInput`. |
| `pub fn capacity_exceeded(msg: impl Into<String>) -> Self` | `crates/contextra-privacy/src/error.rs:111` | Konstruiert einen `PrivacyError::CapacityExceeded`. |
| `pub fn policy_violation(msg: impl Into<String>) -> Self` | `crates/contextra-privacy/src/error.rs:117` | Konstruiert einen `PrivacyError::PolicyViolation`. |
| `pub fn internal_error(msg: impl Into<String>) -> Self` | `crates/contextra-privacy/src/error.rs:123` | Konstruiert einen `PrivacyError::InternalError`. |
| `pub fn io(msg: impl Into<String>) -> Self` | `crates/contextra-privacy/src/error.rs:128` | Konstruiert einen `PrivacyError::Io`. |
| `pub fn state_conflict(msg: impl Into<String>) -> Self` | `crates/contextra-privacy/src/error.rs:133` | Konstruiert einen `PrivacyError::StateConflict`. |
| `pub fn corrupt(msg: impl Into<String>) -> Self` | `crates/contextra-privacy/src/error.rs:138` | Konstruiert einen `PrivacyError::Corrupt`. |

#### `type EgressError`
- Ort: `crates/contextra-privacy/src/error.rs:63`
```rust
pub type EgressError = PrivacyError
```
Dokumentation: Abwärtskompatibler Alias für `PrivacyError`.

#### `type Error`
- Ort: `crates/contextra-privacy/src/error.rs:66`
```rust
pub type Error = PrivacyError
```
Dokumentation: Primärer Crate-Fehlertyp-Alias.

#### `type Result`
- Ort: `crates/contextra-privacy/src/error.rs:69`
```rust
pub type Result<T> = std::result::Result<T, PrivacyError>
```
Dokumentation: Result-Typ für contextra-privacy.

### Modul `guarded_payload`

#### `struct GuardedPayload`
- Ort: `crates/contextra-privacy/src/guarded_payload.rs:35`
```rust
pub struct GuardedPayload<S>
```
Dokumentation: Type-State-Wrapper für Cloud-Egress-Payloads. Ein `GuardedPayload<Sanitized>` kann NUR durch den vollständigen Durchlauf aller 5 Egress-Guard-Layer konstruiert werden (`egress_gateway.rs`). Ein `GuardedPayload<Unsanitized>` an `dispatch_to_cloud()` zu übergeben ist ein COMPILE-FEHLER. # Invariante (§12.3) `dispatch_to_cloud()` akzeptiert ausschließlich `GuardedPayload<Sanitized>`. Diese Invariante ist nicht durch Laufzeit-Checks, sondern durch das Typsystem erzwungen.
- Impl-Block (Trait `Drop`) `crates/contextra-privacy/src/guarded_payload.rs:41`
- Impl-Block `crates/contextra-privacy/src/guarded_payload.rs:48`
- Impl-Block `crates/contextra-privacy/src/guarded_payload.rs:56`
- Impl-Block `crates/contextra-privacy/src/guarded_payload.rs:73`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `inner` | `String` | `crates/contextra-privacy/src/guarded_payload.rs:36` |
| `session_id` | `String` | `crates/contextra-privacy/src/guarded_payload.rs:37` |

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn session_id(&self) -> &str` | `crates/contextra-privacy/src/guarded_payload.rs:51` | Gibt eine Referenz auf die zugeordnete Sitzungs-ID (`session_id`) zurück. |
| `pub fn new(raw: String, session_id: String) -> Self` | `crates/contextra-privacy/src/guarded_payload.rs:64` | Erstellt einen neuen unsanitizierten Payload (Eingang aus MCP-Client oder interner Engine). # Ablauf 1. Übernimmt den rohen Eingabestring `raw`. 2. Verknüpft di |
| `pub fn from_sanitized(sanitized: String, session_id: String) -> Self` | `crates/contextra-privacy/src/guarded_payload.rs:82` | Konstruiert einen bereinigten Payload nach erfolgreicher Bereinigung. # Ablauf Nur von `egress_gateway.rs` nach erfolgreichem Durchlauf aller Egress-Filter aufz |
| `pub fn into_inner(mut self) -> String` | `crates/contextra-privacy/src/guarded_payload.rs:95` | Extrahiert den bereinigten String aus dem Schutzbehälter für den Versand. # Ablauf Konsumiert die Instanz und gibt das interne `String`-Objekt frei. |

#### `struct Sanitized`
- Ort: `crates/contextra-privacy/src/guarded_payload.rs:22`
```rust
pub struct Sanitized
```
Dokumentation: Marker-Typ: Payload hat alle 5 Egress-Guard-Layer erfolgreich durchlaufen und ist bereinigt.

#### `struct Unsanitized`
- Ort: `crates/contextra-privacy/src/guarded_payload.rs:18`
```rust
pub struct Unsanitized
```
Dokumentation: Marker-Typ: Payload hat noch nicht alle Egress-Guard-Layer durchlaufen und gilt als unbereinigt.

### Modul `processing_registry`

#### `struct ProcessingActivityRecord`
- Ort: `crates/contextra-privacy/src/processing_registry.rs:58`
```rust
pub struct ProcessingActivityRecord
```
Dokumentation: Ein Datensatz einer Verarbeitungstätigkeit.

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `activity_name` | `String` | `crates/contextra-privacy/src/processing_registry.rs:60` |
| `controller_or_processor_role` | `ProcessorRole` | `crates/contextra-privacy/src/processing_registry.rs:62` |
| `purpose` | `String` | `crates/contextra-privacy/src/processing_registry.rs:64` |
| `legal_basis` | `String` | `crates/contextra-privacy/src/processing_registry.rs:67` |
| `data_categories` | `Vec<String>` | `crates/contextra-privacy/src/processing_registry.rs:69` |
| `data_subject_categories` | `Vec<String>` | `crates/contextra-privacy/src/processing_registry.rs:71` |
| `recipients` | `Vec<String>` | `crates/contextra-privacy/src/processing_registry.rs:73` |
| `third_country_transfers` | `Vec<String>` | `crates/contextra-privacy/src/processing_registry.rs:75` |
| `retention_period` | `String` | `crates/contextra-privacy/src/processing_registry.rs:77` |
| `technical_and_organizational_measures` | `Vec<String>` | `crates/contextra-privacy/src/processing_registry.rs:79` |
| `tenant_id` | `TenantId` | `crates/contextra-privacy/src/processing_registry.rs:81` |
| `version` | `u32` | `crates/contextra-privacy/src/processing_registry.rs:84` |
| `created_at_unix_secs` | `u64` | `crates/contextra-privacy/src/processing_registry.rs:87` |

#### `struct ProcessingRegistry`
- Ort: `crates/contextra-privacy/src/processing_registry.rs:93`
```rust
pub struct ProcessingRegistry
```
Dokumentation: Unveränderliches, append-only Verzeichnis von Verarbeitungstätigkeiten.
- Impl-Block `crates/contextra-privacy/src/processing_registry.rs:97`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new() -> Self` | `crates/contextra-privacy/src/processing_registry.rs:100` | Erstellt ein neues leeres `ProcessingRegistry`. |
| `pub fn append_record(&mut self, mut record: ProcessingActivityRecord, clock: &dyn Clock)` | `crates/contextra-privacy/src/processing_registry.rs:109` | Fügt einen Datensatz dem Verzeichnis hinzu (append-only) und belegt den Erstellungs-Zeitstempel über die injizierte Uhr. |
| `pub fn records(&self) -> &[ProcessingActivityRecord]` | `crates/contextra-privacy/src/processing_registry.rs:120` | Liefert die Datensätze des Verzeichnisses. |
| `pub fn export(&self) -> ProcessingRegistryExport` | `crates/contextra-privacy/src/processing_registry.rs:126` | Exportiert das Verzeichnis in eine `ProcessingRegistryExport`-Struktur. |

#### `struct ProcessingRegistryExport`
- Ort: `crates/contextra-privacy/src/processing_registry.rs:136`
```rust
pub struct ProcessingRegistryExport
```
Dokumentation: Strukturierte Export-Repräsentation des Verzeichnisses von Verarbeitungstätigkeiten.
- Impl-Block `crates/contextra-privacy/src/processing_registry.rs:141`
- Impl-Block (Trait `TextSearchEngine`) `crates/contextra-privacy/src/processing_registry.rs:165`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `records` | `Vec<ProcessingActivityRecord>` | `crates/contextra-privacy/src/processing_registry.rs:138` |

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new(records: Vec<ProcessingActivityRecord>) -> Self` | `crates/contextra-privacy/src/processing_registry.rs:144` | Erstellt eine neue Export-Instanz mit den angegebenen Datensätzen. |
| `pub fn filter_by_tenant(&self, tenant_id: TenantId) -> Self` | `crates/contextra-privacy/src/processing_registry.rs:153` | Filtert alle Verarbeitungstätigkeiten nach der angegebenen Mandanten-ID. Stellt strikte Mandantentrennung bei Auskunftsabfragen sicher, sodass nur Datensätze de |

#### `enum ProcessorRole`
- Ort: `crates/contextra-privacy/src/processing_registry.rs:31`
```rust
pub enum ProcessorRole
```
Dokumentation: Rolle bezüglich der Verarbeitungstätigkeit.
- Impl-Block `crates/contextra-privacy/src/processing_registry.rs:38`

Varianten:

- `Controller` — `crates/contextra-privacy/src/processing_registry.rs:33`
- `Processor` — `crates/contextra-privacy/src/processing_registry.rs:35`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn as_str(&self) -> &'static str` | `crates/contextra-privacy/src/processing_registry.rs:41` | Liefert die deutschsprachige Bezeichnung der Rolle. |

#### `fn default_version`
- Ort: `crates/contextra-privacy/src/processing_registry.rs:51`
```rust
pub fn default_version() -> u32
```
Dokumentation: Standard-Schemaversion für die Deserialisierung von Verarbeitungsdatensätzen.

#### `fn generate_registry`
- Ort: `crates/contextra-privacy/src/processing_registry.rs:231`
```rust
pub fn generate_registry(records: &[ProcessingActivityRecord]) -> ProcessingRegistryExport
```
Dokumentation: Erzeugt aus einer Reihe von Datensätzen eine strukturierte Export-Repräsentation.

#### `fn render_markdown`
- Ort: `crates/contextra-privacy/src/processing_registry.rs:251`
```rust
pub fn render_markdown(export: &ProcessingRegistryExport) -> String
```
Dokumentation: Erzeugt eine druckfertige, deutschsprachige Markdown-Tabelle aus dem Export. Die Tabelle enthält eine Zeile pro `ProcessingActivityRecord` und Spalten für alle geforderten Angaben.

### Modul `types`

#### `struct SystemClock`
- Ort: `crates/contextra-privacy/src/types.rs:20`
```rust
pub struct SystemClock
```
Dokumentation: Standardmäßige Implementierung von [`Clock`] auf Basis von `std::time::SystemTime`.
- Impl-Block `crates/contextra-privacy/src/types.rs:22`
- Impl-Block (Trait `Clock`) `crates/contextra-privacy/src/types.rs:29`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new() -> Self` | `crates/contextra-privacy/src/types.rs:24` | Erstellt eine neue [`SystemClock`]. |

#### `struct TenantId`
- Ort: `crates/contextra-privacy/src/types.rs:48`
```rust
pub struct TenantId(pub u64)
```
- Impl-Block `crates/contextra-privacy/src/types.rs:50`
- Impl-Block (Trait `fmt::Display`) `crates/contextra-privacy/src/types.rs:74`
- Impl-Block (Trait `From<u64>`) `crates/contextra-privacy/src/types.rs:80`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub const fn new(id: u64) -> Self` | `crates/contextra-privacy/src/types.rs:55` | Erstellt einen neuen `TenantId`. |
| `pub fn try_new(id: u64) -> Result<Self, TenantScopeViolation>` | `crates/contextra-privacy/src/types.rs:60` | Versucht, einen Mandanten zu erzeugen (0 ist reserviert/Fehler für Benutzerdaten). |
| `pub const fn as_u64(&self) -> u64` | `crates/contextra-privacy/src/types.rs:69` | Liefert den numerischen Wert. |

Assoziierte Konstanten/Typen:

- `pub const SYSTEM: Self = Self(0)` — `crates/contextra-privacy/src/types.rs:52`

#### `struct TenantScoped`
- Ort: `crates/contextra-privacy/src/types.rs:104`
```rust
pub struct TenantScoped<T>
```
Dokumentation: An einen Mandanten gebundener Datenbehälter zur Durchsetzung von Isolation.
- Impl-Block `crates/contextra-privacy/src/types.rs:109`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub const fn new(tenant_id: TenantId, data: T) -> Self` | `crates/contextra-privacy/src/types.rs:111` | Erstellt einen neuen `TenantScoped`-Container. |
| `pub const fn tenant_id(&self) -> &TenantId` | `crates/contextra-privacy/src/types.rs:116` | Liefert die gebundene Mandanten-ID. |
| `pub const fn inner(&self) -> &T` | `crates/contextra-privacy/src/types.rs:121` | Gibt eine Referenz auf die inneren Daten zurück. |
| `pub fn into_inner_checked(self, expected: &TenantId) -> Result<T, TenantScopeViolation>` | `crates/contextra-privacy/src/types.rs:126` | Prüft den Mandanten und entpackt die inneren Daten. |

#### `enum EgressDecision`
- Ort: `crates/contextra-privacy/src/types.rs:140`
```rust
pub enum EgressDecision
```
Dokumentation: Entscheidung einer Egress-Richtlinienauswertung.

Varianten:

- `Allow` — `crates/contextra-privacy/src/types.rs:142`
- `Redact(String)` — `crates/contextra-privacy/src/types.rs:144`
- `Deny(String)` — `crates/contextra-privacy/src/types.rs:146`

#### `enum TenantScopeViolation`
- Ort: `crates/contextra-privacy/src/types.rs:88`
```rust
pub enum TenantScopeViolation
```
Dokumentation: Fehler bei Mandanten-Scope-Verletzungen.

Varianten:

- `Mismatch{` — `crates/contextra-privacy/src/types.rs:91`
- `InvalidZero` — `crates/contextra-privacy/src/types.rs:99`

#### `trait Clock`
- Ort: `crates/contextra-privacy/src/types.rs:11`
```rust
pub trait Clock: Send + Sync
```
Dokumentation: Trait für monotone und reale Zeitabfragen ohne Abhängigkeit von externen Ports.

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `fn now_unix_nanos(&self) -> u64` | `crates/contextra-privacy/src/types.rs:13` | Liefert die aktuelle Zeit in Unix-Nanosekunden. |
| `fn monotonic_nanos(&self) -> u64` | `crates/contextra-privacy/src/types.rs:15` | Liefert einen monotonen Nanosekunden-Zähler. |

#### `trait EgressPolicy`
- Ort: `crates/contextra-privacy/src/types.rs:150`
```rust
pub trait EgressPolicy: Send + Sync
```
Dokumentation: Crate-eigener dynamischer Trait für Egress-Richtlinien.

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `fn evaluate(&self, payload: &str) -> EgressDecision` | `crates/contextra-privacy/src/types.rs:152` | Bewertet, ob der ausgehende Payload die Richtlinie erfüllt. |

## 9. Impl-Blöcke ohne Typ im selben Crate

- `impl<T: EgressClassifier + ?Sized> EgressClassifierTrace for T` — `crates/contextra-privacy/src/audit_trace.rs:109` (0 pub Methoden)
