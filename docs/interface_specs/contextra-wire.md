# Schnittstellenspezifikation: `contextra-wire`

> Erzeugt durch `tools/extract_interfaces.py` aus dem Quellcode. Keine Handpflege. Jede Angabe trägt Datei:Zeile. Maßgeblich bleibt der Quellcode.

## 1. Kopf

| Feld | Wert |
|---|---|
| Name | `contextra-wire` |
| Version | workspace |
| Pfad | `crates/contextra-wire` |
| Rolle | Wire-Protokoll |
| Bibliothekspfad | src/lib.rs (Standard) |

**Crate-Dokumentation (//!):**

> # Crate `contextra-wire`
>
> **Ring & Klasse:** Ring 0 (Unsafe-Insel 3 von 3), Klasse A.
>
> ## Zweck
> `contextra-wire` stellt performante Zero-Copy-Serialisierungs- und Deserialisierungs-Primitiven
> für die Interprozesskommunikation (IPC = Inter-Process Communication) bereit. Es kapselt:
> 1. Generierten FlatBuffers-Code (`contextra_generated`), der direkt auf Byte-Puffern operiert.
> 2. Gehärtete Verifikationsfunktionen (`verification`), die Puffer vor dem Zugriff strikt validieren.
> 3. Zero-Copy-Adapter (`adapter`), wie den referenzgezählten `WireBuffer`.
> 4. Einen robusten, angriffssicheren JSON-RPC-2.0-Parser (`jsonrpc`) ohne Panics.
> 5. Einen einheitlichen Fehlertyp `Error` (`Result<T>`) gemäß SPEC v4.0 B4.

## 2. Abhängigkeiten

| Name | Art | Quelle | optional | verwendet von (normal/dev) |
|---|---|---|---|---|
| `bytes` | normal | workspace | nein |  |
| `criterion` | dev | workspace | nein |  |
| `flatbuffers` | normal | workspace | nein |  |
| `proptest` | dev | registry | nein |  |
| `serde` | normal | workspace | nein |  |
| `serde_json` | normal | workspace | nein |  |
| `thiserror` | normal | workspace | nein |  |
| `tracing` | normal | workspace | ja |  |

**Verwendet von (Crate-Ebene):** `contextra-mcp`, `contextra-py`, `contextra-router`, `contextra-wire-fuzz`

## 3. Features

| Feature | Aktiviert |
|---|---|
| `default` | `std` |
| `inspect` | – |
| `serde` | – |
| `std` | – |
| `tracing` | `dep:tracing` |

## 4. Metriken (Quellcode)

| Kennzahl | Wert |
|---|---|
| Quelldateien | 6 |
| Quellzeilen | 3234 |
| Öffentliche Items (Modul-Ebene) | 71 |
| davon `fn` | 19 |
| davon `struct` | 32 |
| davon `enum` | 11 |
| davon `trait` | 1 |
| davon `type` | 1 |
| davon `mod` | 7 |
| Öffentliche Makros (`#[macro_export]`) | 0 |
| Öffentliche Re-Exports (`pub use`) | 5 |
| `unsafe`-Vorkommen (Produktionscode) | 45 |
| `.unwrap()` (Produktionscode) | 16 |
| `.expect(` (Produktionscode) | 9 |
| `panic!` (Produktionscode) | 3 |
| `todo!/unimplemented!/unreachable!` | 0 |
| Indexierungen `x[…]` (grob, Produktionscode) | 2 |
| `#[deprecated]` | 0 |
| `#[cfg(feature…)]`-Gates | 0 |
| Parser-Abdeckung (geparste `pub`-Deklarationen / Grep-Treffer) | 200 / 200 = 100 % |

## 5. Modulbaum und Dateien

| Modul | Datei | Zeilen |
|---|---|---|
| `adapter` | `crates/contextra-wire/src/adapter.rs` | 222 |
| `contextra_generated` | `crates/contextra-wire/src/contextra_generated.rs` | 1818 |
| `generated` | `crates/contextra-wire/src/generated.rs` | 38 |
| `jsonrpc` | `crates/contextra-wire/src/jsonrpc.rs` | 738 |
| `crate` | `crates/contextra-wire/src/lib.rs` | 76 |
| `verification` | `crates/contextra-wire/src/verification.rs` | 342 |

## 6. Re-Exports (`pub use`)

- `pub use adapter::*` — `crates/contextra-wire/src/lib.rs:49` (Modul `crate`)
- `pub use generated::contextra_generated` — `crates/contextra-wire/src/lib.rs:50` (Modul `crate`)
- `pub use generated::contextra_generated::contextra::ipc::*` — `crates/contextra-wire/src/lib.rs:51` (Modul `crate`)
- `pub use jsonrpc::*` — `crates/contextra-wire/src/lib.rs:52` (Modul `crate`)
- `pub use verification::*` — `crates/contextra-wire/src/lib.rs:53` (Modul `crate`)

## 8. Öffentliche Schnittstelle je Item

### Modul `adapter`

#### `struct Unverified`
- Ort: `crates/contextra-wire/src/adapter.rs:30`
```rust
pub struct Unverified
```
Dokumentation: Typzustands-Marker: Unverifizierter Puffer. Rohpuffer, dessen FlatBuffers-Offsets, VTables und Bounds noch nicht validiert wurden. Aus diesem Zustand kann nicht direkt auf Root-Tabellen zugegriffen werden.

#### `struct Verified`
- Ort: `crates/contextra-wire/src/adapter.rs:37`
```rust
pub struct Verified<T>
```
Dokumentation: Typzustands-Marker: Verifizierter Puffer für FlatBuffers-Wurzeltabelle `T`. Garantiert durch vorangegangene Verifikation gegen `FlatBufferConfig`, dass alle Offsets, VTables, Vektoren und Bounds des Typs `T` innerhalb des Puffers liegen (`INV-WIRE-BOUNDS-1`).

#### `struct WireBuffer`
- Ort: `crates/contextra-wire/src/adapter.rs:93`
```rust
pub struct WireBuffer<State = Unverified>
```
Dokumentation: Zero-Copy Byte-Puffer-Wrapper für FlatBuffers-Nachrichten mit Typzustand. Kapselt eine Instanz von `bytes::Bytes`, um ein sicheres, referenzgezähltes Weiterreichen von Binärdaten ohne Speicherallokation oder Kopiervorgänge zu ermöglichen. # Invarianten - `INV-WIRE-BOUNDS-1`: Typzustand `Unverified` -> `Verified<T>`. Wurzelzugriff existiert nur auf `Verified<T>`. - Keine Zeiger-Transmutes (`transmute`) oder Lebenszeit-Manipulationen (`lifetime transmutations`). - Threadsicher durch interne atomare Referenzzählung (`Arc`-äquivalent in `Bytes`).
- Impl-Block `crates/contextra-wire/src/adapter.rs:99`
- Impl-Block `crates/contextra-wire/src/adapter.rs:147`
- Impl-Block `crates/contextra-wire/src/adapter.rs:176`
- Impl-Block (Trait `AsRef<[u8]>`) `crates/contextra-wire/src/adapter.rs:217`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new(data: Bytes) -> Self` | `crates/contextra-wire/src/adapter.rs:113` | Erzeugt einen neuen `WireBuffer` im Zustand `Unverified`, der das übergebene `Bytes`-Objekt umhüllt. # Ablauf 1. Übernahme des `Bytes`-Objekts (Besitzübertragun |
| `pub fn verify<T>( self, config: &FlatBufferConfig, ) -> Result<WireBuffer<Verified<T>>, FlatBufferError> where T: VerifiableRoot,` | `crates/contextra-wire/src/adapter.rs:131` | Verifiziert den Puffer gegen das angegebene FlatBuffers-Root-Schema `T` und die Schutzgrenzen `config`. # Ablauf (`INV-WIRE-BOUNDS-1`) 1. Übergabe des Rohdaten- |
| `pub fn root<'a>( &'a self, ) -> Result<<T::Table<'a> as flatbuffers::Follow<'a>>::Inner, FlatBufferError>` | `crates/contextra-wire/src/adapter.rs:157` | Liefert die verifizierte FlatBuffers-Wurzel für diesen Puffer unter Verwendung der bei der Verifikation geprüften Grenzwerte. # Garantien (`INV-WIRE-BOUNDS-1`)  |
| `pub fn root_with_config<'a>( &'a self, config: &FlatBufferConfig, ) -> Result<<T::Table<'a> as flatbuffers::Follow<'a>>::Inner, FlatBufferError>` | `crates/contextra-wire/src/adapter.rs:168` | Liefert die verifizierte FlatBuffers-Wurzel für diesen Puffer unter Verwendung der angegebenen Schutzgrenzen. # Garantien (`INV-WIRE-BOUNDS-1`) Sicherer, geprüf |
| `pub fn as_slice(&self) -> &[u8]` | `crates/contextra-wire/src/adapter.rs:187` | Gibt eine unveränderliche Slice-Referenz auf die zugrundeliegenden Rohdaten zurück. # Ablauf 1. Zugriff auf den internen Byte-Puffer. 2. Rückgabe als Byte-Slice |
| `pub fn into_inner(self) -> Bytes` | `crates/contextra-wire/src/adapter.rs:198` | Konsumiert den Wrapper und gibt das zugrundeliegende `Bytes`-Objekt frei. # Ablauf 1. Entpacken des internen `data`-Felds. 2. Rückgabe des ursprünglichen `Bytes |
| `pub fn len(&self) -> usize` | `crates/contextra-wire/src/adapter.rs:204` | Gibt die Länge des Puffers in Bytes zurück. |
| `pub fn is_empty(&self) -> bool` | `crates/contextra-wire/src/adapter.rs:210` | Gibt an, ob der Puffer leer ist. |

