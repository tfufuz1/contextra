# Schnittstellenspezifikation: `contextra-vault`

> Erzeugt durch `tools/extract_interfaces.py` aus dem Quellcode. Keine Handpflege. Jede Angabe trägt Datei:Zeile. Maßgeblich bleibt der Quellcode.

## 1. Kopf

| Feld | Wert |
|---|---|
| Name | `contextra-vault` |
| Version | 0.1.0 |
| Pfad | `crates/contextra-vault` |
| Rolle | RAM-Staging (excluded) |
| Bibliothekspfad | src/lib.rs (Standard) |

**Crate-Dokumentation (//!):**

> # contextra-vault
>
> Ephemerer, speichergesicherter Chunk- und Kontext-Tresor (Ebene 1, Kern-Closure `{contextra-sys}`).
>
> Dieses Crate implementiert einen flüchtigen Zwischenspeicher für sensible flüchtige
> In-Memory-Daten, basierend auf `contextra-sys::vault::LockedRegions`.

## 2. Abhängigkeiten

| Name | Art | Quelle | optional | verwendet von (normal/dev) |
|---|---|---|---|---|
| `contextra-sys` | normal | path | nein | contextra-bench, contextra-store, contextra-vault, contextra-vector |
| `contextra-types` | normal | path | ja | contextra, contextra-adapt, contextra-agent, contextra-bench, contextra-checkpoint, contextra-cognition, contextra-crypto, contextra-crypto-fuzz, contextra-db, contextra-engine, contextra-graph, contextra-graph-fuzz, contextra-infer-candle, contextra-infer-ollama, contextra-infer-onnx, contextra-ingest, contextra-kvcache, contextra-mcp, contextra-mvcc, contextra-ports, contextra-py, contextra-rank, contextra-router, contextra-simd, contextra-store, contextra-store-fuzz, contextra-testkit, contextra-text, contextra-vault, contextra-vector, contextra-vector-fuzz |
| `tempfile` | dev | registry | nein |  |
| `thiserror` | normal | registry | nein |  |
| `tracing` | normal | registry | nein |  |
| `zeroize` | normal | registry | nein |  |

**Verwendet von (Crate-Ebene):** `contextra-db`

## 3. Features

| Feature | Aktiviert |
|---|---|
| `default` | – |
| `workspace` | `dep:contextra-types` |

## 4. Metriken (Quellcode)

| Kennzahl | Wert |
|---|---|
| Quelldateien | 4 |
| Quellzeilen | 529 |
| Öffentliche Items (Modul-Ebene) | 14 |
| davon `struct` | 9 |
| davon `enum` | 2 |
| davon `mod` | 3 |
| Öffentliche Makros (`#[macro_export]`) | 0 |
| Öffentliche Re-Exports (`pub use`) | 2 |
| `unsafe`-Vorkommen (Produktionscode) | 0 |
| `.unwrap()` (Produktionscode) | 0 |
| `.expect(` (Produktionscode) | 0 |
| `panic!` (Produktionscode) | 0 |
| `todo!/unimplemented!/unreachable!` | 0 |
| Indexierungen `x[…]` (grob, Produktionscode) | 0 |
| `#[deprecated]` | 0 |
| `#[cfg(feature…)]`-Gates | 1 |
| Parser-Abdeckung (geparste `pub`-Deklarationen / Grep-Treffer) | 33 / 33 = 100 % |

## 5. Modulbaum und Dateien

| Modul | Datei | Zeilen |
|---|---|---|
| `compat` | `crates/contextra-vault/src/compat/mod.rs` | 61 |
| `error` | `crates/contextra-vault/src/error.rs` | 20 |
| `crate` | `crates/contextra-vault/src/lib.rs` | 20 |
| `vault` | `crates/contextra-vault/src/vault.rs` | 428 |

## 6. Re-Exports (`pub use`)

- `pub use error::VaultError` — `crates/contextra-vault/src/lib.rs:16` (Modul `crate`)
- `pub use vault::` — `crates/contextra-vault/src/lib.rs:17` (Modul `crate`)

## 8. Öffentliche Schnittstelle je Item

### Modul `crate`

#### `mod compat`
- Ort: `crates/contextra-vault/src/lib.rs:14`
```rust
pub mod compat
```
- Gate: `#[cfg(feature = "workspace")]`

#### `mod error`
- Ort: `crates/contextra-vault/src/lib.rs:10`
```rust
pub mod error
```

#### `mod vault`
- Ort: `crates/contextra-vault/src/lib.rs:11`
```rust
pub mod vault
```

### Modul `error`

#### `enum VaultError`
- Ort: `crates/contextra-vault/src/error.rs:6`
```rust
pub enum VaultError
```
Dokumentation: Fehlerzustände des flüchtigen Speicher-Puffers.

Varianten:

- `CapacityExceeded{` — `crates/contextra-vault/src/error.rs:9`
- `AlreadyConsumed` — `crates/contextra-vault/src/error.rs:19`

### Modul `vault`

#### `struct CommitReceipt`
- Ort: `crates/contextra-vault/src/vault.rs:182`
```rust
pub struct CommitReceipt
```
Dokumentation: Quittung nach Übergabe der Chunks an die persistente Datenbank via [`VolatileContextVault::commit()`].

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `chunks_committed` | `usize` | `crates/contextra-vault/src/vault.rs:184` |
| `bytes_committed` | `usize` | `crates/contextra-vault/src/vault.rs:186` |
| `tenant_id` | `VaultTenantId` | `crates/contextra-vault/src/vault.rs:188` |
| `tx_id` | `VaultTxId` | `crates/contextra-vault/src/vault.rs:190` |
| `metadata_reference` | `Option<String>` | `crates/contextra-vault/src/vault.rs:192` |

#### `struct VaultChunk`
- Ort: `crates/contextra-vault/src/vault.rs:95`
```rust
pub struct VaultChunk
```
Dokumentation: Ein einzelner Datenblock (Chunk) im `VolatileContextVault`. Sämtliche Inhalte werden beim Drop via [`ZeroizeOnDrop`] kryptographisch überschrieben.
- Impl-Block `crates/contextra-vault/src/vault.rs:112`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `id` | `VaultDocId` | `crates/contextra-vault/src/vault.rs:98` |
| `content` | `Vec<u8>` | `crates/contextra-vault/src/vault.rs:100` |
| `modality` | `SignalModality` | `crates/contextra-vault/src/vault.rs:103` |
| `captured_tx` | `u64` | `crates/contextra-vault/src/vault.rs:106` |
| `label` | `Option<String>` | `crates/contextra-vault/src/vault.rs:109` |

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new( id: VaultDocId, content: Vec<u8>, modality: SignalModality, captured_tx: VaultTxId, ) -> Self` | `crates/contextra-vault/src/vault.rs:120` | Erzeugt einen neuen sensitiven Chunk. # Parameter - `id`: Eindeutige Dokumenten-ID. - `content`: Rohe Nutzdaten (z. B. Text- oder Binärdaten). - `modality`: Art |
| `pub fn with_label(mut self, label: impl Into<String>) -> Self` | `crates/contextra-vault/src/vault.rs:136` | Weist dem Chunk eine beschreibende Bezeichnung (Label) zu. |
| `pub fn size_bytes(&self) -> usize` | `crates/contextra-vault/src/vault.rs:142` | Gibt die Größe des Chunk-Inhalts in Bytes zurück. |

#### `struct VaultChunkMetadata`
- Ort: `crates/contextra-vault/src/vault.rs:396`
```rust
pub struct VaultChunkMetadata
```
Dokumentation: Metadaten-Ansicht eines Chunks ohne sensiblen Nutzdateninhalt.

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `id` | `VaultDocId` | `crates/contextra-vault/src/vault.rs:398` |
| `modality` | `SignalModality` | `crates/contextra-vault/src/vault.rs:400` |
| `size_bytes` | `usize` | `crates/contextra-vault/src/vault.rs:402` |
| `label` | `Option<String>` | `crates/contextra-vault/src/vault.rs:404` |

#### `struct VaultConfig`
- Ort: `crates/contextra-vault/src/vault.rs:149`
```rust
pub struct VaultConfig
```
Dokumentation: Konfiguration für den `VolatileContextVault`.
- Impl-Block (Trait `Default`) `crates/contextra-vault/src/vault.rs:157`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `max_capacity_bytes` | `usize` | `crates/contextra-vault/src/vault.rs:152` |
| `attempt_mlock` | `bool` | `crates/contextra-vault/src/vault.rs:154` |

#### `struct VaultDocId`
- Ort: `crates/contextra-vault/src/vault.rs:32`
```rust
pub struct VaultDocId(pub u64)
```
Dokumentation: Eindeutige ID eines Dokuments/Chunks im Vault (Kern-Typ).
- Impl-Block (Trait `From<DocId>`) `crates/contextra-vault/src/compat/mod.rs:19`
- Impl-Block `crates/contextra-vault/src/vault.rs:34`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub const fn new(id: u64) -> Self` | `crates/contextra-vault/src/vault.rs:36` | Erzeugt eine neue VaultDocId. |
| `pub const fn as_u64(self) -> u64` | `crates/contextra-vault/src/vault.rs:41` | Gibt den inneren numerischen Wert zurück. |

#### `struct VaultTenantId`
- Ort: `crates/contextra-vault/src/vault.rs:64`
```rust
pub struct VaultTenantId(pub u64)
```
Dokumentation: Mandanten-Kennung (Kern-Typ).
- Impl-Block (Trait `From<TenantId>`) `crates/contextra-vault/src/compat/mod.rs:47`
- Impl-Block `crates/contextra-vault/src/vault.rs:66`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub const fn new(id: u64) -> Self` | `crates/contextra-vault/src/vault.rs:68` | Erzeugt eine neue VaultTenantId. |
| `pub const fn as_u64(self) -> u64` | `crates/contextra-vault/src/vault.rs:73` | Gibt den inneren numerischen Wert zurück. |

#### `struct VaultTxId`
- Ort: `crates/contextra-vault/src/vault.rs:48`
```rust
pub struct VaultTxId(pub u64)
```
Dokumentation: Transaktions- und Sequenz-ID (Kern-Typ).
- Impl-Block (Trait `From<TxId>`) `crates/contextra-vault/src/compat/mod.rs:33`
- Impl-Block `crates/contextra-vault/src/vault.rs:50`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub const fn new(id: u64) -> Self` | `crates/contextra-vault/src/vault.rs:52` | Erzeugt eine neue VaultTxId. |
| `pub const fn as_u64(self) -> u64` | `crates/contextra-vault/src/vault.rs:57` | Gibt den inneren numerischen Wert zurück. |

#### `struct VolatileContextVault`
- Ort: `crates/contextra-vault/src/vault.rs:205`
```rust
pub struct VolatileContextVault
```
Dokumentation: Ephemerer RAM-Puffer für sensitive Kontexte im Safe-Modus. # INVARIANTEN - **INV-VAULT-1:** `purge()` zeroized alle Chunk-Inhalte via `ZeroizeOnDrop` vor der Deallokation. `std::mem::forget` auf den Vault hebelt diese Invariante aus. - **INV-VAULT-2:** `ingest()` schreibt NIE in Storage, Log-Writer oder irgendeinen Vektorindex. Einziger Schreibpfad auf dauerhafte Ablage: expliziter Aufruf von `commit()`. - **INV-VAULT-3:** Kapazitätsüberschreitung führt zu `VaultError::CapacityExceeded`. Kein stiller Fallback auf Disk-Speicher.
- Impl-Block `crates/contextra-vault/src/vault.rs:217`
- Impl-Block (Trait `Drop`) `crates/contextra-vault/src/vault.rs:409`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn open(config: VaultConfig) -> Self` | `crates/contextra-vault/src/vault.rs:221` | Öffnet einen neuen leeren Vault im RAM. Führt keinerlei Datei-I/O oder Festplattenzugriffe aus. |
| `pub fn ingest(&mut self, chunk: VaultChunk) -> Result<(), VaultError>` | `crates/contextra-vault/src/vault.rs:248` | Nimmt einen sensitiven Chunk in den RAM-Puffer auf. # Ablauf von `ingest`: 1. Status prüfen: Wurde der Vault bereits konsumiert? Falls ja: `Err(AlreadyConsumed) |
| `pub fn purge_memory(mut self) -> VolatileDrainReceipt` | `crates/contextra-vault/src/vault.rs:288` | Verwirft alle Chunks sofort und überschreibt deren Heap-Inhalte kryptographisch mit Nullen. # Ablauf von `purge`: 1. Alle Chunk-Inhalte im Heap explizit via [`Z |
| `pub fn purge(self) -> VolatileDrainReceipt` | `crates/contextra-vault/src/vault.rs:316` | Neutralisierte Speicherleeren-Methode (Alias für [`Self::purge_memory`]). |
| `pub fn current_size_bytes(&self) -> usize` | `crates/contextra-vault/src/vault.rs:321` | Gibt die Summe der aktuell im flüchtigen RAM liegenden Nutzdaten in Bytes zurück. |
| `pub fn len(&self) -> usize` | `crates/contextra-vault/src/vault.rs:326` | Gibt die Anzahl der aktuell gepufferten Chunks zurück. |
| `pub fn is_empty(&self) -> bool` | `crates/contextra-vault/src/vault.rs:331` | Prüft, ob der Vault leer ist. |
| `pub fn is_memory_locked(&self) -> bool` | `crates/contextra-vault/src/vault.rs:339` | Gibt an, ob alle Chunk-Inhalte im RAM gegen Swap gesichert sind (`mlock`). Liefert `false`, falls `attempt_mlock` deaktiviert war oder das Betriebssystem das Sp |
| `pub fn commit( mut self, tenant_id: VaultTenantId, tx_id: VaultTxId, metadata_reference: Option<impl Into<String>>, ) -> Result<(Vec<VaultChunk>, CommitReceipt), VaultError>` | `crates/contextra-vault/src/vault.rs:350` | Transferiert alle Chunks zur dauerhaften Speicherung und stellt ein [`CommitReceipt`] aus. # Ablauf von `commit`: 1. Status prüfen: Falls bereits konsumiert, `E |
| `pub fn preview_metadata(&self) -> Vec<VaultChunkMetadata>` | `crates/contextra-vault/src/vault.rs:381` | Gibt eine Vorschau der Metadaten zurück, ohne sensible Inhalte zu exponieren. Verhindert versehentliches Ausleiten oder Protokollieren von Geheimnissen. |

#### `struct VolatileDrainReceipt`
- Ort: `crates/contextra-vault/src/vault.rs:169`
```rust
pub struct VolatileDrainReceipt
```
Dokumentation: Quittung nach erfolgreicher flüchtiger Speicherbereinigung via [`VolatileContextVault::purge_memory()`].

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `chunks_purged` | `usize` | `crates/contextra-vault/src/vault.rs:171` |
| `bytes_zeroed` | `usize` | `crates/contextra-vault/src/vault.rs:173` |
| `purged_at` | `Instant` | `crates/contextra-vault/src/vault.rs:175` |

#### `enum SignalModality`
- Ort: `crates/contextra-vault/src/vault.rs:80`
```rust
pub enum SignalModality
```
Dokumentation: Modalität eines Vault-Chunks — bestimmt den Verarbeitungskontext beim späteren Commit.

Varianten:

- `AudioTranscript` — `crates/contextra-vault/src/vault.rs:82`
- `VisualCapture` — `crates/contextra-vault/src/vault.rs:84`
- `TextInput` — `crates/contextra-vault/src/vault.rs:86`
- `StructuredData` — `crates/contextra-vault/src/vault.rs:88`

## 9. Impl-Blöcke ohne Typ im selben Crate

- `impl From<VaultDocId> for DocId` — `crates/contextra-vault/src/compat/mod.rs:12` (0 pub Methoden)
- `impl From<VaultTxId> for TxId` — `crates/contextra-vault/src/compat/mod.rs:26` (0 pub Methoden)
- `impl From<VaultTenantId> for TenantId` — `crates/contextra-vault/src/compat/mod.rs:40` (0 pub Methoden)
- `impl From<VaultError> for ContextraError` — `crates/contextra-vault/src/compat/mod.rs:54` (0 pub Methoden)
