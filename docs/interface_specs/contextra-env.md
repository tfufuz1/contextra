# Schnittstellenspezifikation: `contextra-env`

> Erzeugt durch `tools/extract_interfaces.py` aus dem Quellcode. Keine Handpflege. Jede Angabe trägt Datei:Zeile. Maßgeblich bleibt der Quellcode.

## 1. Kopf

| Feld | Wert |
|---|---|
| Name | `contextra-env` |
| Version | 0.1.0 |
| Pfad | `crates/contextra-env` |
| Rolle | Umgebung (Clock, Rng, FS-Abstraktion) |
| Bibliothekspfad | src/lib.rs (Standard) |

**Crate-Dokumentation (//!):**

> Canonical environment abstractions for Contextra (Clock, Rng, IdGen, Metrics).
>
> # Architecture Role
> Zero internal workspace dependencies (E0 Foundation Micro-Core).
> Pure standard library environment contracts and deterministic test doubles.

## 2. Abhängigkeiten

| Name | Art | Quelle | optional | verwendet von (normal/dev) |
|---|---|---|---|---|
| – | – | – | – | – |

**Verwendet von (Crate-Ebene):** `contextra-checkpoint`, `contextra-crypto`, `contextra-graph`, `contextra-infer-candle`, `contextra-kvcache`, `contextra-mvcc`, `contextra-ports`, `contextra-store`, `contextra-text`, `contextra-vector`

## 3. Features

| Feature | Aktiviert |
|---|---|
| `default` | `std` |
| `std` | – |
| `test-utils` | – |

## 4. Metriken (Quellcode)

| Kennzahl | Wert |
|---|---|
| Quelldateien | 7 |
| Quellzeilen | 1207 |
| Öffentliche Items (Modul-Ebene) | 21 |
| davon `fn` | 1 |
| davon `struct` | 9 |
| davon `trait` | 4 |
| davon `mod` | 7 |
| Öffentliche Makros (`#[macro_export]`) | 0 |
| Öffentliche Re-Exports (`pub use`) | 11 |
| `unsafe`-Vorkommen (Produktionscode) | 0 |
| `.unwrap()` (Produktionscode) | 0 |
| `.expect(` (Produktionscode) | 0 |
| `panic!` (Produktionscode) | 0 |
| `todo!/unimplemented!/unreachable!` | 0 |
| Indexierungen `x[…]` (grob, Produktionscode) | 0 |
| `#[deprecated]` | 0 |
| `#[cfg(feature…)]`-Gates | 2 |
| Parser-Abdeckung (geparste `pub`-Deklarationen / Grep-Treffer) | 32 / 32 = 100 % |

## 5. Modulbaum und Dateien

| Modul | Datei | Zeilen |
|---|---|---|
| `clock` | `crates/contextra-env/src/clock.rs` | 174 |
| `env` | `crates/contextra-env/src/env.rs` | 39 |
| `id_gen` | `crates/contextra-env/src/id_gen.rs` | 148 |
| `crate` | `crates/contextra-env/src/lib.rs` | 84 |
| `manual_clock` | `crates/contextra-env/src/manual_clock.rs` | 71 |
| `metrics` | `crates/contextra-env/src/metrics.rs` | 408 |
| `rng` | `crates/contextra-env/src/rng.rs` | 283 |

## 6. Re-Exports (`pub use`)

- `pub use crate::clock::` — `crates/contextra-env/src/env.rs:13` (Modul `env`)
- `pub use crate::id_gen::` — `crates/contextra-env/src/env.rs:14` (Modul `env`)
- `pub use crate::metrics::` — `crates/contextra-env/src/env.rs:15` (Modul `env`)
- `pub use crate::rng::` — `crates/contextra-env/src/env.rs:18` (Modul `env`)
- `pub use super::` — `crates/contextra-env/src/env.rs:36` (Modul `env::prelude`)
- `pub use clock::` — `crates/contextra-env/src/lib.rs:51` (Modul `crate`)
- `pub use env::*` — `crates/contextra-env/src/lib.rs:52` (Modul `crate`)
- `pub use id_gen::` — `crates/contextra-env/src/lib.rs:53` (Modul `crate`)
- `pub use metrics::` — `crates/contextra-env/src/lib.rs:54` (Modul `crate`)
- `pub use rng::` — `crates/contextra-env/src/lib.rs:57` (Modul `crate`)
- `pub use manual_clock::ManualClock` — `crates/contextra-env/src/lib.rs:60` (Modul `crate`)

## 8. Öffentliche Schnittstelle je Item

### Modul `clock`

#### `struct SystemClock`
- Ort: `crates/contextra-env/src/clock.rs:54`
```rust
pub struct SystemClock
```
Dokumentation: Standard production implementation of [`Clock`] backed by [`SystemTime`] and [`Instant`].
- Impl-Block `crates/contextra-env/src/clock.rs:59`
- Impl-Block (Trait `Default`) `crates/contextra-env/src/clock.rs:75`
- Impl-Block (Trait `Clock`) `crates/contextra-env/src/clock.rs:82`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new() -> Self` | `crates/contextra-env/src/clock.rs:66` | Creates a new [`SystemClock`] using the current [`Instant`] as the monotonic baseline. # Didaktischer Hinweis Die Initialisierung von `start_instant` via `Insta |

#### `trait Clock`
- Ort: `crates/contextra-env/src/clock.rs:18`
```rust
pub trait Clock: Send + Sync + 'static
```
Dokumentation: Port trait for time operations, enabling deterministic testing by decoupling system wall-clock and monotonic time sources. In production, [`SystemClock`] provides access to the real OS clocks. In unit tests or simulations, deterministic doubles like `ManualClock` can be substituted.

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `fn now_unix_nanos(&self) -> u64` | `crates/contextra-env/src/clock.rs:30` | Returns the current wall-clock time as nanoseconds since the Unix epoch. Returns `0` if system time is before the epoch instead of panicking. # Invarianten - `I |
| `fn monotonic_nanos(&self) -> u64` | `crates/contextra-env/src/clock.rs:41` | Returns nanoseconds elapsed since a monotonic baseline reference point. # Invarianten - `INV-ENV-TIME-2`: Garantiert panikfrei und monoton nicht fallend innerha |

### Modul `crate`

#### `mod clock`
- Ort: `crates/contextra-env/src/lib.rs:19`
```rust
pub mod clock
```
Dokumentation: Port trait and default system implementations for wall-clock and monotonic time. LINK clock.rs#clock-root

#### `mod env`
- Ort: `crates/contextra-env/src/lib.rs:23`
```rust
pub mod env
```
Dokumentation: Aggregated environment port definitions and prelude re-exports. LINK env.rs#env-root

#### `mod id_gen`
- Ort: `crates/contextra-env/src/lib.rs:27`
```rust
pub mod id_gen
```
Dokumentation: Port trait and thread-safe atomic unique ID generator. LINK id_gen.rs#idgen-root

#### `mod manual_clock`
- Ort: `crates/contextra-env/src/lib.rs:41`
```rust
pub mod manual_clock
```
Dokumentation: Controllable manual clock implementation for testing determinism. Gated behind optional `test-utils` feature (SPEC K13). LINK manual_clock.rs#manual-clock-root
- Gate: `#[cfg(feature = "test-utils")]`

#### `mod metrics`
- Ort: `crates/contextra-env/src/lib.rs:31`
```rust
pub mod metrics
```
Dokumentation: Observability port trait and in-memory test sink for metrics collection. LINK metrics.rs#metrics-root

#### `mod rng`
- Ort: `crates/contextra-env/src/lib.rs:35`
```rust
pub mod rng
```
Dokumentation: Pseudorandom number generator contract and deterministic SplitMix64 generator. LINK rng.rs#rng-root

#### `fn unix_ms`
- Ort: `crates/contextra-env/src/lib.rs:79`
```rust
pub fn unix_ms(c: &dyn Clock) -> u64
```
Dokumentation: Converts clock time to unix milliseconds by rounding down (SPEC-v2 §4.2, B02). Ensures deterministic millisecond calculation across legacy sub-crates (`text`, `graph`, `vector`) by enforcing standard truncation integer division. # Fehler Keine; garantiert panikfrei, da [`Clock::now_unix_nanos`] stets einen `u64` liefert. # Invarianten - `INV-ENV-TIME-1`: Bei Zeiten vor der Epoche liefert `now_unix_nanos()` 0, somit auch `unix_ms()` 0.

### Modul `env`

#### `mod prelude`
- Ort: `crates/contextra-env/src/env.rs:35`
```rust
pub mod prelude
```
Dokumentation: Prelude for environment ports. Exports the four essential port traits required to build injectable components: - [`Clock`]: Time operations - [`Rng`]: Pseudorandom number generation - [`IdGen`]: Monotonic and unique ID generation - [`MetricsSink`]: Observability and telemetry

### Modul `id_gen`

#### `struct SequentialIdGen`
- Ort: `crates/contextra-env/src/id_gen.rs:39`
```rust
pub struct SequentialIdGen
```
Dokumentation: A thread-safe ID generator that produces monotonically increasing 64-bit IDs. Wraps around safely on integer overflow (`u64::MAX` -> `0`) via atomic standard wrapping.
- Impl-Block `crates/contextra-env/src/id_gen.rs:44`
- Impl-Block (Trait `Default`) `crates/contextra-env/src/id_gen.rs:53`
- Impl-Block (Trait `IdGen`) `crates/contextra-env/src/id_gen.rs:60`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new(start: u64) -> Self` | `crates/contextra-env/src/id_gen.rs:46` | Creates a new [`SequentialIdGen`] starting at `start`. |

#### `trait IdGen`
- Ort: `crates/contextra-env/src/id_gen.rs:14`
```rust
pub trait IdGen: Send + Sync + 'static
```
Dokumentation: Port trait for unique ID generation.

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `fn next_id(&self) -> u64` | `crates/contextra-env/src/id_gen.rs:24` | Generates and returns the next unique 64-bit unsigned identifier. # Invarianten - `INV-ENV-ID-1`: Keine Duplikate bei aufeinanderfolgenden Aufrufen; threadsiche |

### Modul `manual_clock`

#### `struct ManualClock`
- Ort: `crates/contextra-env/src/manual_clock.rs:17`
```rust
pub struct ManualClock
```
Dokumentation: Controllable clock implementation for testing determinism and time regression. Permits precise manual stepping of unix and monotonic nanoseconds via atomic operations.
- Impl-Block `crates/contextra-env/src/manual_clock.rs:24`
- Impl-Block (Trait `Default`) `crates/contextra-env/src/manual_clock.rs:52`
- Impl-Block (Trait `Clock`) `crates/contextra-env/src/manual_clock.rs:59`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new(unix_nanos: u64, mono_nanos: u64) -> Self` | `crates/contextra-env/src/manual_clock.rs:26` | Creates a new `ManualClock` initialized at the given unix and monotonic timestamps. |
| `pub fn advance_nanos(&self, delta: u64)` | `crates/contextra-env/src/manual_clock.rs:36` | Advances the clock by the given delta in nanoseconds. Erhöht sowohl Unix- als auch Monotonie-Nanosekunden um `delta`. |
| `pub fn set_unix_nanos(&self, nanos: u64)` | `crates/contextra-env/src/manual_clock.rs:42` | Sets the unix timestamp to an absolute value in nanoseconds. |
| `pub fn set_monotonic_nanos(&self, nanos: u64)` | `crates/contextra-env/src/manual_clock.rs:47` | Sets the monotonic timestamp to an absolute value in nanoseconds. |

### Modul `metrics`

#### `struct HistogramSnapshot`
- Ort: `crates/contextra-env/src/metrics.rs:119`
```rust
pub struct HistogramSnapshot
```
Dokumentation: Snapshot of a recorded histogram metric.

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `sum` | `f64` | `crates/contextra-env/src/metrics.rs:121` |
| `count` | `u64` | `crates/contextra-env/src/metrics.rs:123` |
| `min` | `Option<f64>` | `crates/contextra-env/src/metrics.rs:125` |
| `max` | `Option<f64>` | `crates/contextra-env/src/metrics.rs:127` |

#### `struct InMemoryMetricsSink`
- Ort: `crates/contextra-env/src/metrics.rs:230`
```rust
pub struct InMemoryMetricsSink
```
Dokumentation: Production-ready local thread-safe implementation of [`MetricsSink`]. Stores metrics strictly in-process with no external network exports (privacy-preserving).
- Impl-Block `crates/contextra-env/src/metrics.rs:236`
- Impl-Block (Trait `MetricsSink`) `crates/contextra-env/src/metrics.rs:277`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new() -> Self` | `crates/contextra-env/src/metrics.rs:238` | Creates a new empty [`InMemoryMetricsSink`]. |
| `pub fn snapshot(&self) -> MetricSnapshot` | `crates/contextra-env/src/metrics.rs:243` | Captures a point-in-time [`MetricSnapshot`] of all recorded counters, gauges, and histograms. |

#### `struct MetricKey`
- Ort: `crates/contextra-env/src/metrics.rs:85`
```rust
pub struct MetricKey
```
Dokumentation: Key identifying a metric by name and sorted key-value labels.
- Impl-Block `crates/contextra-env/src/metrics.rs:92`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `name` | `String` | `crates/contextra-env/src/metrics.rs:87` |

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new(name: impl Into<String>, labels: &[(&str, &str)]) -> Self` | `crates/contextra-env/src/metrics.rs:96` | Creates a new normalized [`MetricKey`]. Sortiert Labels deterministisch lexikografisch, um Hash- und BTreeMap-Konsistenz zu wahren. |

#### `struct MetricSnapshot`
- Ort: `crates/contextra-env/src/metrics.rs:135`
```rust
pub struct MetricSnapshot
```
Dokumentation: In-memory snapshot of all metrics collected by [`InMemoryMetricsSink`].

#### `struct NoopMetricsSink`
- Ort: `crates/contextra-env/src/metrics.rs:62`
```rust
pub struct NoopMetricsSink
```
Dokumentation: A no-op implementation of [`MetricsSink`] that silently discards all metrics events.
- Impl-Block (Trait `MetricsSink`) `crates/contextra-env/src/metrics.rs:64`

#### `trait MetricsSink`
- Ort: `crates/contextra-env/src/metrics.rs:26`
```rust
pub trait MetricsSink: Send + Sync + 'static
```
Dokumentation: Metrics sink port trait for recording system counters, gauges, and histograms. Implemented by observability backends to collect operational metrics from domain components without creating direct dependencies on specific metrics libraries.

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `fn record_counter(&self, name: &str, value: u64, labels: &[(&str, &str)])` | `crates/contextra-env/src/metrics.rs:33` | Records a counter metric increment. # Fehler Keine; garantiert panikfrei und nicht-blockierend. # Crash Reiner flüchtiger RAM-Zähler; keine I/O oder persistente |
| `fn record_gauge(&self, name: &str, value: f64, labels: &[(&str, &str)])` | `crates/contextra-env/src/metrics.rs:41` | Records a gauge metric absolute value. # Fehler Keine; garantiert panikfrei und nicht-blockierend. # Crash Reiner flüchtiger RAM-Messwert; keine I/O oder persis |
| `fn record_histogram(&self, name: &str, value: f64, labels: &[(&str, &str)])` | `crates/contextra-env/src/metrics.rs:49` | Records a histogram observation value. # Fehler Keine; garantiert panikfrei und nicht-blockierend. # Crash Reiner flüchtiger RAM-Histogrammwert; keine I/O oder  |

### Modul `rng`

#### `struct SeededRng`
- Ort: `crates/contextra-env/src/rng.rs:83`
```rust
pub struct SeededRng
```
Dokumentation: A fast, deterministic 64-bit pseudorandom number generator using SplitMix64 with atomic state transitions (`AtomicU64`). Lock-free and thread-safe without requiring `Mutex`. # Entropy Seeding Note that non-deterministic entropy seeding (e.g., via `OsRng` or system entropy) is the responsibility of the Composition Root. Application entry points should gather entropy at startup and pass the initial seed to [`SeededRng::new`].
- Impl-Block `crates/contextra-env/src/rng.rs:88`
- Impl-Block (Trait `Rng`) `crates/contextra-env/src/rng.rs:128`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new(seed: u64) -> Self` | `crates/contextra-env/src/rng.rs:92` | Creates a new [`SeededRng`] initialized with the given seed. Garantiert exakte Bitgleichheit zu den kanonischen Golden-Testvektoren. |
| `pub fn from_entropy() -> Self` | `crates/contextra-env/src/rng.rs:109` | Creates a new [`SeededRng`] seeded from process/runtime entropy. # Non-Cryptographic Usage This entropy source uses [`std::collections::hash_map::RandomState`]  |

#### `trait Rng`
- Ort: `crates/contextra-env/src/rng.rs:32`
```rust
pub trait Rng: Send + Sync + 'static
```
Dokumentation: Port trait for random number generators, enabling deterministic testing by decoupling non-deterministic entropy sources.

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `fn next_u64(&self) -> u64` | `crates/contextra-env/src/rng.rs:42` | Generates the next pseudorandom 64-bit unsigned integer. # Invarianten - `INV-ENV-RNG-1`: Deterministische Reproduzierbarkeit bei gleichem Seed. # Fehler Keine; |
| `fn fill_bytes(&self, dest: &mut [u8])` | `crates/contextra-env/src/rng.rs:50` | Fills `dest` with pseudorandom bytes generated by this RNG. # Fehler Keine; garantiert panikfrei für alle Puffergrößen. # Crash Reiner flüchtiger RAM-Zustand; k |
| `fn next_unit_f64(&self) -> f64` | `crates/contextra-env/src/rng.rs:62` | Returns a pseudorandom `f64` value in the half-open interval `[0.0, 1.0)` with 53 bits of precision. # Invarianten - `INV-ENV-RNG-2`: Ergebnis liegt strikt im B |

## 9. Impl-Blöcke ohne Typ im selben Crate

- `impl<T: Clock + ?Sized> Clock for Arc<T>` — `crates/contextra-env/src/clock.rs:118` (0 pub Methoden)
- `impl<T: IdGen + ?Sized> IdGen for Arc<T>` — `crates/contextra-env/src/id_gen.rs:79` (0 pub Methoden)
- `impl<T: MetricsSink + ?Sized> MetricsSink for Arc<T>` — `crates/contextra-env/src/metrics.rs:330` (0 pub Methoden)
- `impl<T: Rng + ?Sized> Rng for Arc<T>` — `crates/contextra-env/src/rng.rs:185` (0 pub Methoden)
