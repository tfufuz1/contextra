# Schnittstellenspezifikation: `contextra`

> Erzeugt durch `tools/extract_interfaces.py` aus dem Quellcode. Keine Handpflege. Jede Angabe trägt Datei:Zeile. Maßgeblich bleibt der Quellcode.

## 1. Kopf

| Feld | Wert |
|---|---|
| Name | `contextra` |
| Version | workspace |
| Pfad | `crates/contextra` |
| Rolle | Fassade |
| Bibliothekspfad | src/lib.rs (Standard) |

**Crate-Dokumentation (//!):**

> # Modul: Contextra Primary Facade & Composition Root (Ring 4)
>
> Dieses Modul bildet die oberste Einstiegsebene (Fassade bzw. Composition Root)
> des Gesamtsystems **Contextra** innerhalb der Architektur (Ring 4).
>
> ## Zweck & Architektur
> Die Fassade dient als zentrale Abstraktions- und Konfigurationsschicht für anwendende
> Systeme (wie z. B. `contextra-mcp` oder externe SDKs/Applikationen). Sie verbirgt die
> interne Komplexität der tieferen Schichten (Ring 0 bis 3) und stellt eine einheitliche,
> ergonomische Programmierschnittstelle (API = Application Programming Interface) bereit.
>
> ## Ring-Zugehörigkeit & Abhängigkeiten

## 2. Abhängigkeiten

| Name | Art | Quelle | optional | verwendet von (normal/dev) |
|---|---|---|---|---|
| `contextra-db` | normal | workspace | nein | contextra, contextra-agent, contextra-bench, contextra-cognition, contextra-db-fuzz, contextra-engine |
| `contextra-infer-candle` | normal | workspace | ja | contextra, contextra-mcp |
| `contextra-ports` | normal | workspace | nein | contextra, contextra-adapt, contextra-agent, contextra-checkpoint, contextra-cognition, contextra-crypto, contextra-crypto-fuzz, contextra-db, contextra-engine, contextra-graph, contextra-graph-fuzz, contextra-infer-candle, contextra-infer-ollama, contextra-infer-onnx, contextra-kvcache, contextra-mcp, contextra-mvcc, contextra-rank, contextra-router, contextra-store, contextra-store-fuzz, contextra-testkit, contextra-text, contextra-vector, contextra-vector-fuzz |
| `contextra-types` | normal | workspace | nein | contextra, contextra-adapt, contextra-agent, contextra-bench, contextra-checkpoint, contextra-cognition, contextra-crypto, contextra-crypto-fuzz, contextra-db, contextra-engine, contextra-graph, contextra-graph-fuzz, contextra-infer-candle, contextra-infer-ollama, contextra-infer-onnx, contextra-ingest, contextra-kvcache, contextra-mcp, contextra-mvcc, contextra-ports, contextra-py, contextra-rank, contextra-router, contextra-simd, contextra-store, contextra-store-fuzz, contextra-testkit, contextra-text, contextra-vault, contextra-vector, contextra-vector-fuzz |
| `serde` | normal | workspace | nein |  |
| `serde_json` | normal | workspace | nein |  |
| `tempfile` | dev | workspace | nein |  |
| `thiserror` | normal | workspace | nein |  |
| `tokio` | normal | workspace | nein |  |
| `tracing` | normal | workspace | nein |  |
| `tracing-subscriber` | dev | workspace | nein |  |
| `uuid` | normal | registry | nein |  |
| `zeroize` | normal | registry | nein |  |

**Verwendet von (Crate-Ebene):** `contextra-mcp`, `contextra-py`

## 3. Features

| Feature | Aktiviert |
|---|---|
| `candle` | `contextra-infer-candle` |
| `default` | `embedded` |
| `embedded` | – |

## 4. Metriken (Quellcode)

| Kennzahl | Wert |
|---|---|
| Quelldateien | 7 |
| Quellzeilen | 2042 |
| Öffentliche Items (Modul-Ebene) | 31 |
| davon `fn` | 4 |
| davon `struct` | 9 |
| davon `enum` | 9 |
| davon `trait` | 1 |
| davon `const` | 2 |
| davon `mod` | 6 |
| Öffentliche Makros (`#[macro_export]`) | 0 |
| Öffentliche Re-Exports (`pub use`) | 12 |
| `unsafe`-Vorkommen (Produktionscode) | 0 |
| `.unwrap()` (Produktionscode) | 0 |
| `.expect(` (Produktionscode) | 0 |
| `panic!` (Produktionscode) | 0 |
| `todo!/unimplemented!/unreachable!` | 0 |
| Indexierungen `x[…]` (grob, Produktionscode) | 5 |
| `#[deprecated]` | 7 |
| `#[cfg(feature…)]`-Gates | 1 |
| Parser-Abdeckung (geparste `pub`-Deklarationen / Grep-Treffer) | 69 / 69 = 100 % |

## 5. Modulbaum und Dateien

| Modul | Datei | Zeilen |
|---|---|---|
| `agent_memory` | `crates/contextra/src/agent_memory.rs` | 414 |
| `boot` | `crates/contextra/src/boot.rs` | 104 |
| `builder` | `crates/contextra/src/builder.rs` | 506 |
| `collection_profile` | `crates/contextra/src/collection_profile.rs` | 199 |
| `crate` | `crates/contextra/src/lib.rs` | 204 |
| `performance_profile` | `crates/contextra/src/performance_profile.rs` | 343 |
| `query_rewriter` | `crates/contextra/src/query_rewriter.rs` | 272 |

## 6. Re-Exports (`pub use`)

- `pub use contextra_types::types::domain::AutoExtractionMode` — `crates/contextra/src/collection_profile.rs:34` (Modul `collection_profile`)
- `pub use agent_memory::` — `crates/contextra/src/lib.rs:46` (Modul `crate`)
- `pub use builder::` — `crates/contextra/src/lib.rs:47` (Modul `crate`)
- `pub use collection_profile::` — `crates/contextra/src/lib.rs:48` (Modul `crate`)
- `pub use contextra_db::` — `crates/contextra/src/lib.rs:49` (Modul `crate`)
- `pub use contextra_types::error::ContextraError` — `crates/contextra/src/lib.rs:53` (Modul `crate`)
- `pub use contextra_types::types::domain::` — `crates/contextra/src/lib.rs:54` (Modul `crate`)
- `pub use contextra_types::DistanceMetric` — `crates/contextra/src/lib.rs:55` (Modul `crate`)
- `pub use performance_profile::` — `crates/contextra/src/lib.rs:56` (Modul `crate`)
- `pub use query_rewriter::LlmQueryRewriter` — `crates/contextra/src/lib.rs:57` (Modul `crate`)
- `pub use contextra_infer_candle as candle` — `crates/contextra/src/lib.rs:60` (Modul `crate`)
- `pub use contextra_ports::license::VectorDeleteMode` — `crates/contextra/src/performance_profile.rs:33` (Modul `performance_profile`)

## 8. Öffentliche Schnittstelle je Item

### Modul `agent_memory`

#### `struct AgentMemory`
- Ort: `crates/contextra/src/agent_memory.rs:158`
```rust
pub struct AgentMemory
```
Dokumentation: High-Level-Fassade für die Verwaltung des Agentengedächtnisses. Bietet die Kernoperationen: - `remember`: Speichern von Texten und Metadaten. - `recall`: Semantisches Retrieval basierend auf Volltext- und Vektorsuche. - `forget`: Neutrales Entfernen einer Erinnerung ohne rechtliche Löschzusagen. - `relate` & `relate_n_ary`: Verknüpfen von Erinnerungen im Wissensgraphen.
- Impl-Block `crates/contextra/src/agent_memory.rs:163`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new(engine: Contextra) -> Self` | `crates/contextra/src/agent_memory.rs:166` | Erstellt eine neue `AgentMemory`-Instanz, indem die übergebene `Contextra`-Engine in ein `Arc` gehüllt wird. |
| `pub fn new_arc(engine: Arc<Contextra>) -> Self` | `crates/contextra/src/agent_memory.rs:175` | Erstellt eine neue `AgentMemory`-Instanz um ein bereits geteiltes `Arc<Contextra>`. |
| `pub fn new_with_id_gen(engine: Contextra, id_gen: Arc<dyn IdGen>) -> Self` | `crates/contextra/src/agent_memory.rs:184` | Erstellt eine Instanz mit einem explizit injizierten ID-Generator-Port. |
| `pub fn with_id_gen(mut self, id_gen: Arc<dyn IdGen>) -> Self` | `crates/contextra/src/agent_memory.rs:193` | Injiziert oder aktualisiert den ID-Generator nachträglich (Builder-Muster). |
| `pub fn engine(&self) -> &Contextra` | `crates/contextra/src/agent_memory.rs:200` | Liefert eine Referenz auf die zugrundeliegende [`Contextra`]-Engine. |
| `pub async fn remember( &self, text: &str, metadata: Option<Value>, ) -> Result<MemoryId, ContextraError>` | `crates/contextra/src/agent_memory.rs:228` | Speichert eine Text-Erinnerung mit optionalen JSON-Metadaten. # Ablauf von `remember`: 1. ID ermitteln: Über den injizierten `IdGen`-Port oder über `next_fallba |
| `pub async fn recall(&self, query: &str, k: usize) -> Result<Vec<Memory>, ContextraError>` | `crates/contextra/src/agent_memory.rs:254` | Ruft die Top-k relevantesten Erinnerungen zu einer Suchanfrage ab. Delegiert direkt an `Contextra::search_text` und transformiert Ergebnisse in [`Memory`]-Objek |
| `pub async fn forget(&self, id: &MemoryId) -> Result<(), ContextraError>` | `crates/contextra/src/agent_memory.rs:266` | Entfernt eine Erinnerung anhand ihrer [`MemoryId`]. Delegiert direkt an `Contextra::delete`. |
| `pub async fn relate( &self, from: &MemoryId, to: &MemoryId, label: &str, ) -> Result<(), ContextraError>` | `crates/contextra/src/agent_memory.rs:280` | Erstellt eine gerichtete Beziehung zwischen zwei Erinnerungen im Wissensgraphen. # Parameter - `from`: Quell-Erinnerung. - `to`: Ziel-Erinnerung. - `label`: Sem |
| `pub async fn relate_n_ary( &self, predicate: &str, participants: &[(&MemoryId, &str)], source: Option<&MemoryId>, ) -> Result<RelationId, ContextraError>` | `crates/contextra/src/agent_memory.rs:313` | Erstellt eine n-äre Beziehung (Hyperedge) zwischen zwei oder mehr Erinnerungen. # Parameter - `predicate`: Semantische Bezeichnung der Beziehung (darf nicht lee |

#### `struct Memory`
- Ort: `crates/contextra/src/agent_memory.rs:124`
```rust
pub struct Memory
```
Dokumentation: Repräsentation einer aus dem Speicher abgerufenen Erinnerung (`recall`).
- Impl-Block (Trait `From<SearchResult>`) `crates/contextra/src/agent_memory.rs:137`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `id` | `String` | `crates/contextra/src/agent_memory.rs:126` |
| `score` | `f32` | `crates/contextra/src/agent_memory.rs:128` |
| `metadata` | `Option<Value>` | `crates/contextra/src/agent_memory.rs:130` |
| `matched_signals` | `Vec<String>` | `crates/contextra/src/agent_memory.rs:132` |
| `provenance` | `Option<ProvenanceRecord>` | `crates/contextra/src/agent_memory.rs:134` |

#### `struct MemoryId`
- Ort: `crates/contextra/src/agent_memory.rs:70`
```rust
pub struct MemoryId(String)
```
Dokumentation: Eindeutiger, typisierter Identifikator für eine in [`AgentMemory`] abgelegte Erinnerung. Kapselt den internen Dokument-ID-String der Datenbank-Engine.
- Impl-Block `crates/contextra/src/agent_memory.rs:72`
- Impl-Block (Trait `From<String>`) `crates/contextra/src/agent_memory.rs:84`
- Impl-Block (Trait `From<&str>`) `crates/contextra/src/agent_memory.rs:90`
- Impl-Block (Trait `AsRef<str>`) `crates/contextra/src/agent_memory.rs:96`
- Impl-Block (Trait `fmt::Display`) `crates/contextra/src/agent_memory.rs:102`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new(id: impl Into<String>) -> Self` | `crates/contextra/src/agent_memory.rs:74` | Creates a new `MemoryId` from a string identifier. |
| `pub fn as_str(&self) -> &str` | `crates/contextra/src/agent_memory.rs:79` | Returns a string slice reference to the underlying memory ID. |

#### `struct RelationId`
- Ort: `crates/contextra/src/agent_memory.rs:113`
```rust
pub struct RelationId(pub u64)
```
Dokumentation: Eindeutiger Identifikator für eine n-äre Beziehung (Hyperedge) im Wissensgraphen. Kapselt den 64-Bit-Graphenschlüssel der Wissensgraphen-Engine.
- Impl-Block (Trait `fmt::Display`) `crates/contextra/src/agent_memory.rs:115`

### Modul `boot`

#### `enum BootPhase`
- Ort: `crates/contextra/src/boot.rs:15`
```rust
pub enum BootPhase
```
Dokumentation: Repräsentiert die normativen Startphasen B0–B9 gemäß SPEC §13.5.

Varianten:

- `B0ConfigValidation` — `crates/contextra/src/boot.rs:17`
- `B1StoragePath` — `crates/contextra/src/boot.rs:19`
- `B2KeyProvider` — `crates/contextra/src/boot.rs:21`
- `B3SidecarValidation` — `crates/contextra/src/boot.rs:23`
- `B4WalRecovery` — `crates/contextra/src/boot.rs:25`
- `B5IntentApplication` — `crates/contextra/src/boot.rs:27`
- `B6ReadBarrierArming` — `crates/contextra/src/boot.rs:29`
- `B7SagaContinuation` — `crates/contextra/src/boot.rs:31`
- `B8SecurityPortsWiring` — `crates/contextra/src/boot.rs:33`
- `B9OperationalTraffic` — `crates/contextra/src/boot.rs:35`

#### `fn execute_boot_sequence`
- Ort: `crates/contextra/src/boot.rs:39`
```rust
pub fn execute_boot_sequence( path: &Path, dimension: usize, passphrase_configured: bool, empty_passphrase: bool, ) -> Result<(), ContextraError>
```
Dokumentation: Führt die normativen Startphasen B0 bis B9 in fester Reihenfolge aus.

### Modul `builder`

#### `struct ContextraBuilder`
- Ort: `crates/contextra/src/builder.rs:83`
```rust
pub struct ContextraBuilder
```
Dokumentation: Ein Builder zur flexiblen Konfiguration und Instanziierung der Datenbank [`Contextra`]. `ContextraBuilder` fungiert als primärer Einstiegspunkt (Composition Root) in Ring 4. Er sammelt alle Einstellungen (Pfade, Vektordimensionen, Metriken, Lizenzen) und baut bei [`ContextraBuilder::build`] die vollständige Engine zusammen.
- Impl-Block (Trait `std::fmt::Debug`) `crates/contextra/src/builder.rs:93`
- Impl-Block (Trait `Clone`) `crates/contextra/src/builder.rs:109`
- Impl-Block `crates/contextra/src/builder.rs:123`
- Impl-Block (Trait `Default`) `crates/contextra/src/builder.rs:379`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new(dimension: usize) -> Self` | `crates/contextra/src/builder.rs:133` | Erzeugt einen neuen `ContextraBuilder` mit der angegebenen Vektordimension. # Parameter - `dimension`: Anzahl der Dimensionen für Vektoren (z. B. 768). # Standa |
| `pub fn with_storage_path(mut self, path: impl Into<PathBuf>) -> Self` | `crates/contextra/src/builder.rs:153` | Setzt das Speicherverzeichnis für die Datenbankdateien (LSM-SSTables, WAL, Indizes). # Parameter - `path`: Pfad im Dateisystem (z. B. `/var/lib/contextra` oder  |
| `pub fn with_max_elements(mut self, max_elements: usize) -> Self` | `crates/contextra/src/builder.rs:163` | Begrenzt die maximale Anzahl an Vektorelementen im Index. # Parameter - `max_elements`: Maximalwert (z. B. 100_000). |
| `pub fn with_distance_metric(mut self, metric: DistanceMetric) -> Self` | `crates/contextra/src/builder.rs:173` | Legt die Distanzmetrik für die Ähnlichkeitssuche fest. # Parameter - `metric`: Metrik wie Kosinus-Ähnlichkeit (`Cosine`), Euklidischer Abstand (`Euclidean`) ode |
| `pub fn with_encryption(mut self, mode: EncryptionMode) -> Self` | `crates/contextra/src/builder.rs:183` | Konfiguriert den Verschlüsselungsmodus explizit. Validiert bei Passphrasen sofort, dass diese nicht leer sind oder nur aus Leerzeichen bestehen. Leere Passphras |
| `pub fn with_encryption_passphrase(self, passphrase: impl Into<String>) -> Self` | `crates/contextra/src/builder.rs:205` | Bequeme Methode zum Setzen einer Passphrase für die Speicherverschlüsselung. Kapselt die Passphrase direkt in einem [`Zeroizing<String>`], um Rückstände im Spei |
| `pub fn is_encrypted(&self) -> bool` | `crates/contextra/src/builder.rs:212` | Prüft, ob der Builder aktuell auf verschlüsselten Modus mit Passphrase eingestellt ist. |
| `pub fn with_embedding_backend(mut self, backend: EmbeddingBackend) -> Self` | `crates/contextra/src/builder.rs:218` | Wählt das Backend für Text-Embeddings (z. B. intern via Candle oder externer Dienst). |
| `pub fn with_consolidation(mut self, enabled: bool, interval: Duration) -> Self` | `crates/contextra/src/builder.rs:229` | Konfiguriert die automatische Hintergrund-Konsolidierung (Memory Consolidation) von Erinnerungen. # Parameter - `enabled`: Schaltet den periodischen Konsolidier |
| `pub fn with_embedder(mut self, embedder: Arc<dyn TextEmbeddingEngine>) -> Self` | `crates/contextra/src/builder.rs:237` | Bindet eine benutzerdefinierte Implementierung des [`TextEmbeddingEngine`]-Ports ein. |
| `pub fn with_license_gate(self, _gate: Arc<dyn LicenseGate>) -> Self` | `crates/contextra/src/builder.rs:248` | Injiziert ein beliebiges Lizenzgate ([`LicenseGate`]) zur Ring-Autorisierung. # Veraltet Die Lizenzüberprüfung wurde entfernt. Diese Methode ist ein No-op. |
| `pub fn with_performance_profile( mut self, profile: crate::performance_profile::PerformanceProfile, ) -> Self` | `crates/contextra/src/builder.rs:257` | Setzt ein vordefiniertes Performance-Profil ([`PerformanceProfile`]). Profile (wie `Compliance`, `Balanced`, `BareMetal`) steuern standardisierte Kombinationen  |
| `pub fn with_config(mut self, mut config: ContextraConfig) -> Self` | `crates/contextra/src/builder.rs:269` | Überschreibt die Konfiguration vollständig mit einer bestehenden [`ContextraConfig`]. Überträgt auch eventuell in `config` enthaltene Passphrasen in den abgesic |
| `pub async fn build(self) -> Result<Contextra, ContextraError>` | `crates/contextra/src/builder.rs:299` | Baut und initialisiert die [`Contextra`]-Datenbankinstanz asynchron. # Ablauf von `build`: 1. **Vorabprüfung**: Prüfe, ob frühere Fehler (z. B. leere Passphrase |

#### `enum EncryptionMode`
- Ort: `crates/contextra/src/builder.rs:46`
```rust
pub enum EncryptionMode
```
Dokumentation: Konfiguration der Speicherverschlüsselung (Encryption at Rest) für persistente Datenbankschichten. Bestimmt, ob Schreib-Lese-Operationen auf Festplatte (LSM-Tree, WAL = Write-Ahead-Log) über AES-GCM-SIV verschlüsselt werden.
- Impl-Block (Trait `std::fmt::Debug`) `crates/contextra/src/builder.rs:53`

Varianten:

- `Passphrase(Zeroizing<String>)` — `crates/contextra/src/builder.rs:48`
- `Unencrypted` — `crates/contextra/src/builder.rs:50`

#### `trait ContextraConfigExt`
- Ort: `crates/contextra/src/builder.rs:64`
```rust
pub trait ContextraConfigExt
```
Dokumentation: Erweiterungstraits für [`ContextraConfig`] zur Abfrage des Verschlüsselungsstatus.

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `fn is_encrypted(&self) -> bool` | `crates/contextra/src/builder.rs:66` | Returns `true` if storage encryption is configured with a non-empty passphrase. |

### Modul `collection_profile`

#### `struct CollectionProfile`
- Ort: `crates/contextra/src/collection_profile.rs:83`
```rust
pub struct CollectionProfile
```
Dokumentation: Vollständiges Profil zur Konfiguration einer Collection. Bündelt Performance-, Persistenz- und Datenschutzaspekte für eine Sammlung.
- Impl-Block `crates/contextra/src/collection_profile.rs:100`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `performance` | `PerformanceProfile` | `crates/contextra/src/collection_profile.rs:84` |
| `kv_delete_mode` | `KvDeleteMode` | `crates/contextra/src/collection_profile.rs:85` |
| `lsm_tuning` | `LsmTuning` | `crates/contextra/src/collection_profile.rs:86` |
| `auto_extraction` | `AutoExtractionMode` | `crates/contextra/src/collection_profile.rs:87` |

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn validate(&self) -> Result<(), CollectionProfileError>` | `crates/contextra/src/collection_profile.rs:110` | Validiert die interne Konsistenz des Profils. # Ablauf von `validate`: 1. Performance-Profil auflösen und dessen Konfiguration prüfen (`resolved.validate()`). 2 |
| `pub fn validate_with_license( &self, _gate: &dyn LicenseGate, ) -> Result<(), CollectionProfileError>` | `crates/contextra/src/collection_profile.rs:126` | Validiert das Profil zusätzlich gegen ein Lizenzgate. # Veraltet Die Lizenzprüfung wurde entfernt. Diese Methode ist ein No-op bezüglich des Lizenzgates und val |

#### `struct LsmTuning`
- Ort: `crates/contextra/src/collection_profile.rs:71`
```rust
pub struct LsmTuning
```
Dokumentation: Feineinstellungsparameter für die LSM-Speicher-Engine (Log-Structured Merge-Tree). Regelt Speicherbudgets und Flush-Frequenzen, um Systemressourcen optimal auf die Hardware abzustimmen.

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `memtable_size_limit` | `u64` | `crates/contextra/src/collection_profile.rs:72` |
| `max_ram_mb` | `u32` | `crates/contextra/src/collection_profile.rs:73` |
| `group_commit_window_micros` | `u32` | `crates/contextra/src/collection_profile.rs:74` |
| `block_cache_shards` | `u16` | `crates/contextra/src/collection_profile.rs:75` |

#### `enum CollectionProfileError`
- Ort: `crates/contextra/src/collection_profile.rs:93`
```rust
pub enum CollectionProfileError
```
Dokumentation: Mögliche Fehler bei der Validierung eines `CollectionProfile`.

Varianten:

- `Performance(#[from] PerformanceProfileError)` — `crates/contextra/src/collection_profile.rs:95`
- `KvDeleteMismatch` — `crates/contextra/src/collection_profile.rs:97`

#### `enum DeploymentTier`
- Ort: `crates/contextra/src/collection_profile.rs:139`
```rust
pub enum DeploymentTier
```
Dokumentation: Vordefinierte Betriebsstufen (Deployment Tiers) für typische Anwendungsszenarien.
- Impl-Block `crates/contextra/src/collection_profile.rs:146`

Varianten:

- `EdgeMinimal` — `crates/contextra/src/collection_profile.rs:140`
- `PowerUserLocal` — `crates/contextra/src/collection_profile.rs:141`
- `EnterpriseShared` — `crates/contextra/src/collection_profile.rs:142`
- `EnterpriseRegulated` — `crates/contextra/src/collection_profile.rs:143`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn resolve(self) -> CollectionProfile` | `crates/contextra/src/collection_profile.rs:151` | Löst die Betriebsstufe in ein konkretes, vorkonfiguriertes [`CollectionProfile`] auf. Garantiert stets ein konsistentes Profil gemäß Invariante INV-COLLECTION-P |

#### `enum KvDeleteMode`
- Ort: `crates/contextra/src/collection_profile.rs:40`
```rust
pub enum KvDeleteMode
```
Dokumentation: Mode specifying KV segment deletion behavior.
- Impl-Block (Trait `Default`) `crates/contextra/src/collection_profile.rs:45`

Varianten:

- `TombstoneOnly` — `crates/contextra/src/collection_profile.rs:41`
- `CryptoShred` — `crates/contextra/src/collection_profile.rs:42`

#### `const DEFAULT_AUTO_EXTRACTION_MODE`
- Ort: `crates/contextra/src/collection_profile.rs:55`
```rust
pub const DEFAULT_AUTO_EXTRACTION_MODE: AutoExtractionMode = AutoExtractionMode::Enabled
```
Dokumentation: Standardmodus für die automatische LLM-gestützte Metadaten-/Entitäten-Extraktion bei Standard-Tiers. Standardmäßig aktiviert (`AutoExtractionMode::Enabled`).

#### `const ENTERPRISE_REGULATED_AUTO_EXTRACTION_DEFAULT`
- Ort: `crates/contextra/src/collection_profile.rs:63`
```rust
pub const ENTERPRISE_REGULATED_AUTO_EXTRACTION_DEFAULT: AutoExtractionMode = AutoExtractionMode::Disabled
```
Dokumentation: Standardmodus für `EnterpriseRegulated`. In EnterpriseRegulated wird kein Dokumenttext automatisch an ein Extraktions-LLM gegeben, solange der Betreiber es nicht ausdrücklich aktiviert (Datenminimierung); explizite Aktivierung pro Collection bleibt möglich.

### Modul `crate`

#### `mod agent_memory`
- Ort: `crates/contextra/src/lib.rs:39`
```rust
pub mod agent_memory
```

#### `mod boot`
- Ort: `crates/contextra/src/lib.rs:40`
```rust
pub mod boot
```

#### `mod builder`
- Ort: `crates/contextra/src/lib.rs:41`
```rust
pub mod builder
```

#### `mod collection_profile`
- Ort: `crates/contextra/src/lib.rs:42`
```rust
pub mod collection_profile
```

#### `mod performance_profile`
- Ort: `crates/contextra/src/lib.rs:43`
```rust
pub mod performance_profile
```

#### `mod query_rewriter`
- Ort: `crates/contextra/src/lib.rs:44`
```rust
pub mod query_rewriter
```

#### `fn builder`
- Ort: `crates/contextra/src/lib.rs:88`
```rust
pub fn builder(dimension: usize) -> ContextraBuilder
```
Dokumentation: Erzeugt einen neuen [`ContextraBuilder`] zur Konfiguration und Instanziierung der Datenbank [`Contextra`]. # Parameter - `dimension`: Die Anzahl der Dimensionen (z. B. 384, 768, 1536) für die Vektoreinbettungen (Embeddings). # Rückgabe Gibt einen vorinitialisierten [`ContextraBuilder`] mit Standardeinstellungen zurück. # Ablauf von `builder`: 1. Übernahme der gewünschten Vektordimension (`dimension`). 2. Initialisierung des Builders mit Default-Parametern (Speicherpfad `./contextra_data`, Fast-Lizenzgate). 3. Rückgabe der Builder-Instanz zur methodenweisen Weiterkonfiguration. # Vorbedingungen

#### `fn open`
- Ort: `crates/contextra/src/lib.rs:130`
```rust
pub async fn open(path: impl AsRef<std::path::Path>) -> Result<Contextra, ContextraError>
```
Dokumentation: Öffnet oder erstellt eine [`Contextra`]-Datenbankinstanz am angegebenen Dateisystempfad mit Standardkonfiguration. Verwendet standardmäßig eine Vektordimension von 768 und das unbeschränkte Entwicklungs-Lizenzgate (`OpenFastGate`). # Parameter - `path`: Dateipfad zum Datenbankverzeichnis (wird bei Bedarf erstellt). # Rückgabe - `Ok(Contextra)`: Vollständig initialisierte Datenbankinstanz. - `Err(ContextraError)`: Fehler bei Dateisystem-I/O, Berechtigungen oder Datenbankkorruption. # Ablauf von `open`: 1. Erzeugung eines Builders mit Standard-Vektordimension 768. 2. Zuweisung des Zielpfads im D

#### `fn open_with_config`
- Ort: `crates/contextra/src/lib.rs:162`
```rust
pub async fn open_with_config( path: impl AsRef<std::path::Path>, config: ContextraConfig, ) -> Result<Contextra, ContextraError>
```
Dokumentation: Öffnet oder erstellt eine [`Contextra`]-Datenbankinstanz mit einer explizit übergebenen Konfiguration. # Parameter - `path`: Dateipfad zum Datenbankverzeichnis. - `config`: Detaillierte Konfiguration ([`ContextraConfig`]) u. a. mit Vektordimension, Speichergrenzen und Tuning-Parametern. # Rückgabe - `Ok(Contextra)`: Bereitgestellte Datenbankinstanz. - `Err(ContextraError)`: Konfigurationskonflikt, I/O-Fehler oder Verstoß gegen Lizenzrichtlinien. # Ablauf von `open_with_config`: 1. Entnahme der Dimension aus der übergebenen `config` (`config.dimension`). 2. Konfiguration des Pfades (`with_stora

### Modul `performance_profile`

#### `struct ResolvedProfileConfig`
- Ort: `crates/contextra/src/performance_profile.rs:118`
```rust
pub struct ResolvedProfileConfig
```
Dokumentation: Vollständig aufgelöste Konfigurationsparameter eines Performance-Profils.
- Impl-Block `crates/contextra/src/performance_profile.rs:170`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `durability_mode` | `DurabilityMode` | `crates/contextra/src/performance_profile.rs:119` |
| `vector_delete_mode` | `VectorDeleteMode` | `crates/contextra/src/performance_profile.rs:120` |
| `integrity_proof_active` | `bool` | `crates/contextra/src/performance_profile.rs:121` |
| `feature_ring` | `FeatureRing` | `crates/contextra/src/performance_profile.rs:122` |

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn enforce_license(&self, _gate: &dyn LicenseGate) -> Result<(), PerformanceProfileError>` | `crates/contextra/src/performance_profile.rs:177` | Prüft den geforderten Feature-Ring gegen das übergebene [`LicenseGate`]. # Veraltet Lizenzprüfung wurde entfernt; diese Methode ist ein No-op und liefert stets  |
| `pub fn validate(&self) -> Result<(), PerformanceProfileError>` | `crates/contextra/src/performance_profile.rs:187` | Validiert die gegenseitige Kompatibilität der Profilparameter. Stellt sicher, dass Durability und Vektor-Bereinigung nicht in einen inkonsistenten Zustand gerat |

#### `enum DurabilityConfigError`
- Ort: `crates/contextra/src/performance_profile.rs:53`
```rust
pub enum DurabilityConfigError
```
Dokumentation: Fehler bei der Validierung des DurabilityMode gegen Ring- und Integritätsfeatures.

Varianten:

- `IncompatibleCombination{` — `crates/contextra/src/performance_profile.rs:55`

#### `enum DurabilityMode`
- Ort: `crates/contextra/src/performance_profile.rs:39`
```rust
pub enum DurabilityMode
```
Dokumentation: Durability mode for WAL persistence and integrity checks.
- Impl-Block (Trait `Default`) `crates/contextra/src/performance_profile.rs:45`
- Impl-Block `crates/contextra/src/performance_profile.rs:68`
- Impl-Block (Trait `PartialEq<contextra_db::DurabilityMode>`) `crates/contextra/src/performance_profile.rs:307`
- Impl-Block (Trait `PartialEq<DurabilityMode>`) `crates/contextra/src/performance_profile.rs:318`
- Impl-Block (Trait `From<DurabilityMode>`) `crates/contextra/src/performance_profile.rs:324`
- Impl-Block (Trait `From<contextra_db::DurabilityMode>`) `crates/contextra/src/performance_profile.rs:334`

Varianten:

- `Full` — `crates/contextra/src/performance_profile.rs:40`
- `WalNoHmac` — `crates/contextra/src/performance_profile.rs:41`
- `MemoryOnly` — `crates/contextra/src/performance_profile.rs:42`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn validate_against_features( self, integrity_proof_active: bool, feature_ring: FeatureRing, ) -> Result<(), DurabilityConfigError>` | `crates/contextra/src/performance_profile.rs:70` | Validiert die Kombination aus DurabilityMode, Integritätsnachweisen und Feature-Ring. |

#### `enum PerformanceProfile`
- Ort: `crates/contextra/src/performance_profile.rs:102`
```rust
pub enum PerformanceProfile
```
Dokumentation: Vordefinierte Leistungsprofile für Contextra-Instanzen.
- Impl-Block (Trait `Default`) `crates/contextra/src/performance_profile.rs:108`
- Impl-Block `crates/contextra/src/performance_profile.rs:138`

Varianten:

- `Compliance` — `crates/contextra/src/performance_profile.rs:103`
- `Balanced` — `crates/contextra/src/performance_profile.rs:104`
- `BareMetal` — `crates/contextra/src/performance_profile.rs:105`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn resolve(self) -> ResolvedProfileConfig` | `crates/contextra/src/performance_profile.rs:146` | Löst das Profil in konkrete Konfigurationsparameter auf. # Ablauf von `resolve`: - `Compliance`: Full Durability, SynchronousRepair, IntegrityProof aktiv, Sover |

#### `enum PerformanceProfileError`
- Ort: `crates/contextra/src/performance_profile.rs:128`
```rust
pub enum PerformanceProfileError
```
Dokumentation: Fehler bei der Validierung oder Lizenzierung eines Performance-Profils.

Varianten:

- `Durability(#[from] DurabilityConfigError)` — `crates/contextra/src/performance_profile.rs:130`
- `DurabilityVectorMismatch` — `crates/contextra/src/performance_profile.rs:132`
- `LicenseRequired(FeatureRing, contextra_ports::license::LicenseError)` — `crates/contextra/src/performance_profile.rs:135`

### Modul `query_rewriter`

#### `struct LlmQueryRewriter`
- Ort: `crates/contextra/src/query_rewriter.rs:40`
```rust
pub struct LlmQueryRewriter
```
Dokumentation: LLM-unterstützter Abfrage-Umformulierer für iteratives oder mehrstufiges Retrieval (Multi-Step Retrieval). Sendet den aktuellen Suchkontext an ein LLM via [`LlmTextGenerator`], um alternative, semantisch präzisere Suchanfragen zu generieren.
- Impl-Block `crates/contextra/src/query_rewriter.rs:47`
- Impl-Block (Trait `QueryRewriter`) `crates/contextra/src/query_rewriter.rs:93`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new(generator: Arc<dyn LlmTextGenerator>) -> Self` | `crates/contextra/src/query_rewriter.rs:53` | Erstellt einen neuen `LlmQueryRewriter` mit Standardgrenzwerten: - `max_subqueries`: 3 alternative Abfragen - `max_context_results`: 3 Dokumente als Kontext - ` |
| `pub fn with_max_subqueries(mut self, n: usize) -> Self` | `crates/contextra/src/query_rewriter.rs:64` | Konfiguriert die maximale Anzahl an generierten Teilabfragen. |
| `pub fn with_max_context_results(mut self, n: usize) -> Self` | `crates/contextra/src/query_rewriter.rs:71` | Konfiguriert die maximale Anzahl an bestehenden Suchergebnissen, die als Kontext in den Prompt fließen. |
| `pub fn with_max_snippet_chars(mut self, n: usize) -> Self` | `crates/contextra/src/query_rewriter.rs:78` | Konfiguriert die maximale Textlänge (in Zeichen) für Textausschnitte (Snippets) im Prompt. |
| `pub async fn rewrite_query( &self, original_query: &str, current_results: &[ScoredEntry], ) -> Result<Vec<String>>` | `crates/contextra/src/query_rewriter.rs:84` | Öffentlicher Einstiegspunkt zur asynchronen Umformulierung einer Suchanfrage. |

## 9. Impl-Blöcke ohne Typ im selben Crate

- `impl ContextraConfigExt for ContextraConfig` — `crates/contextra/src/builder.rs:69` (0 pub Methoden)
- `impl From<DurabilityConfigError> for contextra_types::ContextraError` — `crates/contextra/src/performance_profile.rs:62` (0 pub Methoden)