#### `trait VerifiableRoot`
- Ort: `crates/contextra-wire/src/adapter.rs:42`
```rust
pub trait VerifiableRoot
```
Dokumentation: Helper trait providing higher-ranked follow and verification for root FlatBuffers tables.
- Impl-Block `crates/contextra-wire/src/adapter.rs:60`

Assoziierte Konstanten/Typen:

- `type Table<'a>: flatbuffers::Follow<'a> + flatbuffers::Verifiable + 'a` — `crates/contextra-wire/src/adapter.rs:44`

### Modul `contextra_generated`

#### `mod contextra`
- Ort: `crates/contextra-wire/src/contextra_generated.rs:29`
```rust
pub mod contextra
```

### Modul `contextra_generated::contextra`

#### `mod ipc`
- Ort: `crates/contextra-wire/src/contextra_generated.rs:37`
```rust
pub mod ipc
```

### Modul `contextra_generated::contextra::ipc`

#### `struct Embedding`
- Ort: `crates/contextra-wire/src/contextra_generated.rs:51`
```rust
pub struct Embedding<'a>
```
Dokumentation: Öffentliches FlatBuffers-Item `Embedding` für IPC-Nachrichten.
- Impl-Block (Trait `VerifiableRoot`) `crates/contextra-wire/src/adapter.rs:48`
- Impl-Block (Trait `flatbuffers::Follow<'a>`) `crates/contextra-wire/src/contextra_generated.rs:55`
- Impl-Block `crates/contextra-wire/src/contextra_generated.rs:66`
- Impl-Block (Trait `flatbuffers::Verifiable`) `crates/contextra-wire/src/contextra_generated.rs:146`
- Impl-Block (Trait `core::fmt::Debug`) `crates/contextra-wire/src/contextra_generated.rs:240`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `_tab` | `flatbuffers::Table<'a>` | `crates/contextra-wire/src/contextra_generated.rs:52` |

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub unsafe fn init_from_table(table: flatbuffers::Table<'a>) -> Self` | `crates/contextra-wire/src/contextra_generated.rs:81` | Öffentliches FlatBuffers-Item `init_from_table` für IPC-Nachrichten. |
| `pub fn create<'bldr: 'args, 'args: 'mut_bldr, 'mut_bldr>( _fbb: &'mut_bldr mut flatbuffers::FlatBufferBuilder<'bldr>, args: &'args EmbeddingArgs<'args>, ) -> flatbuffers::WIPOffset<Embedding<'bldr>>` | `crates/contextra-wire/src/contextra_generated.rs:87` | Öffentliches FlatBuffers-Item `create` für IPC-Nachrichten. |
| `pub fn data(&self) -> Option<flatbuffers::Vector<'a, f32>>` | `crates/contextra-wire/src/contextra_generated.rs:105` | Öffentliches FlatBuffers-Item `data` für IPC-Nachrichten. |
| `pub fn quantized_u8(&self) -> Option<flatbuffers::Vector<'a, u8>>` | `crates/contextra-wire/src/contextra_generated.rs:121` | Öffentliches FlatBuffers-Item `quantized_u8` für IPC-Nachrichten. |
| `pub fn metric(&self) -> i8` | `crates/contextra-wire/src/contextra_generated.rs:137` | Öffentliches FlatBuffers-Item `metric` für IPC-Nachrichten. |

Assoziierte Konstanten/Typen:

- `pub const VT_DATA: flatbuffers::VOffsetT = 4` — `crates/contextra-wire/src/contextra_generated.rs:69`
- `pub const VT_QUANTIZED_U8: flatbuffers::VOffsetT = 6` — `crates/contextra-wire/src/contextra_generated.rs:72`
- `pub const VT_METRIC: flatbuffers::VOffsetT = 8` — `crates/contextra-wire/src/contextra_generated.rs:75`

#### `struct EmbeddingArgs`
- Ort: `crates/contextra-wire/src/contextra_generated.rs:171`
```rust
pub struct EmbeddingArgs<'a>
```
Dokumentation: Öffentliches FlatBuffers-Item `EmbeddingArgs` für IPC-Nachrichten.
- Impl-Block (Trait `Default`) `crates/contextra-wire/src/contextra_generated.rs:176`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `metric` | `i8` | `crates/contextra-wire/src/contextra_generated.rs:174` |

#### `struct EmbeddingBuilder`
- Ort: `crates/contextra-wire/src/contextra_generated.rs:189`
```rust
pub struct EmbeddingBuilder<'a: 'b, 'b>
```
Dokumentation: Öffentliches FlatBuffers-Item `EmbeddingBuilder` für IPC-Nachrichten.
- Impl-Block `crates/contextra-wire/src/contextra_generated.rs:193`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn add_data(&mut self, data: flatbuffers::WIPOffset<flatbuffers::Vector<'b, f32>>)` | `crates/contextra-wire/src/contextra_generated.rs:197` | Öffentliches FlatBuffers-Item `add_data` für IPC-Nachrichten. |
| `pub fn add_quantized_u8( &mut self, quantized_u8: flatbuffers::WIPOffset<flatbuffers::Vector<'b, u8>>, )` | `crates/contextra-wire/src/contextra_generated.rs:204` | Öffentliches FlatBuffers-Item `add_quantized_u8` für IPC-Nachrichten. |
| `pub fn add_metric(&mut self, metric: i8)` | `crates/contextra-wire/src/contextra_generated.rs:216` | Öffentliches FlatBuffers-Item `add_metric` für IPC-Nachrichten. |
| `pub fn new( _fbb: &'b mut flatbuffers::FlatBufferBuilder<'a>, ) -> EmbeddingBuilder<'a, 'b>` | `crates/contextra-wire/src/contextra_generated.rs:222` | Öffentliches FlatBuffers-Item `new` für IPC-Nachrichten. |
| `pub fn finish(self) -> flatbuffers::WIPOffset<Embedding<'a>>` | `crates/contextra-wire/src/contextra_generated.rs:234` | Öffentliches FlatBuffers-Item `finish` für IPC-Nachrichten. |

#### `struct HyperEdge`
- Ort: `crates/contextra-wire/src/contextra_generated.rs:1304`
```rust
pub struct HyperEdge<'a>
```
Dokumentation: Öffentliches FlatBuffers-Item `HyperEdge` für IPC-Nachrichten.
- Impl-Block (Trait `VerifiableRoot`) `crates/contextra-wire/src/adapter.rs:78`
- Impl-Block (Trait `flatbuffers::Follow<'a>`) `crates/contextra-wire/src/contextra_generated.rs:1308`
- Impl-Block `crates/contextra-wire/src/contextra_generated.rs:1319`
- Impl-Block (Trait `flatbuffers::Verifiable`) `crates/contextra-wire/src/contextra_generated.rs:1526`
- Impl-Block (Trait `core::fmt::Debug`) `crates/contextra-wire/src/contextra_generated.rs:1702`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `_tab` | `flatbuffers::Table<'a>` | `crates/contextra-wire/src/contextra_generated.rs:1305` |

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub unsafe fn init_from_table(table: flatbuffers::Table<'a>) -> Self` | `crates/contextra-wire/src/contextra_generated.rs:1355` | Öffentliches FlatBuffers-Item `init_from_table` für IPC-Nachrichten. |
| `pub fn create<'bldr: 'args, 'args: 'mut_bldr, 'mut_bldr>( _fbb: &'mut_bldr mut flatbuffers::FlatBufferBuilder<'bldr>, args: &'args HyperEdgeArgs<'args>, ) -> flatbuffers::WIPOffset<HyperEdge<'bldr>>` | `crates/contextra-wire/src/contextra_generated.rs:1361` | Öffentliches FlatBuffers-Item `create` für IPC-Nachrichten. |
| `pub fn id(&self) -> u64` | `crates/contextra-wire/src/contextra_generated.rs:1386` | Öffentliches FlatBuffers-Item `id` für IPC-Nachrichten. |
| `pub fn predicate(&self) -> u8` | `crates/contextra-wire/src/contextra_generated.rs:1396` | Öffentliches FlatBuffers-Item `predicate` für IPC-Nachrichten. |
| `pub fn participants( &self, ) -> Option<flatbuffers::Vector<'a, flatbuffers::ForwardsUOffset<RoleBinding<'a>>>>` | `crates/contextra-wire/src/contextra_generated.rs:1410` | Öffentliches FlatBuffers-Item `participants` für IPC-Nachrichten. |
| `pub fn weight(&self) -> f32` | `crates/contextra-wire/src/contextra_generated.rs:1427` | Öffentliches FlatBuffers-Item `weight` für IPC-Nachrichten. |
| `pub fn tx_valid_from(&self) -> u64` | `crates/contextra-wire/src/contextra_generated.rs:1441` | Öffentliches FlatBuffers-Item `tx_valid_from` für IPC-Nachrichten. |
| `pub fn tx_valid_to(&self) -> u64` | `crates/contextra-wire/src/contextra_generated.rs:1455` | Öffentliches FlatBuffers-Item `tx_valid_to` für IPC-Nachrichten. |
| `pub fn business_valid_from(&self) -> i64` | `crates/contextra-wire/src/contextra_generated.rs:1469` | Öffentliches FlatBuffers-Item `business_valid_from` für IPC-Nachrichten. |
| `pub fn business_valid_to(&self) -> i64` | `crates/contextra-wire/src/contextra_generated.rs:1483` | Öffentliches FlatBuffers-Item `business_valid_to` für IPC-Nachrichten. |
| `pub fn source_doc_id(&self) -> u64` | `crates/contextra-wire/src/contextra_generated.rs:1497` | Öffentliches FlatBuffers-Item `source_doc_id` für IPC-Nachrichten. |
| `pub fn child_edge_ids(&self) -> Option<flatbuffers::Vector<'a, u64>>` | `crates/contextra-wire/src/contextra_generated.rs:1511` | Öffentliches FlatBuffers-Item `child_edge_ids` für IPC-Nachrichten. |

Assoziierte Konstanten/Typen:

- `pub const VT_ID: flatbuffers::VOffsetT = 4` — `crates/contextra-wire/src/contextra_generated.rs:1322`
- `pub const VT_PREDICATE: flatbuffers::VOffsetT = 6` — `crates/contextra-wire/src/contextra_generated.rs:1325`
- `pub const VT_PARTICIPANTS: flatbuffers::VOffsetT = 8` — `crates/contextra-wire/src/contextra_generated.rs:1328`
- `pub const VT_WEIGHT: flatbuffers::VOffsetT = 10` — `crates/contextra-wire/src/contextra_generated.rs:1331`
- `pub const VT_TX_VALID_FROM: flatbuffers::VOffsetT = 12` — `crates/contextra-wire/src/contextra_generated.rs:1334`
- `pub const VT_TX_VALID_TO: flatbuffers::VOffsetT = 14` — `crates/contextra-wire/src/contextra_generated.rs:1337`
- `pub const VT_BUSINESS_VALID_FROM: flatbuffers::VOffsetT = 16` — `crates/contextra-wire/src/contextra_generated.rs:1340`
- `pub const VT_BUSINESS_VALID_TO: flatbuffers::VOffsetT = 18` — `crates/contextra-wire/src/contextra_generated.rs:1343`
- `pub const VT_SOURCE_DOC_ID: flatbuffers::VOffsetT = 20` — `crates/contextra-wire/src/contextra_generated.rs:1346`
- `pub const VT_CHILD_EDGE_IDS: flatbuffers::VOffsetT = 22` — `crates/contextra-wire/src/contextra_generated.rs:1349`

#### `struct HyperEdgeArgs`
- Ort: `crates/contextra-wire/src/contextra_generated.rs:1556`
```rust
pub struct HyperEdgeArgs<'a>
```
Dokumentation: Öffentliches FlatBuffers-Item `HyperEdgeArgs` für IPC-Nachrichten.
- Impl-Block (Trait `Default`) `crates/contextra-wire/src/contextra_generated.rs:1572`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `id` | `u64` | `crates/contextra-wire/src/contextra_generated.rs:1557` |
| `predicate` | `u8` | `crates/contextra-wire/src/contextra_generated.rs:1558` |
| `participants` | `Option<` | `crates/contextra-wire/src/contextra_generated.rs:1559` |
| `weight` | `f32` | `crates/contextra-wire/src/contextra_generated.rs:1564` |
| `tx_valid_from` | `u64` | `crates/contextra-wire/src/contextra_generated.rs:1565` |
| `tx_valid_to` | `u64` | `crates/contextra-wire/src/contextra_generated.rs:1566` |
| `business_valid_from` | `i64` | `crates/contextra-wire/src/contextra_generated.rs:1567` |
| `business_valid_to` | `i64` | `crates/contextra-wire/src/contextra_generated.rs:1568` |
| `source_doc_id` | `u64` | `crates/contextra-wire/src/contextra_generated.rs:1569` |

#### `struct HyperEdgeBuilder`
- Ort: `crates/contextra-wire/src/contextra_generated.rs:1592`
```rust
pub struct HyperEdgeBuilder<'a: 'b, 'b>
```
Dokumentation: Öffentliches FlatBuffers-Item `HyperEdgeBuilder` für IPC-Nachrichten.
- Impl-Block `crates/contextra-wire/src/contextra_generated.rs:1596`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn add_id(&mut self, id: u64)` | `crates/contextra-wire/src/contextra_generated.rs:1600` | Öffentliches FlatBuffers-Item `add_id` für IPC-Nachrichten. |
| `pub fn add_predicate(&mut self, predicate: u8)` | `crates/contextra-wire/src/contextra_generated.rs:1606` | Öffentliches FlatBuffers-Item `add_predicate` für IPC-Nachrichten. |
| `pub fn add_participants( &mut self, participants: flatbuffers::WIPOffset< flatbuffers::Vector<'b, flatbuffers::ForwardsUOffset<RoleBinding<'b>>>, >, )` | `crates/contextra-wire/src/contextra_generated.rs:1613` | Öffentliches FlatBuffers-Item `add_participants` für IPC-Nachrichten. |
| `pub fn add_weight(&mut self, weight: f32)` | `crates/contextra-wire/src/contextra_generated.rs:1627` | Öffentliches FlatBuffers-Item `add_weight` für IPC-Nachrichten. |
| `pub fn add_tx_valid_from(&mut self, tx_valid_from: u64)` | `crates/contextra-wire/src/contextra_generated.rs:1634` | Öffentliches FlatBuffers-Item `add_tx_valid_from` für IPC-Nachrichten. |
| `pub fn add_tx_valid_to(&mut self, tx_valid_to: u64)` | `crates/contextra-wire/src/contextra_generated.rs:1641` | Öffentliches FlatBuffers-Item `add_tx_valid_to` für IPC-Nachrichten. |
| `pub fn add_business_valid_from(&mut self, business_valid_from: i64)` | `crates/contextra-wire/src/contextra_generated.rs:1648` | Öffentliches FlatBuffers-Item `add_business_valid_from` für IPC-Nachrichten. |
| `pub fn add_business_valid_to(&mut self, business_valid_to: i64)` | `crates/contextra-wire/src/contextra_generated.rs:1658` | Öffentliches FlatBuffers-Item `add_business_valid_to` für IPC-Nachrichten. |
| `pub fn add_source_doc_id(&mut self, source_doc_id: u64)` | `crates/contextra-wire/src/contextra_generated.rs:1665` | Öffentliches FlatBuffers-Item `add_source_doc_id` für IPC-Nachrichten. |
| `pub fn add_child_edge_ids( &mut self, child_edge_ids: flatbuffers::WIPOffset<flatbuffers::Vector<'b, u64>>, )` | `crates/contextra-wire/src/contextra_generated.rs:1672` | Öffentliches FlatBuffers-Item `add_child_edge_ids` für IPC-Nachrichten. |
| `pub fn new( _fbb: &'b mut flatbuffers::FlatBufferBuilder<'a>, ) -> HyperEdgeBuilder<'a, 'b>` | `crates/contextra-wire/src/contextra_generated.rs:1684` | Öffentliches FlatBuffers-Item `new` für IPC-Nachrichten. |
| `pub fn finish(self) -> flatbuffers::WIPOffset<HyperEdge<'a>>` | `crates/contextra-wire/src/contextra_generated.rs:1696` | Öffentliches FlatBuffers-Item `finish` für IPC-Nachrichten. |

#### `struct HyperEdgeId`
- Ort: `crates/contextra-wire/src/contextra_generated.rs:1030`
```rust
pub struct HyperEdgeId<'a>
```
Dokumentation: Öffentliches FlatBuffers-Item `HyperEdgeId` für IPC-Nachrichten.
- Impl-Block (Trait `VerifiableRoot`) `crates/contextra-wire/src/adapter.rs:70`
- Impl-Block (Trait `flatbuffers::Follow<'a>`) `crates/contextra-wire/src/contextra_generated.rs:1034`
- Impl-Block `crates/contextra-wire/src/contextra_generated.rs:1045`
- Impl-Block (Trait `flatbuffers::Verifiable`) `crates/contextra-wire/src/contextra_generated.rs:1081`
- Impl-Block (Trait `core::fmt::Debug`) `crates/contextra-wire/src/contextra_generated.rs:1140`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `_tab` | `flatbuffers::Table<'a>` | `crates/contextra-wire/src/contextra_generated.rs:1031` |

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub unsafe fn init_from_table(table: flatbuffers::Table<'a>) -> Self` | `crates/contextra-wire/src/contextra_generated.rs:1054` | Öffentliches FlatBuffers-Item `init_from_table` für IPC-Nachrichten. |
| `pub fn create<'bldr: 'args, 'args: 'mut_bldr, 'mut_bldr>( _fbb: &'mut_bldr mut flatbuffers::FlatBufferBuilder<'bldr>, args: &'args HyperEdgeIdArgs, ) -> flatbuffers::WIPOffset<HyperEdgeId<'bldr>>` | `crates/contextra-wire/src/contextra_generated.rs:1060` | Öffentliches FlatBuffers-Item `create` für IPC-Nachrichten. |
| `pub fn id(&self) -> u64` | `crates/contextra-wire/src/contextra_generated.rs:1072` | Öffentliches FlatBuffers-Item `id` für IPC-Nachrichten. |

Assoziierte Konstanten/Typen:

- `pub const VT_ID: flatbuffers::VOffsetT = 4` — `crates/contextra-wire/src/contextra_generated.rs:1048`

#### `struct HyperEdgeIdArgs`
- Ort: `crates/contextra-wire/src/contextra_generated.rs:1096`
```rust
pub struct HyperEdgeIdArgs
```
Dokumentation: Öffentliches FlatBuffers-Item `HyperEdgeIdArgs` für IPC-Nachrichten.
- Impl-Block (Trait `Default`) `crates/contextra-wire/src/contextra_generated.rs:1099`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `id` | `u64` | `crates/contextra-wire/src/contextra_generated.rs:1097` |

#### `struct HyperEdgeIdBuilder`
- Ort: `crates/contextra-wire/src/contextra_generated.rs:1108`
```rust
pub struct HyperEdgeIdBuilder<'a: 'b, 'b>
```
Dokumentation: Öffentliches FlatBuffers-Item `HyperEdgeIdBuilder` für IPC-Nachrichten.
- Impl-Block `crates/contextra-wire/src/contextra_generated.rs:1112`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn add_id(&mut self, id: u64)` | `crates/contextra-wire/src/contextra_generated.rs:1116` | Öffentliches FlatBuffers-Item `add_id` für IPC-Nachrichten. |
| `pub fn new( _fbb: &'b mut flatbuffers::FlatBufferBuilder<'a>, ) -> HyperEdgeIdBuilder<'a, 'b>` | `crates/contextra-wire/src/contextra_generated.rs:1122` | Öffentliches FlatBuffers-Item `new` für IPC-Nachrichten. |
| `pub fn finish(self) -> flatbuffers::WIPOffset<HyperEdgeId<'a>>` | `crates/contextra-wire/src/contextra_generated.rs:1134` | Öffentliches FlatBuffers-Item `finish` für IPC-Nachrichten. |

#### `struct RoleBinding`
- Ort: `crates/contextra-wire/src/contextra_generated.rs:1153`
```rust
pub struct RoleBinding<'a>
```
Dokumentation: Öffentliches FlatBuffers-Item `RoleBinding` für IPC-Nachrichten.
- Impl-Block (Trait `VerifiableRoot`) `crates/contextra-wire/src/adapter.rs:74`
- Impl-Block (Trait `flatbuffers::Follow<'a>`) `crates/contextra-wire/src/contextra_generated.rs:1157`
- Impl-Block `crates/contextra-wire/src/contextra_generated.rs:1168`
- Impl-Block (Trait `flatbuffers::Verifiable`) `crates/contextra-wire/src/contextra_generated.rs:1222`
- Impl-Block (Trait `core::fmt::Debug`) `crates/contextra-wire/src/contextra_generated.rs:1290`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `_tab` | `flatbuffers::Table<'a>` | `crates/contextra-wire/src/contextra_generated.rs:1154` |

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub unsafe fn init_from_table(table: flatbuffers::Table<'a>) -> Self` | `crates/contextra-wire/src/contextra_generated.rs:1180` | Öffentliches FlatBuffers-Item `init_from_table` für IPC-Nachrichten. |
| `pub fn create<'bldr: 'args, 'args: 'mut_bldr, 'mut_bldr>( _fbb: &'mut_bldr mut flatbuffers::FlatBufferBuilder<'bldr>, args: &'args RoleBindingArgs, ) -> flatbuffers::WIPOffset<RoleBinding<'bldr>>` | `crates/contextra-wire/src/contextra_generated.rs:1186` | Öffentliches FlatBuffers-Item `create` für IPC-Nachrichten. |
| `pub fn role(&self) -> u32` | `crates/contextra-wire/src/contextra_generated.rs:1199` | Öffentliches FlatBuffers-Item `role` für IPC-Nachrichten. |
| `pub fn entity(&self) -> u64` | `crates/contextra-wire/src/contextra_generated.rs:1209` | Öffentliches FlatBuffers-Item `entity` für IPC-Nachrichten. |

Assoziierte Konstanten/Typen:

- `pub const VT_ROLE: flatbuffers::VOffsetT = 4` — `crates/contextra-wire/src/contextra_generated.rs:1171`
- `pub const VT_ENTITY: flatbuffers::VOffsetT = 6` — `crates/contextra-wire/src/contextra_generated.rs:1174`

#### `struct RoleBindingArgs`
- Ort: `crates/contextra-wire/src/contextra_generated.rs:1238`
```rust
pub struct RoleBindingArgs
```
Dokumentation: Öffentliches FlatBuffers-Item `RoleBindingArgs` für IPC-Nachrichten.
- Impl-Block (Trait `Default`) `crates/contextra-wire/src/contextra_generated.rs:1242`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `role` | `u32` | `crates/contextra-wire/src/contextra_generated.rs:1239` |
| `entity` | `u64` | `crates/contextra-wire/src/contextra_generated.rs:1240` |

#### `struct RoleBindingBuilder`
- Ort: `crates/contextra-wire/src/contextra_generated.rs:1251`
```rust
pub struct RoleBindingBuilder<'a: 'b, 'b>
```
Dokumentation: Öffentliches FlatBuffers-Item `RoleBindingBuilder` für IPC-Nachrichten.
- Impl-Block `crates/contextra-wire/src/contextra_generated.rs:1255`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn add_role(&mut self, role: u32)` | `crates/contextra-wire/src/contextra_generated.rs:1259` | Öffentliches FlatBuffers-Item `add_role` für IPC-Nachrichten. |
| `pub fn add_entity(&mut self, entity: u64)` | `crates/contextra-wire/src/contextra_generated.rs:1265` | Öffentliches FlatBuffers-Item `add_entity` für IPC-Nachrichten. |
| `pub fn new( _fbb: &'b mut flatbuffers::FlatBufferBuilder<'a>, ) -> RoleBindingBuilder<'a, 'b>` | `crates/contextra-wire/src/contextra_generated.rs:1272` | Öffentliches FlatBuffers-Item `new` für IPC-Nachrichten. |
| `pub fn finish(self) -> flatbuffers::WIPOffset<RoleBinding<'a>>` | `crates/contextra-wire/src/contextra_generated.rs:1284` | Öffentliches FlatBuffers-Item `finish` für IPC-Nachrichten. |

#### `struct RoleId`
- Ort: `crates/contextra-wire/src/contextra_generated.rs:909`
```rust
pub struct RoleId<'a>
```
Dokumentation: Öffentliches FlatBuffers-Item `RoleId` für IPC-Nachrichten.
- Impl-Block (Trait `VerifiableRoot`) `crates/contextra-wire/src/adapter.rs:66`
- Impl-Block (Trait `flatbuffers::Follow<'a>`) `crates/contextra-wire/src/contextra_generated.rs:913`
- Impl-Block `crates/contextra-wire/src/contextra_generated.rs:924`
- Impl-Block (Trait `flatbuffers::Verifiable`) `crates/contextra-wire/src/contextra_generated.rs:960`
- Impl-Block (Trait `core::fmt::Debug`) `crates/contextra-wire/src/contextra_generated.rs:1017`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `_tab` | `flatbuffers::Table<'a>` | `crates/contextra-wire/src/contextra_generated.rs:910` |

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub unsafe fn init_from_table(table: flatbuffers::Table<'a>) -> Self` | `crates/contextra-wire/src/contextra_generated.rs:933` | Öffentliches FlatBuffers-Item `init_from_table` für IPC-Nachrichten. |
| `pub fn create<'bldr: 'args, 'args: 'mut_bldr, 'mut_bldr>( _fbb: &'mut_bldr mut flatbuffers::FlatBufferBuilder<'bldr>, args: &'args RoleIdArgs, ) -> flatbuffers::WIPOffset<RoleId<'bldr>>` | `crates/contextra-wire/src/contextra_generated.rs:939` | Öffentliches FlatBuffers-Item `create` für IPC-Nachrichten. |
| `pub fn id(&self) -> u32` | `crates/contextra-wire/src/contextra_generated.rs:951` | Öffentliches FlatBuffers-Item `id` für IPC-Nachrichten. |

Assoziierte Konstanten/Typen:

- `pub const VT_ID: flatbuffers::VOffsetT = 4` — `crates/contextra-wire/src/contextra_generated.rs:927`

#### `struct RoleIdArgs`
- Ort: `crates/contextra-wire/src/contextra_generated.rs:975`
```rust
pub struct RoleIdArgs
```
Dokumentation: Öffentliches FlatBuffers-Item `RoleIdArgs` für IPC-Nachrichten.
- Impl-Block (Trait `Default`) `crates/contextra-wire/src/contextra_generated.rs:978`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `id` | `u32` | `crates/contextra-wire/src/contextra_generated.rs:976` |

#### `struct RoleIdBuilder`
- Ort: `crates/contextra-wire/src/contextra_generated.rs:987`
```rust
pub struct RoleIdBuilder<'a: 'b, 'b>
```
Dokumentation: Öffentliches FlatBuffers-Item `RoleIdBuilder` für IPC-Nachrichten.
- Impl-Block `crates/contextra-wire/src/contextra_generated.rs:991`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn add_id(&mut self, id: u32)` | `crates/contextra-wire/src/contextra_generated.rs:995` | Öffentliches FlatBuffers-Item `add_id` für IPC-Nachrichten. |
| `pub fn new(_fbb: &'b mut flatbuffers::FlatBufferBuilder<'a>) -> RoleIdBuilder<'a, 'b>` | `crates/contextra-wire/src/contextra_generated.rs:1001` | Öffentliches FlatBuffers-Item `new` für IPC-Nachrichten. |
| `pub fn finish(self) -> flatbuffers::WIPOffset<RoleId<'a>>` | `crates/contextra-wire/src/contextra_generated.rs:1011` | Öffentliches FlatBuffers-Item `finish` für IPC-Nachrichten. |

#### `struct ScoredDocument`
- Ort: `crates/contextra-wire/src/contextra_generated.rs:255`
```rust
pub struct ScoredDocument<'a>
```
Dokumentation: Öffentliches FlatBuffers-Item `ScoredDocument` für IPC-Nachrichten.
- Impl-Block (Trait `VerifiableRoot`) `crates/contextra-wire/src/adapter.rs:52`
- Impl-Block (Trait `flatbuffers::Follow<'a>`) `crates/contextra-wire/src/contextra_generated.rs:259`
- Impl-Block `crates/contextra-wire/src/contextra_generated.rs:270`
- Impl-Block (Trait `flatbuffers::Verifiable`) `crates/contextra-wire/src/contextra_generated.rs:371`
- Impl-Block (Trait `core::fmt::Debug`) `crates/contextra-wire/src/contextra_generated.rs:476`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `_tab` | `flatbuffers::Table<'a>` | `crates/contextra-wire/src/contextra_generated.rs:256` |

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub unsafe fn init_from_table(table: flatbuffers::Table<'a>) -> Self` | `crates/contextra-wire/src/contextra_generated.rs:288` | Öffentliches FlatBuffers-Item `init_from_table` für IPC-Nachrichten. |
| `pub fn create<'bldr: 'args, 'args: 'mut_bldr, 'mut_bldr>( _fbb: &'mut_bldr mut flatbuffers::FlatBufferBuilder<'bldr>, args: &'args ScoredDocumentArgs<'args>, ) -> flatbuffers::WIPOffset<ScoredDocument<'bldr>>` | `crates/contextra-wire/src/contextra_generated.rs:294` | Öffentliches FlatBuffers-Item `create` für IPC-Nachrichten. |
| `pub fn id(&self) -> Option<&'a str>` | `crates/contextra-wire/src/contextra_generated.rs:315` | Öffentliches FlatBuffers-Item `id` für IPC-Nachrichten. |
| `pub fn score(&self) -> f32` | `crates/contextra-wire/src/contextra_generated.rs:328` | Öffentliches FlatBuffers-Item `score` für IPC-Nachrichten. |
| `pub fn metadata(&self) -> Option<&'a str>` | `crates/contextra-wire/src/contextra_generated.rs:342` | Öffentliches FlatBuffers-Item `metadata` für IPC-Nachrichten. |
| `pub fn embedding(&self) -> Option<Embedding<'a>>` | `crates/contextra-wire/src/contextra_generated.rs:357` | Öffentliches FlatBuffers-Item `embedding` für IPC-Nachrichten. |

Assoziierte Konstanten/Typen:

- `pub const VT_ID: flatbuffers::VOffsetT = 4` — `crates/contextra-wire/src/contextra_generated.rs:273`
- `pub const VT_SCORE: flatbuffers::VOffsetT = 6` — `crates/contextra-wire/src/contextra_generated.rs:276`
- `pub const VT_METADATA: flatbuffers::VOffsetT = 8` — `crates/contextra-wire/src/contextra_generated.rs:279`
- `pub const VT_EMBEDDING: flatbuffers::VOffsetT = 10` — `crates/contextra-wire/src/contextra_generated.rs:282`

#### `struct ScoredDocumentArgs`
- Ort: `crates/contextra-wire/src/contextra_generated.rs:397`
```rust
pub struct ScoredDocumentArgs<'a>
```
Dokumentation: Öffentliches FlatBuffers-Item `ScoredDocumentArgs` für IPC-Nachrichten.
- Impl-Block (Trait `Default`) `crates/contextra-wire/src/contextra_generated.rs:403`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `id` | `Option<flatbuffers::WIPOffset<&'a str>>` | `crates/contextra-wire/src/contextra_generated.rs:398` |
| `score` | `f32` | `crates/contextra-wire/src/contextra_generated.rs:399` |
| `metadata` | `Option<flatbuffers::WIPOffset<&'a str>>` | `crates/contextra-wire/src/contextra_generated.rs:400` |
| `embedding` | `Option<flatbuffers::WIPOffset<Embedding<'a>>>` | `crates/contextra-wire/src/contextra_generated.rs:401` |

#### `struct ScoredDocumentBuilder`
- Ort: `crates/contextra-wire/src/contextra_generated.rs:417`
```rust
pub struct ScoredDocumentBuilder<'a: 'b, 'b>
```
Dokumentation: Öffentliches FlatBuffers-Item `ScoredDocumentBuilder` für IPC-Nachrichten.
- Impl-Block `crates/contextra-wire/src/contextra_generated.rs:421`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn add_id(&mut self, id: flatbuffers::WIPOffset<&'b str>)` | `crates/contextra-wire/src/contextra_generated.rs:425` | Öffentliches FlatBuffers-Item `add_id` für IPC-Nachrichten. |
| `pub fn add_score(&mut self, score: f32)` | `crates/contextra-wire/src/contextra_generated.rs:432` | Öffentliches FlatBuffers-Item `add_score` für IPC-Nachrichten. |
| `pub fn add_metadata(&mut self, metadata: flatbuffers::WIPOffset<&'b str>)` | `crates/contextra-wire/src/contextra_generated.rs:439` | Öffentliches FlatBuffers-Item `add_metadata` für IPC-Nachrichten. |
| `pub fn add_embedding(&mut self, embedding: flatbuffers::WIPOffset<Embedding<'b>>)` | `crates/contextra-wire/src/contextra_generated.rs:448` | Öffentliches FlatBuffers-Item `add_embedding` für IPC-Nachrichten. |
| `pub fn new( _fbb: &'b mut flatbuffers::FlatBufferBuilder<'a>, ) -> ScoredDocumentBuilder<'a, 'b>` | `crates/contextra-wire/src/contextra_generated.rs:458` | Öffentliches FlatBuffers-Item `new` für IPC-Nachrichten. |
| `pub fn finish(self) -> flatbuffers::WIPOffset<ScoredDocument<'a>>` | `crates/contextra-wire/src/contextra_generated.rs:470` | Öffentliches FlatBuffers-Item `finish` für IPC-Nachrichten. |

#### `struct SearchResponse`
- Ort: `crates/contextra-wire/src/contextra_generated.rs:492`
```rust
pub struct SearchResponse<'a>
```
Dokumentation: Öffentliches FlatBuffers-Item `SearchResponse` für IPC-Nachrichten.
- Impl-Block (Trait `VerifiableRoot`) `crates/contextra-wire/src/adapter.rs:56`
- Impl-Block (Trait `flatbuffers::Follow<'a>`) `crates/contextra-wire/src/contextra_generated.rs:496`
- Impl-Block `crates/contextra-wire/src/contextra_generated.rs:507`
- Impl-Block (Trait `flatbuffers::Verifiable`) `crates/contextra-wire/src/contextra_generated.rs:588`
- Impl-Block (Trait `core::fmt::Debug`) `crates/contextra-wire/src/contextra_generated.rs:686`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `_tab` | `flatbuffers::Table<'a>` | `crates/contextra-wire/src/contextra_generated.rs:493` |

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub unsafe fn init_from_table(table: flatbuffers::Table<'a>) -> Self` | `crates/contextra-wire/src/contextra_generated.rs:522` | Öffentliches FlatBuffers-Item `init_from_table` für IPC-Nachrichten. |
| `pub fn create<'bldr: 'args, 'args: 'mut_bldr, 'mut_bldr>( _fbb: &'mut_bldr mut flatbuffers::FlatBufferBuilder<'bldr>, args: &'args SearchResponseArgs<'args>, ) -> flatbuffers::WIPOffset<SearchResponse<'bldr>>` | `crates/contextra-wire/src/contextra_generated.rs:528` | Öffentliches FlatBuffers-Item `create` für IPC-Nachrichten. |
| `pub fn results( &self, ) -> Option<flatbuffers::Vector<'a, flatbuffers::ForwardsUOffset<ScoredDocument<'a>>>>` | `crates/contextra-wire/src/contextra_generated.rs:544` | Öffentliches FlatBuffers-Item `results` für IPC-Nachrichten. |
| `pub fn total_hits(&self) -> u32` | `crates/contextra-wire/src/contextra_generated.rs:561` | Öffentliches FlatBuffers-Item `total_hits` für IPC-Nachrichten. |
| `pub fn processing_time_ms(&self) -> f32` | `crates/contextra-wire/src/contextra_generated.rs:575` | Öffentliches FlatBuffers-Item `processing_time_ms` für IPC-Nachrichten. |

Assoziierte Konstanten/Typen:

- `pub const VT_RESULTS: flatbuffers::VOffsetT = 4` — `crates/contextra-wire/src/contextra_generated.rs:510`
- `pub const VT_TOTAL_HITS: flatbuffers::VOffsetT = 6` — `crates/contextra-wire/src/contextra_generated.rs:513`
- `pub const VT_PROCESSING_TIME_MS: flatbuffers::VOffsetT = 8` — `crates/contextra-wire/src/contextra_generated.rs:516`

#### `struct SearchResponseArgs`
- Ort: `crates/contextra-wire/src/contextra_generated.rs:607`
```rust
pub struct SearchResponseArgs<'a>
```
Dokumentation: Öffentliches FlatBuffers-Item `SearchResponseArgs` für IPC-Nachrichten.
- Impl-Block (Trait `Default`) `crates/contextra-wire/src/contextra_generated.rs:616`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `results` | `Option<` | `crates/contextra-wire/src/contextra_generated.rs:608` |
| `total_hits` | `u32` | `crates/contextra-wire/src/contextra_generated.rs:613` |
| `processing_time_ms` | `f32` | `crates/contextra-wire/src/contextra_generated.rs:614` |

#### `struct SearchResponseBuilder`
- Ort: `crates/contextra-wire/src/contextra_generated.rs:629`
```rust
pub struct SearchResponseBuilder<'a: 'b, 'b>
```
Dokumentation: Öffentliches FlatBuffers-Item `SearchResponseBuilder` für IPC-Nachrichten.
- Impl-Block `crates/contextra-wire/src/contextra_generated.rs:633`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn add_results( &mut self, results: flatbuffers::WIPOffset< flatbuffers::Vector<'b, flatbuffers::ForwardsUOffset<ScoredDocument<'b>>>, >, )` | `crates/contextra-wire/src/contextra_generated.rs:637` | Öffentliches FlatBuffers-Item `add_results` für IPC-Nachrichten. |
| `pub fn add_total_hits(&mut self, total_hits: u32)` | `crates/contextra-wire/src/contextra_generated.rs:651` | Öffentliches FlatBuffers-Item `add_total_hits` für IPC-Nachrichten. |
| `pub fn add_processing_time_ms(&mut self, processing_time_ms: f32)` | `crates/contextra-wire/src/contextra_generated.rs:658` | Öffentliches FlatBuffers-Item `add_processing_time_ms` für IPC-Nachrichten. |
| `pub fn new( _fbb: &'b mut flatbuffers::FlatBufferBuilder<'a>, ) -> SearchResponseBuilder<'a, 'b>` | `crates/contextra-wire/src/contextra_generated.rs:668` | Öffentliches FlatBuffers-Item `new` für IPC-Nachrichten. |
| `pub fn finish(self) -> flatbuffers::WIPOffset<SearchResponse<'a>>` | `crates/contextra-wire/src/contextra_generated.rs:680` | Öffentliches FlatBuffers-Item `finish` für IPC-Nachrichten. |

#### `struct VectorIndexUpdate`
- Ort: `crates/contextra-wire/src/contextra_generated.rs:701`
```rust
pub struct VectorIndexUpdate<'a>
```
Dokumentation: Öffentliches FlatBuffers-Item `VectorIndexUpdate` für IPC-Nachrichten.
- Impl-Block (Trait `flatbuffers::Follow<'a>`) `crates/contextra-wire/src/contextra_generated.rs:705`
- Impl-Block `crates/contextra-wire/src/contextra_generated.rs:716`
- Impl-Block (Trait `flatbuffers::Verifiable`) `crates/contextra-wire/src/contextra_generated.rs:799`
- Impl-Block (Trait `core::fmt::Debug`) `crates/contextra-wire/src/contextra_generated.rs:894`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `_tab` | `flatbuffers::Table<'a>` | `crates/contextra-wire/src/contextra_generated.rs:702` |

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub unsafe fn init_from_table(table: flatbuffers::Table<'a>) -> Self` | `crates/contextra-wire/src/contextra_generated.rs:731` | Öffentliches FlatBuffers-Item `init_from_table` für IPC-Nachrichten. |
| `pub fn create<'bldr: 'args, 'args: 'mut_bldr, 'mut_bldr>( _fbb: &'mut_bldr mut flatbuffers::FlatBufferBuilder<'bldr>, args: &'args VectorIndexUpdateArgs<'args>, ) -> flatbuffers::WIPOffset<VectorIndexUpdate<'bldr>>` | `crates/contextra-wire/src/contextra_generated.rs:737` | Öffentliches FlatBuffers-Item `create` für IPC-Nachrichten. |
| `pub fn id(&self) -> Option<&'a str>` | `crates/contextra-wire/src/contextra_generated.rs:757` | Öffentliches FlatBuffers-Item `id` für IPC-Nachrichten. |
| `pub fn embedding(&self) -> Option<Embedding<'a>>` | `crates/contextra-wire/src/contextra_generated.rs:770` | Öffentliches FlatBuffers-Item `embedding` für IPC-Nachrichten. |
| `pub fn metadata(&self) -> Option<&'a str>` | `crates/contextra-wire/src/contextra_generated.rs:785` | Öffentliches FlatBuffers-Item `metadata` für IPC-Nachrichten. |

Assoziierte Konstanten/Typen:

- `pub const VT_ID: flatbuffers::VOffsetT = 4` — `crates/contextra-wire/src/contextra_generated.rs:719`
- `pub const VT_EMBEDDING: flatbuffers::VOffsetT = 6` — `crates/contextra-wire/src/contextra_generated.rs:722`
- `pub const VT_METADATA: flatbuffers::VOffsetT = 8` — `crates/contextra-wire/src/contextra_generated.rs:725`

#### `struct VectorIndexUpdateArgs`
- Ort: `crates/contextra-wire/src/contextra_generated.rs:824`
```rust
pub struct VectorIndexUpdateArgs<'a>
```
Dokumentation: Öffentliches FlatBuffers-Item `VectorIndexUpdateArgs` für IPC-Nachrichten.
- Impl-Block (Trait `Default`) `crates/contextra-wire/src/contextra_generated.rs:829`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `id` | `Option<flatbuffers::WIPOffset<&'a str>>` | `crates/contextra-wire/src/contextra_generated.rs:825` |
| `embedding` | `Option<flatbuffers::WIPOffset<Embedding<'a>>>` | `crates/contextra-wire/src/contextra_generated.rs:826` |
| `metadata` | `Option<flatbuffers::WIPOffset<&'a str>>` | `crates/contextra-wire/src/contextra_generated.rs:827` |

#### `struct VectorIndexUpdateBuilder`
- Ort: `crates/contextra-wire/src/contextra_generated.rs:842`
```rust
pub struct VectorIndexUpdateBuilder<'a: 'b, 'b>
```
Dokumentation: Öffentliches FlatBuffers-Item `VectorIndexUpdateBuilder` für IPC-Nachrichten.
- Impl-Block `crates/contextra-wire/src/contextra_generated.rs:846`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn add_id(&mut self, id: flatbuffers::WIPOffset<&'b str>)` | `crates/contextra-wire/src/contextra_generated.rs:850` | Öffentliches FlatBuffers-Item `add_id` für IPC-Nachrichten. |
| `pub fn add_embedding(&mut self, embedding: flatbuffers::WIPOffset<Embedding<'b>>)` | `crates/contextra-wire/src/contextra_generated.rs:857` | Öffentliches FlatBuffers-Item `add_embedding` für IPC-Nachrichten. |
| `pub fn add_metadata(&mut self, metadata: flatbuffers::WIPOffset<&'b str>)` | `crates/contextra-wire/src/contextra_generated.rs:867` | Öffentliches FlatBuffers-Item `add_metadata` für IPC-Nachrichten. |
| `pub fn new( _fbb: &'b mut flatbuffers::FlatBufferBuilder<'a>, ) -> VectorIndexUpdateBuilder<'a, 'b>` | `crates/contextra-wire/src/contextra_generated.rs:876` | Öffentliches FlatBuffers-Item `new` für IPC-Nachrichten. |
| `pub fn finish(self) -> flatbuffers::WIPOffset<VectorIndexUpdate<'a>>` | `crates/contextra-wire/src/contextra_generated.rs:888` | Öffentliches FlatBuffers-Item `finish` für IPC-Nachrichten. |

#### `enum EmbeddingOffset`
- Ort: `crates/contextra-wire/src/contextra_generated.rs:47`
```rust
pub enum EmbeddingOffset
```
Dokumentation: Öffentliches FlatBuffers-Item `EmbeddingOffset` für IPC-Nachrichten.

#### `enum HyperEdgeIdOffset`
- Ort: `crates/contextra-wire/src/contextra_generated.rs:1026`
```rust
pub enum HyperEdgeIdOffset
```
Dokumentation: Öffentliches FlatBuffers-Item `HyperEdgeIdOffset` für IPC-Nachrichten.

#### `enum HyperEdgeOffset`
- Ort: `crates/contextra-wire/src/contextra_generated.rs:1300`
```rust
pub enum HyperEdgeOffset
```
Dokumentation: Öffentliches FlatBuffers-Item `HyperEdgeOffset` für IPC-Nachrichten.

#### `enum RoleBindingOffset`
- Ort: `crates/contextra-wire/src/contextra_generated.rs:1149`
```rust
pub enum RoleBindingOffset
```
Dokumentation: Öffentliches FlatBuffers-Item `RoleBindingOffset` für IPC-Nachrichten.

#### `enum RoleIdOffset`
- Ort: `crates/contextra-wire/src/contextra_generated.rs:905`
```rust
pub enum RoleIdOffset
```
Dokumentation: Öffentliches FlatBuffers-Item `RoleIdOffset` für IPC-Nachrichten.

#### `enum ScoredDocumentOffset`
- Ort: `crates/contextra-wire/src/contextra_generated.rs:251`
```rust
pub enum ScoredDocumentOffset
```
Dokumentation: Öffentliches FlatBuffers-Item `ScoredDocumentOffset` für IPC-Nachrichten.

#### `enum SearchResponseOffset`
- Ort: `crates/contextra-wire/src/contextra_generated.rs:488`
```rust
pub enum SearchResponseOffset
```
Dokumentation: Öffentliches FlatBuffers-Item `SearchResponseOffset` für IPC-Nachrichten.

#### `enum VectorIndexUpdateOffset`
- Ort: `crates/contextra-wire/src/contextra_generated.rs:697`
```rust
pub enum VectorIndexUpdateOffset
```
Dokumentation: Öffentliches FlatBuffers-Item `VectorIndexUpdateOffset` für IPC-Nachrichten.

#### `fn finish_search_response_buffer`
- Ort: `crates/contextra-wire/src/contextra_generated.rs:1801`
```rust
pub fn finish_search_response_buffer<'a, 'b>( fbb: &'b mut flatbuffers::FlatBufferBuilder<'a>, root: flatbuffers::WIPOffset<SearchResponse<'a>>, )
```
Dokumentation: Öffentliches FlatBuffers-Item `finish_search_response_buffer` für IPC-Nachrichten.

#### `fn finish_size_prefixed_search_response_buffer`
- Ort: `crates/contextra-wire/src/contextra_generated.rs:1811`
```rust
pub fn finish_size_prefixed_search_response_buffer<'a, 'b>( fbb: &'b mut flatbuffers::FlatBufferBuilder<'a>, root: flatbuffers::WIPOffset<SearchResponse<'a>>, )
```
Dokumentation: Öffentliches FlatBuffers-Item `finish_size_prefixed_search_response_buffer` für IPC-Nachrichten.

#### `fn root_as_search_response`
- Ort: `crates/contextra-wire/src/contextra_generated.rs:1727`
```rust
pub fn root_as_search_response( buf: &[u8], ) -> Result<SearchResponse, flatbuffers::InvalidFlatbuffer>
```
Dokumentation: Öffentliches FlatBuffers-Item `root_as_search_response` für IPC-Nachrichten.

#### `fn root_as_search_response_unchecked`
- Ort: `crates/contextra-wire/src/contextra_generated.rs:1783`
```rust
pub unsafe fn root_as_search_response_unchecked(buf: &[u8]) -> SearchResponse
```
Dokumentation: Öffentliches FlatBuffers-Item `root_as_search_response_unchecked` für IPC-Nachrichten.

#### `fn root_as_search_response_with_opts`
- Ort: `crates/contextra-wire/src/contextra_generated.rs:1755`
```rust
pub fn root_as_search_response_with_opts<'b, 'o>( opts: &'o flatbuffers::VerifierOptions, buf: &'b [u8], ) -> Result<SearchResponse<'b>, flatbuffers::InvalidFlatbuffer>
```
Dokumentation: Öffentliches FlatBuffers-Item `root_as_search_response_with_opts` für IPC-Nachrichten.

#### `fn size_prefixed_root_as_search_response`
- Ort: `crates/contextra-wire/src/contextra_generated.rs:1741`
```rust
pub fn size_prefixed_root_as_search_response( buf: &[u8], ) -> Result<SearchResponse, flatbuffers::InvalidFlatbuffer>
```
Dokumentation: Öffentliches FlatBuffers-Item `size_prefixed_root_as_search_response` für IPC-Nachrichten.

#### `fn size_prefixed_root_as_search_response_unchecked`
- Ort: `crates/contextra-wire/src/contextra_generated.rs:1793`
```rust
pub unsafe fn size_prefixed_root_as_search_response_unchecked( buf: &[u8], ) -> SearchResponse
```
Dokumentation: Öffentliches FlatBuffers-Item `size_prefixed_root_as_search_response_unchecked` für IPC-Nachrichten.

#### `fn size_prefixed_root_as_search_response_with_opts`
- Ort: `crates/contextra-wire/src/contextra_generated.rs:1770`
```rust
pub fn size_prefixed_root_as_search_response_with_opts<'b, 'o>( opts: &'o flatbuffers::VerifierOptions, buf: &'b [u8], ) -> Result<SearchResponse<'b>, flatbuffers::InvalidFlatbuffer>
```
Dokumentation: Öffentliches FlatBuffers-Item `size_prefixed_root_as_search_response_with_opts` für IPC-Nachrichten.

### Modul `crate`

#### `mod adapter`
- Ort: `crates/contextra-wire/src/lib.rs:44`
```rust
pub mod adapter
```

#### `mod generated`
- Ort: `crates/contextra-wire/src/lib.rs:40`
```rust
pub mod generated
```

#### `mod jsonrpc`
- Ort: `crates/contextra-wire/src/lib.rs:45`
```rust
pub mod jsonrpc
```

#### `mod verification`
- Ort: `crates/contextra-wire/src/lib.rs:46`
```rust
pub mod verification
```

#### `enum Error`
- Ort: `crates/contextra-wire/src/lib.rs:61`
```rust
pub enum Error
```
Dokumentation: Zentraler, typisierter Fehlertyp für `contextra-wire` gemäß SPEC v4.0 (B4). Fasst FlatBuffers-Verifikationsfehler und JSON-RPC-Protokollfehler zusammen.

Varianten:

- `FlatBuffer(#[from] FlatBufferError)` — `crates/contextra-wire/src/lib.rs:64`
- `JsonRpc(#[from] JsonRpcError)` — `crates/contextra-wire/src/lib.rs:68`
- `InvalidInput(&'static str)` — `crates/contextra-wire/src/lib.rs:72`

#### `type Result`
- Ort: `crates/contextra-wire/src/lib.rs:76`
```rust
pub type Result<T> = core::result::Result<T, Error>
```
Dokumentation: Standard-Result-Typ für `contextra-wire`.

### Modul `generated`

#### `mod contextra_generated`
- Ort: `crates/contextra-wire/src/generated.rs:38`
```rust
pub mod contextra_generated
```

### Modul `jsonrpc`

#### `struct JsonRpcConfig`
- Ort: `crates/contextra-wire/src/jsonrpc.rs:33`
```rust
pub struct JsonRpcConfig
```
Dokumentation: Configurable limits and security parameters for JSON-RPC message processing. Konfigurierbare Obergrenzen und Sicherheitsparameter für die JSON-RPC-Verarbeitung.
- Impl-Block (Trait `Default`) `crates/contextra-wire/src/jsonrpc.rs:48`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `max_message_size` | `usize` | `crates/contextra-wire/src/jsonrpc.rs:36` |
| `max_depth` | `usize` | `crates/contextra-wire/src/jsonrpc.rs:39` |
| `max_batch_size` | `usize` | `crates/contextra-wire/src/jsonrpc.rs:42` |
| `reject_duplicate_keys` | `bool` | `crates/contextra-wire/src/jsonrpc.rs:45` |

#### `struct JsonRpcError`
- Ort: `crates/contextra-wire/src/jsonrpc.rs:149`
```rust
pub struct JsonRpcError
```
Dokumentation: JSON-RPC 2.0 Error object. Standardisiertes JSON-RPC 2.0 Fehlerobjekt.
- Impl-Block (Trait `std::fmt::Display`) `crates/contextra-wire/src/jsonrpc.rs:162`
- Impl-Block (Trait `std::error::Error`) `crates/contextra-wire/src/jsonrpc.rs:168`
- Impl-Block `crates/contextra-wire/src/jsonrpc.rs:170`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `code` | `i32` | `crates/contextra-wire/src/jsonrpc.rs:152` |
| `message` | `String` | `crates/contextra-wire/src/jsonrpc.rs:155` |
| `data` | `Option<Value>` | `crates/contextra-wire/src/jsonrpc.rs:159` |

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn parse_error(message: impl Into<String>) -> Self` | `crates/contextra-wire/src/jsonrpc.rs:174` | Parse error (-32700): Invalid JSON received by server. Erzeugt einen Parse error (-32700): Ungültiges JSON empfangen. |
| `pub fn invalid_request(message: impl Into<String>) -> Self` | `crates/contextra-wire/src/jsonrpc.rs:185` | Invalid Request (-32600): JSON sent is not a valid Request object. Erzeugt einen Invalid Request (-32600): Gesendetes JSON ist keine gültige Anfrage. |
| `pub fn method_not_found(message: impl Into<String>) -> Self` | `crates/contextra-wire/src/jsonrpc.rs:196` | Method not found (-32601): The method does not exist / is not available. Erzeugt einen Method not found (-32601): Methode existiert nicht oder ist nicht verfügb |
| `pub fn invalid_params(message: impl Into<String>) -> Self` | `crates/contextra-wire/src/jsonrpc.rs:207` | Invalid params (-32602): Invalid method parameter(s). Erzeugt einen Invalid params (-32602): Ungültige Methodenparameter übergeben. |
| `pub fn internal_error(message: impl Into<String>) -> Self` | `crates/contextra-wire/src/jsonrpc.rs:218` | Internal error (-32603): Internal JSON-RPC error. Erzeugt einen Internal error (-32603): Interner JSON-RPC Serverfehler. |

#### `struct JsonRpcRequest`
- Ort: `crates/contextra-wire/src/jsonrpc.rs:82`
```rust
pub struct JsonRpcRequest
```
Dokumentation: Incoming JSON-RPC 2.0 message (Request or Notification). Eingehende JSON-RPC 2.0-Nachricht (Anfrage oder Benachrichtigung). Handelt es sich um eine Benachrichtigung (Notification), ist das Feld `id` auf `None` gesetzt.
- Impl-Block `crates/contextra-wire/src/jsonrpc.rs:98`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `jsonrpc` | `String` | `crates/contextra-wire/src/jsonrpc.rs:85` |
| `id` | `Option<Value>` | `crates/contextra-wire/src/jsonrpc.rs:88` |
| `method` | `String` | `crates/contextra-wire/src/jsonrpc.rs:91` |
| `params` | `Value` | `crates/contextra-wire/src/jsonrpc.rs:95` |

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn validate(&self) -> Result<(), JsonRpcError>` | `crates/contextra-wire/src/jsonrpc.rs:106` | Validates that the request complies with the JSON-RPC 2.0 specification (fail-closed). # Ablauf 1. Prüfen, ob `jsonrpc` exakt `"2.0"` entspricht. Falls nicht -> |

#### `struct JsonRpcResponse`
- Ort: `crates/contextra-wire/src/jsonrpc.rs:127`
```rust
pub struct JsonRpcResponse
```
Dokumentation: Outgoing JSON-RPC 2.0 response. Ausgehende JSON-RPC 2.0-Antwort (Erfolg oder Fehler).
- Impl-Block `crates/contextra-wire/src/jsonrpc.rs:227`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `jsonrpc` | `String` | `crates/contextra-wire/src/jsonrpc.rs:130` |
| `id` | `Option<Value>` | `crates/contextra-wire/src/jsonrpc.rs:133` |
| `result` | `Option<Value>` | `crates/contextra-wire/src/jsonrpc.rs:137` |
| `error` | `Option<JsonRpcError>` | `crates/contextra-wire/src/jsonrpc.rs:141` |

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn ok(id: Option<Value>, result: Value) -> Self` | `crates/contextra-wire/src/jsonrpc.rs:236` | Constructs a successful JSON-RPC response. Konstruiert eine erfolgreiche JSON-RPC 2.0-Antwort mit Ergebnisdaten. # Parameter - `id`: ID der beantworteten Anfrag |
| `pub fn err(id: Option<Value>, code: i32, message: impl Into<String>) -> Self` | `crates/contextra-wire/src/jsonrpc.rs:254` | Constructs an error JSON-RPC response. Konstruiert eine JSON-RPC 2.0-Fehlerantwort. # Parameter - `id`: ID der Anfrage (oder `None`, falls unbekannt). - `code`: |

#### `enum JsonRpcPayload`
- Ort: `crates/contextra-wire/src/jsonrpc.rs:67`
```rust
pub enum JsonRpcPayload
```
Dokumentation: Parsed JSON-RPC payload representing either a single request or a batch of requests. Geparste JSON-RPC-Payload: Repräsentiert entweder eine Einzelanfrage oder einen Batch von Anfragen.

Varianten:

- `Single(JsonRpcRequest)` — `crates/contextra-wire/src/jsonrpc.rs:70`
- `Batch(Vec<JsonRpcRequest>)` — `crates/contextra-wire/src/jsonrpc.rs:73`

#### `fn parse_jsonrpc_payload`
- Ort: `crates/contextra-wire/src/jsonrpc.rs:272`
```rust
pub fn parse_jsonrpc_payload(raw: &[u8]) -> Result<JsonRpcPayload, JsonRpcError>
```
Dokumentation: Parses raw bytes into a `JsonRpcPayload` using default configuration limits. Parst rohe Bytes in ein `JsonRpcPayload` unter Verwendung der Standard-Konfiguration.

#### `fn parse_jsonrpc_payload_with_config`
- Ort: `crates/contextra-wire/src/jsonrpc.rs:294`
```rust
pub fn parse_jsonrpc_payload_with_config( raw: &[u8], config: &JsonRpcConfig, ) -> Result<JsonRpcPayload, JsonRpcError>
```
Dokumentation: Parses raw bytes into a `JsonRpcPayload` with strict security and size validation. Parst rohe Bytes in ein `JsonRpcPayload` mit strikter Sicherheits- und Größenvalidierung. # Ablauf von `parse_jsonrpc_payload_with_config`: 1. Puffergröße gegen `config.max_message_size` prüfen. 2. UTF-8-Konformität sicherstellen. 3. Vorvalidierung der Struktur via `validate_json_structure_and_limits` (Schachtelungstiefe und Duplikate). 4. Deserialisierung in generischen `serde_json::Value`. 5. Verzweigung nach Wurzelelement: - `Value::Array`: Batch-Verarbeitung. Prüfen auf Nicht-Leerheit und `max_batch_size`. J

### Modul `verification`

#### `struct FlatBufferConfig`
- Ort: `crates/contextra-wire/src/verification.rs:73`
```rust
pub struct FlatBufferConfig
```
Dokumentation: Configuration parameters for FlatBuffers verification. Definiert Schutzgrenzen gegen Denial-of-Service-Angriffe (DoS) durch übergroße oder tief geschachtelte Strukturen.
- Impl-Block (Trait `Default`) `crates/contextra-wire/src/verification.rs:85`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `max_message_size` | `usize` | `crates/contextra-wire/src/verification.rs:76` |
| `max_depth` | `usize` | `crates/contextra-wire/src/verification.rs:79` |
| `max_tables` | `usize` | `crates/contextra-wire/src/verification.rs:82` |

#### `enum FlatBufferError`
- Ort: `crates/contextra-wire/src/verification.rs:34`
```rust
pub enum FlatBufferError
```
Dokumentation: Error type for FlatBuffers verification failures. Signalisiert Verletzungen der Spezifikation oder Sicherheitsgrenzen beim Einlesen von Byte-Puffern.
- Impl-Block (Trait `fmt::Display`) `crates/contextra-wire/src/verification.rs:49`
- Impl-Block (Trait `std::error::Error`) `crates/contextra-wire/src/verification.rs:65`

Varianten:

- `EmptyBuffer` — `crates/contextra-wire/src/verification.rs:37`
- `BufferTooLarge{ size: usize, max_size: usize }` — `crates/contextra-wire/src/verification.rs:40`
- `VerificationFailed(String)` — `crates/contextra-wire/src/verification.rs:43`
- `IdentifierMismatch` — `crates/contextra-wire/src/verification.rs:46`

#### `fn verify_and_parse_embedding`
- Ort: `crates/contextra-wire/src/verification.rs:160`
```rust
pub fn verify_and_parse_embedding<'a>( buf: &'a [u8], config: &FlatBufferConfig, ) -> Result<Embedding<'a>, FlatBufferError>
```
Dokumentation: Verifies and parses an `Embedding` root table from bytes. # Ablauf von `verify_and_parse_embedding`: 1. Validiert Puffergrenzen und Tabellenstruktur für `Embedding`. 2. Stellt sicher, dass Vektor-Floats bzw. quantisierte Bytes im Pufferbereich liegen. 3. Gibt das verifizierte `Embedding` zurück.

#### `fn verify_and_parse_hyper_edge`
- Ort: `crates/contextra-wire/src/verification.rs:244`
```rust
pub fn verify_and_parse_hyper_edge<'a>( buf: &'a [u8], config: &FlatBufferConfig, ) -> Result<HyperEdge<'a>, FlatBufferError>
```
Dokumentation: Verifies and parses a `HyperEdge` root table from bytes. # Ablauf von `verify_and_parse_hyper_edge`: 1. Validiert Puffergrenzen und Tabellenstruktur für `HyperEdge`. 2. Verifiziert Hyperkanten-ID, Attribute und Rollenbindungen. 3. Gibt die verifizierte `HyperEdge` zurück.

#### `fn verify_and_parse_hyper_edge_id`
- Ort: `crates/contextra-wire/src/verification.rs:216`
```rust
pub fn verify_and_parse_hyper_edge_id<'a>( buf: &'a [u8], config: &FlatBufferConfig, ) -> Result<HyperEdgeId<'a>, FlatBufferError>
```
Dokumentation: Verifies and parses a `HyperEdgeId` root table from bytes. # Ablauf von `verify_and_parse_hyper_edge_id`: 1. Validiert Puffergrenzen und Tabellenstruktur für `HyperEdgeId`. 2. Verifiziert die u64-Hyperkanten-ID. 3. Gibt das verifizierte `HyperEdgeId` zurück.

#### `fn verify_and_parse_role_binding`
- Ort: `crates/contextra-wire/src/verification.rs:230`
```rust
pub fn verify_and_parse_role_binding<'a>( buf: &'a [u8], config: &FlatBufferConfig, ) -> Result<RoleBinding<'a>, FlatBufferError>
```
Dokumentation: Verifies and parses a `RoleBinding` root table from bytes. # Ablauf von `verify_and_parse_role_binding`: 1. Validiert Puffergrenzen und Tabellenstruktur für `RoleBinding`. 2. Verifiziert Rollen-ID und zugeordnete Knoten-IDs. 3. Gibt das verifizierte `RoleBinding` zurück.

#### `fn verify_and_parse_role_id`
- Ort: `crates/contextra-wire/src/verification.rs:202`
```rust
pub fn verify_and_parse_role_id<'a>( buf: &'a [u8], config: &FlatBufferConfig, ) -> Result<RoleId<'a>, FlatBufferError>
```
Dokumentation: Verifies and parses a `RoleId` root table from bytes. # Ablauf von `verify_and_parse_role_id`: 1. Validiert Puffergrenzen und Tabellenstruktur für `RoleId`. 2. Verifiziert die u32-Rollenidentifikation. 3. Gibt das verifizierte `RoleId` zurück.

#### `fn verify_and_parse_scored_document`
- Ort: `crates/contextra-wire/src/verification.rs:174`
```rust
pub fn verify_and_parse_scored_document<'a>( buf: &'a [u8], config: &FlatBufferConfig, ) -> Result<ScoredDocument<'a>, FlatBufferError>
```
Dokumentation: Verifies and parses a `ScoredDocument` root table from bytes. # Ablauf von `verify_and_parse_scored_document`: 1. Validiert Puffergrenzen und Tabellenstruktur für `ScoredDocument`. 2. Verifiziert Dokument-ID, Score, Metadaten-String und optionales Embedding. 3. Gibt das verifizierte `ScoredDocument` zurück.

#### `fn verify_and_parse_search_response`
- Ort: `crates/contextra-wire/src/verification.rs:146`
```rust
pub fn verify_and_parse_search_response<'a>( buf: &'a [u8], config: &FlatBufferConfig, ) -> Result<SearchResponse<'a>, FlatBufferError>
```
Dokumentation: Verifies and parses a `SearchResponse` root table from bytes. # Ablauf von `verify_and_parse_search_response`: 1. Übergibt Puffer und Konfiguration an `verify_flatbuffer::<SearchResponse<'a>>`. 2. Validiert Header, Result-Vektor und Dokumente. 3. Gibt bei Erfolg die verifizierte `SearchResponse` zurück.

#### `fn verify_and_parse_vector_index_update`
- Ort: `crates/contextra-wire/src/verification.rs:188`
```rust
pub fn verify_and_parse_vector_index_update<'a>( buf: &'a [u8], config: &FlatBufferConfig, ) -> Result<VectorIndexUpdate<'a>, FlatBufferError>
```
Dokumentation: Verifies and parses a `VectorIndexUpdate` root table from bytes. # Ablauf von `verify_and_parse_vector_index_update`: 1. Validiert Puffergrenzen und Tabellenstruktur für `VectorIndexUpdate`. 2. Verifiziert Dokument-ID, Operationstyp und Vektordaten. 3. Gibt das verifizierte `VectorIndexUpdate` zurück.

#### `fn verify_flatbuffer`
- Ort: `crates/contextra-wire/src/verification.rs:111`
```rust
pub fn verify_flatbuffer<'a, T>( buf: &'a [u8], config: &FlatBufferConfig, ) -> Result<T::Inner, FlatBufferError> where T: flatbuffers::Follow<'a> + flatbuffers::Verifiable + 'a,
```
Dokumentation: Generic verified root accessor for any FlatBuffers root table `T`. # Ablauf von `verify_flatbuffer`: 1. Puffer prüfen: Ist der Puffer leer, wird sofort `Err(FlatBufferError::EmptyBuffer)` zurückgegeben. 2. Längenprüfung: Überschreitet `buf.len()` das Limit `config.max_message_size`, wird mit `Err(FlatBufferError::BufferTooLarge)` abgelehnt. 3. Verifier konfigurieren: Erstellen der `VerifierOptions` mit Limits für Tiefe, Tabellenanzahl und scheinbare Größe. 4. Strukturprüfung via `flatbuffers::root_with_opts::<T>`: Prüft Offsets, VTables und Feldausrichtung im Byte-Array. 5. Bei erfolgreicher V
