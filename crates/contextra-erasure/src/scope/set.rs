//! Auflösung von Löschumfängen und Scope-Mengen (SPEC §11.1, ER-17).
//!
//! Ein `LayerScopeSet` sammelt die Scope-Auflösungen aller Schichten für eine Löschung.
//! Bei kleinem Umfang wird er direkt in den Journal-Eintrag eingebettet (kein zusätzlicher fsync).

use serde::{Deserialize, Serialize};

/// Kennung einer Schicht (für Scope-Zuordnung).
pub type ScopeLayerId = [u8; 16];

/// Auflösung einer einzelnen Schicht für eine Löschung (ER-17).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ResolvedLayerScope {
    /// Schicht-ID.
    pub layer_id: ScopeLayerId,
    /// Anzahl betroffener Einheiten (Dateien, Segmente, etc.).
    pub unit_count: u64,
    /// Opake Einheitsliste (Schicht-spezifisch).
    pub units: Vec<u64>,
}

impl ResolvedLayerScope {
    /// Erstellt einen leeren Scope für eine Schicht.
    pub fn empty(layer_id: ScopeLayerId) -> Self {
        Self {
            layer_id,
            unit_count: 0,
            units: Vec::new(),
        }
    }

    /// Erstellt einen Scope mit Einheiten.
    pub fn with_units(layer_id: ScopeLayerId, units: Vec<u64>) -> Self {
        let count = units.len() as u64;
        Self {
            layer_id,
            unit_count: count,
            units,
        }
    }

    /// Prüft ob der Scope klein genug ist um eingebettet zu werden (ER-17).
    ///
    /// Schwellwert: ≤ 16 Einheiten → Einbettung in Journal-Eintrag.
    pub fn is_embeddable(&self) -> bool {
        self.unit_count <= 16
    }
}

/// Gesamtmenge der Scope-Auflösungen aller Schichten für eine Löschung.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct LayerScopeSet {
    /// Scopes je Schicht.
    pub scopes: Vec<ResolvedLayerScope>,
}

impl LayerScopeSet {
    /// Erstellt eine leere Scope-Menge.
    pub fn new() -> Self {
        Self { scopes: Vec::new() }
    }

    /// Fügt einen Scope hinzu.
    pub fn push(&mut self, scope: ResolvedLayerScope) {
        self.scopes.push(scope);
    }

    /// Gesamtanzahl betroffener Einheiten über alle Schichten.
    pub fn total_units(&self) -> u64 {
        self.scopes.iter().map(|s| s.unit_count).sum()
    }

    /// Prüft ob der gesamte Scope einbettbar ist (ER-17).
    pub fn is_embeddable(&self) -> bool {
        self.scopes.iter().all(|s| s.is_embeddable())
    }
}
