# Schnittstellenspezifikation: `contextra-adapt`

> Erzeugt durch `tools/extract_interfaces.py` aus dem Quellcode. Keine Handpflege. Jede Angabe trägt Datei:Zeile. Maßgeblich bleibt der Quellcode.

## 1. Kopf

| Feld | Wert |
|---|---|
| Name | `contextra-adapt` |
| Version | workspace |
| Pfad | `crates/contextra-adapt` |
| Rolle | Adaptive Regelung |
| Bibliothekspfad | src/lib.rs (Standard) |

**Crate-Dokumentation (//!):**

> Adaptive controllers, PID regulators, bandits, and Lyapunov drift watchers for Contextra.

## 2. Abhängigkeiten

| Name | Art | Quelle | optional | verwendet von (normal/dev) |
|---|---|---|---|---|
| `ahash` | normal | workspace | nein |  |
| `blake3` | normal | workspace | nein |  |
| `contextra-ports` | normal | path | ja | contextra, contextra-adapt, contextra-agent, contextra-checkpoint, contextra-cognition, contextra-crypto, contextra-crypto-fuzz, contextra-db, contextra-engine, contextra-graph, contextra-graph-fuzz, contextra-infer-candle, contextra-infer-ollama, contextra-infer-onnx, contextra-kvcache, contextra-mcp, contextra-mvcc, contextra-rank, contextra-router, contextra-store, contextra-store-fuzz, contextra-testkit, contextra-text, contextra-vector, contextra-vector-fuzz |
| `contextra-types` | normal | path | ja | contextra, contextra-adapt, contextra-agent, contextra-bench, contextra-checkpoint, contextra-cognition, contextra-crypto, contextra-crypto-fuzz, contextra-db, contextra-engine, contextra-graph, contextra-graph-fuzz, contextra-infer-candle, contextra-infer-ollama, contextra-infer-onnx, contextra-ingest, contextra-kvcache, contextra-mcp, contextra-mvcc, contextra-ports, contextra-py, contextra-rank, contextra-router, contextra-simd, contextra-store, contextra-store-fuzz, contextra-testkit, contextra-text, contextra-vault, contextra-vector, contextra-vector-fuzz |
| `criterion` | dev | workspace | nein |  |
| `proptest` | dev | registry | nein |  |
| `serde` | normal | workspace | nein |  |
| `serde_json` | normal | workspace | nein |  |
| `thiserror` | normal | workspace | nein |  |
| `tracing` | normal | workspace | nein |  |

**Verwendet von (Crate-Ebene):** `contextra-db`, `contextra-router`

## 3. Features

| Feature | Aktiviert |
|---|---|
| `bandit-routing` | – |
| `default` | `std` |
| `exp-bandit-routing` | `bandit-routing` |
| `exp-flow-thompson` | `bandit-routing` |
| `exp-rie-greedy` | – |
| `flow-corrected-thompson` | `exp-flow-thompson` |
| `ports-compat` | `dep:contextra-ports` |
| `rie-greedy-personalization` | `exp-rie-greedy` |
| `std` | – |
| `types-compat` | `dep:contextra-types` |
| `workspace` | `dep:contextra-types`, `dep:contextra-ports` |

## 4. Metriken (Quellcode)

| Kennzahl | Wert |
|---|---|
| Quelldateien | 20 |
| Quellzeilen | 5428 |
| Öffentliche Items (Modul-Ebene) | 92 |
| davon `fn` | 2 |
| davon `struct` | 34 |
| davon `enum` | 9 |
| davon `trait` | 12 |
| davon `type` | 1 |
| davon `const` | 9 |
| davon `mod` | 25 |
| Öffentliche Makros (`#[macro_export]`) | 0 |
| Öffentliche Re-Exports (`pub use`) | 29 |
| `unsafe`-Vorkommen (Produktionscode) | 0 |
| `.unwrap()` (Produktionscode) | 0 |
| `.expect(` (Produktionscode) | 0 |
| `panic!` (Produktionscode) | 0 |
| `todo!/unimplemented!/unreachable!` | 0 |
| Indexierungen `x[…]` (grob, Produktionscode) | 188 |
| `#[deprecated]` | 4 |
| `#[cfg(feature…)]`-Gates | 25 |
| Parser-Abdeckung (geparste `pub`-Deklarationen / Grep-Treffer) | 195 / 195 = 100 % |

## 5. Modulbaum und Dateien

| Modul | Datei | Zeilen |
|---|---|---|
| `bandit` | `crates/contextra-adapt/src/bandit.rs` | 1219 |
| `clock` | `crates/contextra-adapt/src/clock.rs` | 69 |
| `compat::clock` | `crates/contextra-adapt/src/compat/clock.rs` | 18 |
| `compat::error` | `crates/contextra-adapt/src/compat/error.rs` | 77 |
| `compat` | `crates/contextra-adapt/src/compat/mod.rs` | 11 |
| `contract` | `crates/contextra-adapt/src/contract.rs` | 44 |
| `decay_controller` | `crates/contextra-adapt/src/decay_controller.rs` | 255 |
| `drift` | `crates/contextra-adapt/src/drift.rs` | 322 |
| `error` | `crates/contextra-adapt/src/error.rs` | 110 |
| `flow_thompson` | `crates/contextra-adapt/src/flow_thompson.rs` | 918 |
| `homeostat` | `crates/contextra-adapt/src/homeostat.rs` | 232 |
| `crate` | `crates/contextra-adapt/src/lib.rs` | 107 |
| `lyapunov` | `crates/contextra-adapt/src/lyapunov.rs` | 554 |
| `offpolicy` | `crates/contextra-adapt/src/offpolicy.rs` | 307 |
| `pid` | `crates/contextra-adapt/src/pid.rs` | 465 |
| `pid_latency_controller` | `crates/contextra-adapt/src/pid_latency_controller.rs` | 309 |
| `rie_greedy` | `crates/contextra-adapt/src/rie_greedy.rs` | 221 |
| `rng` | `crates/contextra-adapt/src/rng.rs` | 44 |
| `shadow_mode` | `crates/contextra-adapt/src/shadow_mode.rs` | 126 |
| `strategy` | `crates/contextra-adapt/src/strategy.rs` | 20 |

## 6. Re-Exports (`pub use`)

- `pub use clock::PortsClockAdapter` — `crates/contextra-adapt/src/compat/mod.rs:11` (Modul `compat`)
- `pub use crate::clock::Clock` — `crates/contextra-adapt/src/contract.rs:9` (Modul `contract`)
- `pub use crate::rng::Rng` — `crates/contextra-adapt/src/contract.rs:10` (Modul `contract`)
- `pub use clock::` — `crates/contextra-adapt/src/lib.rs:36` (Modul `crate`)
- `pub use error::` — `crates/contextra-adapt/src/lib.rs:37` (Modul `crate`)
- `pub use rng::` — `crates/contextra-adapt/src/lib.rs:38` (Modul `crate`)
- `pub use strategy::RetrievalStrategy` — `crates/contextra-adapt/src/lib.rs:39` (Modul `crate`)
- `pub use crate::homeostat::*` — `crates/contextra-adapt/src/lib.rs:45` (Modul `control`)
- `pub use crate::lyapunov::*` — `crates/contextra-adapt/src/lib.rs:46` (Modul `control`)
- `pub use crate::pid::*` — `crates/contextra-adapt/src/lib.rs:47` (Modul `control`)
- `pub use crate::pid_latency_controller::*` — `crates/contextra-adapt/src/lib.rs:48` (Modul `control`)
- `pub use crate::bandit::*` — `crates/contextra-adapt/src/lib.rs:54` (Modul `bandit_algorithms`)
- `pub use crate::flow_thompson::*` — `crates/contextra-adapt/src/lib.rs:56` (Modul `bandit_algorithms`)
- `pub use crate::rie_greedy::*` — `crates/contextra-adapt/src/lib.rs:58` (Modul `bandit_algorithms`)
- `pub use crate::offpolicy::*` — `crates/contextra-adapt/src/lib.rs:63` (Modul `policy`)
- `pub use crate::shadow_mode::*` — `crates/contextra-adapt/src/lib.rs:64` (Modul `policy`)
- `pub use crate::drift::*` — `crates/contextra-adapt/src/lib.rs:69` (Modul `drift_detection`)
- `pub use crate::decay_controller::*` — `crates/contextra-adapt/src/lib.rs:74` (Modul `decay`)
- `pub use bandit::*` — `crates/contextra-adapt/src/lib.rs:95` (Modul `crate`)
- `pub use decay_controller::*` — `crates/contextra-adapt/src/lib.rs:96` (Modul `crate`)
- `pub use drift::*` — `crates/contextra-adapt/src/lib.rs:97` (Modul `crate`)
- `pub use flow_thompson::*` — `crates/contextra-adapt/src/lib.rs:99` (Modul `crate`)
- `pub use homeostat::*` — `crates/contextra-adapt/src/lib.rs:100` (Modul `crate`)
- `pub use lyapunov::*` — `crates/contextra-adapt/src/lib.rs:101` (Modul `crate`)
- `pub use offpolicy::*` — `crates/contextra-adapt/src/lib.rs:102` (Modul `crate`)
- `pub use pid::*` — `crates/contextra-adapt/src/lib.rs:103` (Modul `crate`)
- `pub use pid_latency_controller::*` — `crates/contextra-adapt/src/lib.rs:104` (Modul `crate`)
- `pub use rie_greedy::` — `crates/contextra-adapt/src/lib.rs:106` (Modul `crate`)
- `pub use shadow_mode::*` — `crates/contextra-adapt/src/lib.rs:107` (Modul `crate`)

## 8. Öffentliche Schnittstelle je Item

### Modul `bandit`

#### `struct AlignedF32Vec`
- Ort: `crates/contextra-adapt/src/bandit.rs:71`
```rust
pub struct AlignedF32Vec(pub Vec<f32>)
```
- Impl-Block (Trait `std::ops::Deref`) `crates/contextra-adapt/src/bandit.rs:73`
- Impl-Block (Trait `std::ops::DerefMut`) `crates/contextra-adapt/src/bandit.rs:80`
- Impl-Block `crates/contextra-adapt/src/bandit.rs:86`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new(vec: Vec<f32>) -> Self` | `crates/contextra-adapt/src/bandit.rs:88` | Erstellt einen neuen `AlignedF32Vec` aus einem `Vec<f32>`. |
| `pub fn zeros(len: usize) -> Self` | `crates/contextra-adapt/src/bandit.rs:93` | Erstellt einen neuen `AlignedF32Vec` mit `len` Nullen. |

#### `struct BanditProfileState`
- Ort: `crates/contextra-adapt/src/bandit.rs:320`
```rust
pub struct BanditProfileState
```
- Impl-Block (Trait `BanditPolicy`) `crates/contextra-adapt/src/bandit.rs:372`
- Impl-Block `crates/contextra-adapt/src/bandit.rs:389`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `theta` | `Vec<f32>` | `crates/contextra-adapt/src/bandit.rs:322` |
| `sigma_sq` | `Vec<f32>` | `crates/contextra-adapt/src/bandit.rs:324` |
| `inv_a` | `AlignedF32Vec` | `crates/contextra-adapt/src/bandit.rs:327` |
| `work_buf` | `AlignedF32Vec` | `crates/contextra-adapt/src/bandit.rs:333` |
| `alpha` | `f32` | `crates/contextra-adapt/src/bandit.rs:335` |
| `alpha_base` | `f32` | `crates/contextra-adapt/src/bandit.rs:338` |
| `alpha_max_multiplier` | `f32` | `crates/contextra-adapt/src/bandit.rs:341` |
| `lambda` | `f32` | `crates/contextra-adapt/src/bandit.rs:343` |
| `mu` | `f32` | `crates/contextra-adapt/src/bandit.rs:345` |
| `gamma` | `f32` | `crates/contextra-adapt/src/bandit.rs:349` |
| `drift_gamma` | `f32` | `crates/contextra-adapt/src/bandit.rs:352` |
| `drift_steps_remaining` | `usize` | `crates/contextra-adapt/src/bandit.rs:355` |
| `drift_decay_window` | `usize` | `crates/contextra-adapt/src/bandit.rs:358` |
| `implementation` | `BanditImplementation` | `crates/contextra-adapt/src/bandit.rs:360` |
| `seed` | `u64` | `crates/contextra-adapt/src/bandit.rs:363` |
| `sketch_matrix` | `Option<SketchMatrix>` | `crates/contextra-adapt/src/bandit.rs:366` |
| `sketch_theta` | `Vec<f32>` | `crates/contextra-adapt/src/bandit.rs:369` |

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn cold_start(d: usize, alpha_base: f32) -> Self` | `crates/contextra-adapt/src/bandit.rs:392` |  |
| `pub fn ensure_sketched_state(&mut self, projected_dim: usize) -> Result<(), BanditError>` | `crates/contextra-adapt/src/bandit.rs:426` |  |
| `pub fn expected_dim(&self) -> usize` | `crates/contextra-adapt/src/bandit.rs:491` | Gibt die erwartete Dimension der Feature-Vektoren zurück. |
| `pub fn score( &self, x: &[f32], cost: f32, is_cloud_transport: bool, ) -> Result<f32, BanditError>` | `crates/contextra-adapt/src/bandit.rs:516` |  |
| `pub fn update( &mut self, x: &[f32], r_outcome: f32, cost: f32, is_cloud_transport: bool, ) -> Result<(), BanditError>` | `crates/contextra-adapt/src/bandit.rs:660` |  |
| `pub fn on_drift_detected(&mut self, k_drift: f32)` | `crates/contextra-adapt/src/bandit.rs:803` |  |

#### `struct SketchMatrix`
- Ort: `crates/contextra-adapt/src/bandit.rs:124`
```rust
pub struct SketchMatrix
```
- Impl-Block `crates/contextra-adapt/src/bandit.rs:135`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `data` | `AlignedF32Vec` | `crates/contextra-adapt/src/bandit.rs:126` |
| `projected_dim` | `usize` | `crates/contextra-adapt/src/bandit.rs:128` |
| `original_dim` | `usize` | `crates/contextra-adapt/src/bandit.rs:130` |
| `seed` | `u64` | `crates/contextra-adapt/src/bandit.rs:132` |

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn derive_seed_u64(tenant_id: u64, base_seed: u64) -> u64` | `crates/contextra-adapt/src/bandit.rs:138` |  |
| `pub fn derive_seed(namespace_id: u64, base_seed: u64) -> u64` | `crates/contextra-adapt/src/bandit.rs:149` | Leitet einen isolierten PRNG-Seed via BLAKE3-Hash ab. |
| `pub fn from_seed( seed: u64, original_dim: usize, projected_dim: usize, ) -> Result<Self, BanditError>` | `crates/contextra-adapt/src/bandit.rs:155` |  |
| `pub fn project(&self, x: &[f32]) -> Vec<f32>` | `crates/contextra-adapt/src/bandit.rs:186` |  |

#### `enum BanditError`
- Ort: `crates/contextra-adapt/src/bandit.rs:36`
```rust
pub enum BanditError
```

Varianten:

- `DimensionMismatch{` — `crates/contextra-adapt/src/bandit.rs:39`
- `InvalidConfig(String)` — `crates/contextra-adapt/src/bandit.rs:47`
- `NonFiniteVariance(f32)` — `crates/contextra-adapt/src/bandit.rs:50`
- `PrecisionMatrixDriftDetected{` — `crates/contextra-adapt/src/bandit.rs:53`

#### `enum BanditImplementation`
- Ort: `crates/contextra-adapt/src/bandit.rs:102`
```rust
pub enum BanditImplementation
```

Varianten:

- `ShermanMorrison` — `crates/contextra-adapt/src/bandit.rs:105`
- `DiagonalApproximation` — `crates/contextra-adapt/src/bandit.rs:107`
- `FlowCorrectedThompson` — `crates/contextra-adapt/src/bandit.rs:113`
- `SketchedProjection{` — `crates/contextra-adapt/src/bandit.rs:115`

#### `trait BanditPolicy`
- Ort: `crates/contextra-adapt/src/bandit.rs:197`
```rust
pub trait BanditPolicy: Send + Sync
```

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `fn apply_drift_penalty(&mut self, k_drift: f32, alpha_max: f32, gamma: f32)` | `crates/contextra-adapt/src/bandit.rs:199` | Wendet Drift-Penalty / Covariance Discounting bei erkannter concept drift an. |

#### `const SHERMAN_MORRISON_REFACTORIZATION_INTERVAL`
- Ort: `crates/contextra-adapt/src/bandit.rs:31`
```rust
pub const SHERMAN_MORRISON_REFACTORIZATION_INTERVAL: u64 = 1000
```

### Modul `clock`

#### `struct SystemClock`
- Ort: `crates/contextra-adapt/src/clock.rs:19`
```rust
pub struct SystemClock
```
Dokumentation: Standard-Implementierung von [`Clock`] unter Verwendung des Betriebssystem-Zeitgebers.
- Impl-Block (Trait `Default`) `crates/contextra-adapt/src/clock.rs:24`
- Impl-Block `crates/contextra-adapt/src/clock.rs:30`
- Impl-Block (Trait `Clock`) `crates/contextra-adapt/src/clock.rs:50`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new() -> Self` | `crates/contextra-adapt/src/clock.rs:33` | Erstellt eine neue `SystemClock`-Instanz. |

#### `trait Clock`
- Ort: `crates/contextra-adapt/src/clock.rs:9`
```rust
pub trait Clock: Send + Sync + 'static
```
Dokumentation: Trait für deterministische Zeitgeber.

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `fn now_unix_nanos(&self) -> u64` | `crates/contextra-adapt/src/clock.rs:11` | Liefert die aktuelle Unix-Epochenzeit in Nanosekunden. |
| `fn monotonic_nanos(&self) -> u64` | `crates/contextra-adapt/src/clock.rs:14` | Liefert einen monoton steigenden Zeitstempel in Nanosekunden. |

### Modul `compat`

#### `mod clock`
- Ort: `crates/contextra-adapt/src/compat/mod.rs:6`
```rust
pub mod clock
```
- Gate: `#[cfg(feature = "ports-compat")]`

#### `mod error`
- Ort: `crates/contextra-adapt/src/compat/mod.rs:8`
```rust
pub mod error
```
- Gate: `#[cfg(feature = "types-compat")]`

### Modul `compat::clock`

#### `struct PortsClockAdapter`
- Ort: `crates/contextra-adapt/src/compat/clock.rs:8`
```rust
pub struct PortsClockAdapter(pub Arc<dyn contextra_ports::Clock>)
```
Dokumentation: Adapter für einen dynamischen `contextra_ports::Clock`-Referenzzeiger auf das lokale `Clock`-Trait.
- Impl-Block (Trait `Clock`) `crates/contextra-adapt/src/compat/clock.rs:10`

### Modul `contract`

#### `trait AdaptiveController`
- Ort: `crates/contextra-adapt/src/contract.rs:13`
```rust
pub trait AdaptiveController: Send + Sync
```
Dokumentation: Trait für Regler mit Anti-Windup und Stellgrößenbegrenzung.

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `fn update(&mut self, dt: Duration, measurement: f64) -> f64` | `crates/contextra-adapt/src/contract.rs:15` | Aktualisiert den Reglerzustand und berechnet das Stellsignal. |
| `fn reset(&mut self)` | `crates/contextra-adapt/src/contract.rs:17` | Setzt den internen Akkumulator zurück. |

#### `trait BanditArmSelector`
- Ort: `crates/contextra-adapt/src/contract.rs:21`
```rust
pub trait BanditArmSelector: Send + Sync
```
Dokumentation: Trait für kontextuelle Bandit-Arm-Auswahl und Regressions-Updates.

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `fn select_arm(&mut self, context: &[f32]) -> Result<usize>` | `crates/contextra-adapt/src/contract.rs:23` | Wählt einen Arm basierend auf dem Kontextvektor aus. |
| `fn update(&mut self, arm: usize, context: &[f32], reward: f64) -> Result<()>` | `crates/contextra-adapt/src/contract.rs:25` | Aktualisiert den Zustand für den gewählten Arm mit beobachteter Belohnung. |

#### `trait StateSnapshot`
- Ort: `crates/contextra-adapt/src/contract.rs:29`
```rust
pub trait StateSnapshot
```
Dokumentation: Serialisierbarer Zustandsschnappschuss für Regler und Bandit-Profile.

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `fn to_snapshot_bytes(&self) -> Result<Vec<u8>>` | `crates/contextra-adapt/src/contract.rs:31` | Serialisiert den Zustand in einen defensiven Byte-Puffer. |
| `fn from_snapshot_bytes(bytes: &[u8]) -> Result<Self> where Self: Sized` | `crates/contextra-adapt/src/contract.rs:33` | Stellt den Zustand aus einem Byte-Puffer wieder her. |

#### `fn _assert_contract_conformance`
- Ort: `crates/contextra-adapt/src/contract.rs:40`
```rust
pub fn _assert_contract_conformance<T: StateSnapshot>(item: &T, bytes: &[u8]) -> Result<()>
```
Dokumentation: Konformitätsprüfung für Contract-Traits.

### Modul `crate`

#### `mod bandit`
- Ort: `crates/contextra-adapt/src/lib.rs:80`
```rust
pub mod bandit
```
- Gate: `#[cfg(feature = "bandit-routing")]`

#### `mod bandit_algorithms`
- Ort: `crates/contextra-adapt/src/lib.rs:52`
```rust
pub mod bandit_algorithms
```
Dokumentation: Bandit-Algorithmen (LinUCB, Sketched-Bandits, Flow-Corrected Thompson Sampling, RIE-Greedy).

#### `mod clock`
- Ort: `crates/contextra-adapt/src/lib.rs:26`
```rust
pub mod clock
```
- Gate: `#[cfg(feature = "std")]`

#### `mod clock`
- Ort: `crates/contextra-adapt/src/lib.rs:28`
```rust
pub mod clock
```

#### `mod compat`
- Ort: `crates/contextra-adapt/src/lib.rs:30`
```rust
pub mod compat
```
- Gate: `#[cfg(feature = "workspace")]`

#### `mod contract`
- Ort: `crates/contextra-adapt/src/lib.rs:31`
```rust
pub mod contract
```

#### `mod control`
- Ort: `crates/contextra-adapt/src/lib.rs:44`
```rust
pub mod control
```
Dokumentation: Regelungstechnik & Latenzsteuerung (PID, Latency Controller, Homeostat, Lyapunov).

#### `mod decay`
- Ort: `crates/contextra-adapt/src/lib.rs:73`
```rust
pub mod decay
```
Dokumentation: Zeitliches und Score-basiertes Abklingen (Adaptive Decay Controller).

#### `mod decay_controller`
- Ort: `crates/contextra-adapt/src/lib.rs:81`
```rust
pub mod decay_controller
```

#### `mod drift`
- Ort: `crates/contextra-adapt/src/lib.rs:82`
```rust
pub mod drift
```

#### `mod drift_detection`
- Ort: `crates/contextra-adapt/src/lib.rs:68`
```rust
pub mod drift_detection
```
Dokumentation: Konzept- und Verteilungs-Drift (Catoni M-Estimator, Ensemble Drift Watcher, Policy Bridge).

#### `mod error`
- Ort: `crates/contextra-adapt/src/lib.rs:32`
```rust
pub mod error
```

#### `mod flow_thompson`
- Ort: `crates/contextra-adapt/src/lib.rs:84`
```rust
pub mod flow_thompson
```
- Gate: `#[cfg(feature = "flow-corrected-thompson")]`

#### `mod homeostat`
- Ort: `crates/contextra-adapt/src/lib.rs:85`
```rust
pub mod homeostat
```

#### `mod lyapunov`
- Ort: `crates/contextra-adapt/src/lib.rs:86`
```rust
pub mod lyapunov
```

#### `mod offpolicy`
- Ort: `crates/contextra-adapt/src/lib.rs:87`
```rust
pub mod offpolicy
```

#### `mod pid`
- Ort: `crates/contextra-adapt/src/lib.rs:88`
```rust
pub mod pid
```

#### `mod pid_latency_controller`
- Ort: `crates/contextra-adapt/src/lib.rs:89`
```rust
pub mod pid_latency_controller
```

#### `mod policy`
- Ort: `crates/contextra-adapt/src/lib.rs:62`
```rust
pub mod policy
```
Dokumentation: Policy-Evaluation & Shadow Execution (Off-Policy IPS, Shadow Mode).

#### `mod rie_greedy`
- Ort: `crates/contextra-adapt/src/lib.rs:91`
```rust
pub mod rie_greedy
```
- Gate: `#[cfg(feature = "rie-greedy-personalization")]`

#### `mod rng`
- Ort: `crates/contextra-adapt/src/lib.rs:33`
```rust
pub mod rng
```

#### `mod shadow_mode`
- Ort: `crates/contextra-adapt/src/lib.rs:92`
```rust
pub mod shadow_mode
```

#### `mod strategy`
- Ort: `crates/contextra-adapt/src/lib.rs:34`
```rust
pub mod strategy
```

### Modul `decay_controller`

#### `struct AdaptiveDecayController`
- Ort: `crates/contextra-adapt/src/decay_controller.rs:67`
```rust
pub struct AdaptiveDecayController
```
Dokumentation: Adaptive Decay Controller für time-weighted Cache-Eviction.
- Impl-Block `crates/contextra-adapt/src/decay_controller.rs:71`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new(config: DecayControllerConfig) -> Self` | `crates/contextra-adapt/src/decay_controller.rs:73` | Erstellt einen neuen `AdaptiveDecayController` mit der angegebenen Konfiguration. |
| `pub fn with_defaults() -> Self` | `crates/contextra-adapt/src/decay_controller.rs:78` | Erstellt einen neuen `AdaptiveDecayController` mit Standardwerten. |
| `pub fn config(&self) -> &DecayControllerConfig` | `crates/contextra-adapt/src/decay_controller.rs:83` | Liefer eine Referenz auf die Konfiguration zurück. |
| `pub fn system_temperature(&self, inputs: &DecaySignalInputs) -> f32` | `crates/contextra-adapt/src/decay_controller.rs:89` | Berechnet aktuelle Systemtemperatur T(t) ∈ `[0, 1]`. |
| `pub fn effective_half_life(&self, temperature: f32) -> f32` | `crates/contextra-adapt/src/decay_controller.rs:96` | Effektive Half-Life unter aktueller Temperatur. |
| `pub fn effective_score( &self, base_score: f32, elapsed_tx: u64, inputs: &DecaySignalInputs, ) -> f32` | `crates/contextra-adapt/src/decay_controller.rs:103` | Temperatur-adjustierter Score für einen Chunk. `elapsed_tx`: TxId(now) - TxId(chunk_created). ADR-016: kein SystemTime. |
| `pub fn should_evict( &self, base_score: f32, elapsed_tx: u64, inputs: &DecaySignalInputs, ) -> bool` | `crates/contextra-adapt/src/decay_controller.rs:119` | Gibt true wenn Chunk unter Eviction-Schwelle. Wird von Background Workers (background_workers.rs) für Sweep verwendet. |

#### `struct DecayControllerConfig`
- Ort: `crates/contextra-adapt/src/decay_controller.rs:46`
```rust
pub struct DecayControllerConfig
```
Dokumentation: Konfiguration für Adaptive Decay Controller (Cache-Eviction mit Time-Weighted Decay) (via MaintenanceConfig).
- Impl-Block (Trait `Default`) `crates/contextra-adapt/src/decay_controller.rs:55`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `kappa` | `f32` | `crates/contextra-adapt/src/decay_controller.rs:48` |
| `base_half_life_tx` | `u64` | `crates/contextra-adapt/src/decay_controller.rs:50` |
| `eviction_threshold` | `f32` | `crates/contextra-adapt/src/decay_controller.rs:52` |

#### `struct DecaySignalInputs`
- Ort: `crates/contextra-adapt/src/decay_controller.rs:20`
```rust
pub struct DecaySignalInputs
```
Dokumentation: Inputs for Adaptive Decay Controller (Cache-Eviction mit Time-Weighted Decay) aus bestehenden System-Metriken.
- Impl-Block (Trait `Default`) `crates/contextra-adapt/src/decay_controller.rs:33`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `tombstone_ratio` | `f32` | `crates/contextra-adapt/src/decay_controller.rs:22` |
| `query_load_inverse` | `f32` | `crates/contextra-adapt/src/decay_controller.rs:26` |
| `w_tombstone` | `f32` | `crates/contextra-adapt/src/decay_controller.rs:28` |
| `w_query` | `f32` | `crates/contextra-adapt/src/decay_controller.rs:30` |

### Modul `drift`

#### `struct CatoniDriftDetector`
- Ort: `crates/contextra-adapt/src/drift.rs:40`
```rust
pub struct CatoniDriftDetector
```
Dokumentation: Catoni-M-estimator based robust drift detector (arXiv:2505.20051 & arXiv:2501.10974). Uses influence function $\psi(x) = \frac{x}{1 + |x|}$ for $O(1)$ robust mean estimation, paired with CUSUM sequential change-point accumulation. Deterministic (P28) and robust against isolated reward outliers.
- Impl-Block (Trait `Default`) `crates/contextra-adapt/src/drift.rs:53`
- Impl-Block `crates/contextra-adapt/src/drift.rs:59`
- Impl-Block (Trait `DriftDetector`) `crates/contextra-adapt/src/drift.rs:78`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `catoni_mu` | `f64` | `crates/contextra-adapt/src/drift.rs:42` |
| `cusum_sum` | `f64` | `crates/contextra-adapt/src/drift.rs:44` |
| `threshold` | `f64` | `crates/contextra-adapt/src/drift.rs:46` |
| `alpha` | `f64` | `crates/contextra-adapt/src/drift.rs:48` |
| `baseline_reward` | `f64` | `crates/contextra-adapt/src/drift.rs:50` |

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new(baseline_reward: f64, threshold: f64, alpha: f64) -> Self` | `crates/contextra-adapt/src/drift.rs:61` | Creates a new `CatoniDriftDetector` with specified baseline, threshold, and learning rate. |
| `pub fn reset(&mut self)` | `crates/contextra-adapt/src/drift.rs:72` | Resets the CUSUM accumulator and restores `catoni_mu` to baseline. |

#### `struct DriftPolicyBridge`
- Ort: `crates/contextra-adapt/src/drift.rs:154`
```rust
pub struct DriftPolicyBridge
```
Dokumentation: Adapter zur Kopplung des `LyapunovDriftWatcher` mit einer `BanditPolicy`.
- Impl-Block (Trait `Default`) `crates/contextra-adapt/src/drift.rs:165`
- Impl-Block `crates/contextra-adapt/src/drift.rs:176`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `watcher` | `LyapunovDriftWatcher` | `crates/contextra-adapt/src/drift.rs:156` |
| `k_drift` | `f32` | `crates/contextra-adapt/src/drift.rs:158` |
| `alpha_max` | `f32` | `crates/contextra-adapt/src/drift.rs:160` |
| `gamma` | `f32` | `crates/contextra-adapt/src/drift.rs:162` |

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new(window_size: usize, k_drift: f32, alpha_max: f32, gamma: f32) -> Self` | `crates/contextra-adapt/src/drift.rs:178` | Erstellt eine neue `DriftPolicyBridge` mit benutzerdefinierten Parametern. |
| `pub fn observe_and_react<P: crate::bandit::BanditPolicy + ?Sized>( &mut self, current_scores: &[f32], policy: &mut P, ) -> LyapunovResult` | `crates/contextra-adapt/src/drift.rs:190` | Nimmt aktuelle Non-Conformity-Scores auf, wertet den Drift-Status aus und wendet bei erkannter Drift eine Strafe auf die `BanditPolicy` an. |

#### `struct EnsembleDriftWatcher`
- Ort: `crates/contextra-adapt/src/drift.rs:110`
```rust
pub struct EnsembleDriftWatcher
```
Dokumentation: Ensemble combination of [`LyapunovDriftWatcher`] and [`CatoniDriftDetector`]. Triggers drift response ONLY if BOTH detectors independently detect drift, significantly reducing false positive rates under noisy or outlier-prone reward streams.
- Impl-Block (Trait `Default`) `crates/contextra-adapt/src/drift.rs:117`
- Impl-Block `crates/contextra-adapt/src/drift.rs:126`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `lyapunov` | `LyapunovDriftWatcher` | `crates/contextra-adapt/src/drift.rs:112` |
| `catoni` | `CatoniDriftDetector` | `crates/contextra-adapt/src/drift.rs:114` |

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new(lyapunov: LyapunovDriftWatcher, catoni: CatoniDriftDetector) -> Self` | `crates/contextra-adapt/src/drift.rs:128` | Creates a new `EnsembleDriftWatcher`. |
| `pub fn observe_and_decide(&mut self, reward: f32) -> bool` | `crates/contextra-adapt/src/drift.rs:133` | Observes a new reward / score and decides if drift is detected by BOTH detectors. |
| `pub fn set_baseline(&mut self, baseline_scores: &[f32])` | `crates/contextra-adapt/src/drift.rs:140` | Sets the baseline distribution for the Lyapunov watcher and baseline mean for the Catoni detector. |

#### `enum DriftSignal`
- Ort: `crates/contextra-adapt/src/drift.rs:13`
```rust
pub enum DriftSignal
```
Dokumentation: Drift detection signal returned by a [`DriftDetector`].

Varianten:

- `Stable` — `crates/contextra-adapt/src/drift.rs:15`
- `Detected` — `crates/contextra-adapt/src/drift.rs:17`

#### `trait DriftDetector`
- Ort: `crates/contextra-adapt/src/drift.rs:27`
```rust
pub trait DriftDetector
```
Dokumentation: Generic interface for sequential O(1) drift detectors. # Abgrenzung zu `contextra-rank::DriftDetector` `contextra-adapt::DriftDetector` ist ein O(1) sequentielles Online-Signal-Trait für synchrone Regler und Bandit-Policies in Ring 0. Es arbeitet direkt auf Skalaren (Scores/Rewards). Im Gegensatz dazu führt `contextra-rank::DriftDetector` höherstufige Verteilungs- und Ranking-Analysen auf gespeicherten Resultatlisten und Metrikserien durch.

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `fn observe(&mut self, reward: f32) -> DriftSignal` | `crates/contextra-adapt/src/drift.rs:29` | Observes a reward or non-conformity score and returns the updated drift signal. |
| `fn is_drifting(&self) -> bool` | `crates/contextra-adapt/src/drift.rs:31` | Returns `true` if drift is currently detected. |

### Modul `error`

#### `enum AdaptError`
- Ort: `crates/contextra-adapt/src/error.rs:10`
```rust
pub enum AdaptError
```
Dokumentation: Zentraler Fehlertyp für alle adaptiven Regler, Bandits und Drift-Watcher.
- Impl-Block (Trait `From<crate::bandit::BanditError>`) `crates/contextra-adapt/src/error.rs:47`
- Impl-Block (Trait `From<crate::flow_thompson::FcTsError>`) `crates/contextra-adapt/src/error.rs:69`
- Impl-Block (Trait `From<crate::flow_thompson::OffPolicyError>`) `crates/contextra-adapt/src/error.rs:84`
- Impl-Block (Trait `From<crate::rie_greedy::RieGreedyError>`) `crates/contextra-adapt/src/error.rs:95`

Varianten:

- `DimensionMismatch{` — `crates/contextra-adapt/src/error.rs:13`
- `InvalidConfig(String)` — `crates/contextra-adapt/src/error.rs:22`
- `NonFinite(String)` — `crates/contextra-adapt/src/error.rs:26`
- `PrecisionMatrixDriftDetected{` — `crates/contextra-adapt/src/error.rs:30`
- `ZeroPropensityViolation(String)` — `crates/contextra-adapt/src/error.rs:39`
- `InvalidInput(String)` — `crates/contextra-adapt/src/error.rs:43`

#### `type Result`
- Ort: `crates/contextra-adapt/src/error.rs:110`
```rust
pub type Result<T> = std::result::Result<T, AdaptError>
```
Dokumentation: Ergebnistyp für Operationen in `contextra-adapt`.

### Modul `flow_thompson`

#### `struct BanditShadowReport`
- Ort: `crates/contextra-adapt/src/flow_thompson.rs:861`
```rust
pub struct BanditShadowReport
```

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `sample_count` | `u64` | `crates/contextra-adapt/src/flow_thompson.rs:863` |
| `cumulative_regret` | `f64` | `crates/contextra-adapt/src/flow_thompson.rs:865` |
| `p99_latency_delta_us` | `f64` | `crates/contextra-adapt/src/flow_thompson.rs:867` |
| `reward_improvement_ratio` | `Option<f32>` | `crates/contextra-adapt/src/flow_thompson.rs:869` |

#### `struct DefaultBanditFlipGate`
- Ort: `crates/contextra-adapt/src/flow_thompson.rs:885`
```rust
pub struct DefaultBanditFlipGate
```
- Impl-Block (Trait `BanditDefaultFlipGate`) `crates/contextra-adapt/src/flow_thompson.rs:887`

#### `struct DiagonalApproximationBandit`
- Ort: `crates/contextra-adapt/src/flow_thompson.rs:640`
```rust
pub struct DiagonalApproximationBandit
```
- Impl-Block `crates/contextra-adapt/src/flow_thompson.rs:645`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new( propensities: std::collections::BTreeMap<crate::strategy::RetrievalStrategy, f32>, ) -> Self` | `crates/contextra-adapt/src/flow_thompson.rs:647` | Erstellt eine neue `DiagonalApproximationBandit`-Referenzpolitik mit der gegebenen Propensitätsmap. |
| `pub fn from_pairs( pairs: impl IntoIterator<Item = (crate::strategy::RetrievalStrategy, f32)>, ) -> Self` | `crates/contextra-adapt/src/flow_thompson.rs:665` | Erstellt eine `DiagonalApproximationBandit`-Referenzpolitik aus Strategie-Propensitäts-Paaren. ``` use contextra_adapt::flow_thompson::DiagonalApproximationBand |
| `pub fn propensity(&self, strategy: crate::strategy::RetrievalStrategy) -> f32` | `crates/contextra-adapt/src/flow_thompson.rs:674` | Liefert die Propensity P(a \| x) für eine gegebene RetrievalStrategy. |

#### `struct FcTsArmSet`
- Ort: `crates/contextra-adapt/src/flow_thompson.rs:763`
```rust
pub struct FcTsArmSet
```
- Impl-Block `crates/contextra-adapt/src/flow_thompson.rs:768`
- Impl-Block (Trait `OffPolicyCompatibility`) `crates/contextra-adapt/src/flow_thompson.rs:826`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `arms` | `Vec<FlowCorrectedThompsonBandit>` | `crates/contextra-adapt/src/flow_thompson.rs:765` |

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn with_shadow_sink( mut self, sink: std::sync::Arc<dyn crate::shadow_mode::ShadowSink<u32>>, ) -> Self` | `crates/contextra-adapt/src/flow_thompson.rs:770` | Konfiguriert eine Diskrepanz-Senke für alle Arme in der Menge (§12). |
| `pub fn select_arm(&self, context: &[f32], rng: &mut dyn FcTsRng) -> Result<u32, FcTsError>` | `crates/contextra-adapt/src/flow_thompson.rs:785` | Wählt den Arm mit dem höchsten gesampelten Score aus (Argmax). Bei Gleichstand wird der kleinste Arm-Index gewählt. Bei konfiguriertem Shadow-Sink wird zusätzli |

#### `struct FcTsConfig`
- Ort: `crates/contextra-adapt/src/flow_thompson.rs:96`
```rust
pub struct FcTsConfig
```
- Impl-Block (Trait `Default`) `crates/contextra-adapt/src/flow_thompson.rs:113`
- Impl-Block `crates/contextra-adapt/src/flow_thompson.rs:127`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `dim` | `usize` | `crates/contextra-adapt/src/flow_thompson.rs:98` |
| `lambda` | `f32` | `crates/contextra-adapt/src/flow_thompson.rs:100` |
| `noise_var` | `f32` | `crates/contextra-adapt/src/flow_thompson.rs:102` |
| `window_capacity` | `usize` | `crates/contextra-adapt/src/flow_thompson.rs:104` |
| `explore_scale` | `f32` | `crates/contextra-adapt/src/flow_thompson.rs:106` |
| `drift_ridge` | `f32` | `crates/contextra-adapt/src/flow_thompson.rs:108` |
| `max_drift_norm` | `f32` | `crates/contextra-adapt/src/flow_thompson.rs:110` |

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn validate(&self) -> Result<(), FcTsError>` | `crates/contextra-adapt/src/flow_thompson.rs:129` | Validiert die Konfigurationsparameter. |

#### `struct FcTsSamplingDistribution`
- Ort: `crates/contextra-adapt/src/flow_thompson.rs:704`
```rust
pub struct FcTsSamplingDistribution
```
- Impl-Block `crates/contextra-adapt/src/flow_thompson.rs:709`
- Impl-Block (Trait `OffPolicyCompatibility`) `crates/contextra-adapt/src/flow_thompson.rs:743`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new( probabilities: std::collections::BTreeMap<crate::strategy::RetrievalStrategy, f32>, ) -> Self` | `crates/contextra-adapt/src/flow_thompson.rs:711` | Erstellt eine neue `FcTsSamplingDistribution`-Instanz mit der gegebenen Wahrscheinlichkeitsmap. |
| `pub fn from_pairs( pairs: impl IntoIterator<Item = (crate::strategy::RetrievalStrategy, f32)>, ) -> Self` | `crates/contextra-adapt/src/flow_thompson.rs:729` | Erstellt eine `FcTsSamplingDistribution`-Instanz aus Strategie-Wahrscheinlichkeits-Paaren. ``` use contextra_adapt::flow_thompson::FcTsSamplingDistribution; use |
| `pub fn probability(&self, strategy: crate::strategy::RetrievalStrategy) -> f32` | `crates/contextra-adapt/src/flow_thompson.rs:738` | Liefert die FC-TS-Sampling-Wahrscheinlichkeit für eine gegebene RetrievalStrategy. |

#### `struct FlowCorrectedThompsonBandit`
- Ort: `crates/contextra-adapt/src/flow_thompson.rs:187`
```rust
pub struct FlowCorrectedThompsonBandit
```
- Impl-Block (Trait `std::fmt::Debug`) `crates/contextra-adapt/src/flow_thompson.rs:223`
- Impl-Block `crates/contextra-adapt/src/flow_thompson.rs:239`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new(config: FcTsConfig) -> Result<Self, FcTsError>` | `crates/contextra-adapt/src/flow_thompson.rs:241` | Erstellt eine neue FC-TS-Bandit-Instanz mit der gegebenen Konfiguration. |
| `pub fn with_shadow_sink( mut self, sink: std::sync::Arc<dyn crate::shadow_mode::ShadowSink<u32>>, ) -> Self` | `crates/contextra-adapt/src/flow_thompson.rs:273` | Konfiguriert eine Diskrepanz-Senke für den Shadow-Mode-Vergleich (§12). |
| `pub fn shadow_sink(&self) -> Option<&std::sync::Arc<dyn crate::shadow_mode::ShadowSink<u32>>>` | `crates/contextra-adapt/src/flow_thompson.rs:282` | Gibt eine Referenz auf die optional konfigurierte Diskrepanz-Senke zurück. |
| `pub fn config(&self) -> &FcTsConfig` | `crates/contextra-adapt/src/flow_thompson.rs:287` | Gibt eine Referenz auf die Konfiguration zurück. |
| `pub fn mu(&self) -> &[f32]` | `crates/contextra-adapt/src/flow_thompson.rs:292` | Gibt den aktuellen Mittelwert-Vektor $\mu$ zurück. |
| `pub fn drift_rate(&self) -> &[f32]` | `crates/contextra-adapt/src/flow_thompson.rs:297` | Gibt die aktuelle Drift-Rate $\delta$ zurück. |
| `pub fn inv_a(&self) -> &[f32]` | `crates/contextra-adapt/src/flow_thompson.rs:302` | Gibt die inverse Kovarianzmatrix $A^{-1}$ ($d \times d$, row-major) zurück. |
| `pub fn update_with_flow( &mut self, context: &[f32], reward: f32, observation_time: u64, confidence_weight: f32, ) -> Result<(), FcTsError>` | `crates/contextra-adapt/src/flow_thompson.rs:317` | Akkumuliert ein neues Beobachtungstupel mit konfidenzgewichtetem Transport (§21.3). # Parameter - `context`: Kontext-Vektor $x \in \mathbb{R}^d$. - `reward`: Be |
| `pub fn sample_score(&self, context: &[f32], rng: &mut dyn FcTsRng) -> Result<f32, FcTsError>` | `crates/contextra-adapt/src/flow_thompson.rs:430` | Berechnet einen zufälligen Thompson-Sample Score für den Kontext $x$. $w_i = \mu_i + v \cdot \sqrt{\max(A^{-1}_{i,i}, 0)} \cdot z_i$, $\text{Score} = \langle w, |
| `pub fn baseline_score(&self, context: &[f32]) -> Result<f32, FcTsError>` | `crates/contextra-adapt/src/flow_thompson.rs:461` | Berechnet den Baseline-Mittelwert-Score (Sherman-Morrison $\langle \mu, x \rangle$) für den Kontext $x$. |
| `pub fn select_arm(&self, context: &[f32], rng: &mut dyn FcTsRng) -> Result<u32, FcTsError>` | `crates/contextra-adapt/src/flow_thompson.rs:487` | Wählt die Aktion für diese Bandit-Instanz und protokolliert Diskrepanzen bei konfiguriertem Shadow-Sink. |
| `pub fn recompute_drift_rate_from_window(&mut self, _token: &Ring3Token)` | `crates/contextra-adapt/src/flow_thompson.rs:514` | Berechnet die Drift-Rate $\delta$ aus dem aktuellen Fenster neu via Ridge-Regression (AK-18). Nutzt Conjugate Gradient zur Lösung von $(Z^T Z + \rho I) \delta = |

#### `struct Ring3Token`
- Ort: `crates/contextra-adapt/src/flow_thompson.rs:170`
```rust
pub struct Ring3Token
```

#### `struct SplitMix64`
- Ort: `crates/contextra-adapt/src/flow_thompson.rs:72`
```rust
pub struct SplitMix64
```
- Impl-Block `crates/contextra-adapt/src/flow_thompson.rs:76`
- Impl-Block (Trait `FcTsRng`) `crates/contextra-adapt/src/flow_thompson.rs:83`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new(seed: u64) -> Self` | `crates/contextra-adapt/src/flow_thompson.rs:78` | Erstellt eine neue `SplitMix64`-Instanz mit dem gegebenen Seed. |

#### `enum FcTsError`
- Ort: `crates/contextra-adapt/src/flow_thompson.rs:29`
```rust
pub enum FcTsError
```

Varianten:

- `DimensionMismatch{` — `crates/contextra-adapt/src/flow_thompson.rs:32`
- `NonFinite` — `crates/contextra-adapt/src/flow_thompson.rs:40`
- `InvalidConfig(String)` — `crates/contextra-adapt/src/flow_thompson.rs:43`

#### `enum OffPolicyError`
- Ort: `crates/contextra-adapt/src/flow_thompson.rs:695`
```rust
pub enum OffPolicyError
```

Varianten:

- `ZeroPropensityViolation(crate::strategy::RetrievalStrategy)` — `crates/contextra-adapt/src/flow_thompson.rs:698`

#### `trait BanditDefaultFlipGate`
- Ort: `crates/contextra-adapt/src/flow_thompson.rs:874`
```rust
pub trait BanditDefaultFlipGate
```

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `fn should_flip(&self, report: &BanditShadowReport) -> bool` | `crates/contextra-adapt/src/flow_thompson.rs:879` | `true` ⇔ alle drei Bedingungen erfüllt: 1. `sample_count >= MIN_BANDIT_SHADOW_SAMPLES` (10_000) 2. `cumulative_regret <= MAX_CUMULATIVE_REGRET_THRESHOLD` (0.0)  |

#### `trait FcTsRng`
- Ort: `crates/contextra-adapt/src/flow_thompson.rs:48`
```rust
pub trait FcTsRng
```

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `fn next_u64(&mut self) -> u64` | `crates/contextra-adapt/src/flow_thompson.rs:50` | Generiert die nächste 64-Bit vorzeichenlose Ganzzahl. |
| `fn next_f32(&mut self) -> f32` | `crates/contextra-adapt/src/flow_thompson.rs:53` | Generiert eine gleichverteilte Fließkommazahl im halboffenen Intervall `[0, 1)`. |
| `fn next_standard_normal(&mut self) -> f32` | `crates/contextra-adapt/src/flow_thompson.rs:59` | Generiert eine standardnormalverteilte Fließkommazahl $\mathcal{N}(0, 1)$ mittels Box-Muller-Transformation. |

#### `trait OffPolicyCompatibility`
- Ort: `crates/contextra-adapt/src/flow_thompson.rs:681`
```rust
pub trait OffPolicyCompatibility
```

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `fn verify_positivity( &self, reference_policy: &DiagonalApproximationBandit, ) -> Result<(), OffPolicyError>` | `crates/contextra-adapt/src/flow_thompson.rs:686` | Gibt `Err` zurück, wenn für irgendeinen Arm eine Propensity von 0 unter der Referenz-Policy gemessen wird, während FC-TS diesem Arm eine positive Sampling-Wahrs |

#### `fn ring3_background_task_token`
- Ort: `crates/contextra-adapt/src/flow_thompson.rs:180`
```rust
pub fn ring3_background_task_token() -> Ring3Token
```

#### `const MAX_CUMULATIVE_REGRET_THRESHOLD`
- Ort: `crates/contextra-adapt/src/flow_thompson.rs:856`
```rust
pub const MAX_CUMULATIVE_REGRET_THRESHOLD: f64 = 0.0
```
Dokumentation: Maximales zulässiges kumulatives Regret für den Bandit-Default-Flip (≤ 0.0).

#### `const MIN_BANDIT_SHADOW_SAMPLES`
- Ort: `crates/contextra-adapt/src/flow_thompson.rs:854`
```rust
pub const MIN_BANDIT_SHADOW_SAMPLES: u64 = 10_000
```

### Modul `homeostat`

#### `struct RerankDeadline`
- Ort: `crates/contextra-adapt/src/homeostat.rs:102`
```rust
pub struct RerankDeadline<C: ?Sized + Clock = SystemClock>
```
Dokumentation: Hard deadline manager for candidate retrieval and Cross-Encoder reranking phases (P11 requirement). # Intended Integration Point Evaluated before or during candidate hydration and Cross-Encoder batch inference steps in `Collection::hybrid_search_reranked`. If `deadline_exceeded(started_at)` or `deadline_exceeded_nanos` returns `true`, the search pipeline should immediately break out and return the partially hydrated/reranked candidate results accumulated up to that point rather than blocking indefinitely.
- Impl-Block `crates/contextra-adapt/src/homeostat.rs:108`
- Impl-Block `crates/contextra-adapt/src/homeostat.rs:115`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `budget` | `Duration` | `crates/contextra-adapt/src/homeostat.rs:104` |

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new(budget: Duration) -> Self` | `crates/contextra-adapt/src/homeostat.rs:110` | Creates a new `RerankDeadline` with the specified duration budget. |
| `pub fn new_with_clock(budget: Duration, clock: std::sync::Arc<C>) -> Self` | `crates/contextra-adapt/src/homeostat.rs:117` | Creates a new `RerankDeadline` with explicit `Clock` port injection. |
| `pub fn start_nanos(&self) -> u64` | `crates/contextra-adapt/src/homeostat.rs:122` | Returns current monotonic timestamp in nanoseconds from injected clock. |
| `pub fn deadline_exceeded_nanos(&self, started_at_nanos: u64) -> bool` | `crates/contextra-adapt/src/homeostat.rs:127` | Checks if elapsed nanoseconds since `started_at_nanos` equals or exceeds deadline budget. |
| `pub fn deadline_exceeded(&self, started_at: std::time::Instant) -> bool` | `crates/contextra-adapt/src/homeostat.rs:137` | Legacy helper using monotonic_nanos delta based on Instant baseline. |

#### `struct RerankPidController`
- Ort: `crates/contextra-adapt/src/homeostat.rs:21`
```rust
pub struct RerankPidController
```
- Impl-Block (Trait `Default`) `crates/contextra-adapt/src/homeostat.rs:26`
- Impl-Block `crates/contextra-adapt/src/homeostat.rs:33`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new( target_p95_latency_ms: f32, k_min: usize, k_max: usize, initial_pool: usize, ) -> Self` | `crates/contextra-adapt/src/homeostat.rs:35` | Creates a new `RerankPidController` with default PID coefficients and configured parameters. |
| `pub fn kp(&self) -> f32` | `crates/contextra-adapt/src/homeostat.rs:52` | Accessor for `kp`. |
| `pub fn ki(&self) -> f32` | `crates/contextra-adapt/src/homeostat.rs:57` | Accessor for `ki`. |
| `pub fn kd(&self) -> f32` | `crates/contextra-adapt/src/homeostat.rs:62` | Accessor for `kd`. |
| `pub fn target_p95_latency_ms(&self) -> f32` | `crates/contextra-adapt/src/homeostat.rs:67` | Accessor for target latency. |
| `pub fn k_pool(&self) -> usize` | `crates/contextra-adapt/src/homeostat.rs:72` | Accessor for current `k_pool`. |
| `pub fn k_min(&self) -> usize` | `crates/contextra-adapt/src/homeostat.rs:79` | Accessor for `k_min`. |
| `pub fn k_max(&self) -> usize` | `crates/contextra-adapt/src/homeostat.rs:84` | Accessor for `k_max`. |
| `pub fn update(&mut self, observed_p95_latency_ms: f32) -> usize` | `crates/contextra-adapt/src/homeostat.rs:89` | Updates the controller state with observed latency and returns new pool size. |

### Modul `lyapunov`

#### `struct DriftReason`
- Ort: `crates/contextra-adapt/src/lyapunov.rs:48`
```rust
pub struct DriftReason
```
Dokumentation: Grund für erkannte Verteilungsverschiebung (Distributional Drift).

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `kl_divergence` | `f32` | `crates/contextra-adapt/src/lyapunov.rs:50` |
| `lyapunov_exponent` | `f32` | `crates/contextra-adapt/src/lyapunov.rs:52` |

#### `struct LyapunovDriftWatcher`
- Ort: `crates/contextra-adapt/src/lyapunov.rs:102`
```rust
pub struct LyapunovDriftWatcher
```
Dokumentation: Proaktiver Drift-Wächter auf Basis diskreter Lyapunov-Exponenten über KL-Divergenzen.
- Impl-Block (Trait `Default`) `crates/contextra-adapt/src/lyapunov.rs:113`
- Impl-Block `crates/contextra-adapt/src/lyapunov.rs:119`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `window_size` | `usize` | `crates/contextra-adapt/src/lyapunov.rs:104` |
| `divergence_history` | `VecDeque<f32>` | `crates/contextra-adapt/src/lyapunov.rs:106` |
| `baseline_distribution` | `Vec<f32>` | `crates/contextra-adapt/src/lyapunov.rs:108` |
| `latest_result` | `Option<LyapunovResult>` | `crates/contextra-adapt/src/lyapunov.rs:110` |

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new(window_size: usize) -> Self` | `crates/contextra-adapt/src/lyapunov.rs:121` | Erstellt einen neuen `LyapunovDriftWatcher` mit der angegebenen Fenstergröße. |
| `pub fn observe_score(&mut self, score: f32) -> LyapunovResult` | `crates/contextra-adapt/src/lyapunov.rs:134` | Fügt einen einzelnen Non-Conformity-Score zur Historie hinzu und aktualisiert den Drift-Status. |
| `pub fn analyze(&self) -> LyapunovResult` | `crates/contextra-adapt/src/lyapunov.rs:139` | Gibt das neueste Analyseergebnis der Lyapunov-Drift-Berechnung zurück. |
| `pub fn latest_result(&self) -> Option<&LyapunovResult>` | `crates/contextra-adapt/src/lyapunov.rs:146` | Gibt eine Referenz auf das neueste Analyseergebnis zurück. |
| `pub fn status_str(&self) -> &'static str` | `crates/contextra-adapt/src/lyapunov.rs:151` | Gibt den aktuellen Drift-Status als menschenlesbare Zeichenkette ("stabil", "warnung", "kritisch", "unbekannt") zurück. |
| `pub fn set_baseline(&mut self, baseline_scores: &[f32])` | `crates/contextra-adapt/src/lyapunov.rs:168` | Setzt die Baseline-Verteilung der Non-Conformity-Scores aus dem Kalibrierungs-Warmup. |
| `pub fn update(&mut self, current_scores: &[f32]) -> LyapunovResult` | `crates/contextra-adapt/src/lyapunov.rs:176` | Nimmt aktuelle Non-Conformity-Scores auf und berechnet den Drift-Status. |

#### `enum LyapunovResult`
- Ort: `crates/contextra-adapt/src/lyapunov.rs:58`
```rust
pub enum LyapunovResult
```
Dokumentation: Ergebnis der Lyapunov-Drift-Analyse.
- Impl-Block `crates/contextra-adapt/src/lyapunov.rs:75`

Varianten:

- `Stable{` — `crates/contextra-adapt/src/lyapunov.rs:60`
- `DriftDetected{` — `crates/contextra-adapt/src/lyapunov.rs:65`
- `InsufficientData` — `crates/contextra-adapt/src/lyapunov.rs:72`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn is_drift_detected(&self) -> bool` | `crates/contextra-adapt/src/lyapunov.rs:77` | Returns `true` if this result represents a detected distributional drift. |
| `pub fn apply_drift_if_detected<P: crate::bandit::BanditPolicy + ?Sized>( &self, policy: &mut P, k_drift: f32, alpha_max: f32, gamma: f32, ) -> bool` | `crates/contextra-adapt/src/lyapunov.rs:84` | Wendet `apply_drift_penalty` auf die gegebene `BanditPolicy` an, falls `DriftDetected` vorliegt. Gibt `true` zurück, falls Drift erkannt und die Penalty angewen |

### Modul `offpolicy`

#### `struct OffPolicyEvaluator`
- Ort: `crates/contextra-adapt/src/offpolicy.rs:81`
```rust
pub struct OffPolicyEvaluator
```
Dokumentation: Evaluator für Inverse Propensity Scoring (IPS) zur kontrafaktischen Off-Policy-Schätzung.
- Impl-Block `crates/contextra-adapt/src/offpolicy.rs:87`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new() -> Self` | `crates/contextra-adapt/src/offpolicy.rs:92` | Erstellt einen neuen `OffPolicyEvaluator` mit leeren Akkumulatoren. *Hinweis zur Spezifikation:* Von der Spezifikation normativ nicht explizit genannt, jedoch f |
| `pub fn observe( &mut self, target_action: u32, logged_action: u32, propensity: f32, reward: f32, )` | `crates/contextra-adapt/src/offpolicy.rs:110` | Beobachtet ein geloggtes Event und akkumuliert den IPS-Wert. - `target_action`: Die Aktion, die von der zu evaluierenden Ziel-Policy gewählt worden wäre. - `log |
| `pub fn estimate(&self) -> f64` | `crates/contextra-adapt/src/offpolicy.rs:134` | Liefert den aktuellen IPS-Schätzwert V_IPS = cumulative_ips / samples. Falls noch keine Samples beobachtet wurden (`samples == 0`), wird 0.0 zurückgegeben. |
| `pub fn cumulative_ips(&self) -> f64` | `crates/contextra-adapt/src/offpolicy.rs:139` | Liefert die kumulative IPS-Summe Σ (r_i / max(p_i, 0.01)). |
| `pub fn samples(&self) -> u64` | `crates/contextra-adapt/src/offpolicy.rs:144` | Liefert die Gesamtanzahl beobachteter Samples (inklusive verworfener). |
| `pub fn discarded(&self) -> u64` | `crates/contextra-adapt/src/offpolicy.rs:149` | Liefert die Anzahl verworfener Ungültig-Propensity-Samples. |
| `pub fn stats(&self) -> OffPolicyStats` | `crates/contextra-adapt/src/offpolicy.rs:154` | Liefert die Off-Policy-Evaluator-Statistiken. |

#### `struct OffPolicyStats`
- Ort: `crates/contextra-adapt/src/offpolicy.rs:74`
```rust
pub struct OffPolicyStats
```
Dokumentation: Statistiken über verworrene/ungültige Samples der Off-Policy-Evaluation.

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `discarded` | `u64` | `crates/contextra-adapt/src/offpolicy.rs:76` |

#### `struct RandomizedLoggingPolicy`
- Ort: `crates/contextra-adapt/src/offpolicy.rs:22`
```rust
pub struct RandomizedLoggingPolicy
```
Dokumentation: Randomisierte Logging-Policy für unverzerrte Datensammlung zur Off-Policy-Evaluation (§B.5.2.4 & §8.5).
- Impl-Block `crates/contextra-adapt/src/offpolicy.rs:29`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `epsilon` | `f32` | `crates/contextra-adapt/src/offpolicy.rs:24` |
| `num_actions` | `u32` | `crates/contextra-adapt/src/offpolicy.rs:26` |

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new(epsilon: f32, num_actions: u32) -> Self` | `crates/contextra-adapt/src/offpolicy.rs:31` | Erstellt eine neue `RandomizedLoggingPolicy`. |
| `pub fn compute_propensities(&self, greedy_action: u32) -> Vec<f32>` | `crates/contextra-adapt/src/offpolicy.rs:39` | Berechnet die Propensity P(a \| x) für jede Aktion 0..K-1 bei gegebener greedy/target Aktion. |
| `pub fn select_action(&self, greedy_action: u32, random_sample: f32) -> (u32, f32)` | `crates/contextra-adapt/src/offpolicy.rs:53` | Wählt eine Aktion aus und gibt den (Aktionsindex, Propensity) Vektor/Tupel zurück. `random_sample` muss im Intervall [0.0, 1.0) liegen. |

### Modul `pid`

#### `struct PidController`
- Ort: `crates/contextra-adapt/src/pid.rs:64`
```rust
pub struct PidController
```
Dokumentation: PID-Regler zur dynamischen Steuerung der Reranking-Kandidatenpool-Größe basierend auf Latenzmessungen. # Anti-Windup Strategien - Standard-Methode [`update`](Self::update): Nutzt einfaches Integrations-Clamping auf `[-max_integral, max_integral]`. - Erweiterte Methode [`update_with_anti_windup`](AntiWindupController::update_with_anti_windup): Stoppt die Integrationsakkumulation vollständig, sobald `is_actuator_saturated = true` signalisiert wird (Back-Calculation Pattern).
- Impl-Block (Trait `Default`) `crates/contextra-adapt/src/pid.rs:84`
- Impl-Block `crates/contextra-adapt/src/pid.rs:95`
- Impl-Block (Trait `AntiWindupController`) `crates/contextra-adapt/src/pid.rs:166`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `kp` | `f32` | `crates/contextra-adapt/src/pid.rs:66` |
| `ki` | `f32` | `crates/contextra-adapt/src/pid.rs:68` |
| `kd` | `f32` | `crates/contextra-adapt/src/pid.rs:70` |
| `target_latency_ms` | `f32` | `crates/contextra-adapt/src/pid.rs:72` |
| `min_pool_size` | `usize` | `crates/contextra-adapt/src/pid.rs:74` |
| `max_pool_size` | `usize` | `crates/contextra-adapt/src/pid.rs:76` |
| `current_pool_size` | `Option<usize>` | `crates/contextra-adapt/src/pid.rs:78` |

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new( target_latency_ms: f32, min_pool_size: usize, max_pool_size: usize, initial_pool: Option<usize>, ) -> Self` | `crates/contextra-adapt/src/pid.rs:99` | Erstellt einen neuen `PidController` mit expliziten Parametern. `min_pool_size` wird strikt auf mindestens 50 erzwungen (`min_pool_size.max(50)`). |
| `pub fn update(&mut self, dt: Duration, measured_latency_ms: f32) -> usize` | `crates/contextra-adapt/src/pid.rs:125` | Verarbeitet eine neue Latenz-Messung und gibt die neue Pool-Größe zurück. ANTI-WINDUP: Führt Integral-Update nur durch, wenn Stellwert nicht gesättigt ist. |
| `pub fn current_pool_size(&self) -> Option<usize>` | `crates/contextra-adapt/src/pid.rs:154` | Gibt die aktuell empfohlene Pool-Größe zurück. |
| `pub fn reset(&mut self)` | `crates/contextra-adapt/src/pid.rs:159` | Setzt Integral und Derivative auf 0 zurück (z.B. bei Konfigurationsänderung). |

#### `trait AntiWindupController`
- Ort: `crates/contextra-adapt/src/pid.rs:40`
```rust
pub trait AntiWindupController: Send + Sync
```
Dokumentation: Erweitert PID-Regler um konditionelles Anti-Windup bei Aktuatorsättigung. Unterschied zu einfachem Integral-Clamping: Der Integrator akkumuliert NUR, wenn die Stellgröße nicht gesättigt ist — verhindert träges Abklingen nach I/O-Spitzen (Back-Calculation Anti-Windup Pattern). # P28-Konformität `dt` MUSS aus einem injizierten Clock-Port abgeleitet werden, nicht aus `SystemTime::now()` direkt.

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `fn update_with_anti_windup( &mut self, dt: Duration, measured_latency_ms: f32, is_actuator_saturated: bool, ) -> usize` | `crates/contextra-adapt/src/pid.rs:49` | Aktualisiert den Regler mit konditionellem Integrator-Update. `dt`: Zeitdelta seit letztem Update (aus injiziertem Clock-Port). `measured_latency_ms`: gemessene |

#### `const PID_MAX_POOL_SIZE_DEFAULT`
- Ort: `crates/contextra-adapt/src/pid.rs:29`
```rust
pub const PID_MAX_POOL_SIZE_DEFAULT: usize = 200
```
Dokumentation: Maximale Kandidaten-Pool-Größe.

#### `const PID_MIN_POOL_SIZE_DEFAULT`
- Ort: `crates/contextra-adapt/src/pid.rs:26`
```rust
pub const PID_MIN_POOL_SIZE_DEFAULT: usize = 50
```
Dokumentation: Minimale Kandidaten-Pool-Größe (Hard Floor = 50, Quality Knee per arXiv:2604.01733).

### Modul `pid_latency_controller`

#### `struct LatencyBudgetGuard`
- Ort: `crates/contextra-adapt/src/pid_latency_controller.rs:161`
```rust
pub struct LatencyBudgetGuard<C: ?Sized + Clock = SystemClock>
```
Dokumentation: Simple, panic-free latency budget guard for iterative multi-step search loops.
- Impl-Block `crates/contextra-adapt/src/pid_latency_controller.rs:167`
- Impl-Block `crates/contextra-adapt/src/pid_latency_controller.rs:174`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new(budget_ms: f64) -> Self` | `crates/contextra-adapt/src/pid_latency_controller.rs:169` | Creates a new `LatencyBudgetGuard` with the given budget in milliseconds using `SystemClock`. |
| `pub fn new_with_clock(budget_ms: f64, clock: Arc<C>) -> Self` | `crates/contextra-adapt/src/pid_latency_controller.rs:176` | Creates a new `LatencyBudgetGuard` with an injected [`Clock`] port for deterministic time tracking. |
| `pub fn elapsed_ms(&self) -> f64` | `crates/contextra-adapt/src/pid_latency_controller.rs:192` | Returns elapsed time in milliseconds since guard instantiation. |
| `pub fn is_exceeded(&self) -> bool` | `crates/contextra-adapt/src/pid_latency_controller.rs:201` | Returns `true` if the elapsed time has exceeded the configured budget. |
| `pub fn budget_ms(&self) -> f64` | `crates/contextra-adapt/src/pid_latency_controller.rs:206` | Returns configured budget in milliseconds. |

#### `struct PidLatencyController`
- Ort: `crates/contextra-adapt/src/pid_latency_controller.rs:38`
```rust
pub struct PidLatencyController
```
Dokumentation: Classical PID controller for dynamically scaling multi-step retrieval parameters (`k_pool`, `max_hops`). HINWEIS ZUR ANTI-WINDUP STRATEGIE: Dieser Controller nutzt einfaches Integrations-Clamping auf `[-MAX_INTEGRAL, MAX_INTEGRAL]`. Für Anwendungsfälle mit expliziter Aktuatorsättigung (z.B. Pool-Größe an Min-/Max-Grenzen) steht in `pid.rs` der Trait [`AntiWindupController`](crate::AntiWindupController) mit `update_with_anti_windup` bereit, welcher die Integrationsakkumulation bei Sättigung vollständig aussetzt.
- Impl-Block (Trait `Default`) `crates/contextra-adapt/src/pid_latency_controller.rs:53`
- Impl-Block `crates/contextra-adapt/src/pid_latency_controller.rs:59`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `kp` | `f64` | `crates/contextra-adapt/src/pid_latency_controller.rs:40` |
| `ki` | `f64` | `crates/contextra-adapt/src/pid_latency_controller.rs:42` |
| `kd` | `f64` | `crates/contextra-adapt/src/pid_latency_controller.rs:44` |
| `target_latency_ms` | `f64` | `crates/contextra-adapt/src/pid_latency_controller.rs:46` |

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new(target_latency_ms: f64) -> Self` | `crates/contextra-adapt/src/pid_latency_controller.rs:61` | Creates a new `PidLatencyController` with specified target latency in milliseconds. |
| `pub fn with_params(kp: f64, ki: f64, kd: f64, target_latency_ms: f64) -> Self` | `crates/contextra-adapt/src/pid_latency_controller.rs:79` | Custom constructor specifying gains and target latency. |
| `pub fn compute_adjustment(&mut self, observed_latency_ms: f64) -> f64` | `crates/contextra-adapt/src/pid_latency_controller.rs:105` | Calculates scaling adjustment factor in range `[0.3, 1.0]` based on observed latency. - Observed latency > target_latency => scaling factor decreases towards 0. |
| `pub fn integral(&self) -> f64` | `crates/contextra-adapt/src/pid_latency_controller.rs:143` | Liefert den akkumulierten Integratorwert. |
| `pub fn previous_error(&self) -> Option<f64>` | `crates/contextra-adapt/src/pid_latency_controller.rs:148` | Liefert den vorherigen Regelfehler. |
| `pub fn reset(&mut self)` | `crates/contextra-adapt/src/pid_latency_controller.rs:153` | Resets the controller integral state and error history. |

#### `const DEFAULT_TARGET_LATENCY_MS`
- Ort: `crates/contextra-adapt/src/pid_latency_controller.rs:18`
```rust
pub const DEFAULT_TARGET_LATENCY_MS: f64 = 100.0
```
Dokumentation: Konservativer Default für das P95-Latenzziel in Millisekunden (100ms). HINWEIS: implementation_plan-db2.md identifiziert den genauen P95-Zielwert als offene Frage (50ms vs. 100ms). Wir wählen 100ms als konservativen Default und machen den Zielwert über `PidLatencyController::new(target_latency_ms)` vollständig konfigurierbar.

#### `const MAX_INTEGRAL`
- Ort: `crates/contextra-adapt/src/pid_latency_controller.rs:28`
```rust
pub const MAX_INTEGRAL: f64 = 10.0
```
Dokumentation: Max-Grenze für den I-Anteil (Anti-Windup Clamping).

#### `const MAX_SCALING_FACTOR`
- Ort: `crates/contextra-adapt/src/pid_latency_controller.rs:25`
```rust
pub const MAX_SCALING_FACTOR: f64 = 1.0
```
Dokumentation: Oberer Clamp für den K-Pool-Skalierungsfaktor (100% des konfigurierten K-Pools).

#### `const MIN_SCALING_FACTOR`
- Ort: `crates/contextra-adapt/src/pid_latency_controller.rs:22`
```rust
pub const MIN_SCALING_FACTOR: f64 = 0.3
```
Dokumentation: Unterer Clamp für den K-Pool-Skalierungsfaktor (30% des konfigurierten K-Pools), um APM-RECALL-COLLAPSE in Standardlastszenarien zu verhindern.

### Modul `rie_greedy`

#### `struct RieGreedyProfile`
- Ort: `crates/contextra-adapt/src/rie_greedy.rs:34`
```rust
pub struct RieGreedyProfile
```
Dokumentation: Laufzeitzustand eines RIE-Greedy-Profils (§10.4.1). Implementiert die deterministisch-gierige Personalisierung mit zeitlich diskontierter Präzisionsmatrix $\Lambda_t$ und Informationsvektor $\eta_t$.
- Impl-Block `crates/contextra-adapt/src/rie_greedy.rs:47`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `precision_inv` | `Vec<f32>` | `crates/contextra-adapt/src/rie_greedy.rs:36` |
| `info_vector` | `Vec<f32>` | `crates/contextra-adapt/src/rie_greedy.rs:38` |
| `dim` | `usize` | `crates/contextra-adapt/src/rie_greedy.rs:40` |
| `lambda` | `f32` | `crates/contextra-adapt/src/rie_greedy.rs:42` |
| `gamma` | `f32` | `crates/contextra-adapt/src/rie_greedy.rs:44` |

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new(dim: usize, lambda: f32, gamma: f32) -> Self` | `crates/contextra-adapt/src/rie_greedy.rs:49` | Erstellt ein neues RIE-Greedy-Profil mit initialer Präzisionsmatrix $\Lambda_0 = \lambda I_d$. |
| `pub fn mu_hat(&self) -> Result<Vec<f32>, RieGreedyError>` | `crates/contextra-adapt/src/rie_greedy.rs:78` | Berechnet die Parameter-Schätzung $\hat{\mu}_t = \Lambda_t^{-1} \eta_t$. |
| `pub fn predict(&self, context: &[f32]) -> Result<f32, RieGreedyError>` | `crates/contextra-adapt/src/rie_greedy.rs:96` | Berechnet die deterministisch-gierige Erwartungswert-Vorhersage $\hat{\mu}_t \cdot x$. |
| `pub fn update(&mut self, context: &[f32], reward: f32) -> Result<(), RieGreedyError>` | `crates/contextra-adapt/src/rie_greedy.rs:124` | Führt die Diskontierung VOR dem Sherman-Morrison Rang-1 Update durch (§10.4.1, Spec §9.2). 1. Diskontierung: $\Lambda^{-1} \leftarrow \gamma^{-1} \Lambda^{-1}$, |
| `pub fn trace_precision_inv(&self) -> f32` | `crates/contextra-adapt/src/rie_greedy.rs:197` | Berechnet die Spur der inversen Präzisionsmatrix $\text{Tr}(\Lambda^{-1})$. |

#### `enum RieGreedyError`
- Ort: `crates/contextra-adapt/src/rie_greedy.rs:12`
```rust
pub enum RieGreedyError
```
Dokumentation: Fehlerzustände für den RIE-Greedy-Personalismus-Mechanismus (§10.4.1).

Varianten:

- `DimensionMismatch{` — `crates/contextra-adapt/src/rie_greedy.rs:15`
- `NonFinite` — `crates/contextra-adapt/src/rie_greedy.rs:23`
- `InvalidConfig(String)` — `crates/contextra-adapt/src/rie_greedy.rs:26`

### Modul `rng`

#### `struct SeededRng`
- Ort: `crates/contextra-adapt/src/rng.rs:25`
```rust
pub struct SeededRng
```
Dokumentation: Deterministischer SplitMix64 PRNG.
- Impl-Block `crates/contextra-adapt/src/rng.rs:29`
- Impl-Block (Trait `Rng`) `crates/contextra-adapt/src/rng.rs:36`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new(seed: u64) -> Self` | `crates/contextra-adapt/src/rng.rs:31` | Erstellt einen neuen `SeededRng` mit dem Startzustand `seed`. |

#### `trait Rng`
- Ort: `crates/contextra-adapt/src/rng.rs:6`
```rust
pub trait Rng: Send + Sync
```
Dokumentation: Trait für Pseudozufallsgeneratoren in Bandit- und Sampling-Algorithmen.

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `fn next_u64(&mut self) -> u64` | `crates/contextra-adapt/src/rng.rs:8` | Generiert die nächste vorzeichenlose 64-Bit-Ganzzahl. |
| `fn next_f32(&mut self) -> f32` | `crates/contextra-adapt/src/rng.rs:11` | Generiert eine 32-Bit-Fließkommazahl im halboffenen Intervall `[0, 1)`. |
| `fn next_f64(&mut self) -> f64` | `crates/contextra-adapt/src/rng.rs:17` | Generiert eine 64-Bit-Fließkommazahl im halboffenen Intervall `[0.0, 1.0)`. |

### Modul `shadow_mode`

#### `struct ShadowDiscrepancy`
- Ort: `crates/contextra-adapt/src/shadow_mode.rs:11`
```rust
pub struct ShadowDiscrepancy<T>
```
Dokumentation: Diskrepanz-Eintrag: Baseline-Ergebnis vs. Kandidaten-Ergebnis, ohne Einfluss auf das tatsächlich zurückgegebene Nutzerergebnis (§12 Integrationsregel).

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `baseline` | `T` | `crates/contextra-adapt/src/shadow_mode.rs:13` |
| `candidate` | `T` | `crates/contextra-adapt/src/shadow_mode.rs:15` |
| `context_id` | `u64` | `crates/contextra-adapt/src/shadow_mode.rs:17` |

#### `struct TracingShadowSink`
- Ort: `crates/contextra-adapt/src/shadow_mode.rs:29`
```rust
pub struct TracingShadowSink
```
Dokumentation: Referenzimplementierung über `tracing::debug!` — kein zusätzlicher I/O-Pfad, keine Rng/Clock-Portverletzung (P28), da rein diagnostisch.
- Impl-Block (Trait `ShadowSink<T>`) `crates/contextra-adapt/src/shadow_mode.rs:31`

#### `trait ShadowSink`
- Ort: `crates/contextra-adapt/src/shadow_mode.rs:21`
```rust
pub trait ShadowSink<T>: Send + Sync
```
Dokumentation: Dyn-kompatibler Port (P27) für Diskrepanz-Senken (In-Memory für Tests, Tracing/Log für Produktion).

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `fn record(&self, discrepancy: ShadowDiscrepancy<T>)` | `crates/contextra-adapt/src/shadow_mode.rs:23` | Zeichnet einen Diskrepanz-Eintrag auf. |

### Modul `strategy`

#### `enum RetrievalStrategy`
- Ort: `crates/contextra-adapt/src/strategy.rs:9`
```rust
pub enum RetrievalStrategy
```
Dokumentation: Retrieval-Strategie für Multi-Armed- und Contextual-Bandits.

Varianten:

- `Vector` — `crates/contextra-adapt/src/strategy.rs:11`
- `Text` — `crates/contextra-adapt/src/strategy.rs:13`
- `Graph` — `crates/contextra-adapt/src/strategy.rs:15`
- `Hybrid` — `crates/contextra-adapt/src/strategy.rs:17`
- `Global` — `crates/contextra-adapt/src/strategy.rs:19`

## 9. Impl-Blöcke ohne Typ im selben Crate

- `impl<T: Clock + ?Sized> Clock for Arc<T>` — `crates/contextra-adapt/src/clock.rs:61` (0 pub Methoden)
- `impl From<AdaptError> for contextra_types::ContextraError` — `crates/contextra-adapt/src/compat/error.rs:5` (0 pub Methoden)
