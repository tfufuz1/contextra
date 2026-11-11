# Äquivalenzprüfung Regler / Adaptive Decay Controller (S-60)

- **Untersuchte Module:**
  - `crates/contextra-engine/src/decay_controller.rs`
  - `crates/contextra-adapt/src/decay_controller.rs`
  - `crates/contextra-engine/tests/legacy_reference/decay_controller_legacy.rs`
- **Test-Suite:** `crates/contextra-engine/tests/proof_regulators_vs_adapt.rs` (`#![cfg(feature = "adaptive-candidate-pool-sizing")]`)
- **Ergebnis Testlauf:** 2 passed; 0 failed (inkl. 10.000 Proptest-Fälle f32-Bitgleichheit)

## Methoden- und Feldparität

| Element | In `contextra-engine` | In `contextra-adapt` | Differenz (f32-Bitgleich) | Status |
|---|---|---|---|---|
| `DecaySignalInputs` | Ja (`tombstone_ratio`, `query_load_inverse`, `w_tombstone`, `w_query`) | Ja (identische Felder) | Keine | Identisch |
| `DecayControllerConfig` | Ja (`kappa`, `base_half_life_tx`, `eviction_threshold`) | Ja (identische Felder) | Keine | Identisch |
| `AdaptiveDecayController::new` | Ja | Ja | Keine | Identisch |
| `AdaptiveDecayController::with_defaults` | Ja | Ja | Keine | Identisch |
| `AdaptiveDecayController::system_temperature` | Ja | Ja | Keine (`to_bits()` identisch) | Bitgleich |
| `AdaptiveDecayController::effective_half_life` | Ja | Ja | Keine (`to_bits()` identisch) | Bitgleich |
| `AdaptiveDecayController::effective_score` | Ja | Ja | Keine (`to_bits()` identisch) | Bitgleich |
| `AdaptiveDecayController::should_evict` | Ja | Ja | Keine | Identisch |
| `AdaptiveDecayController::with_clock` | Ja (Injektion `Clock`) | **Nein** (fehlt in `adapt`) | n/a | Übergabe an `adapt` |
| `AdaptiveDecayController::set_clock` | Ja | **Nein** (fehlt in `adapt`) | n/a | Übergabe an `adapt` |
| `AdaptiveDecayController::with_base_half_life_ms` | Ja | **Nein** (fehlt in `adapt`) | n/a | Übergabe an `adapt` |
| `AdaptiveDecayController::effective_half_life_ms` | Ja | **Nein** (fehlt in `adapt`) | n/a | Übergabe an `adapt` |
| `AdaptiveDecayController::effective_score_wall_clock` | Ja | **Nein** (fehlt in `adapt`) | n/a | Übergabe an `adapt` |
| `AdaptiveDecayController::should_evict_wall_clock` | Ja | **Nein** (fehlt in `adapt`) | n/a | Übergabe an `adapt` |

## Gründe gegen eine sofortige Ablösung (Verbleib als Engine-Modul)
1. **Optionale Abhängigkeit:** `contextra-adapt` ist in `contextra-engine` nur unter dem optionalen Feature `adaptive-candidate-pool-sizing` eingebunden (`Cargo.toml`). Eine Ersetzung durch Re-Export würde `AdaptiveDecayController` in Standard-Builds (`default = []`) entfernen und damit Aufrufer in `src/collection/maintenance.rs`, `src/background_workers/expiry_workers.rs` und Fremdnutzer (`contextra-db`) brechen.
2. **Fehlende Methoden in `contextra-adapt`:**
   In `adapt` fehlen Clock-Injektion und Wall-Clock-Decay (`with_clock`, `set_clock`, `with_base_half_life_ms`, `effective_half_life_ms`, `effective_score_wall_clock`, `should_evict_wall_clock`).

## Übergaben an den Maintainer / `adapt`-Eigner
- **Übergabe 1 (Methoden für `contextra-adapt`):**
  - `pub fn with_clock(config: DecayControllerConfig, clock: Arc<dyn Clock>) -> Self`
  - `pub fn set_clock(&mut self, clock: Arc<dyn Clock>)`
  - `pub fn with_base_half_life_ms(mut self, base_ms: u64) -> Self`
  - `pub fn effective_half_life_ms(&self, temperature: f32) -> f32`
  - `pub fn effective_score_wall_clock(&self, base_score: f32, created_at_ms: u64, inputs: &DecaySignalInputs) -> f32`
  - `pub fn should_evict_wall_clock(&self, base_score: f32, created_at_ms: u64, inputs: &DecaySignalInputs) -> bool`
- **Übergabe 2 (Manifest-Entscheidung durch ENG-A):**
  - Sobald `contextra-adapt` diese Methoden anbietet und als normale Abhängigkeit in `crates/contextra-engine/Cargo.toml` deklariert wird, kann `src/decay_controller.rs` als Re-Export abgelöst werden.
