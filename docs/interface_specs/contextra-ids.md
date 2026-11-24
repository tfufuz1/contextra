# Schnittstellenspezifikation: `contextra-ids`

> Erzeugt durch `tools/extract_interfaces.py` aus dem Quellcode. Keine Handpflege. Jede Angabe trägt Datei:Zeile. Maßgeblich bleibt der Quellcode.

## 1. Kopf

| Feld | Wert |
|---|---|
| Name | `contextra-ids` |
| Version | 0.1.0 |
| Pfad | `crates/contextra-ids` |
| Rolle | Identifikatoren |
| Bibliothekspfad | src/lib.rs (Standard) |

**Crate-Dokumentation (//!):**

> Canonical identity, tenant, and reference types for Contextra.
>
> # Architecture Role
> Zero internal dependencies. Standalone micro-kernel for identifiers.

## 2. Abhängigkeiten

| Name | Art | Quelle | optional | verwendet von (normal/dev) |
|---|---|---|---|---|
| `bincode` | dev | registry | nein |  |
| `blake3` | normal | registry | nein |  |
| `hex` | dev | registry | nein |  |
| `serde` | normal | registry | nein |  |
| `serde_json` | dev | registry | nein |  |
| `thiserror` | normal | registry | nein |  |

**Verwendet von (Crate-Ebene):** `contextra-checkpoint`, `contextra-crypto`, `contextra-graph`, `contextra-mvcc`, `contextra-store`, `contextra-text`, `contextra-types`, `contextra-vector`

## 3. Features

| Feature | Aktiviert |
|---|---|
| `default` | `std` |
| `docid-128` | – |
| `std` | `blake3/std`, `serde/std` |

## 4. Metriken (Quellcode)

| Kennzahl | Wert |
|---|---|
| Quelldateien | 1 |
| Quellzeilen | 720 |
| Öffentliche Items (Modul-Ebene) | 19 |
| davon `struct` | 12 |
| davon `enum` | 2 |
| davon `type` | 1 |
| davon `const` | 4 |
| Öffentliche Makros (`#[macro_export]`) | 0 |
| Öffentliche Re-Exports (`pub use`) | 0 |
| `unsafe`-Vorkommen (Produktionscode) | 0 |
| `.unwrap()` (Produktionscode) | 0 |
| `.expect(` (Produktionscode) | 0 |
| `panic!` (Produktionscode) | 0 |
| `todo!/unimplemented!/unreachable!` | 0 |
| Indexierungen `x[…]` (grob, Produktionscode) | 8 |
| `#[deprecated]` | 2 |
| `#[cfg(feature…)]`-Gates | 7 |
| Parser-Abdeckung (geparste `pub`-Deklarationen / Grep-Treffer) | 79 / 83 = 95 % |

## 5. Modulbaum und Dateien

| Modul | Datei | Zeilen |
|---|---|---|
| `crate` | `crates/contextra-ids/src/lib.rs` | 720 |

## 8. Öffentliche Schnittstelle je Item

### Modul `crate`

#### `struct CollectionId`
- Ort: `crates/contextra-ids/src/lib.rs:148`
```rust
pub struct CollectionId(pub u64)
```
Dokumentation: Collection-ID.
- Impl-Block `crates/contextra-ids/src/lib.rs:150`
- Impl-Block (Trait `Default`) `crates/contextra-ids/src/lib.rs:184`
- Impl-Block (Trait `From<u64>`) `crates/contextra-ids/src/lib.rs:190`
- Impl-Block (Trait `fmt::Display`) `crates/contextra-ids/src/lib.rs:197`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub const fn new(id: u64) -> Self` | `crates/contextra-ids/src/lib.rs:158` | Creates a new `CollectionId` wrapping the provided `u64` identifier. |
| `pub fn try_new(id: u64) -> Result<Self>` | `crates/contextra-ids/src/lib.rs:163` | Creates a new `CollectionId`, ensuring `id != 0`. |
| `pub const fn inner(self) -> u64` | `crates/contextra-ids/src/lib.rs:173` | Returns the inner raw `u64` identifier. |
| `pub const fn as_u64(self) -> u64` | `crates/contextra-ids/src/lib.rs:179` | Returns the inner raw `u64` identifier. |

Assoziierte Konstanten/Typen:

- `pub const SYSTEM: Self = Self(0)` — `crates/contextra-ids/src/lib.rs:152`
- `pub const INVALID: Self = Self(0)` — `crates/contextra-ids/src/lib.rs:154`

#### `struct DocId`
- Ort: `crates/contextra-ids/src/lib.rs:224`
```rust
pub struct DocId(pub u64)
```
Dokumentation: Dokument-ID.
- Impl-Block `crates/contextra-ids/src/lib.rs:233`
- Impl-Block `crates/contextra-ids/src/lib.rs:267`
- Impl-Block (Trait `From<u64>`) `crates/contextra-ids/src/lib.rs:300`
- Impl-Block (Trait `From<u128>`) `crates/contextra-ids/src/lib.rs:315`
- Impl-Block (Trait `fmt::Display`) `crates/contextra-ids/src/lib.rs:322`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub const fn new(id: u64) -> Self` | `crates/contextra-ids/src/lib.rs:241` | Creates a new `DocId` wrapping the provided `u64` identifier. |
| `pub fn from_key(key: &str) -> Result<Self>` | `crates/contextra-ids/src/lib.rs:246` | Derives a `DocId` from a string key using the first 8 bytes (64 bits) of its BLAKE3 hash. |
| `pub const fn inner(self) -> u64` | `crates/contextra-ids/src/lib.rs:255` | Returns the inner raw identifier. |
| `pub const fn as_u64(self) -> u64` | `crates/contextra-ids/src/lib.rs:261` | Returns the inner raw identifier as `u64`. |
| `pub const fn new(id: u128) -> Self` | `crates/contextra-ids/src/lib.rs:275` | Creates a new `DocId` wrapping the provided `u128` identifier. |
| `pub fn from_key(key: &str) -> Result<Self>` | `crates/contextra-ids/src/lib.rs:280` | Derives a 128-bit `DocId` from a string key using the first 16 bytes (128 bits) of its BLAKE3 hash. |
| `pub const fn inner(self) -> u128` | `crates/contextra-ids/src/lib.rs:289` | Returns the inner raw identifier. |
| `pub const fn as_u64(self) -> u64` | `crates/contextra-ids/src/lib.rs:295` | Returns the lower 64 bits of the identifier. |

Assoziierte Konstanten/Typen:

- `pub const MAX: Self = Self(u64::MAX)` — `crates/contextra-ids/src/lib.rs:235`
- `pub const MIN: Self = Self(0)` — `crates/contextra-ids/src/lib.rs:237`
- `pub const MAX: Self = Self(u128::MAX)` — `crates/contextra-ids/src/lib.rs:269`
- `pub const MIN: Self = Self(0)` — `crates/contextra-ids/src/lib.rs:271`

#### `struct DocId`
- Ort: `crates/contextra-ids/src/lib.rs:230`
```rust
pub struct DocId(pub u128)
```
Dokumentation: Dokument-ID (128-Bit).
- Gate: `#[cfg(feature = "docid-128")]`

#### `struct DocRef`
- Ort: `crates/contextra-ids/src/lib.rs:649`
```rust
pub struct DocRef
```
Dokumentation: Eindeutige Dokumentenadresse: Betroffener ist unveränderlicher Teil der Identität (SPEC 7.2, D-70).

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `subject` | `Option<SubjectRef>` | `crates/contextra-ids/src/lib.rs:651` |
| `key` | `DocKey` | `crates/contextra-ids/src/lib.rs:653` |

#### `struct EntityId`
- Ort: `crates/contextra-ids/src/lib.rs:331`
```rust
pub struct EntityId(pub u64)
```
Dokumentation: Entity-ID für Graph- und semantische Knoten.
- Impl-Block `crates/contextra-ids/src/lib.rs:333`
- Impl-Block (Trait `From<u64>`) `crates/contextra-ids/src/lib.rs:375`
- Impl-Block (Trait `From<&str>`) `crates/contextra-ids/src/lib.rs:382`
- Impl-Block (Trait `From<String>`) `crates/contextra-ids/src/lib.rs:392`
- Impl-Block (Trait `fmt::Display`) `crates/contextra-ids/src/lib.rs:398`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub const fn new(id: u64) -> Self` | `crates/contextra-ids/src/lib.rs:336` | Creates a new `EntityId` wrapping the provided `u64` identifier. |
| `pub const fn inner(self) -> u64` | `crates/contextra-ids/src/lib.rs:342` | Returns the inner raw `u64` identifier. |
| `pub const fn as_u64(self) -> u64` | `crates/contextra-ids/src/lib.rs:348` | Returns the inner raw `u64` identifier. |
| `pub fn as_bytes(&self) -> Vec<u8>` | `crates/contextra-ids/src/lib.rs:353` | Converts the entity identifier into its string byte representation. |
| `pub fn from_doc_id(doc_id: DocId) -> Self` | `crates/contextra-ids/src/lib.rs:359` | Creates an `EntityId` directly from a `DocId`. |
| `pub fn from_doc_id(doc_id: DocId) -> Self` | `crates/contextra-ids/src/lib.rs:365` | Creates an `EntityId` directly from a `DocId`. |
| `pub fn from_key(key: &str) -> Result<Self>` | `crates/contextra-ids/src/lib.rs:370` | Derives an `EntityId` from a string key using the first 8 bytes of its BLAKE3 hash. |

#### `struct FileId`
- Ort: `crates/contextra-ids/src/lib.rs:498`
```rust
pub struct FileId(pub [u8
```
Dokumentation: 16-Byte Dateikennung (SPEC F.1).
- Impl-Block `crates/contextra-ids/src/lib.rs:500`
- Impl-Block (Trait `fmt::Debug`) `crates/contextra-ids/src/lib.rs:512`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub const fn new(bytes: [u8` | `crates/contextra-ids/src/lib.rs:502` | Erzeugt eine `FileId` aus rohen 16 Bytes. |
| `pub const fn as_bytes(&self) -> &[u8` | `crates/contextra-ids/src/lib.rs:507` | Liefert die 16 Bytes als Slice. |

#### `struct Seq`
- Ort: `crates/contextra-ids/src/lib.rs:552`
```rust
pub struct Seq(pub u64)
```
- Impl-Block `crates/contextra-ids/src/lib.rs:554`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub const fn new(val: u64) -> Self` | `crates/contextra-ids/src/lib.rs:559` | Erzeugt eine neue Sequenznummer. |
| `pub const fn as_u64(&self) -> u64` | `crates/contextra-ids/src/lib.rs:564` | Liefert den numerischen Wert als u64. |
| `pub const fn next(self) -> Self` | `crates/contextra-ids/src/lib.rs:569` | Liefert die nächste Sequenznummer. |

Assoziierte Konstanten/Typen:

- `pub const ZERO: Self = Self(0)` — `crates/contextra-ids/src/lib.rs:556`

#### `struct StoreId`
- Ort: `crates/contextra-ids/src/lib.rs:524`
```rust
pub struct StoreId(pub [u8
```
Dokumentation: 16-Byte Speicherinstanz-Kennung (SPEC F.1).
- Impl-Block `crates/contextra-ids/src/lib.rs:526`
- Impl-Block (Trait `fmt::Debug`) `crates/contextra-ids/src/lib.rs:538`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub const fn new(bytes: [u8` | `crates/contextra-ids/src/lib.rs:528` | Erzeugt eine `StoreId` aus rohen 16 Bytes. |
| `pub const fn as_bytes(&self) -> &[u8` | `crates/contextra-ids/src/lib.rs:533` | Liefert die 16 Bytes als Slice. |

#### `struct TenantId`
- Ort: `crates/contextra-ids/src/lib.rs:79`
```rust
pub struct TenantId(pub u64)
```
Dokumentation: Mandanten-ID.
- Impl-Block `crates/contextra-ids/src/lib.rs:81`
- Impl-Block (Trait `Default`) `crates/contextra-ids/src/lib.rs:124`
- Impl-Block (Trait `TryFrom<u64>`) `crates/contextra-ids/src/lib.rs:130`
- Impl-Block (Trait `fmt::Display`) `crates/contextra-ids/src/lib.rs:139`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub const fn new(id: u64) -> Self` | `crates/contextra-ids/src/lib.rs:92` | Creates a new `TenantId` wrapping the provided `u64` identifier. |
| `pub fn try_new(id: u64) -> Result<Self>` | `crates/contextra-ids/src/lib.rs:97` | Sicherer Konstruktor. Gibt Err wenn id == 0. |
| `pub const fn inner(self) -> u64` | `crates/contextra-ids/src/lib.rs:107` | Returns the inner raw `u64` identifier. |
| `pub const fn as_u64(self) -> u64` | `crates/contextra-ids/src/lib.rs:113` | Returns the inner raw `u64` identifier. |
| `pub const fn is_system(self) -> bool` | `crates/contextra-ids/src/lib.rs:119` | Returns `true` if this tenant is the SYSTEM tenant (`TenantId(0)`). |

Assoziierte Konstanten/Typen:

- `pub const SYSTEM: Self = Self(0)` — `crates/contextra-ids/src/lib.rs:83`
- `pub const DEFAULT: Self = Self(0)` — `crates/contextra-ids/src/lib.rs:86`
- `pub const INVALID: Self = Self(0)` — `crates/contextra-ids/src/lib.rs:88`

#### `struct TenantScoped`
- Ort: `crates/contextra-ids/src/lib.rs:674`
```rust
pub struct TenantScoped<T>
```
Dokumentation: Ein Wrapper-Typ, der einen Wert untrennbar an eine `TenantId` bindet.
- Impl-Block `crates/contextra-ids/src/lib.rs:679`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new(tenant_id: TenantId, value: T) -> Self` | `crates/contextra-ids/src/lib.rs:682` | Creates a new `TenantScoped` value bound to the specified `TenantId`. |
| `pub fn tenant_id(&self) -> &TenantId` | `crates/contextra-ids/src/lib.rs:688` | Returns a reference to the bound `TenantId`. |
| `pub fn value(&self) -> &T` | `crates/contextra-ids/src/lib.rs:694` | Referenz auf den inneren Wert. |
| `pub fn into_inner_checked( self, expected: &TenantId, ) -> core::result::Result<T, TenantScopeViolation>` | `crates/contextra-ids/src/lib.rs:699` | Gibt den Wert nur zurück, wenn die aufrufende Seite die erwartete `TenantId` nennt. |
| `pub fn map<U>(self, f: impl FnOnce(T) -> U) -> TenantScoped<U>` | `crates/contextra-ids/src/lib.rs:714` | Transforms the inner value `T` into `U` using `f` while preserving the bound `TenantId`. |

#### `struct Timestamp`
- Ort: `crates/contextra-ids/src/lib.rs:578`
```rust
pub struct Timestamp(pub u64)
```
- Impl-Block `crates/contextra-ids/src/lib.rs:580`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub const fn from_micros(micros: u64) -> Self` | `crates/contextra-ids/src/lib.rs:585` | Erzeugt einen neuen Zeitstempel aus Mikrosekunden seit Unix-Epoche. |
| `pub const fn as_micros(&self) -> u64` | `crates/contextra-ids/src/lib.rs:590` | Liefert den Mikrosekunden-Wert. |

Assoziierte Konstanten/Typen:

- `pub const ZERO: Self = Self(0)` — `crates/contextra-ids/src/lib.rs:582`

#### `struct TxId`
- Ort: `crates/contextra-ids/src/lib.rs:407`
```rust
pub struct TxId(pub u64)
```
Dokumentation: Transaktions-Identifikator.
- Impl-Block `crates/contextra-ids/src/lib.rs:409`
- Impl-Block (Trait `Default`) `crates/contextra-ids/src/lib.rs:475`
- Impl-Block (Trait `From<u64>`) `crates/contextra-ids/src/lib.rs:481`
- Impl-Block (Trait `fmt::Display`) `crates/contextra-ids/src/lib.rs:488`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub const fn new(id: u64) -> Self` | `crates/contextra-ids/src/lib.rs:421` | Creates a new `TxId` wrapping the provided `u64` identifier. |
| `pub const fn inner(self) -> u64` | `crates/contextra-ids/src/lib.rs:427` | Returns the inner raw `u64` identifier. |
| `pub const fn as_u64(self) -> u64` | `crates/contextra-ids/src/lib.rs:434` | Deprecated backwards compatibility alias for inner. |
| `pub const fn internal() -> Self` | `crates/contextra-ids/src/lib.rs:440` | Returns a new internal/system transaction ID. |
| `pub fn try_from_internal_offset(offset: u64) -> Result<Self>` | `crates/contextra-ids/src/lib.rs:445` | Safely constructs an internal system TxId from an offset relative to `TxId::INTERNAL_BASE`. |
| `pub fn is_valid_origin(&self) -> bool` | `crates/contextra-ids/src/lib.rs:454` | Checks if the transaction ID originates from a valid range. |
| `pub const fn is_internal(self) -> bool` | `crates/contextra-ids/src/lib.rs:460` | Returns `true` if this `TxId` was allocated by the internal system range. |
| `pub const fn is_valid(self) -> bool` | `crates/contextra-ids/src/lib.rs:466` | Returns `true` if this `TxId` represents a valid initialized transaction (`id > 0`). |

Assoziierte Konstanten/Typen:

- `pub const INVALID: Self = Self(0)` — `crates/contextra-ids/src/lib.rs:411`
- `pub const MAX_COLLECTION_SEQUENCE: u64 = 1_000_000_000_000` — `crates/contextra-ids/src/lib.rs:414`
- `pub const INTERNAL_BASE: u64 = u64::MAX - 1_000_000` — `crates/contextra-ids/src/lib.rs:417`

#### `enum IdError`
- Ort: `crates/contextra-ids/src/lib.rs:42`
```rust
pub enum IdError
```
Dokumentation: Specific error type for identifier, reference, and tenant scope operations.

Varianten:

- `EmptyKey` — `crates/contextra-ids/src/lib.rs:45`
- `EmptyIdentifier` — `crates/contextra-ids/src/lib.rs:48`
- `IdentifierTooLong` — `crates/contextra-ids/src/lib.rs:51`
- `TenantSystemReserved` — `crates/contextra-ids/src/lib.rs:54`
- `CollectionZero` — `crates/contextra-ids/src/lib.rs:57`
- `TenantScopeViolation{` — `crates/contextra-ids/src/lib.rs:60`
- `TxIdAllocationOverflow(u64)` — `crates/contextra-ids/src/lib.rs:68`

#### `enum TenantScopeViolation`
- Ort: `crates/contextra-ids/src/lib.rs:660`
```rust
pub enum TenantScopeViolation
```
Dokumentation: Errors raised when attempting an illegal or mismatched tenant scope unpacking operation.

Varianten:

- `Mismatch{` — `crates/contextra-ids/src/lib.rs:664`

#### `type Result`
- Ort: `crates/contextra-ids/src/lib.rs:72`
```rust
pub type Result<T> = core::result::Result<T, IdError>
```
Dokumentation: Result alias for identifier operations.

#### `const ID_WIDTH_BITS`
- Ort: `crates/contextra-ids/src/lib.rs:15`
```rust
pub const ID_WIDTH_BITS: u32 = if cfg!(feature = "")
```
Dokumentation: Bit width of `DocId` and associated identifiers in the current build.

#### `const MAX_REF_LEN`
- Ort: `crates/contextra-ids/src/lib.rs:37`
```rust
pub const MAX_REF_LEN: usize = 256
```
Dokumentation: Maximale Bytelänge für Referenz-Strings / Byte-Identifier.

#### `const MAX_SEARCH_K`
- Ort: `crates/contextra-ids/src/lib.rs:34`
```rust
pub const MAX_SEARCH_K: usize = 1_000
```
Dokumentation: Maximale Anzahl an Suchergebnissen (Top-K Begrenzung).

#### `const TOMBSTONE_BIT`
- Ort: `crates/contextra-ids/src/lib.rs:31`
```rust
pub const TOMBSTONE_BIT: u64 = 1 << 63
```
Dokumentation: Bit 63 einer Sequenznummer kennzeichnet einen Tombstone (Soft-Deletion).
