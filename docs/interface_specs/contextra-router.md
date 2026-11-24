# Schnittstellenspezifikation: `contextra-router`

> Erzeugt durch `tools/extract_interfaces.py` aus dem Quellcode. Keine Handpflege. Jede Angabe trägt Datei:Zeile. Maßgeblich bleibt der Quellcode.

## 1. Kopf

| Feld | Wert |
|---|---|
| Name | `contextra-router` |
| Version | workspace |
| Pfad | `crates/contextra-router` |
| Rolle | Modell-Routing |
| Bibliothekspfad | src/lib.rs (Standard) |

**Crate-Dokumentation (//!):**

> # contextra-router
>
> Intelligentes Routing von Anfragen an geeignete Small Language Models (SLM).
> Bewertet Modell-Profile (z. B. Token-Limits, Domänen-Affinitäten), kalibriert Konfidenzwerte
> über konforme Vorhersagen (Conformal Prediction) und steuert Multi-Armed-Bandit-Verfahren (LinUCB / Thompson Sampling).
>
> **Abhängigkeiten:**
> - Upstream: `contextra-types`, `contextra-ports`, `contextra-adapt`, `contextra-wire` (optional).
> - Downstream: MCP-Server (`contextra-mcp`).
>
> **Hauptkomponenten:**
> - [`RouterEngine`]: Primärer Routing-Endpunkt zur Entscheidung für das beste Modell.

## 2. Abhängigkeiten

| Name | Art | Quelle | optional | verwendet von (normal/dev) |
|---|---|---|---|---|
| `arc-swap` | normal | registry | nein |  |
| `contextra-adapt` | normal | workspace | nein | contextra-db, contextra-router |
| `contextra-ports` | normal | workspace | nein | contextra, contextra-adapt, contextra-agent, contextra-checkpoint, contextra-cognition, contextra-crypto, contextra-crypto-fuzz, contextra-db, contextra-engine, contextra-graph, contextra-graph-fuzz, contextra-infer-candle, contextra-infer-ollama, contextra-infer-onnx, contextra-kvcache, contextra-mcp, contextra-mvcc, contextra-rank, contextra-router, contextra-store, contextra-store-fuzz, contextra-testkit, contextra-text, contextra-vector, contextra-vector-fuzz |
| `contextra-types` | normal | workspace | nein | contextra, contextra-adapt, contextra-agent, contextra-bench, contextra-checkpoint, contextra-cognition, contextra-crypto, contextra-crypto-fuzz, contextra-db, contextra-engine, contextra-graph, contextra-graph-fuzz, contextra-infer-candle, contextra-infer-ollama, contextra-infer-onnx, contextra-ingest, contextra-kvcache, contextra-mcp, contextra-mvcc, contextra-ports, contextra-py, contextra-rank, contextra-router, contextra-simd, contextra-store, contextra-store-fuzz, contextra-testkit, contextra-text, contextra-vault, contextra-vector, contextra-vector-fuzz |
| `contextra-wire` | normal | workspace | ja | contextra-mcp, contextra-py, contextra-router, contextra-wire-fuzz |
| `criterion` | dev | registry | nein |  |
| `parking_lot` | normal | workspace | nein |  |
| `proptest` | dev | registry | nein |  |
| `serde` | normal | workspace | nein |  |
| `serde_json` | normal | workspace | nein |  |
| `shlex` | normal | registry | ja |  |
| `tempfile` | dev | registry | nein |  |
| `thiserror` | normal | workspace | nein |  |
| `tokio` | dev | workspace | nein |  |
| `tracing` | normal | workspace | nein |  |
| `tracing-subscriber` | dev | workspace | nein |  |

**Verwendet von (Crate-Ebene):** `contextra-agent`, `contextra-mcp`

## 3. Features

| Feature | Aktiviert |
|---|---|
| `bandit-routing` | `egress-sherman-morrison`, `contextra-adapt/bandit-routing` |
| `cloud-egress-guard` | – |
| `default` | – |
| `egress-sherman-morrison` | – |
| `exp-fc-ts` | `flow-corrected-thompson` |
| `exp-routing-strategy` | `bandit-routing` |
| `flow-corrected-thompson` | `contextra-adapt/flow-corrected-thompson` |
| `subprocess-dispatch` | `dep:contextra-wire`, `dep:tokio`, `dep:shlex` |

## 4. Metriken (Quellcode)

| Kennzahl | Wert |
|---|---|
| Quelldateien | 30 |
| Quellzeilen | 8337 |
| Öffentliche Items (Modul-Ebene) | 53 |
| davon `fn` | 9 |
| davon `struct` | 22 |
| davon `enum` | 6 |
| davon `trait` | 1 |
| davon `type` | 2 |
| davon `mod` | 13 |
| Öffentliche Makros (`#[macro_export]`) | 0 |
| Öffentliche Re-Exports (`pub use`) | 11 |
| `unsafe`-Vorkommen (Produktionscode) | 0 |
| `.unwrap()` (Produktionscode) | 183 |
| `.expect(` (Produktionscode) | 10 |
| `panic!` (Produktionscode) | 4 |
| `todo!/unimplemented!/unreachable!` | 0 |
| Indexierungen `x[…]` (grob, Produktionscode) | 20 |
| `#[deprecated]` | 3 |
| `#[cfg(feature…)]`-Gates | 54 |
| Parser-Abdeckung (geparste `pub`-Deklarationen / Grep-Treffer) | 131 / 131 = 100 % |

## 5. Modulbaum und Dateien

| Modul | Datei | Zeilen |
|---|---|---|
| `arm_registry` | `crates/contextra-router/src/arm_registry.rs` | 238 |
| `bandit_regret_tests` | `crates/contextra-router/src/bandit_regret_tests.rs` | 323 |
| `builder` | `crates/contextra-router/src/builder.rs` | 260 |
| `dispatch` | `crates/contextra-router/src/dispatch.rs` | 291 |
| `fc_ts_dispatch` | `crates/contextra-router/src/fc_ts_dispatch.rs` | 92 |
| `crate` | `crates/contextra-router/src/lib.rs` | 67 |
| `outcome` | `crates/contextra-router/src/outcome.rs` | 202 |
| `ports_local` | `crates/contextra-router/src/ports_local.rs` | 35 |
| `profile` | `crates/contextra-router/src/profile.rs` | 540 |
| `router::dispatch_core` | `crates/contextra-router/src/router/dispatch_core.rs` | 654 |
| `router::lifecycle` | `crates/contextra-router/src/router/lifecycle.rs` | 264 |
| `router` | `crates/contextra-router/src/router/mod.rs` | 322 |
| `router::outcomes` | `crates/contextra-router/src/router/outcomes.rs` | 224 |
| `router::tests` | `crates/contextra-router/src/router/tests.rs` | 221 |
| `routing_strategy` | `crates/contextra-router/src/routing_strategy.rs` | 72 |
| `seam` | `crates/contextra-router/src/seam.rs` | 154 |
| `serde_helpers` | `crates/contextra-router/src/serde_helpers.rs` | 67 |
| `tests::bandit_tests` | `crates/contextra-router/src/tests/bandit_tests.rs` | 99 |
| `tests::fixtures` | `crates/contextra-router/src/tests/fixtures.rs` | 385 |
| `tests::id_gen_determinism_test` | `crates/contextra-router/src/tests/id_gen_determinism_test.rs` | 40 |
| `tests::latency_tests` | `crates/contextra-router/src/tests/latency_tests.rs` | 22 |
| `tests::misc_tests::part1` | `crates/contextra-router/src/tests/misc_tests/part1.rs` | 1014 |
| `tests::misc_tests::part2` | `crates/contextra-router/src/tests/misc_tests/part2.rs` | 821 |
| `tests::misc_tests` | `crates/contextra-router/src/tests/misc_tests.rs` | 23 |
| `tests` | `crates/contextra-router/src/tests/mod.rs` | 26 |
| `tests::routing_tests::dispatch` | `crates/contextra-router/src/tests/routing_tests/dispatch.rs` | 319 |
| `tests::routing_tests::engine_part1` | `crates/contextra-router/src/tests/routing_tests/engine_part1.rs` | 669 |
| `tests::routing_tests::engine_part2` | `crates/contextra-router/src/tests/routing_tests/engine_part2.rs` | 776 |
| `tests::routing_tests` | `crates/contextra-router/src/tests/routing_tests.rs` | 25 |
| `transport` | `crates/contextra-router/src/transport.rs` | 92 |

## 6. Re-Exports (`pub use`)

- `pub use arm_registry::` — `crates/contextra-router/src/lib.rs:52` (Modul `crate`)
- `pub use builder::RouterEngineBuilder` — `crates/contextra-router/src/lib.rs:53` (Modul `crate`)
- `pub use dispatch::dispatch_to_slm` — `crates/contextra-router/src/lib.rs:55` (Modul `crate`)
- `pub use outcome::` — `crates/contextra-router/src/lib.rs:56` (Modul `crate`)
- `pub use profile::` — `crates/contextra-router/src/lib.rs:58` (Modul `crate`)
- `pub use router::` — `crates/contextra-router/src/lib.rs:59` (Modul `crate`)
- `pub use seam::` — `crates/contextra-router/src/lib.rs:60` (Modul `crate`)
- `pub use fc_ts_dispatch::` — `crates/contextra-router/src/lib.rs:63` (Modul `crate`)
- `pub use routing_strategy::RoutingStrategy` — `crates/contextra-router/src/lib.rs:65` (Modul `crate`)
- `pub use transport::Transport` — `crates/contextra-router/src/lib.rs:67` (Modul `crate`)
- `pub use contextra_ports::` — `crates/contextra-router/src/ports_local.rs:16` (Modul `ports_local`)

## 8. Öffentliche Schnittstelle je Item

### Modul `arm_registry`

#### `struct ArmRegistry`
- Ort: `crates/contextra-router/src/arm_registry.rs:27`
```rust
pub struct ArmRegistry
```
Dokumentation: Verlustfreie, deterministische Abbildung Arm-Index <-> RetrievalStrategy. Lebt in `contextra-router` (wo Arme registriert und ausgewertet werden), NICHT im generischen `BanditPolicy`-Port in Ring 0. # Invariante Hält genau 5 registrierte Arme in fester Reihenfolge (Indizes 0 bis 4).
- Impl-Block (Trait `Default`) `crates/contextra-router/src/arm_registry.rs:40`
- Impl-Block `crates/contextra-router/src/arm_registry.rs:64`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn strategy_for(&self, arm: u32) -> Result<RetrievalStrategy, ArmRegistryError>` | `crates/contextra-router/src/arm_registry.rs:84` | Liefert die [`RetrievalStrategy`] für den gegebenen Arm-Index (0..=4). # Ablauf 1. Konvertierung des `u32`-Index nach `usize`. 2. Abruf des Eintrags aus dem int |
| `pub fn arm_for(&self, strategy: RetrievalStrategy) -> u32` | `crates/contextra-router/src/arm_registry.rs:113` |  |

#### `enum ArmRegistryError`
- Ort: `crates/contextra-router/src/arm_registry.rs:34`
```rust
pub enum ArmRegistryError
```
Dokumentation: Fehlerklasse für ungültige Arm-Indizes — crate-lokal, P6-konform an Systemgrenzen konvertierbar.

Varianten:

- `OutOfRange(u32)` — `crates/contextra-router/src/arm_registry.rs:37`

### Modul `builder`

#### `struct RouterEngineBuilder`
- Ort: `crates/contextra-router/src/builder.rs:33`
```rust
pub struct RouterEngineBuilder
```
Dokumentation: Builder zur flexiblen und typsicheren Konstruktion einer [`RouterEngine`].
- Impl-Block (Trait `Default`) `crates/contextra-router/src/builder.rs:48`
- Impl-Block `crates/contextra-router/src/builder.rs:54`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new() -> Self` | `crates/contextra-router/src/builder.rs:57` | Erzeugt einen neuen leeren `RouterEngineBuilder`. |
| `pub fn search_provider(mut self, provider: Arc<dyn HybridSearchProvider>) -> Self` | `crates/contextra-router/src/builder.rs:75` | Konfiguriert den hybriden Such-Provider (Pflichtfeld). |
| `pub fn community_resolver(mut self, resolver: Arc<dyn CommunityResolver>) -> Self` | `crates/contextra-router/src/builder.rs:81` | Konfiguriert den Community-Resolver (Pflichtfeld). |
| `pub fn context_preparer(mut self, preparer: Arc<dyn ContextPreparer>) -> Self` | `crates/contextra-router/src/builder.rs:87` | Konfiguriert den Kontext-Vorbereiter (Pflichtfeld). |
| `pub fn profiles(mut self, profiles: Vec<SlmProfile>) -> Self` | `crates/contextra-router/src/builder.rs:93` | Konfiguriert die zu berücksichtigenden SLM-Profile. |
| `pub fn add_profile(mut self, profile: SlmProfile) -> Self` | `crates/contextra-router/src/builder.rs:99` | Fügt ein einzelnes Profil zur Profil-Liste hinzu. |
| `pub fn calibration_source(mut self, source: Box<dyn CalibrationSource>) -> Self` | `crates/contextra-router/src/builder.rs:105` | Konfiguriert eine benutzerdefinierte Kalibrierungsquelle ([`CalibrationSource`]). |
| `pub fn calibration_file(mut self, path: PathBuf) -> Self` | `crates/contextra-router/src/builder.rs:111` | Konfiguriert das Lesen der Kalibrierungsdaten über das Standard-Dateisystem (`std::fs`). |
| `pub fn calibration_vfs(mut self, vfs: Arc<dyn contextra_ports::vfs::Vfs>, path: PathBuf) -> Self` | `crates/contextra-router/src/builder.rs:118` | Konfiguriert das Lesen der Kalibrierungsdaten über einen virtuellen Dateisystem-Port ([`contextra_ports::vfs::Vfs`]). |
| `pub fn calibration_store_path(mut self, path: Option<PathBuf>) -> Self` | `crates/contextra-router/src/builder.rs:125` | Setzt den Pfad zur Kalibrierungsdatei (unter Beibehaltung der konfigurierten Quelle). |
| `pub fn clock(mut self, clock: Arc<dyn Clock>) -> Self` | `crates/contextra-router/src/builder.rs:131` | Injiziert eine benutzerdefinierte System-Uhr ([`Clock`]). |
| `pub fn initial_decision_id(mut self, start: u64) -> Self` | `crates/contextra-router/src/builder.rs:137` | Setzt den Startwert für monotone Entscheidungs-IDs. |
| `pub fn routing_strategy( mut self, strategy: RoutingStrategy, epsilon: f32, seed: u64, ) -> Self` | `crates/contextra-router/src/builder.rs:144` | Konfiguriert die Routing-Strategie (Cascade oder Bandit) mit festem Epsilon und Seed. |
| `pub fn bandit_rng( mut self, strategy: RoutingStrategy, epsilon: f32, rng: Box<dyn contextra_adapt::Rng + Send>, ) -> Self` | `crates/contextra-router/src/builder.rs:157` | Konfiguriert die Bandit-Exploration mit injiziertem Rng-Port. |
| `pub fn build(mut self) -> Result<RouterEngine>` | `crates/contextra-router/src/builder.rs:173` | Validiert alle Eingaben und erzeugt die [`RouterEngine`]. # Fehler - [`ContextraError::InvalidInput`]: Wenn ein Pflicht-Port fehlt oder Profile ungültig sind. |

### Modul `crate`

#### `mod arm_registry`
- Ort: `crates/contextra-router/src/lib.rs:28`
```rust
pub mod arm_registry
```

#### `mod builder`
- Ort: `crates/contextra-router/src/lib.rs:29`
```rust
pub mod builder
```

#### `mod dispatch`
- Ort: `crates/contextra-router/src/lib.rs:31`
```rust
pub mod dispatch
```
- Gate: `#[cfg(feature = "subprocess-dispatch")]`

#### `mod fc_ts_dispatch`
- Ort: `crates/contextra-router/src/lib.rs:40`
```rust
pub mod fc_ts_dispatch
```
- Gate: `#[cfg(feature = "flow-corrected-thompson")]`

#### `mod outcome`
- Ort: `crates/contextra-router/src/lib.rs:32`
```rust
pub mod outcome
```

#### `mod ports_local`
- Ort: `crates/contextra-router/src/lib.rs:33`
```rust
pub mod ports_local
```

#### `mod profile`
- Ort: `crates/contextra-router/src/lib.rs:34`
```rust
pub mod profile
```

#### `mod router`
- Ort: `crates/contextra-router/src/lib.rs:35`
```rust
pub mod router
```

#### `mod routing_strategy`
- Ort: `crates/contextra-router/src/lib.rs:42`
```rust
pub mod routing_strategy
```

#### `mod seam`
- Ort: `crates/contextra-router/src/lib.rs:36`
```rust
pub mod seam
```

#### `mod serde_helpers`
- Ort: `crates/contextra-router/src/lib.rs:37`
```rust
pub mod serde_helpers
```

#### `mod transport`
- Ort: `crates/contextra-router/src/lib.rs:44`
```rust
pub mod transport
```
- Gate: `#[cfg(feature = "cloud-egress-guard")]`

### Modul `dispatch`

#### `fn dispatch_to_slm`
- Ort: `crates/contextra-router/src/dispatch.rs:98`
```rust
pub async fn dispatch_to_slm(decision: &RoutingDecision) -> Result<String>
```
Dokumentation: Führt den MCP-Aufruf an das in der Routing-Entscheidung ausgewählte SLM aus. # Ablauf 1. Überprüfung des Transport-Kanals (aktuell Stdio JSON-RPC 2.0). 2. MCP-Endpunkt zerlegen (`split_endpoint`) und Subprozess spawnen. 3. JSON-RPC 2.0 Request (`tools/call` oder `generate`) serialisieren und auf Stdin schreiben. 4. Antwortzeile von Stdout asynchron einlesen und deserialisieren. 5. Validierung des JSON-RPC-Ergebnisses und Rückgabe des Antworttexts. # Vorbedingungen - `decision.profile.mcp_endpoint` muss ein gültiges ausführbares Programm spezifizieren. - Das System muss Prozess-Spawns erlauben.

### Modul `fc_ts_dispatch`

#### `enum FcTsDispatchError`
- Ort: `crates/contextra-router/src/fc_ts_dispatch.rs:21`
```rust
pub enum FcTsDispatchError
```
Dokumentation: Fehler der FC-TS-Profilauswahl für contextra-router.

Varianten:

- `ArmSet(#[from] FcTsError)` — `crates/contextra-router/src/fc_ts_dispatch.rs:24`
- `ProfileArmMismatch{ profiles: usize, arms: usize }` — `crates/contextra-router/src/fc_ts_dispatch.rs:27`

#### `fn deterministic_fc_ts_rng`
- Ort: `crates/contextra-router/src/fc_ts_dispatch.rs:90`
```rust
pub fn deterministic_fc_ts_rng(seed: u64) -> impl FcTsRng
```

#### `fn select_profile_fc_ts`
- Ort: `crates/contextra-router/src/fc_ts_dispatch.rs:54`
```rust
pub fn select_profile_fc_ts( profile_names: &[String], arm_set: &FcTsArmSet, context: &[f32], rng: &mut dyn FcTsRng, ) -> Result<(usize, String), FcTsDispatchError>
```

### Modul `outcome`

#### `struct DecisionId`
- Ort: `crates/contextra-router/src/outcome.rs:32`
```rust
pub struct DecisionId(u64)
```
Dokumentation: Eindeutige ID einer Routing-Entscheidung. Wird von `RouterEngine::route()` erzeugt und vergeben und später von `RouterEngine::record_outcome()` konsumiert, um das Feedback dem richtigen Modellprofil zuzuordnen.
- Impl-Block `crates/contextra-router/src/outcome.rs:34`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub const fn from_raw(raw: u64) -> Self` | `crates/contextra-router/src/outcome.rs:40` | Erzeugt eine [`DecisionId`] aus einem rohen 64-Bit-Ganzzahlwert. # Parameter - `raw`: Rohwert der ID. |
| `pub const fn inner(self) -> u64` | `crates/contextra-router/src/outcome.rs:46` | Liefert den zugrundeliegenden `u64`-Wert der [`DecisionId`]. |

#### `struct DecisionIdGenerator`
- Ort: `crates/contextra-router/src/outcome.rs:56`
```rust
pub struct DecisionIdGenerator
```
Dokumentation: Instanzgebundener Generator für monoton steigende [`DecisionId`]s (Spec §3 P29). Garantiert Thread-Sicherheit über atomare Inkrement-Operationen ohne globale Sperren.
- Impl-Block `crates/contextra-router/src/outcome.rs:60`
- Impl-Block (Trait `Default`) `crates/contextra-router/src/outcome.rs:93`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new(start: u64) -> Self` | `crates/contextra-router/src/outcome.rs:66` | Erstellt einen neuen Generator mit dem angegebenen Startwert. # Parameter - `start`: Anfangswert für die Sequenz (meist 0). |
| `pub fn next(&self) -> DecisionId` | `crates/contextra-router/src/outcome.rs:88` |  |

#### `enum RoutingOutcome`
- Ort: `crates/contextra-router/src/outcome.rs:106`
```rust
pub enum RoutingOutcome
```
Dokumentation: Tatsächliches Ergebnis einer getroffenen Routing-Entscheidung. Wird vom Agent-Orchestrator oder Evaluator NACH dem SLM-Aufruf an den Router zurückgemeldet.
- Impl-Block `crates/contextra-router/src/outcome.rs:116`

Varianten:

- `Success` — `crates/contextra-router/src/outcome.rs:108`
- `Escalated{ escalated_to: String }` — `crates/contextra-router/src/outcome.rs:111`
- `Rejected{ reason: Option<String> }` — `crates/contextra-router/src/outcome.rs:113`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn non_conformity_score(&self) -> f32` | `crates/contextra-router/src/outcome.rs:127` | Berechnet den Non-Conformity-Score: 0.0 = perfekt konform, 1.0 = maximal non-konform / fehlerhaft. # Werte - [`RoutingOutcome::Success`] => `0.0` - [`RoutingOut |

### Modul `profile`

#### `struct ProfileCalibrationState`
- Ort: `crates/contextra-router/src/profile.rs:308`
```rust
pub struct ProfileCalibrationState
```
Dokumentation: Laufzeit-Zustand der Kalibrierung eines Profils inklusive Auswahlhäufigkeit und Konfidenz.
- Impl-Block (Trait `Default`) `crates/contextra-router/src/profile.rs:326`
- Impl-Block `crates/contextra-router/src/profile.rs:339`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `times_selected` | `u64` | `crates/contextra-router/src/profile.rs:310` |
| `cumulative_confidence` | `f64` | `crates/contextra-router/src/profile.rs:313` |
| `calibrated_min_score` | `f32` | `crates/contextra-router/src/profile.rs:316` |
| `original_min_score` | `f32` | `crates/contextra-router/src/profile.rs:318` |
| `conformal` | `ProfileCalibrator` | `crates/contextra-router/src/profile.rs:320` |
| `last_calibrated_fingerprint` | `Option<ConfigFingerprint>` | `crates/contextra-router/src/profile.rs:323` |

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new(original_min_score: f32) -> Self` | `crates/contextra-router/src/profile.rs:342` | Erzeugt einen neuen Initialzustand für die Kalibrierung eines Profils. |
| `pub fn check_and_invalidate_fingerprint(&mut self, active_fp: Option<&ConfigFingerprint>)` | `crates/contextra-router/src/profile.rs:356` | Prüft, ob der aktive Konfigurations-Fingerprint abweicht, und setzt bei Diskrepanz die Kalibrierung zurück (INV-P8-1). |
| `pub fn is_calibrated(&self, active_fp: Option<&ConfigFingerprint>) -> bool` | `crates/contextra-router/src/profile.rs:380` | Prüft, ob das Profil kalibriert und der Fingerprint noch valide ist. |
| `pub fn average_confidence(&self) -> f64` | `crates/contextra-router/src/profile.rs:393` | Liefert die durchschnittliche Konfidenz über alle Auswahlen. |
| `pub fn recalibrate_conformal(&mut self, non_conformity_score: f32) -> bool` | `crates/contextra-router/src/profile.rs:403` | Führt eine Neukalibrierung anhand des beobachteten Non-Conformity-Scores durch. |
| `pub fn reset(&mut self)` | `crates/contextra-router/src/profile.rs:416` | Setzt alle Kalibrierungsstatistiken auf die Ausgangswerte zurück. |

#### `struct ProfileCalibrator`
- Ort: `crates/contextra-router/src/profile.rs:207`
```rust
pub struct ProfileCalibrator
```
Dokumentation: Konformaler Kalibrator nach Gibbs & Candès zur dynamischen Schwellwertanpassung.
- Impl-Block (Trait `Default`) `crates/contextra-router/src/profile.rs:230`
- Impl-Block `crates/contextra-router/src/profile.rs:242`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `alpha` | `f32` | `crates/contextra-router/src/profile.rs:209` |
| `gamma` | `f32` | `crates/contextra-router/src/profile.rs:211` |
| `quantile_threshold` | `f32` | `crates/contextra-router/src/profile.rs:214` |
| `window_errors` | `u64` | `crates/contextra-router/src/profile.rs:216` |
| `window_total` | `u64` | `crates/contextra-router/src/profile.rs:218` |

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new(alpha: f32, gamma: f32, initial_threshold: f32) -> Self` | `crates/contextra-router/src/profile.rs:246` | Erstellt einen neuen `ProfileCalibrator` mit Fehlertoleranz `alpha`, Schrittweite `gamma` und Startschwelle. |
| `pub fn update(&mut self, non_conformity_score: f32) -> bool` | `crates/contextra-router/src/profile.rs:267` | Aktualisiert den Quantils-Schwellwert basierend auf einem beobachteten Non-Conformity-Score. # Ablauf 1. Fehlerprüfung: War der Score größer als die bisherige S |
| `pub fn empirical_error_rate(&self) -> f32` | `crates/contextra-router/src/profile.rs:287` | Berechnet die empirische Fehlerrate im aktuellen Beobachtungsfenster. |
| `pub fn reset_window(&mut self)` | `crates/contextra-router/src/profile.rs:297` | Setzt die Fensterzähler für Fehler und Gesamtzahl zurück. |

#### `struct SlmProfile`
- Ort: `crates/contextra-router/src/profile.rs:47`
```rust
pub struct SlmProfile
```
Dokumentation: Repräsentiert das Profil eines Small Language Models (SLM) mit Token-Budget und Domänen-Zuständigkeiten.
- Impl-Block `crates/contextra-router/src/profile.rs:82`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `name` | `String` | `crates/contextra-router/src/profile.rs:49` |
| `mcp_endpoint` | `String` | `crates/contextra-router/src/profile.rs:51` |
| `domain_communities` | `BTreeSet<u64>` | `crates/contextra-router/src/profile.rs:54` |
| `token_budget` | `TokenBudget` | `crates/contextra-router/src/profile.rs:56` |
| `min_relevance_score` | `f32` | `crates/contextra-router/src/profile.rs:58` |
| `resource_cost_estimate` | `f32` | `crates/contextra-router/src/profile.rs:62` |
| `fingerprint` | `Option<ConfigFingerprint>` | `crates/contextra-router/src/profile.rs:67` |
| `transport` | `crate::transport::Transport` | `crates/contextra-router/src/profile.rs:73` |
| `bandit_state` | `Option<contextra_adapt::BanditProfileState>` | `crates/contextra-router/src/profile.rs:79` |

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new( name: impl Into<String>, mcp_endpoint: impl Into<String>, domain_communities: impl IntoIterator<Item = u64>, token_budget: TokenBudget, min_relevance_score: f32, ) -> Self` | `crates/contextra-router/src/profile.rs:93` | Erzeugt ein neues `SlmProfile` mit Standardwerten. # Ablauf 1. Belegung aller Pflichtfelder (Name, Endpunkt, Communities, Budget, Mindestscore). 2. Initialisier |
| `pub fn with_fingerprint(mut self, fingerprint: ConfigFingerprint) -> Self` | `crates/contextra-router/src/profile.rs:118` | Weist dem Profil einen Konfigurations-Fingerprint zu (P8-Pflicht). |
| `pub fn with_resource_cost_estimate(mut self, resource_cost_estimate: f32) -> Self` | `crates/contextra-router/src/profile.rs:126` | Weist dem Profil geschätzte Ressourcenkosten zu. |
| `pub fn estimated_cost(&self) -> f32` | `crates/contextra-router/src/profile.rs:135` | Liefert die geschätzten Kosten des Profils (Fallback auf `token_budget.limit`). |
| `pub fn validate(&self) -> Result<()>` | `crates/contextra-router/src/profile.rs:153` | Validiert alle Invarianten des Profils. # Invarianten - Name darf nicht leer sein. - MCP-Endpunkt darf nicht leer sein. - `min_relevance_score` muss in `[0.0, 1 |
| `pub fn try_new( name: impl Into<String>, mcp_endpoint: impl Into<String>, domain_communities: impl IntoIterator<Item = u64>, token_budget: TokenBudget, min_relevance_score: f32, ) -> Result<Self>` | `crates/contextra-router/src/profile.rs:180` | Erzeugt ein neues `SlmProfile` und validiert es sofort strikt. |

#### `enum QuantizationLevel`
- Ort: `crates/contextra-router/src/profile.rs:35`
```rust
pub enum QuantizationLevel
```
Dokumentation: Quantisierungsstufe des Modells (z. B. F16, Q8_0, Q4_K_M).

Varianten:

- `F16` — `crates/contextra-router/src/profile.rs:36`
- `Q8_0` — `crates/contextra-router/src/profile.rs:37`
- `Q4_K_M` — `crates/contextra-router/src/profile.rs:38`
- `Unknown` — `crates/contextra-router/src/profile.rs:40`

#### `type ConformalCalibrator`
- Ort: `crates/contextra-router/src/profile.rs:227`
```rust
pub type ConformalCalibrator = ProfileCalibrator
```
Dokumentation: Typ-Alias für `ProfileCalibrator` gemäß Spec-Namenskonvention.

### Modul `profile::serde_sorted_u64_set`

#### `fn deserialize`
- Ort: `crates/contextra-router/src/profile.rs:433`
```rust
pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<BTreeSet<u64>, D::Error>
```

#### `fn serialize`
- Ort: `crates/contextra-router/src/profile.rs:429`
```rust
pub fn serialize<S: Serializer>(set: &BTreeSet<u64>, s: S) -> Result<S::Ok, S::Error>
```

### Modul `router`

#### `struct ConfidenceMetrics`
- Ort: `crates/contextra-router/src/router/mod.rs:110`
```rust
pub struct ConfidenceMetrics
```
Dokumentation: Statistische Konfidenzmetriken einer Routing-Entscheidung (Conformal Prediction).

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `score_lower` | `Option<f32>` | `crates/contextra-router/src/router/mod.rs:112` |
| `score_upper` | `Option<f32>` | `crates/contextra-router/src/router/mod.rs:114` |
| `calibrated` | `bool` | `crates/contextra-router/src/router/mod.rs:116` |
| `quantile_threshold` | `f32` | `crates/contextra-router/src/router/mod.rs:118` |
| `non_conformity_score` | `f32` | `crates/contextra-router/src/router/mod.rs:120` |
| `selection_margin` | `f32` | `crates/contextra-router/src/router/mod.rs:122` |

#### `struct RouterEngine`
- Ort: `crates/contextra-router/src/router/mod.rs:158`
```rust
pub struct RouterEngine
```
Dokumentation: Die primäre Routing-Engine zur Evaluierung und Zuweisung von SLM-Profilen.
- Impl-Block `crates/contextra-router/src/router/dispatch_core.rs:27`
- Impl-Block `crates/contextra-router/src/router/lifecycle.rs:43`
- Impl-Block (Trait `LocalDriftStatusProvider`) `crates/contextra-router/src/router/mod.rs:318`
- Impl-Block `crates/contextra-router/src/router/outcomes.rs:19`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `search_provider` | `Arc<dyn HybridSearchProvider>` | `crates/contextra-router/src/router/mod.rs:159` |
| `community_resolver` | `Arc<dyn CommunityResolver>` | `crates/contextra-router/src/router/mod.rs:160` |
| `context_preparer` | `Arc<dyn ContextPreparer>` | `crates/contextra-router/src/router/mod.rs:161` |
| `clock` | `Arc<dyn Clock>` | `crates/contextra-router/src/router/mod.rs:162` |
| `state` | `ArcSwap<RouterState>` | `crates/contextra-router/src/router/mod.rs:163` |
| `decision_ids` | `DecisionIdGenerator` | `crates/contextra-router/src/router/mod.rs:165` |
| `routing_strategy` | `RoutingStrategy` | `crates/contextra-router/src/router/mod.rs:167` |
| `bandit_exploration` | `BanditExploration` | `crates/contextra-router/src/router/mod.rs:169` |

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub async fn route( &self, query_embedding: &[f32], query_text: &str, ) -> Result<RoutingDecision>` | `crates/contextra-router/src/router/dispatch_core.rs:254` | Führt eine vollständige Routing-Entscheidung für die übergebene Anfrage aus. # Ablauf 1. Suchergebnisse via `search_provider` abrufen und mit Graph-Communities  |
| `pub fn new( search_provider: Arc<dyn HybridSearchProvider>, community_resolver: Arc<dyn CommunityResolver>, context_preparer: Arc<dyn ContextPreparer>, profiles: Vec<SlmProfile>, calibration_store_path: Option<std::path::PathBuf>, ) -> Self` | `crates/contextra-router/src/router/lifecycle.rs:47` | Erzeugt eine neue `RouterEngine` mit Standardeinstellungen. |
| `pub fn new_with_vfs( search_provider: Arc<dyn HybridSearchProvider>, community_resolver: Arc<dyn CommunityResolver>, context_preparer: Arc<dyn ContextPreparer>, profiles: Vec<SlmProfile>, calibration_store_path: Option<std::path::PathBuf>, vfs: Option<Arc<dyn contextra_ports::vfs::Vfs>>, ) -> Self` | `crates/contextra-router/src/router/lifecycle.rs:79` | Erzeugt eine neue `RouterEngine` mit injiziertem VFS (M4: I/O-Entkopplung). # Ablauf 1. Kalibrierungszustand für alle Profile initialisieren. 2. Lyapunov-Wächte |
| `pub fn with_vfs(self, _vfs: Arc<dyn contextra_ports::vfs::Vfs>) -> Self` | `crates/contextra-router/src/router/lifecycle.rs:115` |  |
| `pub fn with_clock(mut self, clock: Arc<dyn Clock>) -> Self` | `crates/contextra-router/src/router/lifecycle.rs:122` | Injiziert eine benutzerdefinierte System-Uhr (Port-Entwurf für Tests). |
| `pub fn with_initial_decision_id(mut self, start: u64) -> Self` | `crates/contextra-router/src/router/lifecycle.rs:130` | Setzt den Startwert für monotone Entscheidungs-IDs. |
| `pub fn with_routing_strategy( mut self, strategy: RoutingStrategy, epsilon: f32, seed: u64, ) -> Self` | `crates/contextra-router/src/router/lifecycle.rs:139` | Konfiguriert die Routing-Strategie (Cascade oder Bandit). |
| `pub fn with_bandit_rng( mut self, strategy: RoutingStrategy, epsilon: f32, rng: Box<dyn contextra_adapt::Rng + Send>, ) -> Self` | `crates/contextra-router/src/router/lifecycle.rs:152` | Builder-Methode zur Konfiguration der Bandit-Exploration mit injiziertem Rng-Port (WP-R03). |
| `pub fn try_new( search_provider: Arc<dyn HybridSearchProvider>, community_resolver: Arc<dyn CommunityResolver>, context_preparer: Arc<dyn ContextPreparer>, profiles: Vec<SlmProfile>, calibration_store_path: Option<std::path::PathBuf>, ) -> Result<Self>` | `crates/contextra-router/src/router/lifecycle.rs:166` | Erzeugt eine neue `RouterEngine` und validiert alle übergebenen Profile. |
| `pub fn try_new_with_vfs( search_provider: Arc<dyn HybridSearchProvider>, community_resolver: Arc<dyn CommunityResolver>, context_preparer: Arc<dyn ContextPreparer>, profiles: Vec<SlmProfile>, calibration_store_path: Option<std::path::PathBuf>, vfs: Option<Arc<dyn contextra_ports::vfs::Vfs>>, ) -> Result<Self>` | `crates/contextra-router/src/router/lifecycle.rs:191` | Erzeugt eine neue `RouterEngine` mit injiziertem VFS und validiert alle Profile. # Fehler Gibt [`ContextraError`] zurück, wenn ein Profil die Validierung nicht  |
| `pub fn update_profiles(&self, new_profiles: Vec<SlmProfile>)` | `crates/contextra-router/src/router/lifecycle.rs:215` | Aktualisiert die Profile im laufenden Betrieb atomar (Hot-Reload via ArcSwap). |
| `pub fn try_update_profiles(&self, new_profiles: Vec<SlmProfile>) -> Result<()>` | `crates/contextra-router/src/router/lifecycle.rs:250` | Aktualisiert Profile atomar nach vorheriger strikter Validierung. |
| `pub fn profiles(&self) -> Vec<SlmProfile>` | `crates/contextra-router/src/router/lifecycle.rs:261` | Liefert eine Momentaufnahme der aktuell aktiven Profile. |
| `pub fn bandit_decision_propensity(&self, id: DecisionId) -> Option<f32>` | `crates/contextra-router/src/router/outcomes.rs:24` | Liefert die Aktionswahrscheinlichkeit (Propensity) einer offenen Bandit-Entscheidung. |
| `pub fn calibration_stats(&self) -> BTreeMap<String, ProfileCalibrationState>` | `crates/contextra-router/src/router/outcomes.rs:31` | Liefert eine Momentaufnahme aller aktuellen Profil-Kalibrierungen. |
| `pub fn reset_calibration(&self, profile_name: &str)` | `crates/contextra-router/src/router/outcomes.rs:38` | Setzt die Kalibrierungsstatistik für ein einzelnes Profil zurück. |
| `pub fn drift_status(&self, profile_name: &str) -> Option<LyapunovResult>` | `crates/contextra-router/src/router/outcomes.rs:52` | Fragt den Ljapunow-Drift-Status für ein bestimmtes Profil ab. |
| `pub fn overall_drift_status(&self) -> String` | `crates/contextra-router/src/router/outcomes.rs:63` | Liefert eine textuelle Gesamtübersicht des System-Drifts. |
| `pub fn set_lyapunov_baseline(&self, profile_name: &str, baseline: &[f32]) -> bool` | `crates/contextra-router/src/router/outcomes.rs:90` | Setzt die Referenz-Baseline für die Drift-Erkennung eines Profils. |
| `pub fn record_outcome(&self, decision_id: DecisionId, outcome: RoutingOutcome) -> bool` | `crates/contextra-router/src/router/outcomes.rs:127` | Protokolliert das Ergebnis einer Routing-Entscheidung und aktualisiert Kalibrierung und Bandit-Statistiken. |
| `pub fn pending_decision_count(&self) -> usize` | `crates/contextra-router/src/router/outcomes.rs:209` | Liefert die Anzahl offener Entscheidungen, die auf Feedback warten. |
| `pub fn reset_all_calibration(&self)` | `crates/contextra-router/src/router/outcomes.rs:216` | Setzt alle Kalibrierungen systemweit auf ihre Initialwerte zurück. |

#### `struct RouterState`
- Ort: `crates/contextra-router/src/router/mod.rs:146`
```rust
pub struct RouterState
```
Dokumentation: Unveränderlicher Snapshot des Router-Zustands für atomares Swapping via `ArcSwap`.

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `profiles` | `Vec<SlmProfile>` | `crates/contextra-router/src/router/mod.rs:147` |

#### `struct RoutingDecision`
- Ort: `crates/contextra-router/src/router/mod.rs:129`
```rust
pub struct RoutingDecision
```
Dokumentation: Vollständige Entscheidung des Routers mit ausgewähltem Modellprofil, Kontextfenster und Metriken.

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `profile` | `SlmProfile` | `crates/contextra-router/src/router/mod.rs:131` |
| `context` | `ContextWindow` | `crates/contextra-router/src/router/mod.rs:133` |
| `confidence` | `Option<ConfidenceMetrics>` | `crates/contextra-router/src/router/mod.rs:135` |
| `decision_id` | `DecisionId` | `crates/contextra-router/src/router/mod.rs:137` |
| `drift_status` | `Option<LyapunovResult>` | `crates/contextra-router/src/router/mod.rs:139` |

#### `type DefaultRouterEngine`
- Ort: `crates/contextra-router/src/router/mod.rs:154`
```rust
pub type DefaultRouterEngine = RouterEngine
```
Dokumentation: Standard-Typalias für die Routing-Engine.

### Modul `routing_strategy`

#### `enum RoutingStrategy`
- Ort: `crates/contextra-router/src/routing_strategy.rs:27`
```rust
pub enum RoutingStrategy
```
Dokumentation: Auswahl-Strategie für SLM-Profil-Routing. Steuert, wie die `RouterEngine` unter den verfügbaren Profilen das am besten geeignete Modell bestimmt. # Invariante (§13.3) `ContextualBandit` bleibt Default-off bis `test_bandit_vs_cascade_regret_comparison` messbaren Regret-Vorteil zeigt (P7-Nachweis-Pflicht).

Varianten:

- `Cascade` — `crates/contextra-router/src/routing_strategy.rs:31`
- `ContextualBandit` — `crates/contextra-router/src/routing_strategy.rs:35`
- `FlowCorrectedThompson` — `crates/contextra-router/src/routing_strategy.rs:39`

### Modul `seam`

#### `struct NoCalibrationSource`
- Ort: `crates/contextra-router/src/seam.rs:28`
```rust
pub struct NoCalibrationSource
```
Dokumentation: Deaktiviertes Laden von Kalibrierungsdaten (No-Op).
- Impl-Block (Trait `CalibrationSource`) `crates/contextra-router/src/seam.rs:30`

#### `struct StdFsCalibrationSource`
- Ort: `crates/contextra-router/src/seam.rs:41`
```rust
pub struct StdFsCalibrationSource
```
Dokumentation: Standard-Dateisystem-Implementierung über `std::fs`. Dies ist der einzige Ort im Crate, an dem synchrones `std::fs::read` verwendet wird.
- Impl-Block (Trait `CalibrationSource`) `crates/contextra-router/src/seam.rs:43`

#### `struct VfsCalibrationSource`
- Ort: `crates/contextra-router/src/seam.rs:52`
```rust
pub struct VfsCalibrationSource
```
Dokumentation: Wrapper für einen virtuellen Dateisystem-Port ([`contextra_ports::vfs::Vfs`]).
- Impl-Block `crates/contextra-router/src/seam.rs:56`
- Impl-Block (Trait `CalibrationSource`) `crates/contextra-router/src/seam.rs:63`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new(vfs: Arc<dyn contextra_ports::vfs::Vfs>) -> Self` | `crates/contextra-router/src/seam.rs:58` | Erzeugt eine neue Kalibrierungsquelle aus einem VFS-Port. |

#### `trait CalibrationSource`
- Ort: `crates/contextra-router/src/seam.rs:17`
```rust
pub trait CalibrationSource: Send + Sync
```
Dokumentation: Trait zur Abstraktion des Lesens persistierter Kalibrierungsdaten.

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `fn read_calibration(&self, path: &Path) -> Option<Vec<u8>>` | `crates/contextra-router/src/seam.rs:22` | Liest rohe Kalibrierungsbytes von einem gegebenen Pfad. Gibt `None` zurück, wenn die Datei nicht existiert oder nicht lesbar ist. Darf niemals panizieren. |

### Modul `serde_helpers`

#### `mod sorted_u64_set`
- Ort: `crates/contextra-router/src/serde_helpers.rs:18`
```rust
pub mod sorted_u64_set
```
Dokumentation: Modul zur deterministischen Serialisierung von `BTreeSet<u64>` als sortierter Vektor.

### Modul `serde_helpers::sorted_u64_set`

#### `fn deserialize`
- Ort: `crates/contextra-router/src/serde_helpers.rs:64`
```rust
pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<BTreeSet<u64>, D::Error>
```
Dokumentation: Deserialisiert eine Sequenz von `u64`-Werten in ein geordnetes `BTreeSet<u64>`. # Ablauf 1. Sequenz aus dem Deserializer als `Vec<u64>` einlesen. 2. Vektor in ein `BTreeSet` überführen (Duplikate werden automatisch dedupliziert). # Vorbedingungen Der Deserializer liefert eine gültige Sequenz von ganzzahligen 64-Bit-Werten. # Nachbedingungen Liefert ein vollständiges `BTreeSet<u64>` zurück. # Fehler Gibt `D::Error` zurück, falls das Eingabeformat ungültig ist. # Nebenwirkungen Keine.

#### `fn serialize`
- Ort: `crates/contextra-router/src/serde_helpers.rs:41`
```rust
pub fn serialize<S: Serializer>(set: &BTreeSet<u64>, s: S) -> Result<S::Ok, S::Error>
```

### Modul `tests::fixtures`

#### `struct CollectionAdapter`
- Ort: `crates/contextra-router/src/tests/fixtures.rs:180`
```rust
pub struct CollectionAdapter
```
- Impl-Block `crates/contextra-router/src/tests/fixtures.rs:184`
- Impl-Block (Trait `HybridSearchProvider`) `crates/contextra-router/src/tests/fixtures.rs:190`
- Impl-Block (Trait `CommunityResolver`) `crates/contextra-router/src/tests/fixtures.rs:202`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `collection` | `Arc<TestCollection>` | `crates/contextra-router/src/tests/fixtures.rs:181` |

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new(collection: Arc<TestCollection>) -> Self` | `crates/contextra-router/src/tests/fixtures.rs:185` |  |

#### `struct Contextra`
- Ort: `crates/contextra-router/src/tests/fixtures.rs:290`
```rust
pub struct Contextra
```
- Impl-Block `crates/contextra-router/src/tests/fixtures.rs:294`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `collection` | `Arc<TestCollection>` | `crates/contextra-router/src/tests/fixtures.rs:291` |

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub async fn open_with_config( _path: impl AsRef<std::path::Path>, config: ContextraConfig, ) -> Result<Self>` | `crates/contextra-router/src/tests/fixtures.rs:295` |  |
| `pub async fn collection(&self, _name: &str) -> Result<Arc<TestCollection>>` | `crates/contextra-router/src/tests/fixtures.rs:304` |  |
| `pub fn allocate_tx(&self) -> Result<TxId>` | `crates/contextra-router/src/tests/fixtures.rs:308` |  |
| `pub fn inner_storage(&self) -> TestInnerStorage` | `crates/contextra-router/src/tests/fixtures.rs:312` |  |

#### `struct ContextraConfig`
- Ort: `crates/contextra-router/src/tests/fixtures.rs:276`
```rust
pub struct ContextraConfig
```
- Impl-Block (Trait `Default`) `crates/contextra-router/src/tests/fixtures.rs:281`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `dimension` | `usize` | `crates/contextra-router/src/tests/fixtures.rs:277` |
| `_reserved` | `u64` | `crates/contextra-router/src/tests/fixtures.rs:278` |

#### `struct MockDoc`
- Ort: `crates/contextra-router/src/tests/fixtures.rs:51`
```rust
pub struct MockDoc
```

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `entity_id` | `Option<EntityId>` | `crates/contextra-router/src/tests/fixtures.rs:52` |
| `key` | `String` | `crates/contextra-router/src/tests/fixtures.rs:53` |
| `vector` | `Vec<f32>` | `crates/contextra-router/src/tests/fixtures.rs:54` |
| `text` | `String` | `crates/contextra-router/src/tests/fixtures.rs:55` |

#### `struct TestClock`
- Ort: `crates/contextra-router/src/tests/fixtures.rs:22`
```rust
pub struct TestClock
```
- Impl-Block `crates/contextra-router/src/tests/fixtures.rs:26`
- Impl-Block (Trait `Clock`) `crates/contextra-router/src/tests/fixtures.rs:40`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `nanos` | `AtomicU64` | `crates/contextra-router/src/tests/fixtures.rs:23` |

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new(initial_nanos: u64) -> Self` | `crates/contextra-router/src/tests/fixtures.rs:27` |  |
| `pub fn advance(&self, duration: std::time::Duration)` | `crates/contextra-router/src/tests/fixtures.rs:34` |  |

#### `struct TestCollection`
- Ort: `crates/contextra-router/src/tests/fixtures.rs:60`
```rust
pub struct TestCollection
```
- Impl-Block `crates/contextra-router/src/tests/fixtures.rs:68`
- Impl-Block (Trait `HybridSearchProvider`) `crates/contextra-router/src/tests/fixtures.rs:106`
- Impl-Block (Trait `CommunityResolver`) `crates/contextra-router/src/tests/fixtures.rs:171`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `dimension` | `usize` | `crates/contextra-router/src/tests/fixtures.rs:61` |
| `docs` | `RwLock<Vec<MockDoc>>` | `crates/contextra-router/src/tests/fixtures.rs:62` |
| `next_doc_id` | `AtomicU64` | `crates/contextra-router/src/tests/fixtures.rs:65` |

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new(dimension: usize) -> Self` | `crates/contextra-router/src/tests/fixtures.rs:69` |  |
| `pub async fn insert(&self, key: &str, vector: &[f32], metadata: Option<Value>) -> Result<()>` | `crates/contextra-router/src/tests/fixtures.rs:79` |  |
| `pub fn set_community(&self, entity_id: EntityId, comm_id: u64)` | `crates/contextra-router/src/tests/fixtures.rs:97` |  |
| `pub async fn relate(&self, _key_a: &str, _key_b: &str, _relation: &str) -> Result<()>` | `crates/contextra-router/src/tests/fixtures.rs:101` |  |

#### `struct TestContextPreparer`
- Ort: `crates/contextra-router/src/tests/fixtures.rs:208`
```rust
pub struct TestContextPreparer
```
- Impl-Block (Trait `ContextPreparer`) `crates/contextra-router/src/tests/fixtures.rs:210`

#### `struct TestInnerStorage`
- Ort: `crates/contextra-router/src/tests/fixtures.rs:246`
```rust
pub struct TestInnerStorage
```
- Impl-Block `crates/contextra-router/src/tests/fixtures.rs:250`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `collection` | `Arc<TestCollection>` | `crates/contextra-router/src/tests/fixtures.rs:247` |

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub async fn put(&self, _tx: TxId, key: &[u8], val: &[u8]) -> Result<()>` | `crates/contextra-router/src/tests/fixtures.rs:251` |  |
| `pub async fn commit(&self, _tx: TxId) -> Result<()>` | `crates/contextra-router/src/tests/fixtures.rs:271` |  |

#### `fn create_test_router`
- Ort: `crates/contextra-router/src/tests/fixtures.rs:319`
```rust
pub fn create_test_router( collection: Arc<TestCollection>, profiles: Vec<SlmProfile>, calibration_store_path: Option<std::path::PathBuf>, ) -> RouterEngine
```

#### `fn try_create_test_router`
- Ort: `crates/contextra-router/src/tests/fixtures.rs:336`
```rust
pub fn try_create_test_router( collection: Arc<TestCollection>, profiles: Vec<SlmProfile>, calibration_store_path: Option<std::path::PathBuf>, ) -> Result<RouterEngine>
```

### Modul `transport`

#### `enum Transport`
- Ort: `crates/contextra-router/src/transport.rs:29`
```rust
pub enum Transport
```
Dokumentation: Transport-Kanal für SLM-Routing-Entscheidungen. Definiert, über welches Medium das Sprachmodell (SLM = Small Language Model) angesprochen wird. # Invariante `HttpCloud` darf nur mit aktivem Feature `cloud-egress-guard` und einem `GuardedPayload<Sanitized>` verwendet werden — Compile-Zeit-Schutz via `dispatch_to_cloud()` Signatur.
- Impl-Block `crates/contextra-router/src/transport.rs:37`
- Impl-Block (Trait `Default`) `crates/contextra-router/src/transport.rs:62`

Varianten:

- `StdioMcp` — `crates/contextra-router/src/transport.rs:31`
- `HttpCloud{ url: String }` — `crates/contextra-router/src/transport.rs:34`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn is_cloud(&self) -> bool` | `crates/contextra-router/src/transport.rs:53` | Prüft, ob dieser Transport einen externen Cloud-Endpunkt repräsentiert. # Ablauf 1. Pattern-Matching auf die Transport-Variante. 2. `false` für `StdioMcp`, `tru |

## 9. Impl-Blöcke ohne Typ im selben Crate

- `impl CalibrationSource for Arc<dyn contextra_ports::vfs::Vfs>` — `crates/contextra-router/src/seam.rs:70` (0 pub Methoden)
