# Schnittstellenspezifikation: `contextra-rank`

> Erzeugt durch `tools/extract_interfaces.py` aus dem Quellcode. Keine Handpflege. Jede Angabe trägt Datei:Zeile. Maßgeblich bleibt der Quellcode.

## 1. Kopf

| Feld | Wert |
|---|---|
| Name | `contextra-rank` |
| Version | workspace |
| Pfad | `crates/contextra-rank` |
| Rolle | Ranking und Fusion |
| Bibliothekspfad | src/lib.rs (Standard) |

**Crate-Dokumentation (//!):**

> # Contextra Rank (`contextra-rank`)
>
> **Ring:** Ring 0 (Ranking & Kalibrierung / Kernlogik)
> **Klasse:** Rand `[R]`
> **Reifegrad:** Experimentell / Aktiv (Kern v1)
>
> ## Zweck des Crates
> Dieses Crate implementiert Methoden zur Fusion, Kalibrierung und Erklärung
> von Retrieval-Ergebnissen im Contextra-System. Dazu gehören:
> - **4-Signal-Fusion:** Verknüpfung von Vektor-, Text- (BM25), Graph- und
> Rerank-Scores mittels reziproker Rangfusion (RRF = Reciprocal Rank Fusion)
> und score-basierter Normalisierung.

## 2. Abhängigkeiten

| Name | Art | Quelle | optional | verwendet von (normal/dev) |
|---|---|---|---|---|
| `ahash` | normal | registry | nein |  |
| `contextra-ports` | dev | path | nein | contextra, contextra-adapt, contextra-agent, contextra-checkpoint, contextra-cognition, contextra-crypto, contextra-crypto-fuzz, contextra-db, contextra-engine, contextra-graph, contextra-graph-fuzz, contextra-infer-candle, contextra-infer-ollama, contextra-infer-onnx, contextra-kvcache, contextra-mcp, contextra-mvcc, contextra-rank, contextra-router, contextra-store, contextra-store-fuzz, contextra-testkit, contextra-text, contextra-vector, contextra-vector-fuzz |
| `contextra-types` | dev | path | nein | contextra, contextra-adapt, contextra-agent, contextra-bench, contextra-checkpoint, contextra-cognition, contextra-crypto, contextra-crypto-fuzz, contextra-db, contextra-engine, contextra-graph, contextra-graph-fuzz, contextra-infer-candle, contextra-infer-ollama, contextra-infer-onnx, contextra-ingest, contextra-kvcache, contextra-mcp, contextra-mvcc, contextra-ports, contextra-py, contextra-rank, contextra-router, contextra-simd, contextra-store, contextra-store-fuzz, contextra-testkit, contextra-text, contextra-vault, contextra-vector, contextra-vector-fuzz |
| `criterion` | dev | workspace | nein |  |
| `proptest` | dev | registry | nein |  |
| `serde` | normal | registry | nein |  |
| `serde_json` | normal | workspace | nein |  |
| `thiserror` | normal | workspace | nein |  |
| `tokio` | dev | registry | nein |  |
| `tracing` | normal | workspace | nein |  |

**Verwendet von (Crate-Ebene):** `contextra-bench`, `contextra-cognition`, `contextra-db`, `contextra-engine`, `contextra-infer-candle`, `contextra-mcp`

## 3. Features

| Feature | Aktiviert |
|---|---|
| `default` | `std`, `exp-dibud`, `exp-resonance`, `experimental-rank` |
| `dibud` | `exp-dibud` |
| `exp-dibud` | – |
| `exp-resonance` | – |
| `experimental-all` | `exp-dibud`, `exp-resonance` |
| `experimental-rank` | `experimental-all` |
| `ports-compat` | `workspace` |
| `std` | – |
| `types-compat` | `workspace` |
| `workspace` | `dep:contextra-types`, `dep:contextra-ports` |

## 4. Metriken (Quellcode)

| Kennzahl | Wert |
|---|---|
| Quelldateien | 31 |
| Quellzeilen | 5852 |
| Öffentliche Items (Modul-Ebene) | 105 |
| davon `fn` | 33 |
| davon `struct` | 29 |
| davon `enum` | 11 |
| davon `trait` | 5 |
| davon `type` | 3 |
| davon `const` | 1 |
| davon `mod` | 23 |
| Öffentliche Makros (`#[macro_export]`) | 0 |
| Öffentliche Re-Exports (`pub use`) | 36 |
| `unsafe`-Vorkommen (Produktionscode) | 0 |
| `.unwrap()` (Produktionscode) | 0 |
| `.expect(` (Produktionscode) | 0 |
| `panic!` (Produktionscode) | 0 |
| `todo!/unimplemented!/unreachable!` | 0 |
| Indexierungen `x[…]` (grob, Produktionscode) | 32 |
| `#[deprecated]` | 0 |
| `#[cfg(feature…)]`-Gates | 9 |
| Parser-Abdeckung (geparste `pub`-Deklarationen / Grep-Treffer) | 200 / 200 = 100 % |

## 5. Modulbaum und Dateien

| Modul | Datei | Zeilen |
|---|---|---|
| `calibration::conformal` | `crates/contextra-rank/src/calibration/conformal.rs` | 228 |
| `calibration::isotonic` | `crates/contextra-rank/src/calibration/isotonic.rs` | 424 |
| `calibration` | `crates/contextra-rank/src/calibration/mod.rs` | 125 |
| `calibration::platt` | `crates/contextra-rank/src/calibration/platt.rs` | 166 |
| `compat::clock` | `crates/contextra-rank/src/compat/clock.rs` | 30 |
| `compat::error` | `crates/contextra-rank/src/compat/error.rs` | 45 |
| `compat` | `crates/contextra-rank/src/compat/mod.rs` | 18 |
| `compat::types` | `crates/contextra-rank/src/compat/types.rs` | 234 |
| `contract` | `crates/contextra-rank/src/contract.rs` | 15 |
| `dibud::driver` | `crates/contextra-rank/src/dibud/driver.rs` | 70 |
| `dibud` | `crates/contextra-rank/src/dibud/mod.rs` | 14 |
| `dibud::state` | `crates/contextra-rank/src/dibud/state.rs` | 307 |
| `dibud::types` | `crates/contextra-rank/src/dibud/types.rs` | 127 |
| `drift` | `crates/contextra-rank/src/drift.rs` | 137 |
| `error` | `crates/contextra-rank/src/error.rs` | 23 |
| `explain` | `crates/contextra-rank/src/explain.rs` | 465 |
| `fusion::ext` | `crates/contextra-rank/src/fusion/ext.rs` | 37 |
| `fusion::global` | `crates/contextra-rank/src/fusion/global.rs` | 233 |
| `fusion` | `crates/contextra-rank/src/fusion/mod.rs` | 44 |
| `fusion::mrrf` | `crates/contextra-rank/src/fusion/mrrf.rs` | 224 |
| `fusion::normalized` | `crates/contextra-rank/src/fusion/normalized.rs` | 196 |
| `fusion::provenance` | `crates/contextra-rank/src/fusion/provenance.rs` | 306 |
| `fusion::resonance` | `crates/contextra-rank/src/fusion/resonance.rs` | 96 |
| `fusion::rrf` | `crates/contextra-rank/src/fusion/rrf.rs` | 753 |
| `fusion::signal` | `crates/contextra-rank/src/fusion/signal.rs` | 200 |
| `fusion::topk` | `crates/contextra-rank/src/fusion/topk.rs` | 99 |
| `fusion::types` | `crates/contextra-rank/src/fusion/types.rs` | 216 |
| `crate` | `crates/contextra-rank/src/lib.rs` | 103 |
| `local_types` | `crates/contextra-rank/src/local_types.rs` | 211 |
| `metrics` | `crates/contextra-rank/src/metrics.rs` | 321 |
| `temporal` | `crates/contextra-rank/src/temporal.rs` | 385 |

## 6. Re-Exports (`pub use`)

- `pub use conformal::` — `crates/contextra-rank/src/calibration/mod.rs:11` (Modul `calibration`)
- `pub use isotonic::` — `crates/contextra-rank/src/calibration/mod.rs:12` (Modul `calibration`)
- `pub use platt::PlattScaler` — `crates/contextra-rank/src/calibration/mod.rs:13` (Modul `calibration`)
- `pub use clock::` — `crates/contextra-rank/src/compat/mod.rs:13` (Modul `compat`)
- `pub use types::` — `crates/contextra-rank/src/compat/mod.rs:14` (Modul `compat`)
- `pub use driver::` — `crates/contextra-rank/src/dibud/mod.rs:12` (Modul `dibud`)
- `pub use state::` — `crates/contextra-rank/src/dibud/mod.rs:13` (Modul `dibud`)
- `pub use types::` — `crates/contextra-rank/src/dibud/mod.rs:14` (Modul `dibud`)
- `pub use crate::fusion::SignalKind` — `crates/contextra-rank/src/explain.rs:15` (Modul `explain`)
- `pub use crate::local_types::` — `crates/contextra-rank/src/fusion/mod.rs:20` (Modul `fusion`)
- `pub use ext::ResonanceModifier` — `crates/contextra-rank/src/fusion/mod.rs:21` (Modul `fusion`)
- `pub use ext::` — `crates/contextra-rank/src/fusion/mod.rs:23` (Modul `fusion`)
- `pub use global::` — `crates/contextra-rank/src/fusion/mod.rs:24` (Modul `fusion`)
- `pub use crate::compat::types::weights_to_signal_factors_types_compat` — `crates/contextra-rank/src/fusion/mod.rs:26` (Modul `fusion`)
- `pub use normalized::` — `crates/contextra-rank/src/fusion/mod.rs:27` (Modul `fusion`)
- `pub use provenance::ProvenanceBuilder` — `crates/contextra-rank/src/fusion/mod.rs:28` (Modul `fusion`)
- `pub use resonance::` — `crates/contextra-rank/src/fusion/mod.rs:30` (Modul `fusion`)
- `pub use rrf::` — `crates/contextra-rank/src/fusion/mod.rs:31` (Modul `fusion`)
- `pub use signal::` — `crates/contextra-rank/src/fusion/mod.rs:37` (Modul `fusion`)
- `pub use topk::BoundedTopK` — `crates/contextra-rank/src/fusion/mod.rs:38` (Modul `fusion`)
- `pub use types::` — `crates/contextra-rank/src/fusion/mod.rs:39` (Modul `fusion`)
- `pub use crate::compat::types::signal_contribution_synthesized_from_types` — `crates/contextra-rank/src/fusion/mod.rs:44` (Modul `fusion`)
- `pub use crate::local_types::FusionStrategy` — `crates/contextra-rank/src/fusion/rrf.rs:27` (Modul `fusion::rrf`)
- `pub use super::mrrf::` — `crates/contextra-rank/src/fusion/rrf.rs:193` (Modul `fusion::rrf`)
- `pub use crate::local_types::SignalFailurePolicy` — `crates/contextra-rank/src/fusion/types.rs:9` (Modul `fusion::types`)
- `pub use error::` — `crates/contextra-rank/src/lib.rs:65` (Modul `crate`)
- `pub use local_types::` — `crates/contextra-rank/src/lib.rs:66` (Modul `crate`)
- `pub use calibration::` — `crates/contextra-rank/src/lib.rs:74` (Modul `crate`)
- `pub use dibud::` — `crates/contextra-rank/src/lib.rs:79` (Modul `crate`)
- `pub use drift::` — `crates/contextra-rank/src/lib.rs:83` (Modul `crate`)
- `pub use explain::` — `crates/contextra-rank/src/lib.rs:84` (Modul `crate`)
- `pub use fusion::` — `crates/contextra-rank/src/lib.rs:85` (Modul `crate`)
- `pub use metrics::` — `crates/contextra-rank/src/lib.rs:94` (Modul `crate`)
- `pub use temporal::` — `crates/contextra-rank/src/lib.rs:96` (Modul `crate`)
- `pub use fusion::` — `crates/contextra-rank/src/lib.rs:102` (Modul `crate`)
- `pub use fusion::ResonanceModifier` — `crates/contextra-rank/src/lib.rs:103` (Modul `crate`)

## 8. Öffentliche Schnittstelle je Item

### Modul `calibration`

#### `mod conformal`
- Ort: `crates/contextra-rank/src/calibration/mod.rs:7`
```rust
pub mod conformal
```

#### `mod isotonic`
- Ort: `crates/contextra-rank/src/calibration/mod.rs:8`
```rust
pub mod isotonic
```

#### `mod platt`
- Ort: `crates/contextra-rank/src/calibration/mod.rs:9`
```rust
pub mod platt
```

#### `enum Calibrator`
- Ort: `crates/contextra-rank/src/calibration/mod.rs:20`
```rust
pub enum Calibrator
```
Dokumentation: Versionierter Container für Kalibrierungsmodelle (Klassenvertrag RANK-1).
- Impl-Block `crates/contextra-rank/src/calibration/mod.rs:47`

Varianten:

- `Platt{` — `crates/contextra-rank/src/calibration/mod.rs:23`
- `Isotonic{` — `crates/contextra-rank/src/calibration/mod.rs:31`
- `Conformal{` — `crates/contextra-rank/src/calibration/mod.rs:39`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn platt(scaler: PlattScaler) -> Self` | `crates/contextra-rank/src/calibration/mod.rs:49` | Erzeugt einen neuen `Calibrator` basierend auf einem `PlattScaler`. |
| `pub fn isotonic(calibrator: IsotonicCalibrator) -> Self` | `crates/contextra-rank/src/calibration/mod.rs:57` | Erzeugt einen neuen `Calibrator` basierend auf einem `IsotonicCalibrator`. |
| `pub fn conformal(calibrator: AdaptiveConformalCalibrator) -> Self` | `crates/contextra-rank/src/calibration/mod.rs:65` | Erzeugt einen neuen `Calibrator` basierend auf einem `AdaptiveConformalCalibrator`. |

#### `fn calibrate_score`
- Ort: `crates/contextra-rank/src/calibration/mod.rs:78`
```rust
pub fn calibrate_score(raw: f32, calibrator: &Calibrator) -> Result<f32>
```
Dokumentation: Kalibriert einen Roh-Score anhand eines übergebenen Kalibrierers (Klassenvertrag RANK-1). # Fehler Gibt `RankError::InvalidInput` zurück, wenn `raw` nicht endlich ist (NaN oder Inf) oder wenn die Format-Version des Kalibrierers inkompatibel ist.

### Modul `calibration::conformal`

#### `struct AdaptiveConformalCalibrator`
- Ort: `crates/contextra-rank/src/calibration/conformal.rs:74`
```rust
pub struct AdaptiveConformalCalibrator
```
Dokumentation: Adaptive Conformal Calibrator for quantile-based online threshold estimation under covariate shift. Implements online stochastic gradient updates on the weighted pinball loss for target coverage level $1 - \alpha$. When a non-conformity score exceeds the current threshold ($S_t > \hat{q}_t$), the threshold increases by $\gamma \cdot w_t \cdot (1 - \alpha)$. Otherwise, it decreases by $\gamma \cdot w_t \cdot \alpha$. Adaptiver konformer Kalibrierer für Online-Schwellenwerte unter Covariate Shift.
- Impl-Block `crates/contextra-rank/src/calibration/conformal.rs:84`
- Impl-Block (Trait `ConformalCalibrator`) `crates/contextra-rank/src/calibration/conformal.rs:189`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new(alpha: f32) -> Result<Self, ConformalError>` | `crates/contextra-rank/src/calibration/conformal.rs:100` | Creates a new `AdaptiveConformalCalibrator` with target error rate `alpha` in `(0.0, 1.0)`. Uses default initial threshold (0.5) and default learning rate (0.02 |
| `pub fn with_params( alpha: f32, initial_threshold: f32, learning_rate: f32, ) -> Result<Self, ConformalError>` | `crates/contextra-rank/src/calibration/conformal.rs:109` | Creates an `AdaptiveConformalCalibrator` with custom parameters. |
| `pub fn with_bounds( mut self, min_threshold: f32, max_threshold: f32, ) -> Result<Self, ConformalError>` | `crates/contextra-rank/src/calibration/conformal.rs:136` | Sets bounding constraints `[min_threshold, max_threshold]` for the estimated threshold. |
| `pub fn alpha(&self) -> f32` | `crates/contextra-rank/src/calibration/conformal.rs:157` | Returns the target error rate $\alpha$. |
| `pub fn learning_rate(&self) -> f32` | `crates/contextra-rank/src/calibration/conformal.rs:162` | Returns the learning rate $\gamma$. |
| `pub fn observation_count(&self) -> u64` | `crates/contextra-rank/src/calibration/conformal.rs:167` | Returns the total number of observations processed. |
| `pub fn invalidate_on_config_change(&mut self, new_fingerprint: impl Into<ConfigFingerprint>)` | `crates/contextra-rank/src/calibration/conformal.rs:172` | Resets state if ConfigFingerprint changes (P8 Compliance). |

Assoziierte Konstanten/Typen:

- `pub const DEFAULT_LEARNING_RATE: f32 = 0.02` — `crates/contextra-rank/src/calibration/conformal.rs:86`
- `pub const DEFAULT_INITIAL_THRESHOLD: f32 = 0.5` — `crates/contextra-rank/src/calibration/conformal.rs:89`
- `pub const DEFAULT_MIN_THRESHOLD: f32 = 0.0` — `crates/contextra-rank/src/calibration/conformal.rs:92`
- `pub const DEFAULT_MAX_THRESHOLD: f32 = 1.0` — `crates/contextra-rank/src/calibration/conformal.rs:95`

#### `enum ConformalError`
- Ort: `crates/contextra-rank/src/calibration/conformal.rs:14`
```rust
pub enum ConformalError
```
Dokumentation: Error type for conformal calibration and threshold updates. Fehlertypen für die konforme Kalibrierung.

Varianten:

- `Alpha(f32)` — `crates/contextra-rank/src/calibration/conformal.rs:17`
- `PropensityWeight(f32)` — `crates/contextra-rank/src/calibration/conformal.rs:21`
- `NonConformityScore(f32)` — `crates/contextra-rank/src/calibration/conformal.rs:25`
- `Threshold(f32)` — `crates/contextra-rank/src/calibration/conformal.rs:29`
- `LearningRate(f32)` — `crates/contextra-rank/src/calibration/conformal.rs:33`

#### `trait ConformalCalibrator`
- Ort: `crates/contextra-rank/src/calibration/conformal.rs:38`
```rust
pub trait ConformalCalibrator
```
Dokumentation: Trait for deterministic, online conformal calibration and threshold estimation. Trait für deterministische konforme Schwellenwertschätzung.

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `fn update( &mut self, non_conformity_score: f32, propensity_weight: f32, ) -> Result<bool, ConformalError>` | `crates/contextra-rank/src/calibration/conformal.rs:56` |  |
| `fn threshold(&self) -> f32` | `crates/contextra-rank/src/calibration/conformal.rs:63` | Returns the currently valid threshold $\hat{q}$. |

### Modul `calibration::isotonic`

#### `struct IsotonicCalibrator`
- Ort: `crates/contextra-rank/src/calibration/isotonic.rs:76`
```rust
pub struct IsotonicCalibrator
```
Dokumentation: Isotonic Calibrator using Pool-Adjacent Violators Algorithm. Nicht-parametrischer Kalibrierer auf Basis von PAVA.
- Impl-Block (Trait `PartialEq`) `crates/contextra-rank/src/calibration/isotonic.rs:94`
- Impl-Block (Trait `fmt::Debug`) `crates/contextra-rank/src/calibration/isotonic.rs:108`
- Impl-Block `crates/contextra-rank/src/calibration/isotonic.rs:128`
- Impl-Block (Trait `IsotonicPortsClockExt`) `crates/contextra-rank/src/compat/clock.rs:26`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new(warmup_required: u32, max_observations: usize) -> Self` | `crates/contextra-rank/src/calibration/isotonic.rs:130` | Creates a new `IsotonicCalibrator`. |
| `pub fn with_clock(mut self, clock: Arc<dyn Clock>) -> Self` | `crates/contextra-rank/src/calibration/isotonic.rs:146` | Sets a custom clock port for deterministic time access. |
| `pub fn with_defaults() -> Self` | `crates/contextra-rank/src/calibration/isotonic.rs:153` | Creates an `IsotonicCalibrator` with defaults (Warmup: 50, Max Obs: 2000). |
| `pub fn record_outcome(&mut self, raw_score: f32, outcome: bool)` | `crates/contextra-rank/src/calibration/isotonic.rs:159` | Records a new observation `(raw_score, outcome)`. Non-finite `raw_score` values are safely ignored. Nimmt eine neue Beobachtung `(raw_score, outcome)` auf. |
| `pub fn observation_count(&self) -> usize` | `crates/contextra-rank/src/calibration/isotonic.rs:176` | Returns current observation count. |
| `pub fn is_calibrated(&self) -> bool` | `crates/contextra-rank/src/calibration/isotonic.rs:181` | Checks if warmup observations threshold has been reached. |
| `pub fn last_calibration_at(&self) -> Option<u64>` | `crates/contextra-rank/src/calibration/isotonic.rs:186` | Returns timestamp (UNIX epoch in seconds) of last model rebuild. |
| `pub fn cached_ece(&self) -> Option<f32>` | `crates/contextra-rank/src/calibration/isotonic.rs:191` | Returns cached Expected Calibration Error (ECE). |
| `pub fn calibrated_probability(&mut self, raw_score: f32) -> Option<f32>` | `crates/contextra-rank/src/calibration/isotonic.rs:197` | Returns calibrated probability if warm, or `None` if below warmup requirement. Liefert die kalibrierte Wahrscheinlichkeit für einen Roh-Score. |
| `pub fn force_rebuild(&mut self)` | `crates/contextra-rank/src/calibration/isotonic.rs:211` | Forces immediate PAVA model rebuild. |
| `pub fn invalidate_on_config_change(&mut self, new_fingerprint: impl Into<ConfigFingerprint>)` | `crates/contextra-rank/src/calibration/isotonic.rs:218` | Resets observations if ConfigFingerprint changes (P8 Compliance). |
| `pub fn expected_calibration_error(&mut self) -> Option<f32>` | `crates/contextra-rank/src/calibration/isotonic.rs:334` | Calculates Expected Calibration Error over 10 bins. |

#### `struct SystemClock`
- Ort: `crates/contextra-rank/src/calibration/isotonic.rs:26`
```rust
pub struct SystemClock
```
Dokumentation: Standard SystemClock basierend auf std::time::SystemTime.
- Impl-Block `crates/contextra-rank/src/calibration/isotonic.rs:28`
- Impl-Block (Trait `Clock`) `crates/contextra-rank/src/calibration/isotonic.rs:35`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new() -> Self` | `crates/contextra-rank/src/calibration/isotonic.rs:30` | Erzeugt eine neue `SystemClock`. |

#### `trait Clock`
- Ort: `crates/contextra-rank/src/calibration/isotonic.rs:14`
```rust
pub trait Clock: Send + Sync
```
Dokumentation: Lokaler Clock-Port für deterministische Zeitstempel (B11-Konformität).

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `fn now_unix_nanos(&self) -> u64` | `crates/contextra-rank/src/calibration/isotonic.rs:16` | Gibt den aktuellen UNIX-Zeitstempel in Nanosekunden zurück. |
| `fn monotonic_nanos(&self) -> u64` | `crates/contextra-rank/src/calibration/isotonic.rs:19` | Gibt eine monotone Zeit in Nanosekunden zurück. |

### Modul `calibration::platt`

#### `struct PlattScaler`
- Ort: `crates/contextra-rank/src/calibration/platt.rs:13`
```rust
pub struct PlattScaler
```
Dokumentation: Platt Scaler for parametric logit/score calibration. Parametrischer Kalibrierer nach Platt (1999).
- Impl-Block (Trait `Default`) `crates/contextra-rank/src/calibration/platt.rs:19`
- Impl-Block `crates/contextra-rank/src/calibration/platt.rs:25`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new(a: f32, b: f32) -> Self` | `crates/contextra-rank/src/calibration/platt.rs:27` | Creates a new `PlattScaler` with specified parameters `a` and `b`. Falls back to identity if non-finite. |
| `pub fn identity() -> Self` | `crates/contextra-rank/src/calibration/platt.rs:39` | Uncalibrated default fallback ($A=1.0, B=0.0$). |
| `pub fn is_identity(&self) -> bool` | `crates/contextra-rank/src/calibration/platt.rs:48` | Checks if this scaler is identical to uncalibrated default ($A=1.0, B=0.0$). |
| `pub fn is_fitted(&self) -> bool` | `crates/contextra-rank/src/calibration/platt.rs:53` | Checks if model has been fitted. |
| `pub fn params(&self) -> (f32, f32)` | `crates/contextra-rank/src/calibration/platt.rs:58` | Returns current parameters `(a, b)`. |
| `pub fn predict(&self, raw_score: f32) -> f32` | `crates/contextra-rank/src/calibration/platt.rs:63` | Calculates calibrated probability for raw score. |
| `pub fn apply(&self, logit: f32) -> f32` | `crates/contextra-rank/src/calibration/platt.rs:68` | Alias for `transform(logit)`. |
| `pub fn transform(&self, logit: f32) -> f32` | `crates/contextra-rank/src/calibration/platt.rs:74` | Applies Platt Scaling `sigmoid(A * logit + B)`. Führt die logistische Transformation `sigmoid(A * logit + B)` aus. |
| `pub fn invalidate_on_config_change(&mut self, new_fingerprint: impl Into<ConfigFingerprint>)` | `crates/contextra-rank/src/calibration/platt.rs:86` | Resets scaler to identity if ConfigFingerprint changes (P8 Compliance). |
| `pub fn fit(observations: &[(f32, bool)]) -> Self` | `crates/contextra-rank/src/calibration/platt.rs:103` | Fits parameters `A` and `B` via Negative Log-Likelihood minimization with L2 regularization and target smoothing (Platt, 1999). Trainiert die Parameter A und B  |

### Modul `compat`

#### `mod clock`
- Ort: `crates/contextra-rank/src/compat/mod.rs:9`
```rust
pub mod clock
```

#### `mod error`
- Ort: `crates/contextra-rank/src/compat/mod.rs:10`
```rust
pub mod error
```

#### `mod types`
- Ort: `crates/contextra-rank/src/compat/mod.rs:11`
```rust
pub mod types
```

### Modul `compat::clock`

#### `struct PortsClockAdapter`
- Ort: `crates/contextra-rank/src/compat/clock.rs:8`
```rust
pub struct PortsClockAdapter(pub Arc<dyn contextra_ports::Clock>)
```
Dokumentation: Adapter für einen dynamischen `contextra_ports::Clock` Referenzzeiger auf das lokale `Clock`-Trait.
- Impl-Block (Trait `Clock`) `crates/contextra-rank/src/compat/clock.rs:10`

#### `trait IsotonicPortsClockExt`
- Ort: `crates/contextra-rank/src/compat/clock.rs:21`
```rust
pub trait IsotonicPortsClockExt
```
Dokumentation: Extension-Trait für `IsotonicCalibrator` zur Ports-Clock-Konfiguration.

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `fn with_ports_clock(self, clock: Arc<dyn contextra_ports::Clock>) -> Self` | `crates/contextra-rank/src/compat/clock.rs:23` | Konfiguriert den Kalibrator mit einer externen `contextra_ports::Clock`. |

### Modul `compat::types`

#### `fn apply_temporal_validity_filter_with_ports_clock`
- Ort: `crates/contextra-rank/src/compat/types.rs:161`
```rust
pub fn apply_temporal_validity_filter_with_ports_clock( results: Vec<crate::fusion::SearchResult>, current_tx: TxId, clock: &dyn contextra_ports::Clock, ) -> Vec<crate::fusion::SearchResult>
```
Dokumentation: Bi-temporal filter helper mit Ports-Clock.

#### `fn signal_contribution_synthesized_from_types`
- Ort: `crates/contextra-rank/src/compat/types.rs:145`
```rust
pub fn signal_contribution_synthesized_from_types(source_doc_ids: &[contextra_types::DocId]) -> crate::fusion::ProvenanceRecord
```
Dokumentation: Helper für Synthese aus `contextra_types::DocId`.

#### `fn weights_to_signal_factors_types_compat`
- Ort: `crates/contextra-rank/src/compat/types.rs:151`
```rust
pub fn weights_to_signal_factors_types_compat( weights: Option<&contextra_types::FusionWeights>, ) -> (f32, f32, f32)
```
Dokumentation: Konvertiert optionale `contextra_types::FusionWeights`.

### Modul `contract`

#### `trait Clock`
- Ort: `crates/contextra-rank/src/contract.rs:7`
```rust
pub trait Clock: Send + Sync
```
Dokumentation: Minimaler Clock-Port für deterministische Zeitmessung im Kern.

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `fn now_unix_nanos(&self) -> u64` | `crates/contextra-rank/src/contract.rs:9` | Gibt den aktuellen UNIX-Zeitstempel in Nanosekunden zurück. |
| `fn monotonic_nanos(&self) -> u64` | `crates/contextra-rank/src/contract.rs:12` | Gibt eine monotone Zeit in Nanosekunden zurück. |

### Modul `crate`

#### `mod calibration`
- Ort: `crates/contextra-rank/src/lib.rs:55`
```rust
pub mod calibration
```

#### `mod compat`
- Ort: `crates/contextra-rank/src/lib.rs:54`
```rust
pub mod compat
```
- Gate: `#[cfg(feature = "workspace")]`

#### `mod contract`
- Ort: `crates/contextra-rank/src/lib.rs:56`
```rust
pub mod contract
```

#### `mod dibud`
- Ort: `crates/contextra-rank/src/lib.rs:72`
```rust
pub mod dibud
```
- Gate: `#[cfg(feature = "dibud")]`

#### `mod drift`
- Ort: `crates/contextra-rank/src/lib.rs:57`
```rust
pub mod drift
```

#### `mod error`
- Ort: `crates/contextra-rank/src/lib.rs:58`
```rust
pub mod error
```

#### `mod explain`
- Ort: `crates/contextra-rank/src/lib.rs:59`
```rust
pub mod explain
```

#### `mod fusion`
- Ort: `crates/contextra-rank/src/lib.rs:60`
```rust
pub mod fusion
```

#### `mod local_types`
- Ort: `crates/contextra-rank/src/lib.rs:61`
```rust
pub mod local_types
```

#### `mod metrics`
- Ort: `crates/contextra-rank/src/lib.rs:62`
```rust
pub mod metrics
```

#### `mod temporal`
- Ort: `crates/contextra-rank/src/lib.rs:63`
```rust
pub mod temporal
```

### Modul `dibud`

#### `mod driver`
- Ort: `crates/contextra-rank/src/dibud/mod.rs:8`
```rust
pub mod driver
```

#### `mod state`
- Ort: `crates/contextra-rank/src/dibud/mod.rs:9`
```rust
pub mod state
```

#### `mod types`
- Ort: `crates/contextra-rank/src/dibud/mod.rs:10`
```rust
pub mod types
```

### Modul `dibud::driver`

#### `fn fuse_exact_prefix`
- Ort: `crates/contextra-rank/src/dibud/driver.rs:17`
```rust
pub fn fuse_exact_prefix<I1, I2, I3>( mut state: DiBudFusionState, vector: &mut I1, text: &mut I2, graph: &mut I3, edge_reinforcement: impl Fn(DocId) -> f32, budget: &FusionBudget, ) -> Result<DiBudOutcome, RankError> where I1: Iterator<Item = DocId>, I2: Iterator<Item = DocId>, I3: Iterator<Item = DocId>,
```
Dokumentation: Synchronously executes DiBud fusion using three channel iterators. Führt die DiBud-Streaming-Fusion synchron über Iterator-Kanäle aus.

#### `fn fuse_exact_prefix_async`
- Ort: `crates/contextra-rank/src/dibud/driver.rs:50`
```rust
pub async fn fuse_exact_prefix_async<P, Fut>( mut state: DiBudFusionState, mut poll: P, edge_reinforcement: impl Fn(DocId) -> f32, budget: &FusionBudget, ) -> Result<DiBudOutcome, RankError> where P: FnMut(BudgetedChannel) -> Fut, Fut: Future<Output = Result<Option<DocId>, RankError>>,
```
Dokumentation: Asynchronously executes DiBud fusion using a channel poll closure returning a std `Future`. This function relies purely on `core::future::Future` and does not require an async runtime. Führt die DiBud-Streaming-Fusion asynchron aus.

### Modul `dibud::state`

#### `struct DiBudFusionState`
- Ort: `crates/contextra-rank/src/dibud/state.rs:39`
```rust
pub struct DiBudFusionState
```
Dokumentation: Core state machine tracking channel depths, observed document scores, and prefix certification. Zustandsautomat für die budgetierte Streaming-Rangfusion.
- Impl-Block (Trait `Default`) `crates/contextra-rank/src/dibud/state.rs:48`
- Impl-Block `crates/contextra-rank/src/dibud/state.rs:54`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new(capacity: usize) -> Self` | `crates/contextra-rank/src/dibud/state.rs:56` | Creates a new `DiBudFusionState` with pre-allocated map capacities. |
| `pub fn channel_depths(&self) -> [usize` | `crates/contextra-rank/src/dibud/state.rs:68` | Returns current channel depths `[Vector, Text, Graph]`. |
| `pub fn exhausted(&self) -> [bool` | `crates/contextra-rank/src/dibud/state.rs:73` | Returns exhausted status for each channel `[Vector, Text, Graph]`. |
| `pub fn accesses(&self) -> usize` | `crates/contextra-rank/src/dibud/state.rs:78` | Returns total read accesses performed. |
| `pub fn provenance_of(&self, doc_id: &DocId) -> Option<&ProvenanceRecord>` | `crates/contextra-rank/src/dibud/state.rs:83` | Returns reference to provenance record for a given document if present. |
| `pub fn feed( &mut self, ch: BudgetedChannel, item: Option<DocId>, er: impl Fn(DocId) -> f32, budget: &FusionBudget, ) -> Result<(), RankError>` | `crates/contextra-rank/src/dibud/state.rs:90` | Feeds the result of reading a channel into the state machine. Raw scores are unknown in pure `DocId` streams and are set to 0.0 in `ProvenanceRecord`. |
| `pub fn next_request(&self, budget: &FusionBudget) -> DiBudStep` | `crates/contextra-rank/src/dibud/state.rs:128` | Determines whether fusion is done or returns the next channel to poll. Bestimmt die nächste Abfrageaktion oder schließt die Fusion ab. |

#### `enum DiBudStep`
- Ort: `crates/contextra-rank/src/dibud/state.rs:22`
```rust
pub enum DiBudStep
```
Dokumentation: Represents the step action requested by `DiBudFusionState::next_request`. Nächste durchzuführende Aktion im DiBud-Zustandsautomaten.

Varianten:

- `Poll(BudgetedChannel)` — `crates/contextra-rank/src/dibud/state.rs:24`
- `Done(DiBudOutcome)` — `crates/contextra-rank/src/dibud/state.rs:26`

### Modul `dibud::types`

#### `struct DiBudOutcome`
- Ort: `crates/contextra-rank/src/dibud/types.rs:118`
```rust
pub struct DiBudOutcome
```
Dokumentation: Final outcome of a DiBud fusion execution. Endergebnis der DiBud-Fusion inklusive zertifiziertem Präfix.

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `ranked` | `Vec<DocId>` | `crates/contextra-rank/src/dibud/types.rs:120` |
| `certified_len` | `usize` | `crates/contextra-rank/src/dibud/types.rs:122` |
| `accesses` | `usize` | `crates/contextra-rank/src/dibud/types.rs:124` |
| `budget_exhausted` | `bool` | `crates/contextra-rank/src/dibud/types.rs:126` |

#### `struct FusionBudget`
- Ort: `crates/contextra-rank/src/dibud/types.rs:49`
```rust
pub struct FusionBudget
```
Dokumentation: Dynamic budget configuration for DiBud fusion. Ressourcen- und Abbruchbudget für die Streaming-Rangfusion.
- Impl-Block (Trait `Default`) `crates/contextra-rank/src/dibud/types.rs:62`
- Impl-Block `crates/contextra-rank/src/dibud/types.rs:74`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `max_total_accesses` | `usize` | `crates/contextra-rank/src/dibud/types.rs:51` |
| `min_certified_results` | `usize` | `crates/contextra-rank/src/dibud/types.rs:53` |
| `edge_reinforcement_weight` | `f32` | `crates/contextra-rank/src/dibud/types.rs:55` |
| `channel_weights` | `[f32; 3]` | `crates/contextra-rank/src/dibud/types.rs:57` |
| `edge_reinforcement_upper_bound` | `f32` | `crates/contextra-rank/src/dibud/types.rs:59` |

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn validate(&self) -> Result<(), RankError>` | `crates/contextra-rank/src/dibud/types.rs:79` | Validates budget parameter constraints. Ensures all weights are finite and non-negative, and minimum accesses/certified lengths are at least 1. |

#### `enum BudgetedChannel`
- Ort: `crates/contextra-rank/src/dibud/types.rs:18`
```rust
pub enum BudgetedChannel
```
Dokumentation: Identifies the channels managed by DiBud fusion budget. Order of variants is defined such that `Vector < Text < Graph`, matching the tie-breaking priority requirement `Graph > Text > Vector`. Identifiziert die durch DiBud budgetierten Retrieval-Kanäle.
- Impl-Block `crates/contextra-rank/src/dibud/types.rs:27`

Varianten:

- `Vector= 0` — `crates/contextra-rank/src/dibud/types.rs:20`
- `Text= 1` — `crates/contextra-rank/src/dibud/types.rs:22`
- `Graph= 2` — `crates/contextra-rank/src/dibud/types.rs:24`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub const fn index(self) -> usize` | `crates/contextra-rank/src/dibud/types.rs:30` | Converts channel variant to array index 0..3. |
| `pub const fn from_index(index: usize) -> Option<Self>` | `crates/contextra-rank/src/dibud/types.rs:36` | Converts array index 0..3 to channel variant. |

### Modul `drift`

#### `struct DriftDetector`
- Ort: `crates/contextra-rank/src/drift.rs:59`
```rust
pub struct DriftDetector
```
Dokumentation: Drift detector monitoring score shifts over sliding windows. Detektor zur Überwachung von Score-Verschiebungen über ein gleitendes Fenster.
- Impl-Block `crates/contextra-rank/src/drift.rs:66`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new(window_size: usize, threshold: f32) -> Self` | `crates/contextra-rank/src/drift.rs:68` | Creates a new `DriftDetector` with window size and sensitivity threshold. |
| `pub fn with_defaults() -> Self` | `crates/contextra-rank/src/drift.rs:83` | Creates a `DriftDetector` with default parameters (window size 50, threshold 0.15). |
| `pub fn set_baseline(&mut self, baseline: f32)` | `crates/contextra-rank/src/drift.rs:88` | Sets explicit baseline mean score. |
| `pub fn observe(&mut self, score: f32) -> DriftStatus` | `crates/contextra-rank/src/drift.rs:95` | Records a new score observation and calculates updated drift status. |
| `pub fn analyze(&self) -> DriftStatus` | `crates/contextra-rank/src/drift.rs:109` | Analyzes current observations against baseline. |
| `pub fn overall_drift_status(&self) -> &'static str` | `crates/contextra-rank/src/drift.rs:134` | Returns human-readable overall drift status string. |

#### `enum DriftStatus`
- Ort: `crates/contextra-rank/src/drift.rs:16`
```rust
pub enum DriftStatus
```
Dokumentation: Drift status summary representation. Zusammenfassung des aktuellen Drift-Status einer Signalverteilung.
- Impl-Block `crates/contextra-rank/src/drift.rs:33`

Varianten:

- `Stable{` — `crates/contextra-rank/src/drift.rs:18`
- `DriftDetected{` — `crates/contextra-rank/src/drift.rs:23`
- `InsufficientData` — `crates/contextra-rank/src/drift.rs:30`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn as_str(&self) -> &'static str` | `crates/contextra-rank/src/drift.rs:35` | Returns human-readable summary string ("stabil", "warnung", "kritisch", or "unbekannt"). |

### Modul `error`

#### `enum RankError`
- Ort: `crates/contextra-rank/src/error.rs:8`
```rust
pub enum RankError
```
Dokumentation: Fehlerzustände von `contextra-rank`.

Varianten:

- `InvalidInput(String)` — `crates/contextra-rank/src/error.rs:11`
- `Calibration(String)` — `crates/contextra-rank/src/error.rs:15`
- `FusionFailed(String)` — `crates/contextra-rank/src/error.rs:19`

#### `type Result`
- Ort: `crates/contextra-rank/src/error.rs:23`
```rust
pub type Result<T> = core::result::Result<T, RankError>
```
Dokumentation: Standard-Result-Typ für `contextra-rank`.

### Modul `explain`

#### `struct ExplanationEntry`
- Ort: `crates/contextra-rank/src/explain.rs:20`
```rust
pub struct ExplanationEntry
```
Dokumentation: Breakdown entry representing the contribution of a single search signal. Einzelner Beitrags-Eintrag für eine spezifische Signalquelle innerhalb der Erklärung.

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `signal` | `SignalKind` | `crates/contextra-rank/src/explain.rs:22` |
| `raw_score` | `f32` | `crates/contextra-rank/src/explain.rs:24` |
| `rank` | `Option<u32>` | `crates/contextra-rank/src/explain.rs:26` |
| `weight` | `Option<f32>` | `crates/contextra-rank/src/explain.rs:28` |
| `contribution_share` | `f32` | `crates/contextra-rank/src/explain.rs:30` |

#### `struct RetrievalExplanation`
- Ort: `crates/contextra-rank/src/explain.rs:36`
```rust
pub struct RetrievalExplanation
```
Dokumentation: Structured explanation detailing why a document was retrieved with its given rank. Strukturierte Gesamterklärung zur Nachvollziehbarkeit eines Retrieval-Ergebnisses.
- Impl-Block `crates/contextra-rank/src/explain.rs:60`
- Impl-Block (Trait `fmt::Display`) `crates/contextra-rank/src/explain.rs:111`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `entries` | `Vec<ExplanationEntry>` | `crates/contextra-rank/src/explain.rs:38` |
| `dominant_signal` | `Option<SignalKind>` | `crates/contextra-rank/src/explain.rs:40` |
| `coherence_bonus` | `f32` | `crates/contextra-rank/src/explain.rs:42` |
| `source_collection` | `Option<String>` | `crates/contextra-rank/src/explain.rs:44` |

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn to_human_readable_de(&self) -> String` | `crates/contextra-rank/src/explain.rs:66` | Generates a human-readable German explanation string for audit logging or CLI output. Formats all signal contributions using exact ranks and normalized percenta |

#### `fn explain`
- Ort: `crates/contextra-rank/src/explain.rs:207`
```rust
pub fn explain(record: &ProvenanceRecord) -> RetrievalExplanation
```
Dokumentation: Generates a structured `RetrievalExplanation` from a `ProvenanceRecord`. Computes per-signal `contribution_share` normalized such that all present signals sum to `1.0` (or `0.0` if no signals are present). Erzeugt aus einem maschinenlesbaren `ProvenanceRecord` eine strukturierte `RetrievalExplanation`.

### Modul `fusion`

#### `mod ext`
- Ort: `crates/contextra-rank/src/fusion/mod.rs:8`
```rust
pub mod ext
```

#### `mod mrrf`
- Ort: `crates/contextra-rank/src/fusion/mod.rs:10`
```rust
pub mod mrrf
```

#### `mod resonance`
- Ort: `crates/contextra-rank/src/fusion/mod.rs:14`
```rust
pub mod resonance
```
- Gate: `#[cfg(feature = "exp-resonance")]`

### Modul `fusion::ext`

#### `struct ResonanceConfig`
- Ort: `crates/contextra-rank/src/fusion/ext.rs:17`
```rust
pub struct ResonanceConfig
```
Dokumentation: Stub-Konfiguration für Resonanz-Bonus, wenn `exp-resonance` nicht aktiviert ist.
- Impl-Block (Trait `ResonanceModifier`) `crates/contextra-rank/src/fusion/ext.rs:23`
- Impl-Block (Trait `Default`) `crates/contextra-rank/src/fusion/resonance.rs:22`
- Impl-Block (Trait `crate::fusion::ext::ResonanceModifier`) `crates/contextra-rank/src/fusion/resonance.rs:92`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `enabled` | `bool` | `crates/contextra-rank/src/fusion/ext.rs:19` |

#### `trait ResonanceModifier`
- Ort: `crates/contextra-rank/src/fusion/ext.rs:9`
```rust
pub trait ResonanceModifier
```
Dokumentation: Trait für Signal-Resonanz und Kohärenz-Modulation.

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `fn modify_results(&self, results: Vec<SearchResult>, valid_signal_count: usize) -> Vec<SearchResult>` | `crates/contextra-rank/src/fusion/ext.rs:11` | Wendet Resonanz- oder Kohärenzmodifikationen auf die Ergebnisliste an. |

#### `fn apply_resonance_bonus`
- Ort: `crates/contextra-rank/src/fusion/ext.rs:31`
```rust
pub fn apply_resonance_bonus( results: Vec<SearchResult>, _valid_signal_count: usize, _config: &ResonanceConfig, ) -> Vec<SearchResult>
```
Dokumentation: No-Op Fallback für `apply_resonance_bonus`.

### Modul `fusion::global`

#### `struct GlobalFusionConfig`
- Ort: `crates/contextra-rank/src/fusion/global.rs:19`
```rust
pub struct GlobalFusionConfig
```
Dokumentation: Configuration for `GlobalFusionStrategy`. Konfigurationsparameter für die globale Community-Fusionsstrategie.
- Impl-Block (Trait `Default`) `crates/contextra-rank/src/fusion/global.rs:28`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `max_community_nodes` | `Option<usize>` | `crates/contextra-rank/src/fusion/global.rs:21` |
| `min_community_size` | `Option<usize>` | `crates/contextra-rank/src/fusion/global.rs:23` |
| `k_rrf` | `f32` | `crates/contextra-rank/src/fusion/global.rs:25` |

#### `struct GlobalFusionStrategy`
- Ort: `crates/contextra-rank/src/fusion/global.rs:47`
```rust
pub struct GlobalFusionStrategy
```
Dokumentation: Community-aware fusion strategy for corpus-wide topic retrieval (LeanRAG-backed). Globale Fusionsstrategie für hierarchische und korpusweite Retrieval-Szenarien.
- Impl-Block `crates/contextra-rank/src/fusion/global.rs:51`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new(config: GlobalFusionConfig) -> Self` | `crates/contextra-rank/src/fusion/global.rs:53` | Creates a new `GlobalFusionStrategy` with the given configuration. |
| `pub fn config(&self) -> &GlobalFusionConfig` | `crates/contextra-rank/src/fusion/global.rs:58` | Returns a reference to the configuration. |
| `pub fn fuse( &self, result_sets: Vec<(String, Vec<SearchResult>, f32)>, max_results: usize, ) -> Result<Vec<SearchResult>, RankError>` | `crates/contextra-rank/src/fusion/global.rs:144` | Fuses multi-signal search results using community-aware RRF. Candidates are grouped by community ID (extracted from candidate metadata `"community_id"` or `"com |

### Modul `fusion::mrrf`

#### `fn modulate_and_renormalize_weights`
- Ort: `crates/contextra-rank/src/fusion/mrrf.rs:18`
```rust
pub fn modulate_and_renormalize_weights( result_sets: &[(String, Vec<SearchResult>, f32)], calibration_contexts: Option<&[SignalCalibrationContext]>, beta: f32, ) -> Vec<f32>
```
Dokumentation: Modulates signal weights using mRRF margin formula: $m_s = S_1 - S_2$ (when signal is calibrated AND `drift_status` is `Stable` and $\ge 2$ candidates exist). $w_s' = w_s \times (1 + g(\beta \times m_s))$. Renormalizes modulated weights to maintain sum equal to original weight sum (preserving weight norm in [0.999, 1.001]). Moduliert Signalgewichte anhand der Margin-Differenz (mRRF) und renormalisiert sie.

#### `fn weighted_reciprocal_rank_fusion_mrrf`
- Ort: `crates/contextra-rank/src/fusion/mrrf.rs:176`
```rust
pub fn weighted_reciprocal_rank_fusion_mrrf( mut result_sets: Vec<(String, Vec<SearchResult>, f32)>, max_results: usize, priority: MetadataMergePriority, include_provenance: bool, resonance_config: Option<&ResonanceConfig>, calibration_contexts: Option<&[SignalCalibrationContext]>, beta: f32, ) -> Vec<SearchResult>
```
Dokumentation: Weighted Reciprocal Rank Fusion with mRRF margin-based calibration weight modulation. Gewichtete Rangfusion mit margin-basierter mRRF-Gewichtsmodulation.

### Modul `fusion::normalized`

#### `fn score_normalized_fusion_with_options`
- Ort: `crates/contextra-rank/src/fusion/normalized.rs:126`
```rust
pub fn score_normalized_fusion_with_options( mut result_sets: Vec<(String, Vec<SearchResult>, f32)>, max_results: usize, priority: MetadataMergePriority, include_provenance: bool, resonance_config: Option<&ResonanceConfig>, ) -> Vec<SearchResult>
```
Dokumentation: Score-normalized fusion (CombSUM with MinMax normalization). Führt eine score-normalisierte lineare Fusion mehrerer Signal-Ergebnismengen durch.

#### `fn weights_to_signal_factors`
- Ort: `crates/contextra-rank/src/fusion/normalized.rs:188`
```rust
pub fn weights_to_signal_factors( weights: Option<&crate::local_types::FusionWeights>, ) -> (f32, f32, f32)
```
Dokumentation: Converts optional `FusionWeights` into (vector, text, graph) weight tuple. Konvertiert relative Signalgewichte in Normalisierungsfaktoren.

### Modul `fusion::provenance`

#### `struct ProvenanceBuilder`
- Ort: `crates/contextra-rank/src/fusion/provenance.rs:13`
```rust
pub struct ProvenanceBuilder
```
Dokumentation: Fluent builder for constructing `ProvenanceRecord` instances. Fluent-Builder zum deterministischen Erstellen von `ProvenanceRecord`-Strukturen.
- Impl-Block `crates/contextra-rank/src/fusion/provenance.rs:30`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new(rrf_k: f32) -> Self` | `crates/contextra-rank/src/fusion/provenance.rs:32` | Creates a new `ProvenanceBuilder` with the specified RRF parameter `k`. |
| `pub fn vector(mut self, dist: f32, rank: u32, weight: impl Into<Option<f32>>) -> Self` | `crates/contextra-rank/src/fusion/provenance.rs:57` | Attaches vector signal distance, rank, and optional weight. |
| `pub fn bm25(mut self, score: f32, rank: u32, weight: impl Into<Option<f32>>) -> Self` | `crates/contextra-rank/src/fusion/provenance.rs:67` | Attaches BM25 text signal score, rank, and optional weight. |
| `pub fn graph(mut self, score: f32, rank: u32, weight: impl Into<Option<f32>>) -> Self` | `crates/contextra-rank/src/fusion/provenance.rs:77` | Attaches graph signal score, rank, and optional weight. |
| `pub fn rerank_score(mut self, score: f32) -> Self` | `crates/contextra-rank/src/fusion/provenance.rs:87` | Attaches optional cross-encoder rerank score. |
| `pub fn source_collection(mut self, collection: impl Into<String>) -> Self` | `crates/contextra-rank/src/fusion/provenance.rs:95` | Attaches source collection name. |
| `pub fn index_type(mut self, index_type: impl Into<String>) -> Self` | `crates/contextra-rank/src/fusion/provenance.rs:101` | Attaches index type string. |
| `pub fn expected_total(mut self, total: f32) -> Self` | `crates/contextra-rank/src/fusion/provenance.rs:107` | Sets expected ground truth RRF score for debug invariant checking. |
| `pub fn build(self) -> ProvenanceRecord` | `crates/contextra-rank/src/fusion/provenance.rs:114` | Builds the resulting `ProvenanceRecord`. Schließt die Konstruktion ab und berechnet die RRF-Beiträge je Signal. |
| `pub fn explain_human_readable(&self) -> String` | `crates/contextra-rank/src/fusion/provenance.rs:119` | Generates a deterministic human-readable explanation string from the built provenance record. |

### Modul `fusion::resonance`

#### `struct ResonanceConfig`
- Ort: `crates/contextra-rank/src/fusion/resonance.rs:15`
```rust
pub struct ResonanceConfig
```
Dokumentation: Configuration for Resonance Coherence Bonus. Konfigurationsparameter für die Berechnung des Resonanz-Kohärenz-Bonus.

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `beta` | `f32` | `crates/contextra-rank/src/fusion/resonance.rs:17` |
| `gamma` | `f32` | `crates/contextra-rank/src/fusion/resonance.rs:19` |

#### `fn apply_resonance_bonus`
- Ort: `crates/contextra-rank/src/fusion/resonance.rs:42`
```rust
pub fn apply_resonance_bonus( results: Vec<SearchResult>, valid_signal_count: usize, config: &ResonanceConfig, ) -> Vec<SearchResult>
```
Dokumentation: Applies Resonance Coherence Bonus to fused search results. Wendet den Resonanz-Kohärenz-Bonus auf bereits fusionierte Suchergebnisse an.

#### `fn fuse_signals`
- Ort: `crates/contextra-rank/src/fusion/resonance.rs:33`
```rust
pub fn fuse_signals( result_sets: Vec<(String, Vec<SearchResult>, f32)>, max_results: usize, ) -> Vec<FusedScore>
```
Dokumentation: Convenience API to fuse multi-signal search results. Komfortfunktion zur Zusammenführung mehrerer Signal-Ergebnismengen mittels gewichteter RRF.

### Modul `fusion::rrf`

#### `fn fuse_search_results_with_signal_strategies`
- Ort: `crates/contextra-rank/src/fusion/rrf.rs:656`
```rust
pub fn fuse_search_results_with_signal_strategies( mut result_sets: Vec<(String, Vec<SearchResult>, f32)>, max_results: usize, priority: MetadataMergePriority, include_provenance: bool, resonance_config: Option<&ResonanceConfig>, strategies: crate::local_types::SignalFusionStrategies, ) -> Vec<SearchResult>
```
Dokumentation: Fuses search result sets using per-signal fusion strategies (`SignalFusionStrategies`). Führt Teilergebnisse mit individuellen Strategien je Signalart zusammen.

#### `fn fuse_search_results_with_strategy`
- Ort: `crates/contextra-rank/src/fusion/rrf.rs:737`
```rust
pub fn fuse_search_results_with_strategy( result_sets: Vec<(String, Vec<SearchResult>, f32)>, max_results: usize, priority: MetadataMergePriority, include_provenance: bool, resonance_config: Option<&ResonanceConfig>, strategy: FusionStrategy, ) -> Vec<SearchResult>
```
Dokumentation: Fuses search result sets using the specified `FusionStrategy`. Universeller Einstiegspunkt zur Signal-Fusion basierend auf einer globalen `FusionStrategy`.

#### `fn g`
- Ort: `crates/contextra-rank/src/fusion/rrf.rs:35`
```rust
pub fn g(x: f32) -> f32
```
Dokumentation: Bounded smooth influence function $g(x) = \frac{x}{1 + |x|}$. Returns $x / (1 + |x|)$, bounded in $(-1.0, 1.0)$ for all finite $x$. Returns `0.0` if $x$ is non-finite (`NaN` or `Infinity`). Glatte, beschränkte Einflussfunktion $g(x) = \frac{x}{1 + |x|}$.

#### `fn reciprocal_rank_fusion`
- Ort: `crates/contextra-rank/src/fusion/rrf.rs:121`
```rust
pub fn reciprocal_rank_fusion( result_sets: Vec<Vec<SearchResult>>, max_results: usize, ) -> Vec<SearchResult>
```
Dokumentation: Fuses multiple sets of ranked search results into a single ranked list using Reciprocal Rank Fusion (RRF). Führt mehrere ungewichtete Signal-Ergebnislisten mittels Standard-RRF ($k=60$) zusammen.

#### `fn rrf_fuse`
- Ort: `crates/contextra-rank/src/fusion/rrf.rs:51`
```rust
pub fn rrf_fuse(ranked_lists: &[Vec<ScoredItem>], k: usize) -> Result<Vec<ScoredItem>>
```
Dokumentation: Standard RRF-Fusion gemäß Klassenvertrag RANK-1: `rrf_fuse(ranked_lists: &[Vec<ScoredItem>], k: usize) -> Result<Vec<ScoredItem>>`. Berechnet: $\text{score}(d) = \sum_{l} \frac{1}{k + \text{rank}_l(d)}$ mit 1-basiertem Rang. # Fehler Gibt `RankError::InvalidInput` zurück, wenn $k < 1$ ist oder ein Score $\text{NaN}$ / unendlich ist.

#### `fn weighted_reciprocal_rank_fusion`
- Ort: `crates/contextra-rank/src/fusion/rrf.rs:170`
```rust
pub fn weighted_reciprocal_rank_fusion( result_sets: Vec<(String, Vec<SearchResult>, f32)>, max_results: usize, ) -> Vec<SearchResult>
```
Dokumentation: Weighted Reciprocal Rank Fusion with default signal metadata priority (`VectorFirst`). Gewichtete reziproke Rangfusion (Weighted RRF) mit Standard-Metadatenpriorität.

#### `fn weighted_reciprocal_rank_fusion_with_options`
- Ort: `crates/contextra-rank/src/fusion/rrf.rs:197`
```rust
pub fn weighted_reciprocal_rank_fusion_with_options( result_sets: Vec<(String, Vec<SearchResult>, f32)>, max_results: usize, priority: MetadataMergePriority, include_provenance: bool, resonance_config: Option<&ResonanceConfig>, ) -> Vec<SearchResult>
```
Dokumentation: Weighted Reciprocal Rank Fusion with options. Vollständig konfigurierbare gewichtete Rangfusion mit Resonanz- und Provenance-Optionen.

#### `fn weighted_reciprocal_rank_fusion_with_priority`
- Ort: `crates/contextra-rank/src/fusion/rrf.rs:185`
```rust
pub fn weighted_reciprocal_rank_fusion_with_priority( result_sets: Vec<(String, Vec<SearchResult>, f32)>, max_results: usize, priority: MetadataMergePriority, ) -> Vec<SearchResult>
```
Dokumentation: Weighted Reciprocal Rank Fusion with explicit metadata merge priority. Gewichtete reziproke Rangfusion mit expliziter Metadaten-Zusammenführungspriorität.

### Modul `fusion::signal`

#### `struct SignalCalibrationContext`
- Ort: `crates/contextra-rank/src/fusion/signal.rs:18`
```rust
pub struct SignalCalibrationContext
```
Dokumentation: Calibration and drift context for a search signal during mRRF weight modulation. Kontext zu Modellkalibrierung und Driftzustand eines Signals für die mRRF-Gewichtsmodulation.
- Impl-Block `crates/contextra-rank/src/fusion/signal.rs:27`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `signal_name` | `String` | `crates/contextra-rank/src/fusion/signal.rs:20` |
| `is_calibrated` | `bool` | `crates/contextra-rank/src/fusion/signal.rs:22` |
| `drift_status` | `DriftStatus` | `crates/contextra-rank/src/fusion/signal.rs:24` |

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new( signal_name: impl Into<String>, is_calibrated: bool, drift_status: DriftStatus, ) -> Self` | `crates/contextra-rank/src/fusion/signal.rs:29` | Creates a new `SignalCalibrationContext`. |
| `pub fn should_use_calibrated_scores(&self) -> bool` | `crates/contextra-rank/src/fusion/signal.rs:44` | Evaluates whether calibrated scores should be used for margin calculation. Returns `true` if and only if `is_calibrated` is true AND `drift_status` is `DriftSta |

#### `enum MetadataMergePriority`
- Ort: `crates/contextra-rank/src/fusion/signal.rs:164`
```rust
pub enum MetadataMergePriority
```
Dokumentation: Configures signal priority order for metadata merging during Reciprocal Rank Fusion. Prioritätsstrategie bei der Zusammenführung kollidierender Metadatenschlüssel.
- Impl-Block `crates/contextra-rank/src/fusion/signal.rs:176`

Varianten:

- `VectorFirst` — `crates/contextra-rank/src/fusion/signal.rs:167`
- `TextFirst` — `crates/contextra-rank/src/fusion/signal.rs:169`
- `GraphFirst` — `crates/contextra-rank/src/fusion/signal.rs:171`
- `Custom(Vec<SignalKind>)` — `crates/contextra-rank/src/fusion/signal.rs:173`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn signal_rank(&self, signal_name: &str) -> usize` | `crates/contextra-rank/src/fusion/signal.rs:178` | Returns the precedence rank for a given signal name. |

#### `enum SignalKind`
- Ort: `crates/contextra-rank/src/fusion/signal.rs:94`
```rust
pub enum SignalKind
```
Dokumentation: Identifies the kind of search signal used during fusion. Klassifikation der unterschiedlichen Retrieval-Signalarten im Contextra-System.
- Impl-Block `crates/contextra-rank/src/fusion/signal.rs:105`
- Impl-Block (Trait `fmt::Display`) `crates/contextra-rank/src/fusion/signal.rs:155`

Varianten:

- `Vector` — `crates/contextra-rank/src/fusion/signal.rs:96`
- `Text` — `crates/contextra-rank/src/fusion/signal.rs:98`
- `Graph` — `crates/contextra-rank/src/fusion/signal.rs:100`
- `EdgeReinforcement` — `crates/contextra-rank/src/fusion/signal.rs:102`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn from_name(name: &str) -> Option<Self>` | `crates/contextra-rank/src/fusion/signal.rs:107` | Identifies `SignalKind` from a signal name string. |
| `pub fn as_str(&self) -> &'static str` | `crates/contextra-rank/src/fusion/signal.rs:135` | Converts `SignalKind` to its canonical string representation. |
| `pub fn name_de(&self) -> &'static str` | `crates/contextra-rank/src/fusion/signal.rs:145` | Returns the German display name for the signal. |

### Modul `fusion::topk`

#### `struct BoundedTopK`
- Ort: `crates/contextra-rank/src/fusion/topk.rs:24`
```rust
pub struct BoundedTopK<T>
```
Dokumentation: Size-bounded min-heap for Top-K candidate selection during Reciprocal Rank Fusion. # Abgrenzung Dieser Typ (`contextra_rank::fusion::BoundedTopK`) dient als größenbeschränkter Min-Heap zur Akkumulation der K besten Kandidaten während der Multi-Signal-Fusion (RRF / Score-Normalisierung). Er ist zu unterscheiden von `contextra_text::wand::BoundedTopK`, welcher speziell für den WAND-Algorithmus zur Trefferbeschränkung bei der Invertierten-Index-Suche eingesetzt wird. Größenbeschränkter Min-Heap für die Top-K-Kandidatenselektion während der Rangfusion.
- Impl-Block `crates/contextra-rank/src/fusion/topk.rs:29`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new(capacity: usize) -> Self` | `crates/contextra-rank/src/fusion/topk.rs:31` | Creates a new `BoundedTopK` container with maximum capacity `k`. |
| `pub fn capacity(&self) -> usize` | `crates/contextra-rank/src/fusion/topk.rs:40` | Returns maximum allowed capacity `k`. |
| `pub fn len(&self) -> usize` | `crates/contextra-rank/src/fusion/topk.rs:45` | Returns current number of items held in the container. |
| `pub fn is_empty(&self) -> bool` | `crates/contextra-rank/src/fusion/topk.rs:50` | Returns `true` if the container holds zero items. |
| `pub fn push(&mut self, item: T)` | `crates/contextra-rank/src/fusion/topk.rs:55` | Pushes an item into top-K container, evicting the worst candidate if size exceeds capacity `k`. |
| `pub fn into_sorted_vec(self) -> Vec<T>` | `crates/contextra-rank/src/fusion/topk.rs:70` | Consumes the container and returns items sorted from best to worst. |

### Modul `fusion::types`

#### `struct ProvenanceRecord`
- Ort: `crates/contextra-rank/src/fusion/types.rs:38`
```rust
pub struct ProvenanceRecord
```
Dokumentation: Provenance record for fused search result. Auditierbarer Herkunftsnachweis (Provenance) über alle beteiligten Retrieval-Kanäle.
- Impl-Block `crates/contextra-rank/src/fusion/provenance.rs:124`
- Impl-Block `crates/contextra-rank/src/fusion/types.rs:76`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `vector_distance` | `Option<f32>` | `crates/contextra-rank/src/fusion/types.rs:41` |
| `bm25_score` | `Option<f32>` | `crates/contextra-rank/src/fusion/types.rs:45` |
| `graph_score` | `Option<f32>` | `crates/contextra-rank/src/fusion/types.rs:49` |
| `rerank_score` | `Option<f32>` | `crates/contextra-rank/src/fusion/types.rs:53` |
| `source_collection` | `Option<String>` | `crates/contextra-rank/src/fusion/types.rs:61` |
| `index_type` | `Option<String>` | `crates/contextra-rank/src/fusion/types.rs:65` |
| `coherence_bonus` | `f32` | `crates/contextra-rank/src/fusion/types.rs:73` |

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn explain_human_readable(&self) -> String` | `crates/contextra-rank/src/fusion/provenance.rs:126` | Generates a concise, deterministic human-readable explanation from retrieval signal contributions. |
| `pub fn explain_human_readable_with_params( &self, fusion_mode: Option<&str>, calibrated_threshold: Option<f32>, ) -> String` | `crates/contextra-rank/src/fusion/provenance.rs:131` | Generates a concise, deterministic human-readable explanation with optional fusion mode and calibrated threshold. |
| `pub fn synthesized_from(source_doc_ids: &[DocId]) -> Self` | `crates/contextra-rank/src/fusion/types.rs:78` | Constructs a `ProvenanceRecord` synthesized from source document IDs. |

#### `struct ScoredItem`
- Ort: `crates/contextra-rank/src/fusion/types.rs:107`
```rust
pub struct ScoredItem
```
Dokumentation: Crate-lokaler Typ für gerankte/fusionierte Elemente.
- Impl-Block `crates/contextra-rank/src/fusion/types.rs:117`
- Impl-Block (Trait `From<SearchResult>`) `crates/contextra-rank/src/fusion/types.rs:134`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `id` | `String` | `crates/contextra-rank/src/fusion/types.rs:109` |
| `score` | `f32` | `crates/contextra-rank/src/fusion/types.rs:111` |
| `provenance` | `Option<ProvenanceRecord>` | `crates/contextra-rank/src/fusion/types.rs:114` |

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new(id: impl Into<String>, score: f32) -> Self` | `crates/contextra-rank/src/fusion/types.rs:119` | Erzeugt ein neues `ScoredItem` ohne Provenienz. |
| `pub fn with_provenance(mut self, provenance: ProvenanceRecord) -> Self` | `crates/contextra-rank/src/fusion/types.rs:128` | Fügt dem Element Provenienzinformationen hinzu. |

#### `struct SearchReport`
- Ort: `crates/contextra-rank/src/fusion/types.rs:209`
```rust
pub struct SearchReport
```
Dokumentation: Execution report for search query execution. Detaillierter Audit-Report über den Fusions- und Suchablauf.

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `degraded_signals` | `Vec<Signal>` | `crates/contextra-rank/src/fusion/types.rs:211` |
| `warnings` | `Vec<String>` | `crates/contextra-rank/src/fusion/types.rs:213` |
| `overfetch_stages` | `u8` | `crates/contextra-rank/src/fusion/types.rs:215` |

#### `struct SearchResult`
- Ort: `crates/contextra-rank/src/fusion/types.rs:159`
```rust
pub struct SearchResult
```
Dokumentation: Individual search result candidate for fusion. Standard-Suchergebnis eines einzelnen Retrieval-Signals oder einer Fusionsstufe.
- Impl-Block (Trait `From<ScoredItem>`) `crates/contextra-rank/src/fusion/types.rs:144`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `id` | `String` | `crates/contextra-rank/src/fusion/types.rs:161` |
| `score` | `f32` | `crates/contextra-rank/src/fusion/types.rs:163` |
| `metadata` | `Option<serde_json::Value>` | `crates/contextra-rank/src/fusion/types.rs:165` |
| `matched_signals` | `Vec<String>` | `crates/contextra-rank/src/fusion/types.rs:168` |
| `provenance` | `Option<ProvenanceRecord>` | `crates/contextra-rank/src/fusion/types.rs:171` |

#### `struct SignalContribution`
- Ort: `crates/contextra-rank/src/fusion/types.rs:96`
```rust
pub struct SignalContribution
```
Dokumentation: Signal contribution details. Beitrag eines einzelnen Signals zum aggregierten RRF-Score.

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `raw_score` | `f32` | `crates/contextra-rank/src/fusion/types.rs:98` |
| `rank` | `u32` | `crates/contextra-rank/src/fusion/types.rs:100` |
| `rrf_contribution` | `f32` | `crates/contextra-rank/src/fusion/types.rs:102` |

#### `enum Signal`
- Ort: `crates/contextra-rank/src/fusion/types.rs:187`
```rust
pub enum Signal
```
Dokumentation: Query retrieval signal identifier. # Abgrenzung Dieser Typ (`contextra_rank::fusion::Signal`) bezeichnet die spezifische Suchsignal-Art (z. B. `Vector`, `Text`, `Graph`) innerhalb von Suchberichten (`SearchReport`) der Fusion. Er unterscheidet sich von `Signal`, welches temporale bzw. System-Signale repräsentiert. Kennzeichnung der Signalherkunft.
- Impl-Block (Trait `std::fmt::Display`) `crates/contextra-rank/src/fusion/types.rs:196`

Varianten:

- `Vector` — `crates/contextra-rank/src/fusion/types.rs:189`
- `Text` — `crates/contextra-rank/src/fusion/types.rs:191`
- `Graph` — `crates/contextra-rank/src/fusion/types.rs:193`

#### `fn map_is_empty`
- Ort: `crates/contextra-rank/src/fusion/types.rs:13`
```rust
pub fn map_is_empty<K, V, S>(m: &ahash::AHashMap<K, V, S>) -> bool
```

#### `type FusedScore`
- Ort: `crates/contextra-rank/src/fusion/types.rs:176`
```rust
pub type FusedScore = SearchResult
```
Dokumentation: Type alias for fused score output item. Aggregierter Fusionswert eines Dokuments (Typ-Alias auf SearchResult).

### Modul `local_types`

#### `struct ConfigFingerprint`
- Ort: `crates/contextra-rank/src/local_types.rs:56`
```rust
pub struct ConfigFingerprint
```
Dokumentation: Konfigurations-Fingerprint zur Validierung und Cache-Invalidierung.
- Impl-Block (Trait `From<contextra_types::ConfigFingerprint>`) `crates/contextra-rank/src/compat/types.rs:21`
- Impl-Block `crates/contextra-rank/src/local_types.rs:61`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `hash` | `[u8; 32]` | `crates/contextra-rank/src/local_types.rs:58` |

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new( model_id: impl Into<String>, quantization: impl Into<String>, prompt_template: &str, temperature: f32, ) -> Self` | `crates/contextra-rank/src/local_types.rs:66` | Erzeugt einen Fingerprint aus Modell, Quantisierung, Template und Temperatur. |
| `pub const fn from_bytes(bytes: [u8` | `crates/contextra-rank/src/local_types.rs:88` | Erzeugt einen Fingerprint aus einem 32-Byte-Array. |
| `pub fn as_bytes(&self) -> &[u8` | `crates/contextra-rank/src/local_types.rs:93` | Gibt die rohen Bytes des Fingerprints zurück. |

Assoziierte Konstanten/Typen:

- `pub const ZERO: Self = ConfigFingerprint` — `crates/contextra-rank/src/local_types.rs:63`

#### `struct DocId`
- Ort: `crates/contextra-rank/src/local_types.rs:10`
```rust
pub struct DocId(pub u64)
```
Dokumentation: Opake Dokumentidentität für `contextra-rank`.
- Impl-Block (Trait `From<contextra_types::DocId>`) `crates/contextra-rank/src/compat/types.rs:9`
- Impl-Block (Trait `From<DocId>`) `crates/contextra-rank/src/compat/types.rs:15`
- Impl-Block `crates/contextra-rank/src/local_types.rs:12`
- Impl-Block (Trait `std::fmt::Display`) `crates/contextra-rank/src/local_types.rs:34`
- Impl-Block (Trait `From<u64>`) `crates/contextra-rank/src/local_types.rs:40`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn from_key(key: &str) -> Result<Self, String>` | `crates/contextra-rank/src/local_types.rs:14` | Erzeugt eine `DocId` aus einer hexadezimalen oder numerischen Zeichenkette. |

#### `struct FusionWeights`
- Ort: `crates/contextra-rank/src/local_types.rs:162`
```rust
pub struct FusionWeights
```
Dokumentation: Fusionsgewichte für die Normalisierung.
- Impl-Block (Trait `From<contextra_types::FusionWeights>`) `crates/contextra-rank/src/compat/types.rs:85`
- Impl-Block (Trait `Default`) `crates/contextra-rank/src/local_types.rs:173`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `vector` | `f32` | `crates/contextra-rank/src/local_types.rs:164` |
| `text` | `f32` | `crates/contextra-rank/src/local_types.rs:166` |
| `graph` | `f32` | `crates/contextra-rank/src/local_types.rs:168` |
| `metadata` | `f32` | `crates/contextra-rank/src/local_types.rs:170` |

#### `struct SignalFusionStrategies`
- Ort: `crates/contextra-rank/src/local_types.rs:127`
```rust
pub struct SignalFusionStrategies
```
Dokumentation: Signal-Fusionsstrategien pro Kanal.
- Impl-Block (Trait `From<contextra_types::SignalFusionStrategies>`) `crates/contextra-rank/src/compat/types.rs:65`
- Impl-Block (Trait `From<SignalFusionStrategies>`) `crates/contextra-rank/src/compat/types.rs:75`
- Impl-Block `crates/contextra-rank/src/local_types.rs:136`
- Impl-Block (Trait `From<FusionStrategy>`) `crates/contextra-rank/src/local_types.rs:148`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `vector` | `FusionStrategy` | `crates/contextra-rank/src/local_types.rs:129` |
| `text` | `FusionStrategy` | `crates/contextra-rank/src/local_types.rs:131` |
| `graph` | `FusionStrategy` | `crates/contextra-rank/src/local_types.rs:133` |

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn strategy_for_signal(&self, signal_name: &str) -> FusionStrategy` | `crates/contextra-rank/src/local_types.rs:138` | Gibt die Fusionsstrategie für ein bestimmtes Signal ("vector", "text", "graph") zurück. |

#### `struct TxId`
- Ort: `crates/contextra-rank/src/local_types.rs:187`
```rust
pub struct TxId(pub u64)
```
Dokumentation: Transaktions-Identität für zeitliche und MVCC-Gültigkeit.
- Impl-Block (Trait `From<contextra_types::TxId>`) `crates/contextra-rank/src/compat/types.rs:96`
- Impl-Block (Trait `From<TxId>`) `crates/contextra-rank/src/compat/types.rs:102`
- Impl-Block `crates/contextra-rank/src/local_types.rs:189`
- Impl-Block (Trait `From<u64>`) `crates/contextra-rank/src/local_types.rs:201`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub const fn new(id: u64) -> Self` | `crates/contextra-rank/src/local_types.rs:191` | Erzeugt eine neue `TxId`. |
| `pub const fn as_u64(&self) -> u64` | `crates/contextra-rank/src/local_types.rs:196` | Gibt die rohe u64-Zahl zurück. |

#### `enum FusionStrategy`
- Ort: `crates/contextra-rank/src/local_types.rs:102`
```rust
pub enum FusionStrategy
```
Dokumentation: Fusionsstrategie für hybride Suchen.
- Impl-Block (Trait `From<contextra_types::FusionStrategy>`) `crates/contextra-rank/src/compat/types.rs:29`
- Impl-Block (Trait `From<FusionStrategy>`) `crates/contextra-rank/src/compat/types.rs:38`

Varianten:

- `Rrf` — `crates/contextra-rank/src/local_types.rs:105`
- `ScoreNormalized` — `crates/contextra-rank/src/local_types.rs:107`

#### `enum SignalFailurePolicy`
- Ort: `crates/contextra-rank/src/local_types.rs:115`
```rust
pub enum SignalFailurePolicy
```
Dokumentation: Richtlinie bei Signal-Fehlern.
- Impl-Block (Trait `From<contextra_types::OnSignalFailure>`) `crates/contextra-rank/src/compat/types.rs:47`

Varianten:

- `Fail` — `crates/contextra-rank/src/local_types.rs:118`
- `Degrade` — `crates/contextra-rank/src/local_types.rs:120`

#### `const MAX_SEARCH_K`
- Ort: `crates/contextra-rank/src/local_types.rs:6`
```rust
pub const MAX_SEARCH_K: usize = 10_000
```
Dokumentation: Maximale Anzahl an Suchergebnissen für Top-K-Buffer.

### Modul `metrics`

#### `fn mean_reciprocal_rank`
- Ort: `crates/contextra-rank/src/metrics.rs:46`
```rust
pub fn mean_reciprocal_rank<'a, I>(queries: &[(&[I], &AHashSet<&str>)]) -> f32 where I: AsRef<str> + 'a,
```
Dokumentation: Berechnet den Mean Reciprocal Rank (MRR) über mehrere Abfragen hinweg. Liefert das arithmetische Mittel der Reciprocal Ranks aller Abfragen. Bei einer leeren Menge an Abfragen wird `0.0` zurückgegeben.

#### `fn ndcg_at_k`
- Ort: `crates/contextra-rank/src/metrics.rs:180`
```rust
pub fn ndcg_at_k<'a, I>(ranked_ids: &[I], relevance_scores: &AHashMap<&str, f32>, k: usize) -> f32 where I: AsRef<str> + 'a,
```
Dokumentation: Berechnet den Normalized Discounted Cumulative Gain (nDCG@k). Setzt den tatsächlichen DCG@k ins Verhältnis zum idealen DCG@k (IDCG@k) bei perfekter Sortierung. Liefert einen normalisierten Wert zwischen `0.0` und `1.0`. Falls kein relevantes Dokument existiert (IDCG == 0), wird `0.0` zurückgegeben. # Argumente * `ranked_ids` - Erhaltene Reihenfolge der Dokument-IDs. * `relevance_scores` - Relevanz-Grade der Dokumente (z. B. 0.0 bis 4.0 oder Wahrscheinlichkeiten). * `k` - Maximale Auswertungstiefe.

#### `fn precision_at_k`
- Ort: `crates/contextra-rank/src/metrics.rs:71`
```rust
pub fn precision_at_k<'a, I>(ranked_ids: &[I], relevant_ids: &AHashSet<&str>, k: usize) -> f32 where I: AsRef<str> + 'a,
```
Dokumentation: Berechnet Precision@k (Genauigkeit der Top-k-Ergebnisse). Anteil der relevanten Dokumente unter den ersten `k` Treffern: `|relevant ∩ top_k| / k`. Falls `k == 0` oder `ranked_ids.is_empty()`, wird `0.0` zurückgegeben.

#### `fn recall_at_k`
- Ort: `crates/contextra-rank/src/metrics.rs:105`
```rust
pub fn recall_at_k<'a, I>(ranked_ids: &[I], relevant_ids: &AHashSet<&str>, k: usize) -> f32 where I: AsRef<str> + 'a,
```
Dokumentation: Berechnet Recall@k (Trefferquote bezogen auf alle bekannten relevanten Dokumente). Anteil der gefundenen relevanten Dokumente unter den ersten `k` Treffern: `|relevant ∩ top_k| / |relevant|`. Falls `k == 0`, `ranked_ids.is_empty()` oder `relevant_ids.is_empty()`, wird `0.0` zurückgegeben.

#### `fn reciprocal_rank`
- Ort: `crates/contextra-rank/src/metrics.rs:22`
```rust
pub fn reciprocal_rank<'a, I>(ranked_ids: &[I], relevant_ids: &AHashSet<&str>) -> f32 where I: AsRef<str> + 'a,
```
Dokumentation: Berechnet den Reciprocal Rank (RR) für eine geordnete Liste von Treffer-IDs. Findet die 1-basierte Rangposition des ersten relevanten Dokuments und liefert `1.0 / rank`. Falls kein relevantes Dokument enthalten ist oder die Ergebnisliste leer ist, wird `0.0` zurückgegeben. # Argumente * `ranked_ids` - Nach Relevanz absteigend sortierte Dokument-IDs. * `relevant_ids` - Menge der als relevant geltenden Dokument-IDs.

### Modul `temporal`

#### `struct ValidityWindow`
- Ort: `crates/contextra-rank/src/temporal.rs:27`
```rust
pub struct ValidityWindow
```
Dokumentation: Extracted bi-temporal validity window representation for a document or search candidate.
- Impl-Block `crates/contextra-rank/src/temporal.rs:38`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `tx_valid_from` | `Option<TxId>` | `crates/contextra-rank/src/temporal.rs:29` |
| `tx_valid_to` | `Option<TxId>` | `crates/contextra-rank/src/temporal.rs:31` |
| `business_valid_from` | `Option<i64>` | `crates/contextra-rank/src/temporal.rs:33` |
| `business_valid_to` | `Option<i64>` | `crates/contextra-rank/src/temporal.rs:35` |

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn is_empty(&self) -> bool` | `crates/contextra-rank/src/temporal.rs:44` | Returns `true` if no validity window metadata constraints are set. Candidates without validity window metadata (legacy data) are treated as always valid (Fail-O |
| `pub fn is_valid_bitemporal(&self, as_of_tx: TxId, as_of_business: Option<u64>) -> bool` | `crates/contextra-rank/src/temporal.rs:54` | Evaluates bi-temporal validity using strict AND logic across system transaction time and business time. Intervall-Semantik: `[valid_from, valid_until)` (inclusi |

#### `fn apply_temporal_validity_filter`
- Ort: `crates/contextra-rank/src/temporal.rs:189`
```rust
pub fn apply_temporal_validity_filter( results: Vec<crate::fusion::SearchResult>, current_tx: TxId, query_timestamp: Option<u64>, ) -> Vec<crate::fusion::SearchResult>
```
Dokumentation: Applies bi-temporal validity filtering to candidates post-RRF fusion. Filters out candidate results that are invalid relative to the current transaction `current_tx` or business timestamp `query_timestamp`. Candidates without validity window metadata (legacy data) are retained (Fail-Open).

#### `fn apply_temporal_validity_filter_at`
- Ort: `crates/contextra-rank/src/temporal.rs:209`
```rust
pub fn apply_temporal_validity_filter_at( results: Vec<crate::fusion::SearchResult>, as_of_timestamp: u64, ) -> Vec<crate::fusion::SearchResult>
```
Dokumentation: Applies historical "as-of" bi-temporal validity filtering to candidates post-RRF fusion. Evaluates candidate validity at historical point-in-time `as_of_timestamp` (analogous to Graphitis Episode-Pinning). A candidate is retained ONLY IF `as_of_timestamp` lies within `[valid_from, valid_until)` AND transaction validity (`tx_valid_to`, ADR-033) is active at that historical timestamp. Candidates without validity window metadata (legacy data) are retained (Fail-Open for backward compatibility).

#### `fn apply_temporal_validity_filter_with_clock`
- Ort: `crates/contextra-rank/src/temporal.rs:175`
```rust
pub fn apply_temporal_validity_filter_with_clock( results: Vec<crate::fusion::SearchResult>, current_tx: TxId, clock: &dyn crate::calibration::Clock, ) -> Vec<crate::fusion::SearchResult>
```
Dokumentation: Applies bi-temporal validity filtering using an injected `Clock` port trait (B11).

#### `fn extract_validity_window`
- Ort: `crates/contextra-rank/src/temporal.rs:122`
```rust
pub fn extract_validity_window(metadata: Option<&Value>) -> ValidityWindow
```
Dokumentation: Extracts bi-temporal validity window metadata from candidate document metadata.

#### `fn sort_candidates_deterministic`
- Ort: `crates/contextra-rank/src/temporal.rs:165`
```rust
pub fn sort_candidates_deterministic(results: &mut [crate::fusion::SearchResult])
```
Dokumentation: Sorts candidates deterministically by score descending, breaking ties by document ID ascending (B11).

#### `type FusionResult`
- Ort: `crates/contextra-rank/src/temporal.rs:23`
```rust
pub type FusionResult = crate::fusion::SearchResult
```
Dokumentation: Type alias for search & fusion results in post-retrieval pipelines.

## 9. Impl-Blöcke ohne Typ im selben Crate

- `impl From<RankError> for contextra_types::ContextraError` — `crates/contextra-rank/src/compat/error.rs:5` (0 pub Methoden)
- `impl From<SignalFailurePolicy> for contextra_types::OnSignalFailure` — `crates/contextra-rank/src/compat/types.rs:56` (0 pub Methoden)
- `impl TryFrom<SearchResult> for contextra_types::ContextChunk` — `crates/contextra-rank/src/compat/types.rs:108` (0 pub Methoden)
- `impl From<DocId> for u64` — `crates/contextra-rank/src/local_types.rs:46` (0 pub Methoden)
- `impl From<TxId> for u64` — `crates/contextra-rank/src/local_types.rs:207` (0 pub Methoden)
