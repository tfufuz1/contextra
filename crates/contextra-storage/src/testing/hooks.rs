//! # Test-Hooks
//!
//! Deterministische Test-Hooks nach SPEC §4, §18, §19.2 und IMPL §6 WP-1.3.

#![allow(clippy::unwrap_used, clippy::expect_used)]

pub use contextra_storage_ports::HookPoint;
use contextra_storage_ports::Hooks;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Aufzeichnender Test-Hook für deterministische Synchronisation und Fehlerprüfungen.
///
/// Schreibt alle empfangenen `HookPoint`-Ereignisse in eine lineare Sequenz.
/// Hält keinen globalen Zustand; alle Daten sind an die Instanz gebunden.
#[derive(Default, Clone)]
pub struct TestHooks {
    sequence: Arc<Mutex<Vec<HookPoint>>>,
    counts: Arc<Mutex<HashMap<HookPoint, usize>>>,
}

impl TestHooks {
    /// Erstellt eine neue, leere Instanz von `TestHooks`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Liefert die geordnete Sequenz aller bisher aufgetretenen Hook-Punkte.
    pub fn sequence(&self) -> Vec<HookPoint> {
        self.sequence.lock().unwrap().clone()
    }

    /// Liefert die Anzahl der Aufrufe für einen bestimmten Hook-Punkt.
    pub fn count(&self, point: HookPoint) -> usize {
        let counts = self.counts.lock().unwrap();
        counts.get(&point).copied().unwrap_or(0)
    }

    /// Berechnet einen 64-Bit-Hash über die bisherige Sequenz von Hook-Punkten (für Determinismus-Tests).
    pub fn hash(&self) -> u64 {
        let seq = self.sequence.lock().unwrap();
        let mut h: u64 = 0xcbf29ce484222325;
        for p in seq.iter() {
            let val = *p as u64;
            h ^= val;
            h = h.wrapping_mul(0x100000001b3);
        }
        h
    }

    /// Setzt alle Aufzeichnungen (Sequenz und Zähler) zurück.
    pub fn reset(&self) {
        self.sequence.lock().unwrap().clear();
        self.counts.lock().unwrap().clear();
    }
}

impl Hooks for TestHooks {
    fn point(&self, p: HookPoint) {
        self.sequence.lock().unwrap().push(p);
        let mut counts = self.counts.lock().unwrap();
        *counts.entry(p).or_insert(0) += 1;
    }
}
