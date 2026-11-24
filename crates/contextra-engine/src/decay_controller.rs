//! # Modul: decay_controller.rs
//!
//! **Zweck:** Decay-Controller zur zeitlichen Abwertung und Bereinigung von Wissenselementen.
//!
//! **Architektur:** Gehört zur Crate `contextra-engine` (Ring 2).
//! Eingehaltene Invarianten: Fail-Closed bei Fehlern, kein ungeschützter globaler Zustand.

// FILE-CONTEXT
// ZWECK: Adaptive Decay Controller — WP 2.2: Typen aus contextra-adapt hierher kopiert (archiviert).
// INVARIANTEN: Identische Logik wie contextra-adapt/src/decay_controller.rs (ADR-016).
// NICHT-OFFENSICHTLICH: Kein SystemTime — elapsed_tx = TxId-Differenz (deterministisch).

//! Adaptive Decay Controller (Cache-Eviction mit Time-Weighted Decay) (F-01).
//!
//! Kopiert aus contextra-adapt (archiviert in WP 2.1), um die Kante zu entfernen.
//! ADR-016: TxId-Differenz statt SystemTime (deterministisch, replay-safe).

use contextra_ports::Clock;
use std::sync::Arc;

/// Inputs für den Adaptive Decay Controller.
#[derive(Debug, Clone, Copy)]
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
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DecayControllerConfig {
    /// Sensitivitätskoeffizient für Temperatur.
    pub kappa: f32,
    /// Basis-Halbwertszeit in Transaktionszyklen.
    pub base_half_life_tx: u64,
    /// Basis-Halbwertszeit in Millisekunden für Wall-Clock-Decay.
    pub base_half_life_ms: Option<u64>,
    /// Eviktionsschwelle.
    pub eviction_threshold: f32,
}

impl Default for DecayControllerConfig {
    fn default() -> Self {
        Self {
            kappa: 2.0,
            base_half_life_tx: 10_000,
            base_half_life_ms: Some(86_400_000), // 24h
            eviction_threshold: 0.01,
        }
    }
}

/// Adaptive Decay Controller für time-weighted Cache-Eviction mit injizierter Clock.
#[derive(Clone)]
pub struct AdaptiveDecayController {
    config: DecayControllerConfig,
    clock: Option<Arc<dyn Clock>>,
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
    // ANCHOR[contextra-engine:DEC-001]
    /// Erstellt eine neue Instanz des `AdaptiveDecayController` mit der gegebenen Konfiguration.
    pub fn new(config: DecayControllerConfig) -> Self {
        Self {
            config,
            clock: None,
        }
    }

    /// Erstellt eine neue Instanz des `AdaptiveDecayController` mit injizierter Clock (B11).
    pub fn with_clock(config: DecayControllerConfig, clock: Arc<dyn Clock>) -> Self {
        Self {
            config,
            clock: Some(clock),
        }
    }

    // ANCHOR[contextra-engine:DEC-002]
    /// Initialisiert den Controller mit den Standardwerten der Spezifikation.
    pub fn with_defaults() -> Self {
        Self::new(DecayControllerConfig::default())
    }

    /// Setzt oder aktualisiert die injizierte Clock.
    pub fn set_clock(&mut self, clock: Arc<dyn Clock>) {
        self.clock = Some(clock);
    }

    // ANCHOR[contextra-engine:DEC-003]
    /// Berechnet die aktuelle Systemtemperatur basierend auf Tombstone-Verhältnis und Abfragelast.
    #[inline]
    pub fn system_temperature(&self, inputs: &DecaySignalInputs) -> f32 {
        (inputs.w_tombstone * inputs.tombstone_ratio + inputs.w_query * inputs.query_load_inverse)
            .clamp(0.0, 1.0)
    }

    // ANCHOR[contextra-engine:DEC-004]
    /// Berechnet die temperaturangepasste effektive Halbwertszeit in Transaktionszyklen.
    #[inline]
    pub fn effective_half_life(&self, temperature: f32) -> f32 {
        self.config.base_half_life_tx as f32 * (1.0 + self.config.kappa * (1.0 - temperature))
    }

    /// Berechnet die temperaturangepasste effektive Halbwertszeit in Millisekunden.
    #[inline]
    pub fn effective_half_life_ms(&self, temperature: f32) -> f32 {
        let base = self.config.base_half_life_ms.unwrap_or(86_400_000) as f32;
        base * (1.0 + self.config.kappa * (1.0 - temperature))
    }

    // ANCHOR[contextra-engine:DEC-005]
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

    // ANCHOR[contextra-engine:DEC-006]
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
