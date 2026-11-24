# Schnittstellenspezifikation: `contextra-storage-ports`

> Erzeugt durch `tools/extract_interfaces.py` aus dem Quellcode. Keine Handpflege. Jede Angabe trägt Datei:Zeile. Maßgeblich bleibt der Quellcode.

## 1. Kopf

| Feld | Wert |
|---|---|
| Name | `contextra-storage-ports` |
| Version | workspace |
| Pfad | `crates/contextra-storage/crates/contextra-storage-ports` |
| Rolle | Speicherkern: normative Ports |
| Bibliothekspfad | src/lib.rs (Standard) |

**Crate-Dokumentation (//!):**

> contextra-storage-ports: Verträge, Werttypen, Fehler, Limits. Keine Logik (SP-PORTS).

## 2. Abhängigkeiten

| Name | Art | Quelle | optional | verwendet von (normal/dev) |
|---|---|---|---|---|
| `bytes` | normal | registry | nein |  |
| `thiserror` | normal | registry | nein |  |
| `zeroize` | normal | registry | nein |  |

**Verwendet von (Crate-Ebene):** `contextra-crypto-ref`, `contextra-keys-cohort`, `contextra-keys-file`, `contextra-storage`, `contextra-storage-bridge`, `contextra-storage-fs`

## 3. Features

| Feature | Aktiviert |
|---|---|
| `testing` | – |

## 4. Metriken (Quellcode)

| Kennzahl | Wert |
|---|---|
| Quelldateien | 17 |
| Quellzeilen | 1111 |
| Öffentliche Items (Modul-Ebene) | 108 |
| davon `struct` | 25 |
| davon `enum` | 25 |
| davon `trait` | 19 |
| davon `type` | 1 |
| davon `const` | 22 |
| davon `mod` | 16 |
| Öffentliche Makros (`#[macro_export]`) | 0 |
| Öffentliche Re-Exports (`pub use`) | 10 |
| `unsafe`-Vorkommen (Produktionscode) | 0 |
| `.unwrap()` (Produktionscode) | 0 |
| `.expect(` (Produktionscode) | 0 |
| `panic!` (Produktionscode) | 0 |
| `todo!/unimplemented!/unreachable!` | 0 |
| Indexierungen `x[…]` (grob, Produktionscode) | 0 |
| `#[deprecated]` | 0 |
| `#[cfg(feature…)]`-Gates | 0 |
| Parser-Abdeckung (geparste `pub`-Deklarationen / Grep-Treffer) | 137 / 137 = 100 % |

## 5. Modulbaum und Dateien

| Modul | Datei | Zeilen |
|---|---|---|
| `change` | `crates/contextra-storage/crates/contextra-storage-ports/src/change.rs` | 104 |
| `error` | `crates/contextra-storage/crates/contextra-storage-ports/src/error.rs` | 155 |
| `facade` | `crates/contextra-storage/crates/contextra-storage-ports/src/facade.rs` | 4 |
| `ids` | `crates/contextra-storage/crates/contextra-storage-ports/src/ids.rs` | 145 |
| `keys` | `crates/contextra-storage/crates/contextra-storage-ports/src/keys.rs` | 75 |
| `crate` | `crates/contextra-storage/crates/contextra-storage-ports/src/lib.rs` | 23 |
| `limits` | `crates/contextra-storage/crates/contextra-storage-ports/src/limits.rs` | 29 |
| `traits::anchor` | `crates/contextra-storage/crates/contextra-storage-ports/src/traits/anchor.rs` | 72 |
| `traits::authority` | `crates/contextra-storage/crates/contextra-storage-ports/src/traits/authority.rs` | 48 |
| `traits::cold` | `crates/contextra-storage/crates/contextra-storage-ports/src/traits/cold.rs` | 7 |
| `traits::crypto` | `crates/contextra-storage/crates/contextra-storage-ports/src/traits/crypto.rs` | 37 |
| `traits::engine` | `crates/contextra-storage/crates/contextra-storage-ports/src/traits/engine.rs` | 10 |
| `traits::fs` | `crates/contextra-storage/crates/contextra-storage-ports/src/traits/fs.rs` | 93 |
| `traits::memory` | `crates/contextra-storage/crates/contextra-storage-ports/src/traits/memory.rs` | 48 |
| `traits` | `crates/contextra-storage/crates/contextra-storage-ports/src/traits/mod.rs` | 20 |
| `traits::observe` | `crates/contextra-storage/crates/contextra-storage-ports/src/traits/observe.rs` | 235 |
| `traits::spawn` | `crates/contextra-storage/crates/contextra-storage-ports/src/traits/spawn.rs` | 6 |

## 6. Re-Exports (`pub use`)

- `pub use change::` — `crates/contextra-storage/crates/contextra-storage-ports/src/lib.rs:12` (Modul `crate`)
- `pub use error::` — `crates/contextra-storage/crates/contextra-storage-ports/src/lib.rs:15` (Modul `crate`)
- `pub use ids::` — `crates/contextra-storage/crates/contextra-storage-ports/src/lib.rs:16` (Modul `crate`)
- `pub use keys::` — `crates/contextra-storage/crates/contextra-storage-ports/src/lib.rs:21` (Modul `crate`)
- `pub use anchor::` — `crates/contextra-storage/crates/contextra-storage-ports/src/traits/mod.rs:12` (Modul `traits`)
- `pub use authority::` — `crates/contextra-storage/crates/contextra-storage-ports/src/traits/mod.rs:13` (Modul `traits`)
- `pub use crypto::` — `crates/contextra-storage/crates/contextra-storage-ports/src/traits/mod.rs:14` (Modul `traits`)
- `pub use fs::` — `crates/contextra-storage/crates/contextra-storage-ports/src/traits/mod.rs:15` (Modul `traits`)
- `pub use memory::` — `crates/contextra-storage/crates/contextra-storage-ports/src/traits/mod.rs:16` (Modul `traits`)
- `pub use observe::` — `crates/contextra-storage/crates/contextra-storage-ports/src/traits/mod.rs:17` (Modul `traits`)

## 8. Öffentliche Schnittstelle je Item

### Modul `change`

#### `struct CallerMeta`
- Ort: `crates/contextra-storage/crates/contextra-storage-ports/src/change.rs:34`
```rust
pub struct CallerMeta
```

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `origin` | `Origin` | `crates/contextra-storage/crates/contextra-storage-ports/src/change.rs:35` |
| `caller_seq` | `u64` | `crates/contextra-storage/crates/contextra-storage-ports/src/change.rs:36` |

#### `struct Change`
- Ort: `crates/contextra-storage/crates/contextra-storage-ports/src/change.rs:39`
```rust
pub struct Change
```

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `ops` | `Vec<Op>` | `crates/contextra-storage/crates/contextra-storage-ports/src/change.rs:40` |
| `guards` | `Vec<Guard>` | `crates/contextra-storage/crates/contextra-storage-ports/src/change.rs:41` |
| `idem` | `Option<IdemToken>` | `crates/contextra-storage/crates/contextra-storage-ports/src/change.rs:42` |
| `durability` | `Durability` | `crates/contextra-storage/crates/contextra-storage-ports/src/change.rs:43` |
| `allow_mixed` | `bool` | `crates/contextra-storage/crates/contextra-storage-ports/src/change.rs:44` |
| `caller` | `CallerMeta` | `crates/contextra-storage/crates/contextra-storage-ports/src/change.rs:45` |

#### `struct Receipt`
- Ort: `crates/contextra-storage/crates/contextra-storage-ports/src/change.rs:101`
```rust
pub struct Receipt
```

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `stamp` | `Stamp` | `crates/contextra-storage/crates/contextra-storage-ports/src/change.rs:102` |
| `durable` | `bool` | `crates/contextra-storage/crates/contextra-storage-ports/src/change.rs:103` |

#### `enum ClusterLifecycle`
- Ort: `crates/contextra-storage/crates/contextra-storage-ports/src/change.rs:12`
```rust
pub enum ClusterLifecycle
```

Varianten:

- `Live` — `crates/contextra-storage/crates/contextra-storage-ports/src/change.rs:13`
- `Retired{ at: crate::ids::Stamp }` — `crates/contextra-storage/crates/contextra-storage-ports/src/change.rs:14`
- `Shredded{ at: crate::ids::Stamp }` — `crates/contextra-storage/crates/contextra-storage-ports/src/change.rs:15`
- `Purged{ at: crate::ids::Stamp }` — `crates/contextra-storage/crates/contextra-storage-ports/src/change.rs:16`

#### `enum Durability`
- Ort: `crates/contextra-storage/crates/contextra-storage-ports/src/change.rs:20`
```rust
pub enum Durability
```

Varianten:

- `Durable` — `crates/contextra-storage/crates/contextra-storage-ports/src/change.rs:21`
- `Relaxed` — `crates/contextra-storage/crates/contextra-storage-ports/src/change.rs:22`

#### `enum Guard`
- Ort: `crates/contextra-storage/crates/contextra-storage-ports/src/change.rs:74`
```rust
pub enum Guard
```

Varianten:

- `Key{` — `crates/contextra-storage/crates/contextra-storage-ports/src/change.rs:75`
- `Range{` — `crates/contextra-storage/crates/contextra-storage-ports/src/change.rs:83`

#### `enum Op`
- Ort: `crates/contextra-storage/crates/contextra-storage-ports/src/change.rs:48`
```rust
pub enum Op
```

Varianten:

- `Put{` — `crates/contextra-storage/crates/contextra-storage-ports/src/change.rs:49`
- `Delete{` — `crates/contextra-storage/crates/contextra-storage-ports/src/change.rs:58`
- `RangeDelete{` — `crates/contextra-storage/crates/contextra-storage-ports/src/change.rs:65`

#### `enum Origin`
- Ort: `crates/contextra-storage/crates/contextra-storage-ports/src/change.rs:26`
```rust
pub enum Origin
```

Varianten:

- `User` — `crates/contextra-storage/crates/contextra-storage-ports/src/change.rs:27`
- `Maintenance` — `crates/contextra-storage/crates/contextra-storage-ports/src/change.rs:28`
- `Replication` — `crates/contextra-storage/crates/contextra-storage-ports/src/change.rs:29`
- `Migration` — `crates/contextra-storage/crates/contextra-storage-ports/src/change.rs:30`

#### `enum Precondition`
- Ort: `crates/contextra-storage/crates/contextra-storage-ports/src/change.rs:94`
```rust
pub enum Precondition
```

Varianten:

- `Absent` — `crates/contextra-storage/crates/contextra-storage-ports/src/change.rs:95`
- `Version(Stamp)` — `crates/contextra-storage/crates/contextra-storage-ports/src/change.rs:96`
- `Unchanged{ since: Stamp }` — `crates/contextra-storage/crates/contextra-storage-ports/src/change.rs:97`

#### `enum Protection`
- Ort: `crates/contextra-storage/crates/contextra-storage-ports/src/change.rs:6`
```rust
pub enum Protection
```

Varianten:

- `Sealed` — `crates/contextra-storage/crates/contextra-storage-ports/src/change.rs:7`
- `Plain` — `crates/contextra-storage/crates/contextra-storage-ports/src/change.rs:8`

### Modul `crate`

#### `mod change`
- Ort: `crates/contextra-storage/crates/contextra-storage-ports/src/lib.rs:4`
```rust
pub mod change
```

#### `mod error`
- Ort: `crates/contextra-storage/crates/contextra-storage-ports/src/lib.rs:5`
```rust
pub mod error
```

#### `mod facade`
- Ort: `crates/contextra-storage/crates/contextra-storage-ports/src/lib.rs:6`
```rust
pub mod facade
```

#### `mod ids`
- Ort: `crates/contextra-storage/crates/contextra-storage-ports/src/lib.rs:7`
```rust
pub mod ids
```

#### `mod keys`
- Ort: `crates/contextra-storage/crates/contextra-storage-ports/src/lib.rs:8`
```rust
pub mod keys
```

#### `mod limits`
- Ort: `crates/contextra-storage/crates/contextra-storage-ports/src/lib.rs:9`
```rust
pub mod limits
```

#### `mod traits`
- Ort: `crates/contextra-storage/crates/contextra-storage-ports/src/lib.rs:10`
```rust
pub mod traits
```

### Modul `error`

#### `enum Error`
- Ort: `crates/contextra-storage/crates/contextra-storage-ports/src/error.rs:34`
```rust
pub enum Error
```
- Impl-Block `crates/contextra-storage/crates/contextra-storage-ports/src/error.rs:100`

Varianten:

- `Transient{ role: &'static str }` — `crates/contextra-storage/crates/contextra-storage-ports/src/error.rs:36`
- `DiskFull` — `crates/contextra-storage/crates/contextra-storage-ports/src/error.rs:38`
- `Poisoned{ cause: &'static str }` — `crates/contextra-storage/crates/contextra-storage-ports/src/error.rs:40`
- `Indeterminate{ first_stamp: u64, last_stamp: u64 }` — `crates/contextra-storage/crates/contextra-storage-ports/src/error.rs:42`
- `Corrupt{ place: &'static str }` — `crates/contextra-storage/crates/contextra-storage-ports/src/error.rs:44`
- `Unsupported{ what: &'static str }` — `crates/contextra-storage/crates/contextra-storage-ports/src/error.rs:46`
- `KeyUnavailable{ retry_after_secs: Option<u64> }` — `crates/contextra-storage/crates/contextra-storage-ports/src/error.rs:48`
- `KeyLost` — `crates/contextra-storage/crates/contextra-storage-ports/src/error.rs:50`
- `Shredded` — `crates/contextra-storage/crates/contextra-storage-ports/src/error.rs:52`
- `ClusterRetired` — `crates/contextra-storage/crates/contextra-storage-ports/src/error.rs:54`
- `UnknownCluster` — `crates/contextra-storage/crates/contextra-storage-ports/src/error.rs:56`
- `BelowFloor` — `crates/contextra-storage/crates/contextra-storage-ports/src/error.rs:58`
- `Erased{ at: u64 }` — `crates/contextra-storage/crates/contextra-storage-ports/src/error.rs:60`
- `Restricted` — `crates/contextra-storage/crates/contextra-storage-ports/src/error.rs:62`
- `Conflict` — `crates/contextra-storage/crates/contextra-storage-ports/src/error.rs:64`
- `GuardRetry` — `crates/contextra-storage/crates/contextra-storage-ports/src/error.rs:66`
- `ChangeTooLarge` — `crates/contextra-storage/crates/contextra-storage-ports/src/error.rs:68`
- `IdemConflict` — `crates/contextra-storage/crates/contextra-storage-ports/src/error.rs:70`
- `IdemCapacity` — `crates/contextra-storage/crates/contextra-storage-ports/src/error.rs:72`
- `BranchClosed` — `crates/contextra-storage/crates/contextra-storage-ports/src/error.rs:74`
- `BranchHasChildren` — `crates/contextra-storage/crates/contextra-storage-ports/src/error.rs:76`
- `Lagged` — `crates/contextra-storage/crates/contextra-storage-ports/src/error.rs:78`
- `EraseFanoutExceeded` — `crates/contextra-storage/crates/contextra-storage-ports/src/error.rs:80`
- `ClockUncertain` — `crates/contextra-storage/crates/contextra-storage-ports/src/error.rs:82`
- `ReservedSpace` — `crates/contextra-storage/crates/contextra-storage-ports/src/error.rs:84`
- `Exhausted` — `crates/contextra-storage/crates/contextra-storage-ports/src/error.rs:86`
- `Limit{ limit: &'static str }` — `crates/contextra-storage/crates/contextra-storage-ports/src/error.rs:88`
- `CasLost` — `crates/contextra-storage/crates/contextra-storage-ports/src/error.rs:90`
- `Rollback` — `crates/contextra-storage/crates/contextra-storage-ports/src/error.rs:92`
- `Io{` — `crates/contextra-storage/crates/contextra-storage-ports/src/error.rs:94`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn class(&self) -> ErrorClass` | `crates/contextra-storage/crates/contextra-storage-ports/src/error.rs:101` |  |
| `pub fn hint(&self) -> &'static str` | `crates/contextra-storage/crates/contextra-storage-ports/src/error.rs:122` |  |

#### `enum ErrorClass`
- Ort: `crates/contextra-storage/crates/contextra-storage-ports/src/error.rs:7`
```rust
pub enum ErrorClass
```

Varianten:

- `Transient` — `crates/contextra-storage/crates/contextra-storage-ports/src/error.rs:9`
- `DiskFull` — `crates/contextra-storage/crates/contextra-storage-ports/src/error.rs:11`
- `Poisoned` — `crates/contextra-storage/crates/contextra-storage-ports/src/error.rs:13`
- `Indeterminate` — `crates/contextra-storage/crates/contextra-storage-ports/src/error.rs:15`
- `Corrupt` — `crates/contextra-storage/crates/contextra-storage-ports/src/error.rs:17`
- `Unsupported` — `crates/contextra-storage/crates/contextra-storage-ports/src/error.rs:19`
- `Key` — `crates/contextra-storage/crates/contextra-storage-ports/src/error.rs:21`
- `Usage` — `crates/contextra-storage/crates/contextra-storage-ports/src/error.rs:23`
- `Conflict` — `crates/contextra-storage/crates/contextra-storage-ports/src/error.rs:25`
- `Rollback` — `crates/contextra-storage/crates/contextra-storage-ports/src/error.rs:27`
- `Internal` — `crates/contextra-storage/crates/contextra-storage-ports/src/error.rs:29`

### Modul `ids`

#### `struct BranchId`
- Ort: `crates/contextra-storage/crates/contextra-storage-ports/src/ids.rs:90`
```rust
pub struct BranchId(pub u32)
```

#### `struct ClusterId`
- Ort: `crates/contextra-storage/crates/contextra-storage-ports/src/ids.rs:21`
```rust
pub struct ClusterId([u8
```
- Impl-Block `crates/contextra-storage/crates/contextra-storage-ports/src/ids.rs:23`
- Impl-Block (Trait `fmt::Debug`) `crates/contextra-storage/crates/contextra-storage-ports/src/ids.rs:34`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn from_wire(raw: [u8` | `crates/contextra-storage/crates/contextra-storage-ports/src/ids.rs:26` | Nur für den Wire-Decoder (SP-CODEC). Weitere Konstruktionspfade existieren nicht. |
| `pub fn as_bytes(&self) -> &[u8` | `crates/contextra-storage/crates/contextra-storage-ports/src/ids.rs:29` |  |

#### `struct DurablePosition`
- Ort: `crates/contextra-storage/crates/contextra-storage-ports/src/ids.rs:84`
```rust
pub struct DurablePosition
```
Dokumentation: A durable progress point in one instance's stamp domain.

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `instance` | `InstanceId` | `crates/contextra-storage/crates/contextra-storage-ports/src/ids.rs:85` |
| `stamp` | `Stamp` | `crates/contextra-storage/crates/contextra-storage-ports/src/ids.rs:86` |

#### `struct EraseRequestId`
- Ort: `crates/contextra-storage/crates/contextra-storage-ports/src/ids.rs:103`
```rust
pub struct EraseRequestId([u8
```
Dokumentation: Separate idempotency domain for erase requests.
- Impl-Block `crates/contextra-storage/crates/contextra-storage-ports/src/ids.rs:105`
- Impl-Block (Trait `fmt::Debug`) `crates/contextra-storage/crates/contextra-storage-ports/src/ids.rs:109`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn from_bytes(raw: [u8` | `crates/contextra-storage/crates/contextra-storage-ports/src/ids.rs:106` |  |

#### `struct IdemToken`
- Ort: `crates/contextra-storage/crates/contextra-storage-ports/src/ids.rs:93`
```rust
pub struct IdemToken([u8
```
- Impl-Block `crates/contextra-storage/crates/contextra-storage-ports/src/ids.rs:95`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn from_bytes(raw: [u8` | `crates/contextra-storage/crates/contextra-storage-ports/src/ids.rs:96` |  |

#### `struct InstanceId`
- Ort: `crates/contextra-storage/crates/contextra-storage-ports/src/ids.rs:66`
```rust
pub struct InstanceId([u8
```
Dokumentation: Stable identity of one storage instance.  A stamp is meaningful only with this identity and must never be treated as a globally ordered value.
- Impl-Block `crates/contextra-storage/crates/contextra-storage-ports/src/ids.rs:68`
- Impl-Block (Trait `fmt::Debug`) `crates/contextra-storage/crates/contextra-storage-ports/src/ids.rs:76`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn from_wire(raw: [u8` | `crates/contextra-storage/crates/contextra-storage-ports/src/ids.rs:71` | Used by format and recovery codecs after validating the exact width. |
| `pub fn as_bytes(&self) -> &[u8` | `crates/contextra-storage/crates/contextra-storage-ports/src/ids.rs:73` |  |

#### `struct Redacted`
- Ort: `crates/contextra-storage/crates/contextra-storage-ports/src/ids.rs:12`
```rust
pub struct Redacted<T>(pub T)
```
Dokumentation: Umhüllt einen Wert, dessen Inhalt nie in Debug-Ausgaben erscheinen darf.
- Impl-Block (Trait `fmt::Debug`) `crates/contextra-storage/crates/contextra-storage-ports/src/ids.rs:14`

#### `struct SpaceId`
- Ort: `crates/contextra-storage/crates/contextra-storage-ports/src/ids.rs:41`
```rust
pub struct SpaceId(pub u64)
```
- Impl-Block `crates/contextra-storage/crates/contextra-storage-ports/src/ids.rs:43`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub const fn is_system(self) -> bool` | `crates/contextra-storage/crates/contextra-storage-ports/src/ids.rs:44` |  |
| `pub fn user(value: u64) -> Result<Self, crate::Error>` | `crates/contextra-storage/crates/contextra-storage-ports/src/ids.rs:46` |  |

#### `struct Ticket`
- Ort: `crates/contextra-storage/crates/contextra-storage-ports/src/ids.rs:117`
```rust
pub struct Ticket([u8
```
Dokumentation: Opaque durable erase-ticket identity.
- Impl-Block `crates/contextra-storage/crates/contextra-storage-ports/src/ids.rs:119`
- Impl-Block (Trait `fmt::Debug`) `crates/contextra-storage/crates/contextra-storage-ports/src/ids.rs:125`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn from_bytes(raw: [u8` | `crates/contextra-storage/crates/contextra-storage-ports/src/ids.rs:120` |  |
| `pub fn as_bytes(&self) -> &[u8` | `crates/contextra-storage/crates/contextra-storage-ports/src/ids.rs:122` | Canonical wire representation for durable internal records. |

#### `struct TombId`
- Ort: `crates/contextra-storage/crates/contextra-storage-ports/src/ids.rs:133`
```rust
pub struct TombId([u8
```
Dokumentation: Pseudonymous, domain-separated ledger identifier; never a clear ClusterId.
- Impl-Block `crates/contextra-storage/crates/contextra-storage-ports/src/ids.rs:135`
- Impl-Block (Trait `fmt::Debug`) `crates/contextra-storage/crates/contextra-storage-ports/src/ids.rs:141`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn from_bytes(raw: [u8` | `crates/contextra-storage/crates/contextra-storage-ports/src/ids.rs:136` |  |
| `pub fn as_bytes(&self) -> &[u8` | `crates/contextra-storage/crates/contextra-storage-ports/src/ids.rs:138` | Canonical wire representation for the ledger only. |

#### `enum DataClass`
- Ort: `crates/contextra-storage/crates/contextra-storage-ports/src/ids.rs:58`
```rust
pub enum DataClass
```
Dokumentation: Explicit classification at the storage API boundary (SPEC §4.7). This is deliberately a closed enum.  Storage validates the combinations involving this value before it starts catalog, key, or filesystem I/O; it does not inspect user content to infer a classification.

Varianten:

- `NonPersonal` — `crates/contextra-storage/crates/contextra-storage-ports/src/ids.rs:59`
- `Personal` — `crates/contextra-storage/crates/contextra-storage-ports/src/ids.rs:60`

#### `type Stamp`
- Ort: `crates/contextra-storage/crates/contextra-storage-ports/src/ids.rs:8`
```rust
pub type Stamp = u64
```
Dokumentation: Stamp = globale Versionsmarke, 56 Bit nutzbar.

#### `const FIRST_USER_SPACE_ID`
- Ort: `crates/contextra-storage/crates/contextra-storage-ports/src/ids.rs:5`
```rust
pub const FIRST_USER_SPACE_ID: u64 = SYSTEM_SPACE_ID_MAX + 1
```

#### `const STAMP_MAX`
- Ort: `crates/contextra-storage/crates/contextra-storage-ports/src/ids.rs:9`
```rust
pub const STAMP_MAX: Stamp = (1u64 << 56) - 1
```

#### `const SYSTEM_SPACE_ID_MAX`
- Ort: `crates/contextra-storage/crates/contextra-storage-ports/src/ids.rs:4`
```rust
pub const SYSTEM_SPACE_ID_MAX: u64 = 255
```

### Modul `keys`

#### `struct DestroyReceipt`
- Ort: `crates/contextra-storage/crates/contextra-storage-ports/src/keys.rs:37`
```rust
pub struct DestroyReceipt
```
Dokumentation: Aggregate result of an idempotent destroy request. No key or cluster IDs are included in this value.

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `requested_clusters` | `usize` | `crates/contextra-storage/crates/contextra-storage-ports/src/keys.rs:38` |
| `absent_clusters` | `usize` | `crates/contextra-storage/crates/contextra-storage-ports/src/keys.rs:39` |

#### `struct KekId`
- Ort: `crates/contextra-storage/crates/contextra-storage-ports/src/keys.rs:9`
```rust
pub struct KekId([u8
```
Dokumentation: Opaque identifier for a provider-managed key-encryption key.
- Impl-Block `crates/contextra-storage/crates/contextra-storage-ports/src/keys.rs:11`
- Impl-Block (Trait `fmt::Debug`) `crates/contextra-storage/crates/contextra-storage-ports/src/keys.rs:15`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub const fn from_bytes(raw: [u8` | `crates/contextra-storage/crates/contextra-storage-ports/src/keys.rs:12` |  |

#### `struct Key32`
- Ort: `crates/contextra-storage/crates/contextra-storage-ports/src/keys.rs:43`
```rust
pub struct Key32(Zeroizing<[u8
```
Dokumentation: 32-Byte-Schlüssel. Zeroized bei Drop, Debug redigiert, nicht Clone.
- Impl-Block `crates/contextra-storage/crates/contextra-storage-ports/src/keys.rs:45`
- Impl-Block (Trait `fmt::Debug`) `crates/contextra-storage/crates/contextra-storage-ports/src/keys.rs:54`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn from_array(raw: [u8` | `crates/contextra-storage/crates/contextra-storage-ports/src/keys.rs:46` |  |
| `pub fn expose(&self) -> &[u8` | `crates/contextra-storage/crates/contextra-storage-ports/src/keys.rs:49` |  |

#### `struct KeyProviderInfo`
- Ort: `crates/contextra-storage/crates/contextra-storage-ports/src/keys.rs:26`
```rust
pub struct KeyProviderInfo
```

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `destroy` | `DestroyBehavior` | `crates/contextra-storage/crates/contextra-storage-ports/src/keys.rs:27` |
| `keys_in_backups` | `bool` | `crates/contextra-storage/crates/contextra-storage-ports/src/keys.rs:28` |

#### `enum DestroyBehavior`
- Ort: `crates/contextra-storage/crates/contextra-storage-ports/src/keys.rs:20`
```rust
pub enum DestroyBehavior
```

Varianten:

- `Immediate` — `crates/contextra-storage/crates/contextra-storage-ports/src/keys.rs:21`
- `Scheduled{ max_delay_secs: u64 }` — `crates/contextra-storage/crates/contextra-storage-ports/src/keys.rs:22`

#### `enum KeyState`
- Ort: `crates/contextra-storage/crates/contextra-storage-ports/src/keys.rs:32`
```rust
pub enum KeyState
```

#### `trait KeyProvider`
- Ort: `crates/contextra-storage/crates/contextra-storage-ports/src/keys.rs:60`
```rust
pub trait KeyProvider: Send + Sync
```

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `fn info(&self) -> KeyProviderInfo` | `crates/contextra-storage/crates/contextra-storage-ports/src/keys.rs:62` | Describes key destruction timing and backup coverage. |
| `fn kek(&self, id: KekId) -> Result<Key32, Error>` | `crates/contextra-storage/crates/contextra-storage-ports/src/keys.rs:64` | Returns an available KEK by opaque provider identifier. |
| `fn create_cluster_key(&self, cluster: ClusterId, entropy: &dyn crate::traits::Entropy) -> Result<u32, Error>` | `crates/contextra-storage/crates/contextra-storage-ports/src/keys.rs:66` | Creates or returns the current durable key epoch for a cluster. |
| `fn cluster_key(&self, cluster: ClusterId, epoch: u32) -> Result<Option<Key32>, Error>` | `crates/contextra-storage/crates/contextra-storage-ports/src/keys.rs:69` | Fetches one cluster epoch. `None` means the provider has no usable key; recovery must distinguish legitimate destruction from key loss. |
| `fn probe(&self, cluster: ClusterId) -> Result<KeyState, Error>` | `crates/contextra-storage/crates/contextra-storage-ports/src/keys.rs:71` | Reports provider state across every epoch for a cluster. |
| `fn list(&self) -> Result<Vec<ClusterId>, Error>` | `crates/contextra-storage/crates/contextra-storage-ports/src/keys.rs:73` | Lists provider-owned cluster identifiers for orphan reconciliation. |
| `fn destroy_many(&self, clusters: &[ClusterId]) -> Result<DestroyReceipt, Error>` | `crates/contextra-storage/crates/contextra-storage-ports/src/keys.rs:74` |  |

### Modul `limits`

#### `const CATALOG_ROLLOVER_MIN_BYTES`
- Ort: `crates/contextra-storage/crates/contextra-storage-ports/src/limits.rs:19`
```rust
pub const CATALOG_ROLLOVER_MIN_BYTES: u64 = 8 * MIB as u64
```

#### `const COHORT_MAX`
- Ort: `crates/contextra-storage/crates/contextra-storage-ports/src/limits.rs:26`
```rust
pub const COHORT_MAX: usize = 1024
```

#### `const DEFAULT_MAX_CLUSTERS`
- Ort: `crates/contextra-storage/crates/contextra-storage-ports/src/limits.rs:23`
```rust
pub const DEFAULT_MAX_CLUSTERS: u64 = 4_000_000
```

#### `const DEFAULT_MAX_OPEN_FILES`
- Ort: `crates/contextra-storage/crates/contextra-storage-ports/src/limits.rs:24`
```rust
pub const DEFAULT_MAX_OPEN_FILES: usize = 1024
```

#### `const GIB`
- Ort: `crates/contextra-storage/crates/contextra-storage-ports/src/limits.rs:4`
```rust
pub const GIB: u64 = 1 << 30
```

#### `const ID_RESERVE`
- Ort: `crates/contextra-storage/crates/contextra-storage-ports/src/limits.rs:21`
```rust
pub const ID_RESERVE: u64 = 1 << 16
```

#### `const KEY_GRACE_SECS`
- Ort: `crates/contextra-storage/crates/contextra-storage-ports/src/limits.rs:25`
```rust
pub const KEY_GRACE_SECS: u64 = 600
```

#### `const KIB`
- Ort: `crates/contextra-storage/crates/contextra-storage-ports/src/limits.rs:2`
```rust
pub const KIB: usize = 1 << 10
```

#### `const MAX_BODY_BYTES`
- Ort: `crates/contextra-storage/crates/contextra-storage-ports/src/limits.rs:8`
```rust
pub const MAX_BODY_BYTES: usize = 4 * MIB
```

#### `const MAX_CHANGE_BYTES`
- Ort: `crates/contextra-storage/crates/contextra-storage-ports/src/limits.rs:9`
```rust
pub const MAX_CHANGE_BYTES: usize = 8 * MIB
```

#### `const MAX_GROUP_BYTES`
- Ort: `crates/contextra-storage/crates/contextra-storage-ports/src/limits.rs:10`
```rust
pub const MAX_GROUP_BYTES: usize = 4 * MIB
```

#### `const MAX_LAG_BYTES`
- Ort: `crates/contextra-storage/crates/contextra-storage-ports/src/limits.rs:11`
```rust
pub const MAX_LAG_BYTES: u64 = GIB
```

#### `const MIB`
- Ort: `crates/contextra-storage/crates/contextra-storage-ports/src/limits.rs:3`
```rust
pub const MIB: usize = 1 << 20
```

#### `const MIN_ERASE_RESERVE_BYTES`
- Ort: `crates/contextra-storage/crates/contextra-storage-ports/src/limits.rs:14`
```rust
pub const MIN_ERASE_RESERVE_BYTES: u64 = 2 * GIB
```
Dokumentation: Capacity reserved for erase progress, consumer retention, filesystem metadata headroom, and the dedicated purge scratch slots (SPEC §16).

#### `const REDO_ALIGN`
- Ort: `crates/contextra-storage/crates/contextra-storage-ports/src/limits.rs:7`
```rust
pub const REDO_ALIGN: u64 = 4096
```

#### `const REDO_SEGMENT_BYTES`
- Ort: `crates/contextra-storage/crates/contextra-storage-ports/src/limits.rs:6`
```rust
pub const REDO_SEGMENT_BYTES: u64 = 64 * MIB as u64
```

#### `const RUN_BLOCK_TARGET_BYTES`
- Ort: `crates/contextra-storage/crates/contextra-storage-ports/src/limits.rs:16`
```rust
pub const RUN_BLOCK_TARGET_BYTES: usize = 16 * KIB
```

#### `const RUN_RESTART_INTERVAL`
- Ort: `crates/contextra-storage/crates/contextra-storage-ports/src/limits.rs:17`
```rust
pub const RUN_RESTART_INTERVAL: usize = 16
```

#### `const STAMP_RESERVE`
- Ort: `crates/contextra-storage/crates/contextra-storage-ports/src/limits.rs:20`
```rust
pub const STAMP_RESERVE: u64 = 1 << 20
```

### Modul `traits`

#### `mod anchor`
- Ort: `crates/contextra-storage/crates/contextra-storage-ports/src/traits/mod.rs:2`
```rust
pub mod anchor
```

#### `mod authority`
- Ort: `crates/contextra-storage/crates/contextra-storage-ports/src/traits/mod.rs:3`
```rust
pub mod authority
```

#### `mod cold`
- Ort: `crates/contextra-storage/crates/contextra-storage-ports/src/traits/mod.rs:4`
```rust
pub mod cold
```

#### `mod crypto`
- Ort: `crates/contextra-storage/crates/contextra-storage-ports/src/traits/mod.rs:5`
```rust
pub mod crypto
```

#### `mod engine`
- Ort: `crates/contextra-storage/crates/contextra-storage-ports/src/traits/mod.rs:6`
```rust
pub mod engine
```

#### `mod fs`
- Ort: `crates/contextra-storage/crates/contextra-storage-ports/src/traits/mod.rs:7`
```rust
pub mod fs
```

#### `mod memory`
- Ort: `crates/contextra-storage/crates/contextra-storage-ports/src/traits/mod.rs:8`
```rust
pub mod memory
```

#### `mod observe`
- Ort: `crates/contextra-storage/crates/contextra-storage-ports/src/traits/mod.rs:9`
```rust
pub mod observe
```

#### `mod spawn`
- Ort: `crates/contextra-storage/crates/contextra-storage-ports/src/traits/mod.rs:10`
```rust
pub mod spawn
```

### Modul `traits::anchor`

#### `struct AnchorValue`
- Ort: `crates/contextra-storage/crates/contextra-storage-ports/src/traits/anchor.rs:5`
```rust
pub struct AnchorValue
```

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `erase_epoch` | `u64` | `crates/contextra-storage/crates/contextra-storage-ports/src/traits/anchor.rs:6` |
| `ledger_head` | `u64` | `crates/contextra-storage/crates/contextra-storage-ports/src/traits/anchor.rs:7` |
| `catalog_epoch` | `u64` | `crates/contextra-storage/crates/contextra-storage-ports/src/traits/anchor.rs:8` |

#### `struct ClockReading`
- Ort: `crates/contextra-storage/crates/contextra-storage-ports/src/traits/anchor.rs:23`
```rust
pub struct ClockReading
```
Dokumentation: Injected time sample. `durable_utc_millis` is the persisted UTC high-water mark; adapters must never move it backwards. `skew` is fail-closed when the observed wall clock is implausibly far from that mark.
- Impl-Block `crates/contextra-storage/crates/contextra-storage-ports/src/traits/anchor.rs:34`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `monotonic_nanos` | `u64` | `crates/contextra-storage/crates/contextra-storage-ports/src/traits/anchor.rs:25` |
| `utc_millis` | `u64` | `crates/contextra-storage/crates/contextra-storage-ports/src/traits/anchor.rs:27` |
| `durable_utc_millis` | `u64` | `crates/contextra-storage/crates/contextra-storage-ports/src/traits/anchor.rs:29` |
| `skew` | `ClockSkew` | `crates/contextra-storage/crates/contextra-storage-ports/src/traits/anchor.rs:31` |

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn deadline_utc_millis(self) -> Result<u64, Error>` | `crates/contextra-storage/crates/contextra-storage-ports/src/traits/anchor.rs:38` | Returns UTC usable for durable deadlines, rejecting clock rollback and adapter-reported skew. Monotonic process time remains usable for elapsed-duration measure |

#### `enum AnchorStrength`
- Ort: `crates/contextra-storage/crates/contextra-storage-ports/src/traits/anchor.rs:14`
```rust
pub enum AnchorStrength
```
Dokumentation: Strength is explicit because a local file prevents neither rollback of a copied volume nor two restored hosts from writing concurrently.

Varianten:

- `Weak` — `crates/contextra-storage/crates/contextra-storage-ports/src/traits/anchor.rs:15`
- `Strong` — `crates/contextra-storage/crates/contextra-storage-ports/src/traits/anchor.rs:16`

#### `enum ClockSkew`
- Ort: `crates/contextra-storage/crates/contextra-storage-ports/src/traits/anchor.rs:47`
```rust
pub enum ClockSkew
```

Varianten:

- `Certain` — `crates/contextra-storage/crates/contextra-storage-ports/src/traits/anchor.rs:48`
- `Uncertain` — `crates/contextra-storage/crates/contextra-storage-ports/src/traits/anchor.rs:49`

#### `trait Anchor`
- Ort: `crates/contextra-storage/crates/contextra-storage-ports/src/traits/anchor.rs:52`
```rust
pub trait Anchor: Send + Sync
```

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `fn read(&self) -> Result<AnchorValue, Error>` | `crates/contextra-storage/crates/contextra-storage-ports/src/traits/anchor.rs:53` |  |
| `fn advance(&self, expect: AnchorValue, next: AnchorValue) -> Result<(), Error>` | `crates/contextra-storage/crates/contextra-storage-ports/src/traits/anchor.rs:55` | Monoton; bei verlorenem Wettlauf Error::CasLost. |
| `fn strength(&self) -> AnchorStrength` | `crates/contextra-storage/crates/contextra-storage-ports/src/traits/anchor.rs:56` |  |

#### `trait Clock`
- Ort: `crates/contextra-storage/crates/contextra-storage-ports/src/traits/anchor.rs:59`
```rust
pub trait Clock: Send + Sync
```

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `fn read(&self) -> Result<ClockReading, Error>` | `crates/contextra-storage/crates/contextra-storage-ports/src/traits/anchor.rs:64` | Returns monotone and UTC time plus the durable clock floor. Storage uses monotonic time for process-local expiry. Durable deadlines must fail closed when `skew` |
| `fn advance_utc_floor(&self, expect_millis: u64, next_millis: u64) -> Result<(), Error>` | `crates/contextra-storage/crates/contextra-storage-ports/src/traits/anchor.rs:67` | Durably advances the UTC high-water mark with compare-and-swap semantics. A lower value is rejected; a lost race returns `CasLost`. |

#### `trait Entropy`
- Ort: `crates/contextra-storage/crates/contextra-storage-ports/src/traits/anchor.rs:70`
```rust
pub trait Entropy: Send + Sync
```

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `fn fill(&self, buf: &mut [u8]) -> Result<(), Error>` | `crates/contextra-storage/crates/contextra-storage-ports/src/traits/anchor.rs:71` |  |

### Modul `traits::authority`

#### `struct FourEyesProof`
- Ort: `crates/contextra-storage/crates/contextra-storage-ports/src/traits/authority.rs:24`
```rust
pub struct FourEyesProof(Vec<u8>)
```
- Impl-Block `crates/contextra-storage/crates/contextra-storage-ports/src/traits/authority.rs:26`
- Impl-Block (Trait `fmt::Debug`) `crates/contextra-storage/crates/contextra-storage-ports/src/traits/authority.rs:32`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn from_opaque(bytes: Vec<u8>) -> Self` | `crates/contextra-storage/crates/contextra-storage-ports/src/traits/authority.rs:27` |  |
| `pub fn as_bytes(&self) -> &[u8]` | `crates/contextra-storage/crates/contextra-storage-ports/src/traits/authority.rs:29` |  |

#### `struct Principal`
- Ort: `crates/contextra-storage/crates/contextra-storage-ports/src/traits/authority.rs:10`
```rust
pub struct Principal(Vec<u8>)
```
- Impl-Block `crates/contextra-storage/crates/contextra-storage-ports/src/traits/authority.rs:12`
- Impl-Block (Trait `fmt::Debug`) `crates/contextra-storage/crates/contextra-storage-ports/src/traits/authority.rs:18`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn from_opaque(bytes: Vec<u8>) -> Self` | `crates/contextra-storage/crates/contextra-storage-ports/src/traits/authority.rs:13` |  |
| `pub fn differs_from(&self, other: &Self) -> bool` | `crates/contextra-storage/crates/contextra-storage-ports/src/traits/authority.rs:15` |  |

#### `trait Authority`
- Ort: `crates/contextra-storage/crates/contextra-storage-ports/src/traits/authority.rs:38`
```rust
pub trait Authority: Send + Sync
```

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `fn verify_four_eyes( &self, requester: &Principal, approver: &Principal, proof: &FourEyesProof, ) -> Result<(), Error>` | `crates/contextra-storage/crates/contextra-storage-ports/src/traits/authority.rs:42` | Verifies the policy-layer proof for a lift.  The core rejects equal principals before calling this port; providers must not log the opaque values they receive. |

### Modul `traits::cold`

#### `trait ColdTier`
- Ort: `crates/contextra-storage/crates/contextra-storage-ports/src/traits/cold.rs:4`
```rust
pub trait ColdTier: Send + Sync
```

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `fn archive(&self, key: &str, bytes: &[u8]) -> Result<(), Error>` | `crates/contextra-storage/crates/contextra-storage-ports/src/traits/cold.rs:5` |  |
| `fn fetch(&self, key: &str, out: &mut Vec<u8>) -> Result<(), Error>` | `crates/contextra-storage/crates/contextra-storage-ports/src/traits/cold.rs:6` |  |

### Modul `traits::crypto`

#### `trait Checksum`
- Ort: `crates/contextra-storage/crates/contextra-storage-ports/src/traits/crypto.rs:35`
```rust
pub trait Checksum: Send + Sync
```

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `fn crc32c(&self, data: &[u8]) -> u32` | `crates/contextra-storage/crates/contextra-storage-ports/src/traits/crypto.rs:36` |  |

#### `trait Cipher`
- Ort: `crates/contextra-storage/crates/contextra-storage-ports/src/traits/crypto.rs:5`
```rust
pub trait Cipher: Send + Sync
```

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `fn seal( &self, key: &Key32, nonce: &[u8` | `crates/contextra-storage/crates/contextra-storage-ports/src/traits/crypto.rs:6` |  |
| `fn open( &self, key: &Key32, nonce: &[u8` | `crates/contextra-storage/crates/contextra-storage-ports/src/traits/crypto.rs:15` | Fehlschlag der Authentizität ⇒ Error::Corrupt, nie Teilausgabe. |

#### `trait Kdf`
- Ort: `crates/contextra-storage/crates/contextra-storage-ports/src/traits/crypto.rs:31`
```rust
pub trait Kdf: Send + Sync
```

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `fn derive(&self, master: &Key32, label: &[u8]) -> Result<Key32, Error>` | `crates/contextra-storage/crates/contextra-storage-ports/src/traits/crypto.rs:32` |  |

#### `trait Mac`
- Ort: `crates/contextra-storage/crates/contextra-storage-ports/src/traits/crypto.rs:25`
```rust
pub trait Mac: Send + Sync
```

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `fn tag(&self, key: &Key32, data: &[u8]) -> Result<[u8` | `crates/contextra-storage/crates/contextra-storage-ports/src/traits/crypto.rs:28` | Produces a fixed-width tag.  A fallible port avoids hiding a provider or algorithm setup failure behind a fabricated authentication value. |

### Modul `traits::engine`

#### `trait ConsumerFeed`
- Ort: `crates/contextra-storage/crates/contextra-storage-ports/src/traits/engine.rs:8`
```rust
pub trait ConsumerFeed: Send + Sync
```

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `fn poll(&self, max: usize) -> Result<usize, Error>` | `crates/contextra-storage/crates/contextra-storage-ports/src/traits/engine.rs:9` |  |

#### `trait Memtable`
- Ort: `crates/contextra-storage/crates/contextra-storage-ports/src/traits/engine.rs:4`
```rust
pub trait Memtable: Send + Sync
```

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `fn approx_bytes(&self) -> u64` | `crates/contextra-storage/crates/contextra-storage-ports/src/traits/engine.rs:5` |  |

### Modul `traits::fs`

#### `struct FsCaps`
- Ort: `crates/contextra-storage/crates/contextra-storage-ports/src/traits/fs.rs:26`
```rust
pub struct FsCaps
```

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `durable_file_sync` | `bool` | `crates/contextra-storage/crates/contextra-storage-ports/src/traits/fs.rs:27` |
| `durable_dir_sync` | `bool` | `crates/contextra-storage/crates/contextra-storage-ports/src/traits/fs.rs:28` |
| `atomic_rename` | `bool` | `crates/contextra-storage/crates/contextra-storage-ports/src/traits/fs.rs:29` |
| `preallocation_reserves_space` | `bool` | `crates/contextra-storage/crates/contextra-storage-ports/src/traits/fs.rs:30` |
| `hard_link` | `bool` | `crates/contextra-storage/crates/contextra-storage-ports/src/traits/fs.rs:31` |
| `reflink` | `bool` | `crates/contextra-storage/crates/contextra-storage-ports/src/traits/fs.rs:32` |
| `punch_hole` | `bool` | `crates/contextra-storage/crates/contextra-storage-ports/src/traits/fs.rs:33` |

#### `struct IoCtx`
- Ort: `crates/contextra-storage/crates/contextra-storage-ports/src/traits/fs.rs:20`
```rust
pub struct IoCtx
```

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `role` | `FileRole` | `crates/contextra-storage/crates/contextra-storage-ports/src/traits/fs.rs:21` |
| `purpose` | `&'static str` | `crates/contextra-storage/crates/contextra-storage-ports/src/traits/fs.rs:22` |

#### `struct ReadRange`
- Ort: `crates/contextra-storage/crates/contextra-storage-ports/src/traits/fs.rs:37`
```rust
pub struct ReadRange<'a>
```
Dokumentation: One caller-owned output buffer in a batched ordered read request.

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `offset` | `u64` | `crates/contextra-storage/crates/contextra-storage-ports/src/traits/fs.rs:38` |
| `buffer` | `&'a mut [u8]` | `crates/contextra-storage/crates/contextra-storage-ports/src/traits/fs.rs:39` |
| `bytes_read` | `usize` | `crates/contextra-storage/crates/contextra-storage-ports/src/traits/fs.rs:40` |

#### `enum FileRole`
- Ort: `crates/contextra-storage/crates/contextra-storage-ports/src/traits/fs.rs:6`
```rust
pub enum FileRole
```

Varianten:

- `Catalog` — `crates/contextra-storage/crates/contextra-storage-ports/src/traits/fs.rs:7`
- `Redo` — `crates/contextra-storage/crates/contextra-storage-ports/src/traits/fs.rs:8`
- `Run` — `crates/contextra-storage/crates/contextra-storage-ports/src/traits/fs.rs:9`
- `Blob` — `crates/contextra-storage/crates/contextra-storage-ports/src/traits/fs.rs:10`
- `Keys` — `crates/contextra-storage/crates/contextra-storage-ports/src/traits/fs.rs:11`
- `Anchor` — `crates/contextra-storage/crates/contextra-storage-ports/src/traits/fs.rs:12`
- `Ledger` — `crates/contextra-storage/crates/contextra-storage-ports/src/traits/fs.rs:13`
- `Format` — `crates/contextra-storage/crates/contextra-storage-ports/src/traits/fs.rs:14`
- `PurgeScratch` — `crates/contextra-storage/crates/contextra-storage-ports/src/traits/fs.rs:15`
- `Other` — `crates/contextra-storage/crates/contextra-storage-ports/src/traits/fs.rs:16`

#### `trait File`
- Ort: `crates/contextra-storage/crates/contextra-storage-ports/src/traits/fs.rs:43`
```rust
pub trait File: Send + Sync
```

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `fn read_at(&self, off: u64, buf: &mut [u8], ctx: IoCtx) -> Result<usize, Error>` | `crates/contextra-storage/crates/contextra-storage-ports/src/traits/fs.rs:44` |  |
| `fn read_ranges(&self, ranges: &mut [ReadRange<'_>], ctx: IoCtx) -> Result<(), Error>` | `crates/contextra-storage/crates/contextra-storage-ports/src/traits/fs.rs:47` | Reads caller-owned buffers in the supplied order. Short reads are preserved in each `bytes_read`; implementations must not reorder. |
| `fn append_prealloc(&self, data: &[u8], ctx: IoCtx) -> Result<(), Error>` | `crates/contextra-storage/crates/contextra-storage-ports/src/traits/fs.rs:57` |  |
| `fn preallocate(&self, len: u64, ctx: IoCtx) -> Result<(), Error>` | `crates/contextra-storage/crates/contextra-storage-ports/src/traits/fs.rs:59` | Schreibt Nullen, nie sparse (AN-10). ENOSPC tritt nur hier auf. |
| `fn sync_data(&self, ctx: IoCtx) -> Result<(), Error>` | `crates/contextra-storage/crates/contextra-storage-ports/src/traits/fs.rs:60` |  |
| `fn sync_all(&self, ctx: IoCtx) -> Result<(), Error>` | `crates/contextra-storage/crates/contextra-storage-ports/src/traits/fs.rs:61` |  |
| `fn size(&self) -> Result<u64, Error>` | `crates/contextra-storage/crates/contextra-storage-ports/src/traits/fs.rs:62` |  |

#### `trait Fs`
- Ort: `crates/contextra-storage/crates/contextra-storage-ports/src/traits/fs.rs:65`
```rust
pub trait Fs: Send + Sync
```

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `fn open(&self, path: &Path, ctx: IoCtx) -> Result<Box<dyn File>, Error>` | `crates/contextra-storage/crates/contextra-storage-ports/src/traits/fs.rs:66` |  |
| `fn create_new(&self, path: &Path, ctx: IoCtx) -> Result<Box<dyn File>, Error>` | `crates/contextra-storage/crates/contextra-storage-ports/src/traits/fs.rs:67` |  |
| `fn rename(&self, from: &Path, to: &Path, ctx: IoCtx) -> Result<(), Error>` | `crates/contextra-storage/crates/contextra-storage-ports/src/traits/fs.rs:68` |  |
| `fn unlink(&self, path: &Path, ctx: IoCtx) -> Result<(), Error>` | `crates/contextra-storage/crates/contextra-storage-ports/src/traits/fs.rs:69` |  |
| `fn lock(&self, _path: &Path, _ctx: IoCtx) -> Result<Box<dyn FsLock>, Error>` | `crates/contextra-storage/crates/contextra-storage-ports/src/traits/fs.rs:72` | Acquires a process/host lock for `path`. Implementations without a locking primitive must fail closed instead of returning a dummy lock. |
| `fn link(&self, _from: &Path, _to: &Path, _ctx: IoCtx) -> Result<(), Error>` | `crates/contextra-storage/crates/contextra-storage-ports/src/traits/fs.rs:77` | Optional filesystem capabilities. Unsupported implementations return a typed error and callers must preserve correctness without them. |
| `fn reflink(&self, _from: &Path, _to: &Path, _ctx: IoCtx) -> Result<(), Error>` | `crates/contextra-storage/crates/contextra-storage-ports/src/traits/fs.rs:80` |  |
| `fn punch_hole(&self, _path: &Path, _offset: u64, _length: u64, _ctx: IoCtx) -> Result<(), Error>` | `crates/contextra-storage/crates/contextra-storage-ports/src/traits/fs.rs:83` |  |
| `fn sync_dir(&self, dir: &Path, ctx: IoCtx) -> Result<(), Error>` | `crates/contextra-storage/crates/contextra-storage-ports/src/traits/fs.rs:86` |  |
| `fn list(&self, dir: &Path) -> Result<Vec<PathBuf>, Error>` | `crates/contextra-storage/crates/contextra-storage-ports/src/traits/fs.rs:87` |  |
| `fn free_space(&self, dir: &Path) -> Result<u64, Error>` | `crates/contextra-storage/crates/contextra-storage-ports/src/traits/fs.rs:88` |  |
| `fn caps(&self) -> FsCaps` | `crates/contextra-storage/crates/contextra-storage-ports/src/traits/fs.rs:89` |  |

#### `trait FsLock`
- Ort: `crates/contextra-storage/crates/contextra-storage-ports/src/traits/fs.rs:93`
```rust
pub trait FsLock: Send + Sync
```
Dokumentation: RAII handle for a filesystem lock. Dropping releases the lock.

### Modul `traits::memory`

#### `struct ResourcePermit`
- Ort: `crates/contextra-storage/crates/contextra-storage-ports/src/traits/memory.rs:19`
```rust
pub struct ResourcePermit
```
Dokumentation: A single successful reservation. Dropping it returns exactly the reserved amount to the originating budget; permits are intentionally not cloneable.
- Impl-Block `crates/contextra-storage/crates/contextra-storage-ports/src/traits/memory.rs:25`
- Impl-Block (Trait `Drop`) `crates/contextra-storage/crates/contextra-storage-ports/src/traits/memory.rs:38`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new( class: ResourceClass, bytes: u64, release: impl FnOnce() + Send + Sync + 'static, ) -> Self` | `crates/contextra-storage/crates/contextra-storage-ports/src/traits/memory.rs:26` |  |
| `pub fn bytes(&self) -> u64` | `crates/contextra-storage/crates/contextra-storage-ports/src/traits/memory.rs:34` |  |
| `pub fn class(&self) -> ResourceClass` | `crates/contextra-storage/crates/contextra-storage-ports/src/traits/memory.rs:35` |  |

#### `enum ResourceClass`
- Ort: `crates/contextra-storage/crates/contextra-storage-ports/src/traits/memory.rs:6`
```rust
pub enum ResourceClass
```

Varianten:

- `Memtable` — `crates/contextra-storage/crates/contextra-storage-ports/src/traits/memory.rs:7`
- `ClusterMeta` — `crates/contextra-storage/crates/contextra-storage-ports/src/traits/memory.rs:8`
- `Guard` — `crates/contextra-storage/crates/contextra-storage-ports/src/traits/memory.rs:9`
- `BlockCache` — `crates/contextra-storage/crates/contextra-storage-ports/src/traits/memory.rs:10`
- `KeyCache` — `crates/contextra-storage/crates/contextra-storage-ports/src/traits/memory.rs:11`
- `ExtensionDelta` — `crates/contextra-storage/crates/contextra-storage-ports/src/traits/memory.rs:12`
- `Inflight` — `crates/contextra-storage/crates/contextra-storage-ports/src/traits/memory.rs:13`
- `PurgeScratch` — `crates/contextra-storage/crates/contextra-storage-ports/src/traits/memory.rs:14`

#### `trait MemoryBudget`
- Ort: `crates/contextra-storage/crates/contextra-storage-ports/src/traits/memory.rs:44`
```rust
pub trait MemoryBudget: Send + Sync
```

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `fn try_reserve(&self, class: ResourceClass, bytes: u64) -> Result<ResourcePermit, Error>` | `crates/contextra-storage/crates/contextra-storage-ports/src/traits/memory.rs:47` | Reserves before the caller allocates. Implementations must reject overflow and class/total-limit exhaustion without changing accounting. |

### Modul `traits::observe`

#### `enum EraseStage`
- Ort: `crates/contextra-storage/crates/contextra-storage-ports/src/traits/observe.rs:91`
```rust
pub enum EraseStage
```

#### `enum MaintenancePhase`
- Ort: `crates/contextra-storage/crates/contextra-storage-ports/src/traits/observe.rs:89`
```rust
pub enum MaintenancePhase
```

#### `enum MetricLabel`
- Ort: `crates/contextra-storage/crates/contextra-storage-ports/src/traits/observe.rs:110`
```rust
pub enum MetricLabel
```
Dokumentation: A single bounded label. The enum prevents identifiers and arbitrary text from entering metric dimensions.
- Impl-Block `crates/contextra-storage/crates/contextra-storage-ports/src/traits/observe.rs:121`

Varianten:

- `Durability(Durability)` — `crates/contextra-storage/crates/contextra-storage-ports/src/traits/observe.rs:111`
- `Result(MetricResult)` — `crates/contextra-storage/crates/contextra-storage-ports/src/traits/observe.rs:112`
- `FileRole(FileRole)` — `crates/contextra-storage/crates/contextra-storage-ports/src/traits/observe.rs:113`
- `ResourceClass(ResourceClass)` — `crates/contextra-storage/crates/contextra-storage-ports/src/traits/observe.rs:114`
- `MaintenancePhase(MaintenancePhase)` — `crates/contextra-storage/crates/contextra-storage-ports/src/traits/observe.rs:115`
- `EraseStage(EraseStage)` — `crates/contextra-storage/crates/contextra-storage-ports/src/traits/observe.rs:116`
- `OwnerKind(OwnerKind)` — `crates/contextra-storage/crates/contextra-storage-ports/src/traits/observe.rs:117`
- `ErrorClass(ErrorClass)` — `crates/contextra-storage/crates/contextra-storage-ports/src/traits/observe.rs:118`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub const fn key(self) -> MetricLabelKey` | `crates/contextra-storage/crates/contextra-storage-ports/src/traits/observe.rs:122` |  |
| `pub const fn key_name(self) -> &'static str` | `crates/contextra-storage/crates/contextra-storage-ports/src/traits/observe.rs:135` |  |
| `pub const fn value_name(self) -> &'static str` | `crates/contextra-storage/crates/contextra-storage-ports/src/traits/observe.rs:148` |  |

#### `enum MetricLabelKey`
- Ort: `crates/contextra-storage/crates/contextra-storage-ports/src/traits/observe.rs:96`
```rust
pub enum MetricLabelKey
```

Varianten:

- `Durability` — `crates/contextra-storage/crates/contextra-storage-ports/src/traits/observe.rs:97`
- `Result` — `crates/contextra-storage/crates/contextra-storage-ports/src/traits/observe.rs:98`
- `FileRole` — `crates/contextra-storage/crates/contextra-storage-ports/src/traits/observe.rs:99`
- `ResourceClass` — `crates/contextra-storage/crates/contextra-storage-ports/src/traits/observe.rs:100`
- `Phase` — `crates/contextra-storage/crates/contextra-storage-ports/src/traits/observe.rs:101`
- `Stage` — `crates/contextra-storage/crates/contextra-storage-ports/src/traits/observe.rs:102`
- `OwnerKind` — `crates/contextra-storage/crates/contextra-storage-ports/src/traits/observe.rs:103`
- `ErrorClass` — `crates/contextra-storage/crates/contextra-storage-ports/src/traits/observe.rs:104`

#### `enum MetricName`
- Ort: `crates/contextra-storage/crates/contextra-storage-ports/src/traits/observe.rs:8`
```rust
pub enum MetricName
```
Dokumentation: Fixed metric catalog from SPEC §16.
- Impl-Block `crates/contextra-storage/crates/contextra-storage-ports/src/traits/observe.rs:45`

Varianten:

- `CommitsTotal` — `crates/contextra-storage/crates/contextra-storage-ports/src/traits/observe.rs:9`
- `CommitGroupSize` — `crates/contextra-storage/crates/contextra-storage-ports/src/traits/observe.rs:10`
- `CommitDurationNanoseconds` — `crates/contextra-storage/crates/contextra-storage-ports/src/traits/observe.rs:11`
- `DurableStampLag` — `crates/contextra-storage/crates/contextra-storage-ports/src/traits/observe.rs:12`
- `FsyncDurationNanoseconds` — `crates/contextra-storage/crates/contextra-storage-ports/src/traits/observe.rs:13`
- `MemoryBytes` — `crates/contextra-storage/crates/contextra-storage-ports/src/traits/observe.rs:14`
- `MemoryLimitBytes` — `crates/contextra-storage/crates/contextra-storage-ports/src/traits/observe.rs:15`
- `DiskFull` — `crates/contextra-storage/crates/contextra-storage-ports/src/traits/observe.rs:16`
- `CacheHitRatio` — `crates/contextra-storage/crates/contextra-storage-ports/src/traits/observe.rs:17`
- `DiskReserveBytes` — `crates/contextra-storage/crates/contextra-storage-ports/src/traits/observe.rs:18`
- `ScratchBytes` — `crates/contextra-storage/crates/contextra-storage-ports/src/traits/observe.rs:19`
- `MaintenanceBytesTotal` — `crates/contextra-storage/crates/contextra-storage-ports/src/traits/observe.rs:20`
- `MaintenanceDurationNanoseconds` — `crates/contextra-storage/crates/contextra-storage-ports/src/traits/observe.rs:21`
- `EraseOpenTickets` — `crates/contextra-storage/crates/contextra-storage-ports/src/traits/observe.rs:22`
- `EraseOldestPendingNanoseconds` — `crates/contextra-storage/crates/contextra-storage-ports/src/traits/observe.rs:23`
- `EraseOperationsTotal` — `crates/contextra-storage/crates/contextra-storage-ports/src/traits/observe.rs:24`
- `EraseDeadlineBreachesTotal` — `crates/contextra-storage/crates/contextra-storage-ports/src/traits/observe.rs:25`
- `EraseGateDurationNanoseconds` — `crates/contextra-storage/crates/contextra-storage-ports/src/traits/observe.rs:26`
- `EraseKeyDestroyDurationNanoseconds` — `crates/contextra-storage/crates/contextra-storage-ports/src/traits/observe.rs:27`
- `EraseStageDurationNanoseconds` — `crates/contextra-storage/crates/contextra-storage-ports/src/traits/observe.rs:28`
- `ErasePurgeBytesTotal` — `crates/contextra-storage/crates/contextra-storage-ports/src/traits/observe.rs:29`
- `EraseReserveFreeBytes` — `crates/contextra-storage/crates/contextra-storage-ports/src/traits/observe.rs:30`
- `EraseReserveUsedBytes` — `crates/contextra-storage/crates/contextra-storage-ports/src/traits/observe.rs:31`
- `AnchorLagNanoseconds` — `crates/contextra-storage/crates/contextra-storage-ports/src/traits/observe.rs:32`
- `ConsumerLagBytes` — `crates/contextra-storage/crates/contextra-storage-ports/src/traits/observe.rs:33`
- `ConsumerLagStamps` — `crates/contextra-storage/crates/contextra-storage-ports/src/traits/observe.rs:34`
- `ExtensionBlockedOwners` — `crates/contextra-storage/crates/contextra-storage-ports/src/traits/observe.rs:35`
- `GuardSlowpathRatio` — `crates/contextra-storage/crates/contextra-storage-ports/src/traits/observe.rs:36`
- `BranchCount` — `crates/contextra-storage/crates/contextra-storage-ports/src/traits/observe.rs:37`
- `BranchMaxDepth` — `crates/contextra-storage/crates/contextra-storage-ports/src/traits/observe.rs:38`
- `BranchOldestPinNanoseconds` — `crates/contextra-storage/crates/contextra-storage-ports/src/traits/observe.rs:39`
- `BranchHistoryFloor` — `crates/contextra-storage/crates/contextra-storage-ports/src/traits/observe.rs:40`
- `ErrorsTotal` — `crates/contextra-storage/crates/contextra-storage-ports/src/traits/observe.rs:41`
- `Poisoned` — `crates/contextra-storage/crates/contextra-storage-ports/src/traits/observe.rs:42`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub const fn as_str(self) -> &'static str` | `crates/contextra-storage/crates/contextra-storage-ports/src/traits/observe.rs:46` |  |

#### `enum MetricResult`
- Ort: `crates/contextra-storage/crates/contextra-storage-ports/src/traits/observe.rs:87`
```rust
pub enum MetricResult
```

#### `enum MetricSample`
- Ort: `crates/contextra-storage/crates/contextra-storage-ports/src/traits/observe.rs:207`
```rust
pub enum MetricSample
```

Varianten:

- `Counter(u64)` — `crates/contextra-storage/crates/contextra-storage-ports/src/traits/observe.rs:209`
- `Gauge(u64)` — `crates/contextra-storage/crates/contextra-storage-ports/src/traits/observe.rs:211`
- `Observe(u64)` — `crates/contextra-storage/crates/contextra-storage-ports/src/traits/observe.rs:213`

#### `enum ObserverEvent`
- Ort: `crates/contextra-storage/crates/contextra-storage-ports/src/traits/observe.rs:225`
```rust
pub enum ObserverEvent
```
Dokumentation: Content-free standard observer events. An event cannot carry user data, identifiers, tokens, keys, or arbitrary strings.

Varianten:

- `Commit{ result: MetricResult }` — `crates/contextra-storage/crates/contextra-storage-ports/src/traits/observe.rs:226`
- `FilePoisoned{ role: FileRole }` — `crates/contextra-storage/crates/contextra-storage-ports/src/traits/observe.rs:227`
- `EraseStage{ stage: EraseStage, result: MetricResult }` — `crates/contextra-storage/crates/contextra-storage-ports/src/traits/observe.rs:228`
- `RecoveryQuarantined` — `crates/contextra-storage/crates/contextra-storage-ports/src/traits/observe.rs:229`
- `ExtensionBlocked{ owner: OwnerKind }` — `crates/contextra-storage/crates/contextra-storage-ports/src/traits/observe.rs:230`

#### `enum OwnerKind`
- Ort: `crates/contextra-storage/crates/contextra-storage-ports/src/traits/observe.rs:93`
```rust
pub enum OwnerKind
```

#### `trait Metrics`
- Ort: `crates/contextra-storage/crates/contextra-storage-ports/src/traits/observe.rs:216`
```rust
pub trait Metrics: Send + Sync
```

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `fn record(&self, name: MetricName, sample: MetricSample, labels: &[MetricLabel])` | `crates/contextra-storage/crates/contextra-storage-ports/src/traits/observe.rs:219` | Must be nonblocking. Names and labels are restricted to SPEC §16's closed catalog, so call sites cannot attach identifiers or payloads. |

#### `trait Observer`
- Ort: `crates/contextra-storage/crates/contextra-storage-ports/src/traits/observe.rs:233`
```rust
pub trait Observer: Send + Sync
```

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `fn event(&self, event: ObserverEvent)` | `crates/contextra-storage/crates/contextra-storage-ports/src/traits/observe.rs:234` |  |

### Modul `traits::spawn`

#### `trait Spawner`
- Ort: `crates/contextra-storage/crates/contextra-storage-ports/src/traits/spawn.rs:4`
```rust
pub trait Spawner: Send + Sync
```

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `fn spawn(&self, name: &'static str, job: Box<dyn FnOnce() + Send>) -> Result<(), Error>` | `crates/contextra-storage/crates/contextra-storage-ports/src/traits/spawn.rs:5` |  |
