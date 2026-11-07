//! Unveränderlicher LayerWitness für gemessene Restbestände nach einer Bereinigung (SPEC §12.7.4, INV-WITNESS-MINT-1).
//!
//! # Invarianten
//! - **INV-WITNESS-MINT-1**: Der `LayerWitness` hat keinen öffentlichen Konstruktor außerhalb von
//!   `ScanSession::finish`. Weder `Default`, `Deserialize` noch ungeschützte Fabrikmethoden sind vorhanden.

#![forbid(unsafe_code)]

use crate::layer::{ScanTechnique, WitnessLayer};
use serde::Serialize;

/// Verifiziertes Messergebnis eines Storage-Scanners (SPEC §12.7.4).
///
/// Unter INV-WITNESS-MINT-1 kann ein `LayerWitness` ausschließlich in `ScanSession::finish`
/// entstehen. Ein direkter Konstruktor oder `Default` ist nicht vorhanden.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct LayerWitness {
    layer: WitnessLayer,
    scanned: u64,
    remaining: u64,
    technique: ScanTechnique,
    digest: [u8; 32],
    measured_at: u64,
}

impl LayerWitness {
    /// Modul-interner Konstruktor für ScanSession::finish.
    pub(crate) fn new(
        layer: WitnessLayer,
        scanned: u64,
        remaining: u64,
        technique: ScanTechnique,
        digest: [u8; 32],
        measured_at: u64,
    ) -> Self {
        Self {
            layer,
            scanned,
            remaining,
            technique,
            digest,
            measured_at,
        }
    }

    /// Gibt die gescannte Storage-Schicht als &str zurück.
    pub fn layer(&self) -> &str {
        self.layer.as_str()
    }

    /// Gibt die gescannte Storage-Schicht als &WitnessLayer zurück.
    pub fn witness_layer(&self) -> &WitnessLayer {
        &self.layer
    }

    /// Gibt die Anzahl der während der Verifikation gescannten Einheiten zurück.
    pub fn scanned(&self) -> u64 {
        self.scanned
    }

    /// Gibt die Anzahl der verbleibenden Lebendeinträge zurück.
    pub fn remaining(&self) -> u64 {
        self.remaining
    }

    /// Gibt die angewandte Scantechnik zurück.
    pub fn technique(&self) -> &ScanTechnique {
        &self.technique
    }

    /// Gibt den BLAKE3-Digest der Messung zurück.
    pub fn digest(&self) -> &[u8; 32] {
        &self.digest
    }

    /// Gibt den Zeitstempel der Messung (Unix Nanos) zurück.
    pub fn measured_at(&self) -> u64 {
        self.measured_at
    }
}

/// Hilfsfunktion zur Witness-Erzeugung ausschließlich für Testzwecke (`testkit`-Feature).
#[cfg(any(test, feature = "testkit"))]
pub fn mint_witness_for_testkit(
    layer: WitnessLayer,
    scanned: u64,
    remaining: u64,
    technique: ScanTechnique,
    digest: [u8; 32],
    measured_at: u64,
) -> LayerWitness {
    LayerWitness::new(layer, scanned, remaining, technique, digest, measured_at)
}
