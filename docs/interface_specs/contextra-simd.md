# Schnittstellenspezifikation: `contextra-simd`

> Erzeugt durch `tools/extract_interfaces.py` aus dem Quellcode. Keine Handpflege. Jede Angabe trägt Datei:Zeile. Maßgeblich bleibt der Quellcode.

## 1. Kopf

| Feld | Wert |
|---|---|
| Name | `contextra-simd` |
| Version | workspace |
| Pfad | `crates/contextra-simd` |
| Rolle | SIMD-Kerne |
| Bibliothekspfad | src/lib.rs (Standard) |

**Crate-Dokumentation (//!):**

> # Crate `contextra-simd`
>
> **Ring-Zugehörigkeit:** Ring 0 (Kern, unsafe-Insel).
>
> **Zweck:**
> Diese Crate stellt hochoptimierte Vektor-Distanzberechnungen bereit.
> SIMD (Single Instruction, Multiple Data) nutzt spezielle Prozessorregister (z. B. 128-Bit bei ARM NEON,
> 256-Bit bei x86 AVX2, 512-Bit bei x86 AVX-512), um mehrere Gleitkomma- oder Ganzzahl-Operationen parallel
> mit einem einzigen CPU-Befehl auszuführen.
>
> **Verbindung im System:**
> - **Abhängigkeiten (normal):** `bytemuck`, `thiserror`. Optional: `contextra-types` (nur transitional via Feature `types-compat`), `serde`, `tracing`.

## 2. Abhängigkeiten

| Name | Art | Quelle | optional | verwendet von (normal/dev) |
|---|---|---|---|---|
| `bytemuck` | normal | registry | nein |  |
| `contextra-types` | normal | workspace | ja | contextra, contextra-adapt, contextra-agent, contextra-bench, contextra-checkpoint, contextra-cognition, contextra-crypto, contextra-crypto-fuzz, contextra-db, contextra-engine, contextra-graph, contextra-graph-fuzz, contextra-infer-candle, contextra-infer-ollama, contextra-infer-onnx, contextra-ingest, contextra-kvcache, contextra-mcp, contextra-mvcc, contextra-ports, contextra-py, contextra-rank, contextra-router, contextra-simd, contextra-store, contextra-store-fuzz, contextra-testkit, contextra-text, contextra-vault, contextra-vector, contextra-vector-fuzz |
| `criterion` | dev | registry | nein |  |
| `proptest` | dev | registry | nein |  |
| `serde` | normal | workspace | ja |  |
| `serde_json` | dev | workspace | nein |  |
| `simsimd` | dev | registry | nein |  |
| `thiserror` | normal | workspace | nein |  |
| `tracing` | normal | workspace | ja |  |

**Verwendet von (Crate-Ebene):** `contextra-vector`

## 3. Features

| Feature | Aktiviert |
|---|---|
| `default` | `std` |
| `serde` | `dep:serde` |
| `simd-avx512` | – |
| `std` | – |
| `types-compat` | `workspace` |
| `workspace` | `dep:contextra-types` |

## 4. Metriken (Quellcode)

| Kennzahl | Wert |
|---|---|
| Quelldateien | 11 |
| Quellzeilen | 3627 |
| Öffentliche Items (Modul-Ebene) | 47 |
| davon `fn` | 29 |
| davon `struct` | 4 |
| davon `enum` | 3 |
| davon `trait` | 1 |
| davon `type` | 3 |
| davon `mod` | 7 |
| Öffentliche Makros (`#[macro_export]`) | 0 |
| Öffentliche Re-Exports (`pub use`) | 9 |
| `unsafe`-Vorkommen (Produktionscode) | 286 |
| `.unwrap()` (Produktionscode) | 0 |
| `.expect(` (Produktionscode) | 4 |
| `panic!` (Produktionscode) | 0 |
| `todo!/unimplemented!/unreachable!` | 0 |
| Indexierungen `x[…]` (grob, Produktionscode) | 0 |
| `#[deprecated]` | 0 |
| `#[cfg(feature…)]`-Gates | 4 |
| Parser-Abdeckung (geparste `pub`-Deklarationen / Grep-Treffer) | 62 / 62 = 100 % |

## 5. Modulbaum und Dateien

| Modul | Datei | Zeilen |
|---|---|---|
| `compat` | `crates/contextra-simd/src/compat/mod.rs` | 144 |
| `contract` | `crates/contextra-simd/src/contract.rs` | 65 |
| `dispatch` | `crates/contextra-simd/src/dispatch.rs` | 1199 |
| `error` | `crates/contextra-simd/src/error.rs` | 42 |
| `kernels::avx2` | `crates/contextra-simd/src/kernels/avx2.rs` | 670 |
| `kernels::avx512` | `crates/contextra-simd/src/kernels/avx512.rs` | 680 |
| `kernels` | `crates/contextra-simd/src/kernels/mod.rs` | 47 |
| `kernels::neon` | `crates/contextra-simd/src/kernels/neon.rs` | 176 |
| `kernels::scalar` | `crates/contextra-simd/src/kernels/scalar.rs` | 380 |
| `crate` | `crates/contextra-simd/src/lib.rs` | 202 |
| `metric` | `crates/contextra-simd/src/metric.rs` | 22 |

## 6. Re-Exports (`pub use`)

- `pub use crate::error::` — `crates/contextra-simd/src/contract.rs:12` (Modul `contract`)
- `pub use crate::metric::DistanceMetric` — `crates/contextra-simd/src/contract.rs:13` (Modul `contract`)
- `pub use scalar::*` — `crates/contextra-simd/src/kernels/mod.rs:46` (Modul `kernels`)
- `pub use contract::` — `crates/contextra-simd/src/lib.rs:49` (Modul `crate`)
- `pub use dispatch::` — `crates/contextra-simd/src/lib.rs:50` (Modul `crate`)
- `pub use dispatch::` — `crates/contextra-simd/src/lib.rs:56` (Modul `crate`)
- `pub use error::` — `crates/contextra-simd/src/lib.rs:57` (Modul `crate`)
- `pub use kernels::scalar::` — `crates/contextra-simd/src/lib.rs:58` (Modul `crate`)
- `pub use metric::DistanceMetric` — `crates/contextra-simd/src/lib.rs:65` (Modul `crate`)

## 8. Öffentliche Schnittstelle je Item

### Modul `contract`

#### `struct MetricDistanceFn`
- Ort: `crates/contextra-simd/src/contract.rs:31`
```rust
pub struct MetricDistanceFn
```
Dokumentation: Implementierung von [`DistanceFn`] für das Dispatcher-Gesamtwerkzeug mit konfigurierter Metrik.
- Impl-Block `crates/contextra-simd/src/contract.rs:38`
- Impl-Block (Trait `DistanceFn`) `crates/contextra-simd/src/contract.rs:55`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `metric` | `DistanceMetric` | `crates/contextra-simd/src/contract.rs:33` |
| `dispatcher` | `crate::dispatch::Dispatcher` | `crates/contextra-simd/src/contract.rs:35` |

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new(metric: DistanceMetric) -> Self` | `crates/contextra-simd/src/contract.rs:41` | Erstellt eine neue Distanzfunktion mit der angegebenen Metrik und Auto-Dispatch. |
| `pub const fn with_dispatcher(metric: DistanceMetric, dispatcher: crate::dispatch::Dispatcher) -> Self` | `crates/contextra-simd/src/contract.rs:50` | Erstellt eine neue Distanzfunktion mit benutzerdefiniertem Dispatcher. |

#### `trait DistanceFn`
- Ort: `crates/contextra-simd/src/contract.rs:21`
```rust
pub trait DistanceFn: Send + Sync
```
Dokumentation: Abstrakte Distanzfunktions-Schnittstelle zur dynamischen oder externen Injektion von Berechnungen. Ermöglicht es Vektor-Konsumenten (wie `contextra-vector`), beliebige Distanz-Backends zustandslos und `Send + Sync` einzubinden.

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `fn distance(&self, a: &[f32], b: &[f32]) -> Result<f32>` | `crates/contextra-simd/src/contract.rs:26` | Berechnet die Distanz zwischen zwei gleich langen `f32`-Vektorslices. # Fehler Liefert `Err(SimdError)`, falls Dimensionen nicht übereinstimmen oder NaN/Inf vor |

### Modul `crate`

#### `mod compat`
- Ort: `crates/contextra-simd/src/lib.rs:44`
```rust
pub mod compat
```
- Gate: `#[cfg(feature = "workspace")]`

#### `mod contract`
- Ort: `crates/contextra-simd/src/lib.rs:34`
```rust
pub mod contract
```

#### `mod dispatch`
- Ort: `crates/contextra-simd/src/lib.rs:36`
```rust
pub mod dispatch
```

#### `mod error`
- Ort: `crates/contextra-simd/src/lib.rs:37`
```rust
pub mod error
```

#### `mod kernels`
- Ort: `crates/contextra-simd/src/lib.rs:40`
```rust
pub mod kernels
```

#### `mod metric`
- Ort: `crates/contextra-simd/src/lib.rs:41`
```rust
pub mod metric
```

#### `fn compute_distance`
- Ort: `crates/contextra-simd/src/lib.rs:137`
```rust
pub fn compute_distance(a: &[f32], b: &[f32], metric: DistanceMetric) -> Result<f32>
```
Dokumentation: Berechnet die Distanz zwischen zwei Vektoren unter Verwendung der angegebenen Metrik inklusive Validierung. Diese Funktion ist der primäre Einstiegspunkt für sichere Berechnungen von ungesicherten Eingaben. # Ablauf von `compute_distance`: 1. Vektor `a` auf Nicht-Endlichkeit (NaN / Unendlich) validieren. 2. Vektor `b` auf Nicht-Endlichkeit (NaN / Unendlich) validieren. 3. An [`compute_distance_trusted`] delegieren. # Fehlerfälle: - Gibt `Err(Error::InvalidInput)` zurück, wenn Vektor `a` oder `b` NaN/Unendlich enthält. - Gibt `Err(Error::InvalidInput)` zurück, wenn die Vektorlängen ungleich sin

#### `fn compute_distance_f32_bytes_trusted`
- Ort: `crates/contextra-simd/src/lib.rs:190`
```rust
pub fn compute_distance_f32_bytes_trusted( a: &[f32], b_bytes: &[u8], metric: DistanceMetric, ) -> Result<f32>
```
Dokumentation: Berechnet die Distanz zwischen einem `f32`-Vektor und einem rohen Byte-Puffer (`&[u8]`), der `f32`-Werte darstellt. Dies ist besonders nützlich, wenn Vektoren aus dem Speicher oder Disk-Cache (z. B. LSM-Tree oder Memory-Mapped Files) als Byte-Arrays vorliegen, um Kopier- und Allokationsaufwand zu vermeiden (Zero-Copy). # Ablauf von `compute_distance_f32_bytes_trusted`: 1. Fallunterscheidung nach `DistanceMetric`. 2. Aufruf der byte-kompatiblen Dispatch-Funktion (`cosine_distance_f32_bytes`, etc.). 3. Konvertierung/Reinterpretation der Bytes als `f32` im Ziel-Kernel. # Fehlerfälle: - Gibt `Err(

#### `fn compute_distance_trusted`
- Ort: `crates/contextra-simd/src/lib.rs:162`
```rust
pub fn compute_distance_trusted(a: &[f32], b: &[f32], metric: DistanceMetric) -> Result<f32>
```
Dokumentation: Berechnet die Distanz zwischen zwei bereits validierten `f32`-Vektoren ohne erneute NaN-Prüfung. Wird intern verwendet, wenn Vektoren bereits bei der Aufnahme in den Speicher validiert wurden, um bei tausenden Vergleichen pro Sekunde CPU-Zyklen einzusparen. # Ablauf von `compute_distance_trusted`: 1. Fallunterscheidung anhand der `DistanceMetric`. 2. Aufruf der passenden SIMD-Dispatch-Funktion (`cosine_distance`, `euclidean_distance`, `dot_product_distance`). 3. Längenabgleich und Ausführung im gewählten Hardware-Pfad. # Fehlerfälle: - Gibt `Err(Error::InvalidInput)` zurück, falls Längen abwei

#### `fn validate_vector`
- Ort: `crates/contextra-simd/src/lib.rs:98`
```rust
pub fn validate_vector(vec: &[f32]) -> Result<()>
```
Dokumentation: Prüft, ob ein Vektor gültige endliche Zahlen enthält (kein NaN und kein Unendlich). NaN (Not a Number) und Unendlich führen in SIMD-Registern und Metriken zu undefinierten oder korrupten Ergebnissen. Diese Prüfung verhindert solche Fehler im Vorfeld (Fail-Closed). # Ablauf von `validate_vector`: 1. Iteration über alle Elemente der `f32`-Slice. 2. Prüfung mittels `v.is_finite()`. 3. Abbruch mit Fehler (`Error::invalid_input`), falls mindestens ein Wert ungültig ist. 4. Rückgabe von `Ok(())`, wenn alle Werte endlich sind. # Fehlerfälle: - Gibt `Err(Error::InvalidInput)` zurück, wenn ein Wert `f3

### Modul `dispatch`

#### `struct Dispatcher`
- Ort: `crates/contextra-simd/src/dispatch.rs:80`
```rust
pub struct Dispatcher
```
Dokumentation: Injectable and thread-safe SIMD dispatcher (Design by Contract). Encapsulates SIMD hardware target selection and execution without relying on mutable global state.
- Impl-Block `crates/contextra-simd/src/dispatch.rs:145`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn auto() -> Self` | `crates/contextra-simd/src/dispatch.rs:148` | Creates a new `Dispatcher` with automatic CPU feature detection. |
| `pub const fn scalar() -> Self` | `crates/contextra-simd/src/dispatch.rs:156` | Creates a new `Dispatcher` forcing scalar reference execution. |
| `pub const fn new(target: SimdTarget) -> Self` | `crates/contextra-simd/src/dispatch.rs:164` | Creates a new `Dispatcher` with a specific target configuration. |
| `pub const fn target(&self) -> SimdTarget` | `crates/contextra-simd/src/dispatch.rs:170` | Returns the configured SIMD target. |
| `pub fn cosine_distance(&self, a: &[f32], b: &[f32]) -> Result<f32>` | `crates/contextra-simd/src/dispatch.rs:176` | Computes cosine distance ($1.0 - \text{cosine\_similarity}$) between two `f32` vectors. |
| `pub fn euclidean_distance(&self, a: &[f32], b: &[f32]) -> Result<f32>` | `crates/contextra-simd/src/dispatch.rs:235` | Computes Euclidean distance ($\\|a - b\\|_2$) between two `f32` vectors. |
| `pub fn dot_product_distance(&self, a: &[f32], b: &[f32]) -> Result<f32>` | `crates/contextra-simd/src/dispatch.rs:293` | Computes dot product distance (negated dot product $-a \cdot b$) between two `f32` vectors. |
| `pub fn cosine_distance_f32_bytes(&self, a: &[f32], b_bytes: &[u8]) -> Result<f32>` | `crates/contextra-simd/src/dispatch.rs:351` | Computes cosine distance between an `f32` slice and raw little-endian `f32` byte slice. |
| `pub fn euclidean_distance_f32_bytes(&self, a: &[f32], b_bytes: &[u8]) -> Result<f32>` | `crates/contextra-simd/src/dispatch.rs:396` | Computes Euclidean distance between an `f32` slice and raw little-endian `f32` byte slice. |
| `pub fn dot_product_distance_f32_bytes(&self, a: &[f32], b_bytes: &[u8]) -> Result<f32>` | `crates/contextra-simd/src/dispatch.rs:443` | Computes dot product distance between an `f32` slice and raw little-endian `f32` byte slice. |
| `pub fn dot_product_u8(&self, a: &[u8], b: &[u8]) -> Result<u32>` | `crates/contextra-simd/src/dispatch.rs:488` | Computes integer dot product between two `u8` byte slices. |
| `pub fn euclidean_distance_sq_u8(&self, a: &[u8], b: &[u8]) -> Result<u32>` | `crates/contextra-simd/src/dispatch.rs:538` | Computes squared Euclidean distance ($\\|a - b\\|_2^2$) between two `u8` byte slices. |
| `pub fn cosine_similarity_parts_u8( &self, a: &[u8], b: &[u8], ) -> Result<CosineSimilarityPartsU8>` | `crates/contextra-simd/src/dispatch.rs:583` | Computes cosine similarity component parts (dot product, norm_a_sq, norm_b_sq) for two `u8` byte slices. |

#### `enum SimdTarget`
- Ort: `crates/contextra-simd/src/dispatch.rs:32`
```rust
pub enum SimdTarget
```
Dokumentation: Explicit hardware target configuration for the SIMD dispatcher.

Varianten:

- `Auto` — `crates/contextra-simd/src/dispatch.rs:35`
- `Scalar` — `crates/contextra-simd/src/dispatch.rs:37`
- `Avx2` — `crates/contextra-simd/src/dispatch.rs:39`
- `Avx512` — `crates/contextra-simd/src/dispatch.rs:41`
- `Neon` — `crates/contextra-simd/src/dispatch.rs:43`

#### `fn cosine_distance`
- Ort: `crates/contextra-simd/src/dispatch.rs:662`
```rust
pub fn cosine_distance(a: &[f32], b: &[f32]) -> Result<f32>
```
Dokumentation: Computes cosine distance ($1.0 - \text{cosine\_similarity}$) between two `f32` vectors. Dynamically dispatches to AVX-512, AVX2+FMA, or NEON based on runtime CPU feature detection, falling back to scalar execution if SIMD features are missing or overridden in tests. # Semantics $d(a, b) = 1.0 - \frac{a \cdot b}{\|a\| \|b\|}$. Cosine similarity is clamped to $[-1.0, 1.0]$, yielding distance in $[0.0, 2.0]$. Zero-norm vectors safely return `1.0`. # Errors Returns [`SimdError::DimensionMismatch`] if `a.len() != b.len()`. Returns [`SimdError::InvalidInput`] if a test path override specifies an uns

#### `fn cosine_distance_f32_bytes`
- Ort: `crates/contextra-simd/src/dispatch.rs:755`
```rust
pub fn cosine_distance_f32_bytes(a: &[f32], b_bytes: &[u8]) -> Result<f32>
```
Dokumentation: Computes cosine distance between an `f32` slice and raw little-endian `f32` byte slice. Dynamically dispatches to AVX-512 or AVX2+FMA based on runtime CPU feature detection, falling back to scalar execution if SIMD features are missing or overridden in tests. # Semantics Decodes 4-byte float chunks from `b_bytes` and computes cosine distance against `a`. # Errors Returns [`SimdError::DimensionMismatch`] if `b_bytes.len() < a.len() * 4`. Returns [`SimdError::InvalidInput`] if a test path override specifies an unsupported CPU feature. # Complexity $O(N)$ time, $O(1)$ space where $N = a.len()$. #

#### `fn cosine_similarity_parts_u8`
- Ort: `crates/contextra-simd/src/dispatch.rs:910`
```rust
pub fn cosine_similarity_parts_u8(a: &[u8], b: &[u8]) -> Result<CosineSimilarityPartsU8>
```
Dokumentation: Computes cosine similarity component parts (dot product, norm_a_sq, norm_b_sq) for two `u8` byte slices. Dynamically dispatches to AVX-512 VNNI or AVX2 SIMD implementations, falling back to scalar execution if SIMD features are missing or overridden in tests. # Semantics Computes `CosineSimilarityPartsU8 { dot: sum(a_i * b_i), norm_a_sq: sum(a_i^2), norm_b_sq: sum(b_i^2) }`. # Errors Returns [`SimdError::DimensionMismatch`] if `a.len() != b.len()`. Returns [`SimdError::InvalidInput`] if a test path override specifies an unsupported CPU feature. # Complexity $O(N)$ time, $O(1)$ space where $N =

#### `fn detect_best_target`
- Ort: `crates/contextra-simd/src/dispatch.rs:89`
```rust
pub fn detect_best_target() -> SimdTarget
```
Dokumentation: Detects the best supported SIMD target on the host CPU, with support for caching and env override.

#### `fn dot_product_distance`
- Ort: `crates/contextra-simd/src/dispatch.rs:724`
```rust
pub fn dot_product_distance(a: &[f32], b: &[f32]) -> Result<f32>
```
Dokumentation: Computes dot product distance (negated dot product $-a \cdot b$) between two `f32` vectors. Dynamically dispatches to AVX-512, AVX2+FMA, or NEON based on runtime CPU feature detection, falling back to scalar execution if SIMD features are missing or overridden in tests. # Semantics Returns $- \sum_{i=0}^{N-1} a_i b_i$ so higher dot products yield smaller distance values. # Errors Returns [`SimdError::DimensionMismatch`] if `a.len() != b.len()`. Returns [`SimdError::InvalidInput`] if a test path override specifies an unsupported CPU feature. # Complexity $O(N)$ time, $O(1)$ space where $N = a.l

#### `fn dot_product_distance_f32_bytes`
- Ort: `crates/contextra-simd/src/dispatch.rs:817`
```rust
pub fn dot_product_distance_f32_bytes(a: &[f32], b_bytes: &[u8]) -> Result<f32>
```
Dokumentation: Computes dot product distance between an `f32` slice and raw little-endian `f32` byte slice. Dynamically dispatches to AVX-512 or AVX2+FMA based on runtime CPU feature detection, falling back to scalar execution if SIMD features are missing or overridden in tests. # Semantics Decodes 4-byte float chunks from `b_bytes` and computes negated dot product against `a`. # Errors Returns [`SimdError::DimensionMismatch`] if `b_bytes.len() < a.len() * 4`. Returns [`SimdError::InvalidInput`] if a test path override specifies an unsupported CPU feature. # Complexity $O(N)$ time, $O(1)$ space where $N = a.

#### `fn dot_product_u8`
- Ort: `crates/contextra-simd/src/dispatch.rs:848`
```rust
pub fn dot_product_u8(a: &[u8], b: &[u8]) -> Result<u32>
```
Dokumentation: Computes integer dot product between two `u8` byte slices. Dynamically dispatches to AVX-512 VNNI (`dpbusd`) or AVX2 (`madd_epi16`), falling back to scalar execution if SIMD features are missing or overridden in tests. # Semantics Calculates $\sum_{i=0}^{N-1} a_i b_i$ using 32-bit unsigned accumulation to prevent integer overflow. # Errors Returns [`SimdError::DimensionMismatch`] if `a.len() != b.len()`. Returns [`SimdError::InvalidInput`] if a test path override specifies an unsupported CPU feature. # Complexity $O(N)$ time, $O(1)$ space where $N = a.len()$. # Features Requires runtime detect

#### `fn euclidean_distance`
- Ort: `crates/contextra-simd/src/dispatch.rs:693`
```rust
pub fn euclidean_distance(a: &[f32], b: &[f32]) -> Result<f32>
```
Dokumentation: Computes Euclidean distance ($\|a - b\|_2$) between two `f32` vectors. Dynamically dispatches to AVX-512, AVX2+FMA, or NEON based on runtime CPU feature detection, falling back to scalar execution if SIMD features are missing or overridden in tests. # Semantics $d(a, b) = \sqrt{\sum_{i=0}^{N-1} (a_i - b_i)^2}$. Always non-negative ($\ge 0.0$). # Errors Returns [`SimdError::DimensionMismatch`] if `a.len() != b.len()`. Returns [`SimdError::InvalidInput`] if a test path override specifies an unsupported CPU feature. # Complexity $O(N)$ time, $O(1)$ space where $N = a.len()$. # Features Requires r

#### `fn euclidean_distance_f32_bytes`
- Ort: `crates/contextra-simd/src/dispatch.rs:786`
```rust
pub fn euclidean_distance_f32_bytes(a: &[f32], b_bytes: &[u8]) -> Result<f32>
```
Dokumentation: Computes Euclidean distance between an `f32` slice and raw little-endian `f32` byte slice. Dynamically dispatches to AVX-512 or AVX2+FMA based on runtime CPU feature detection, falling back to scalar execution if SIMD features are missing or overridden in tests. # Semantics Decodes 4-byte float chunks from `b_bytes` and computes Euclidean distance against `a`. # Errors Returns [`SimdError::DimensionMismatch`] if `b_bytes.len() < a.len() * 4`. Returns [`SimdError::InvalidInput`] if a test path override specifies an unsupported CPU feature. # Complexity $O(N)$ time, $O(1)$ space where $N = a.len

#### `fn euclidean_distance_sq_u8`
- Ort: `crates/contextra-simd/src/dispatch.rs:879`
```rust
pub fn euclidean_distance_sq_u8(a: &[u8], b: &[u8]) -> Result<u32>
```
Dokumentation: Computes squared Euclidean distance ($\|a - b\|_2^2$) between two `u8` byte slices. Dynamically dispatches to AVX-512 or AVX2 SIMD implementations, falling back to scalar execution if SIMD features are missing or overridden in tests. # Semantics Calculates $\sum_{i=0}^{N-1} (a_i - b_i)^2$ using 32-bit unsigned accumulation. # Errors Returns [`SimdError::DimensionMismatch`] if `a.len() != b.len()`. Returns [`SimdError::InvalidInput`] if a test path override specifies an unsupported CPU feature. # Complexity $O(N)$ time, $O(1)$ space where $N = a.len()$. # Features Requires runtime detection of 

#### `fn get_dispatch_override`
- Ort: `crates/contextra-simd/src/dispatch.rs:68`
```rust
pub fn get_dispatch_override() -> Option<SimdPathOverride>
```
Dokumentation: Retrieves the current thread-local SIMD dispatch path override.

#### `fn set_dispatch_override`
- Ort: `crates/contextra-simd/src/dispatch.rs:61`
```rust
pub fn set_dispatch_override(path: Option<SimdPathOverride>)
```
Dokumentation: Sets thread-local SIMD dispatch path override for testing.

#### `type SimdPathOverride`
- Ort: `crates/contextra-simd/src/dispatch.rs:51`
```rust
pub type SimdPathOverride = SimdTarget
```
Dokumentation: SIMD path override options for testing purposes.

### Modul `error`

#### `enum SimdError`
- Ort: `crates/contextra-simd/src/error.rs:15`
```rust
pub enum SimdError
```
Dokumentation: Aufzählungs-Typ für alle im Crate auftretenden Fehlerzustände. Alle Varianten sind Fail-Closed konstruiert.

Varianten:

- `InvalidInput(&'static str)` — `crates/contextra-simd/src/error.rs:18`
- `DimensionMismatch{` — `crates/contextra-simd/src/error.rs:22`
- `UnsupportedMetric` — `crates/contextra-simd/src/error.rs:31`

#### `type Error`
- Ort: `crates/contextra-simd/src/error.rs:41`
```rust
pub type Error = SimdError
```
Dokumentation: Typ-Alias `Error` für B4-Konformität (genau ein primärer Fehlertyp `Error`).

#### `type Result`
- Ort: `crates/contextra-simd/src/error.rs:38`
```rust
pub type Result<T> = core::result::Result<T, SimdError>
```
Dokumentation: Crate-lokales Result-Alias mit `SimdError` als Standard-Fehlertyp.

### Modul `kernels`

#### `mod scalar`
- Ort: `crates/contextra-simd/src/kernels/mod.rs:44`
```rust
pub mod scalar
```

### Modul `kernels::scalar`

#### `struct CosineSimilarityPartsF32U8`
- Ort: `crates/contextra-simd/src/kernels/scalar.rs:40`
```rust
pub struct CosineSimilarityPartsF32U8
```
Dokumentation: Struct holding raw float sum components for f32/u8 mixed cosine similarity calculation.

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `dot` | `f32` | `crates/contextra-simd/src/kernels/scalar.rs:42` |
| `norm_a_sq` | `f32` | `crates/contextra-simd/src/kernels/scalar.rs:44` |
| `norm_b_sq` | `f32` | `crates/contextra-simd/src/kernels/scalar.rs:46` |

#### `struct CosineSimilarityPartsU8`
- Ort: `crates/contextra-simd/src/kernels/scalar.rs:28`
```rust
pub struct CosineSimilarityPartsU8
```
Dokumentation: Struct holding raw integer sum components for u8 cosine similarity calculation.

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `dot` | `u32` | `crates/contextra-simd/src/kernels/scalar.rs:30` |
| `norm_a_sq` | `u32` | `crates/contextra-simd/src/kernels/scalar.rs:32` |
| `norm_b_sq` | `u32` | `crates/contextra-simd/src/kernels/scalar.rs:34` |

#### `fn cosine_distance_f32_bytes_scalar`
- Ort: `crates/contextra-simd/src/kernels/scalar.rs:138`
```rust
pub fn cosine_distance_f32_bytes_scalar(a: &[f32], b_bytes: &[u8]) -> f32
```
Dokumentation: Scalar reference implementation for cosine distance between `f32` slice and raw LE `f32` byte slice. # Semantics Decodes `f32` values from `b_bytes` via little-endian bytes and computes cosine distance against `a`. # Complexity $O(N)$ time, $O(1)$ space where $N = a.len()$.

#### `fn cosine_distance_scalar`
- Ort: `crates/contextra-simd/src/kernels/scalar.rs:63`
```rust
pub fn cosine_distance_scalar(a: &[f32], b: &[f32]) -> f32
```
Dokumentation: Scalar reference implementation for cosine distance on `f32` slices. # Semantics Calculates $1.0 - \text{cosine\_similarity}(a, b)$. Cosine similarity is clamped to $[-1.0, 1.0]$. Zero-norm vectors safely return `1.0`. # Complexity $O(N)$ time, $O(1)$ space where $N = \min(a.len(), b.len())$.

#### `fn cosine_similarity_parts_f32_u8`
- Ort: `crates/contextra-simd/src/kernels/scalar.rs:342`
```rust
pub fn cosine_similarity_parts_f32_u8(a: &[f32], b: &[u8]) -> CosineSimilarityPartsF32U8
```
Dokumentation: Scalar reference implementation for mixed `f32`/`u8` cosine similarity component parts. # Semantics Returns [`CosineSimilarityPartsF32U8`] containing dot product and squared norms. # Complexity $O(N)$ time, $O(1)$ space where $N = \min(a.len(), b.len())$.

#### `fn cosine_similarity_parts_u8_scalar`
- Ort: `crates/contextra-simd/src/kernels/scalar.rs:267`
```rust
pub fn cosine_similarity_parts_u8_scalar(a: &[u8], b: &[u8]) -> CosineSimilarityPartsU8
```
Dokumentation: Scalar reference implementation for `u8` cosine similarity component parts. # Semantics Returns [`CosineSimilarityPartsU8`] containing dot product and squared norms. # Complexity $O(N)$ time, $O(1)$ space where $N = \min(a.len(), b.len())$.

#### `fn dot_product_f32_bytes_scalar`
- Ort: `crates/contextra-simd/src/kernels/scalar.rs:201`
```rust
pub fn dot_product_f32_bytes_scalar(a: &[f32], b_bytes: &[u8]) -> f32
```
Dokumentation: Scalar reference implementation for dot product distance between `f32` slice and raw LE `f32` byte slice. # Semantics Decodes `f32` values from `b_bytes` via little-endian bytes and computes negated dot product against `a`. # Complexity $O(N)$ time, $O(1)$ space where $N = a.len()$.

#### `fn dot_product_f32_u8`
- Ort: `crates/contextra-simd/src/kernels/scalar.rs:300`
```rust
pub fn dot_product_f32_u8(a: &[f32], b: &[u8]) -> f32
```
Dokumentation: Scalar reference implementation for mixed `f32`/`u8` dot product. # Semantics Calculates $\sum a_i (b_i \text{ as } f32)$. # Complexity $O(N)$ time, $O(1)$ space where $N = \min(a.len(), b.len())$.

#### `fn dot_product_scalar`
- Ort: `crates/contextra-simd/src/kernels/scalar.rs:116`
```rust
pub fn dot_product_scalar(a: &[f32], b: &[f32]) -> f32
```
Dokumentation: Scalar reference implementation for dot product distance on `f32` slices. # Semantics Calculates negated dot product $- \sum a_i b_i$. # Complexity $O(N)$ time, $O(1)$ space where $N = \min(a.len(), b.len())$.

#### `fn dot_product_u8_scalar`
- Ort: `crates/contextra-simd/src/kernels/scalar.rs:227`
```rust
pub fn dot_product_u8_scalar(a: &[u8], b: &[u8]) -> u32
```
Dokumentation: Scalar reference implementation for unsigned integer dot product between two `u8` byte slices. # Semantics Calculates $\sum a_i b_i$ with 32-bit unsigned accumulation. # Complexity $O(N)$ time, $O(1)$ space where $N = \min(a.len(), b.len())$.

#### `fn euclidean_distance_f32_bytes_scalar`
- Ort: `crates/contextra-simd/src/kernels/scalar.rs:175`
```rust
pub fn euclidean_distance_f32_bytes_scalar(a: &[f32], b_bytes: &[u8]) -> f32
```
Dokumentation: Scalar reference implementation for Euclidean distance between `f32` slice and raw LE `f32` byte slice. # Semantics Decodes `f32` values from `b_bytes` via little-endian bytes and computes Euclidean distance against `a`. # Complexity $O(N)$ time, $O(1)$ space where $N = a.len()$.

#### `fn euclidean_distance_scalar`
- Ort: `crates/contextra-simd/src/kernels/scalar.rs:96`
```rust
pub fn euclidean_distance_scalar(a: &[f32], b: &[f32]) -> f32
```
Dokumentation: Scalar reference implementation for Euclidean distance on `f32` slices. # Semantics Calculates $\sqrt{\sum (a_i - b_i)^2}$. # Complexity $O(N)$ time, $O(1)$ space where $N = \min(a.len(), b.len())$.

#### `fn euclidean_distance_sq_f32_u8`
- Ort: `crates/contextra-simd/src/kernels/scalar.rs:319`
```rust
pub fn euclidean_distance_sq_f32_u8(a: &[f32], b: &[u8], alphas: &[f32], mins: &[f32]) -> f32
```
Dokumentation: Scalar reference implementation for mixed `f32`/`u8` scalar dequantized squared Euclidean distance. # Semantics Calculates $\sum (a_i - (b_i \cdot \alpha_i + \min_i))^2$. # Complexity $O(N)$ time, $O(1)$ space where $N = \min(a.len(), b.len(), \alpha.len(), \min.len())$.

#### `fn euclidean_distance_sq_u8_scalar`
- Ort: `crates/contextra-simd/src/kernels/scalar.rs:246`
```rust
pub fn euclidean_distance_sq_u8_scalar(a: &[u8], b: &[u8]) -> u32
```
Dokumentation: Scalar reference implementation for squared Euclidean distance between two `u8` byte slices. # Semantics Calculates $\sum (a_i - b_i)^2$ with 32-bit unsigned accumulation. # Complexity $O(N)$ time, $O(1)$ space where $N = \min(a.len(), b.len())$.

#### `fn normalize_inplace`
- Ort: `crates/contextra-simd/src/kernels/scalar.rs:371`
```rust
pub fn normalize_inplace(v: &mut [f32])
```
Dokumentation: Normalizes a vector in-place to unit L2 length. # Semantics Divides each element by $\sqrt{\sum v_i^2}$. Zero-norm vectors remain unmodified. # Complexity $O(N)$ time, $O(1)$ space where $N = v.len()$.

### Modul `metric`

#### `enum DistanceMetric`
- Ort: `crates/contextra-simd/src/metric.rs:14`
```rust
pub enum DistanceMetric
```
Dokumentation: Unterstützte Distanzmetriken für Vektorberechnungen. Definiert als geschlossenes Wert-Enum (B09-Entscheidung). Besitzt bitidentische Serialisierung zum Workspace-Typ `DistanceMetric`.
- Impl-Block (Trait `From<DistanceMetric>`) `crates/contextra-simd/src/compat/mod.rs:13`
- Impl-Block (Trait `From<contextra_types::DistanceMetric>`) `crates/contextra-simd/src/compat/mod.rs:26`

Varianten:

- `Cosine` — `crates/contextra-simd/src/metric.rs:16`
- `Euclidean` — `crates/contextra-simd/src/metric.rs:18`
- `DotProduct` — `crates/contextra-simd/src/metric.rs:20`

## 9. Impl-Blöcke ohne Typ im selben Crate

- `impl From<SimdError> for contextra_types::ContextraError` — `crates/contextra-simd/src/compat/mod.rs:42` (0 pub Methoden)
