//! # Modul: decay.rs
//!
//! **Zweck:** Lokale Implementierung des Adaptive Decay Controllers für contextra-cognition (SPEC §6.4, S-63).
//!
//! **Architektur:** Gehört zur Crate `contextra-cognition` (Ebene E2).
//! Eingehaltene Invarianten: Deterministische TxId-Rechnung (INV-5), Zero-Panic (INV-12).

#![forbid(unsafe_code)]

use contextra_ports::Clock;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// Inputs für den Adaptive Decay Controller.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DecaySignalInputs {
    /// Anteil von Tombstones im Index (0.0..1.0).
    pub tombstone_ratio: f32,
    /// Inverse Abfragelast (1.0 = idle, 0.0 = überlastet).
    pub query_load_inverse: f32,
    /// Gewichtung für Tombstones.
    pub w_tombstone: f32,
    /// Gewichtung für Abfragelast.
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
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DecayControllerConfig {
    /// Sensitivitätskoeffizient für Temperatur.
    pub kappa: f32,
    /// Basis-Halbwertszeit in Transaktionszyklen.
    pub base_half_life_tx: u64,
    /// Eviktionsschwelle.
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

/// Adaptive Decay Controller für time-weighted Cache-Eviction mit injizierter Clock.
#[derive(Clone)]
pub struct AdaptiveDecayController {
    config: DecayControllerConfig,
    clock: Option<Arc<dyn Clock>>,
    base_half_life_ms: Option<u64>,
}

impl std::fmt::Debug for AdaptiveDecayController {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AdaptiveDecayController")
            .field("config", &self.config)
            .field("has_clock", &self.clock.is_some())
            .finish()
    }
}

impl AdaptiveDecayController {
    /// Erstellt eine neue Instanz des `AdaptiveDecayController` mit der gegebenen Konfiguration.
    pub fn new(config: DecayControllerConfig) -> Self {
        Self {
            config,
            clock: None,
            base_half_life_ms: None,
        }
    }

    /// Erstellt eine neue Instanz des `AdaptiveDecayController` mit injizierter Clock.
    pub fn with_clock(config: DecayControllerConfig, clock: Arc<dyn Clock>) -> Self {
        Self {
            config,
            clock: Some(clock),
            base_half_life_ms: None,
        }
    }

    /// Setzt die Basis-Halbwertszeit in Millisekunden für Wall-Clock-Decay.
    pub fn with_base_half_life_ms(mut self, base_ms: u64) -> Self {
        self.base_half_life_ms = Some(base_ms);
        self
    }

    /// Initialisiert den Controller mit den Standardwerten der Spezifikation.
    pub fn with_defaults() -> Self {
        Self::new(DecayControllerConfig::default())
    }

    /// Setzt oder aktualisiert die injizierte Clock.
    pub fn set_clock(&mut self, clock: Arc<dyn Clock>) {
        self.clock = Some(clock);
    }

    /// Konfiguration des Controllers abrufen.
    pub fn config(&self) -> &DecayControllerConfig {
        &self.config
    }

    /// Berechnet die aktuelle Systemtemperatur basierend auf Tombstone-Verhältnis und Abfragelast.
    #[inline]
    pub fn system_temperature(&self, inputs: &DecaySignalInputs) -> f32 {
        (inputs.w_tombstone * inputs.tombstone_ratio + inputs.w_query * inputs.query_load_inverse)
            .clamp(0.0, 1.0)
    }

    /// Berechnet die temperaturangepasste effektive Halbwertszeit in Transaktionszyklen.
    #[inline]
    pub fn effective_half_life(&self, temperature: f32) -> f32 {
        self.config.base_half_life_tx as f32 * (1.0 + self.config.kappa * (1.0 - temperature))
    }

    /// Berechnet die temperaturangepasste effektive Halbwertszeit in Millisekunden.
    #[inline]
    pub fn effective_half_life_ms(&self, temperature: f32) -> f32 {
        let base = self.base_half_life_ms.unwrap_or(86_400_000) as f32;
        base * (1.0 + self.config.kappa * (1.0 - temperature))
    }

    /// Berechnet den aktuellen abgewerteten Score eines Objekts unter Berücksichtigung von Alter und Systemtemperatur.
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

    /// Berechnet den abgewerteten Score basierend auf der Zeitdifferenz zur injizierten Clock.
    pub fn effective_score_wall_clock(
        &self,
        base_score: f32,
        created_at_ms: u64,
        inputs: &DecaySignalInputs,
    ) -> f32 {
        let now_ms = if let Some(ref clock) = self.clock {
            clock.now_unix_nanos() / 1_000_000
        } else {
            created_at_ms
        };
        let elapsed_ms = now_ms.saturating_sub(created_at_ms);
        let temperature = self.system_temperature(inputs);
        let half_life = self.effective_half_life_ms(temperature);
        let decay_exponent = -(std::f32::consts::LN_2 / half_life) * elapsed_ms as f32;
        base_score * decay_exponent.exp().max(0.0)
    }

    /// Prüft, ob ein Eintrag aufgrund seines verfallenen Scores unter den Eviktions-Schwellenwert gefallen ist.
    #[inline]
    pub fn should_evict(
        &self,
        base_score: f32,
        elapsed_tx: u64,
        inputs: &DecaySignalInputs,
    ) -> bool {
        self.effective_score(base_score, elapsed_tx, inputs) < self.config.eviction_threshold
    }

    /// Prüft, ob ein Eintrag unter Verwendung der injizierten Clock eviktiert werden soll.
    #[inline]
    pub fn should_evict_wall_clock(
        &self,
        base_score: f32,
        created_at_ms: u64,
        inputs: &DecaySignalInputs,
    ) -> bool {
        self.effective_score_wall_clock(base_score, created_at_ms, inputs)
            < self.config.eviction_threshold
    }
}
