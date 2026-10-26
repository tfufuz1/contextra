// FILE-CONTEXT
// ZWECK: Adaptive Decay Controller — WP 2.2: Typen aus contextra-adapt hierher kopiert (archiviert).
// INVARIANTEN: Identische Logik wie contextra-adapt/src/decay_controller.rs (ADR-016).
// NICHT-OFFENSICHTLICH: Kein SystemTime — elapsed_tx = TxId-Differenz (deterministisch).

//! Adaptive Decay Controller (Cache-Eviction mit Time-Weighted Decay) (F-01).
//!
//! Kopiert aus contextra-adapt (archiviert in WP 2.1), um die Kante zu entfernen.
//! ADR-016: TxId-Differenz statt SystemTime (deterministisch, replay-safe).

/// Inputs für den Adaptive Decay Controller.
#[derive(Debug, Clone, Copy)]
pub struct DecaySignalInputs {
    pub tombstone_ratio: f32,
    pub query_load_inverse: f32,
    pub w_tombstone: f32,
    pub w_query: f32,
}

impl Default for DecaySignalInputs {
    fn default() -> Self {
        Self {
            tombstone_ratio: 0.0,
            query_load_inverse: 0.5,
            w_tombstone: 0.6,
            w_query: 0.4,
        }
    }
}

/// Konfiguration für den Adaptive Decay Controller.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DecayControllerConfig {
    pub kappa: f32,
    pub base_half_life_tx: u64,
    pub eviction_threshold: f32,
}

impl Default for DecayControllerConfig {
    fn default() -> Self {
        Self {
            kappa: 2.0,
            base_half_life_tx: 10_000,
            eviction_threshold: 0.01,
        }
    }
}

/// Adaptive Decay Controller für time-weighted Cache-Eviction.
#[derive(Debug, Clone)]
pub struct AdaptiveDecayController {
    config: DecayControllerConfig,
}

impl AdaptiveDecayController {
    pub fn new(config: DecayControllerConfig) -> Self {
        Self { config }
    }

    pub fn with_defaults() -> Self {
        Self::new(DecayControllerConfig::default())
    }

    #[inline]
    pub fn system_temperature(&self, inputs: &DecaySignalInputs) -> f32 {
        (inputs.w_tombstone * inputs.tombstone_ratio + inputs.w_query * inputs.query_load_inverse)
            .clamp(0.0, 1.0)
    }

    #[inline]
    pub fn effective_half_life(&self, temperature: f32) -> f32 {
        self.config.base_half_life_tx as f32 * (1.0 + self.config.kappa * (1.0 - temperature))
    }

    pub fn effective_score(
        &self,
        base_score: f32,
        elapsed_tx: u64,
        inputs: &DecaySignalInputs,
    ) -> f32 {
        let temperature = self.system_temperature(inputs);
        let half_life = self.effective_half_life(temperature);
        let decay_exponent = -(std::f32::consts::LN_2 / half_life) * elapsed_tx as f32;
        base_score * decay_exponent.exp().max(0.0)
    }

    #[inline]
    pub fn should_evict(&self, base_score: f32, elapsed_tx: u64, inputs: &DecaySignalInputs) -> bool {
        self.effective_score(base_score, elapsed_tx, inputs) < self.config.eviction_threshold
    }
}
