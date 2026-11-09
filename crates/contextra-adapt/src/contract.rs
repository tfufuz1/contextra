//! Freeze-Vertrag und standardisierte Schnittstellen für `contextra-adapt`.
//!
//! SPEC v4.0 §2 (Klasse A) & B1–B12.

use crate::error::Result;
use std::time::Duration;

/// Trait für deterministische Zeitabfragen ohne globale Zeitgeber.
pub trait Clock: Send + Sync {
    /// Liefert die aktuelle Epochenzeit in Unix-Nanosekunden.
    fn now_unix_nanos(&self) -> u64;
    /// Liefert einen monoton steigenden Nanosekunden-Zähler.
    fn monotonic_nanos(&self) -> u64;
}

/// Trait für deterministische Pseudozufallszahlen.
pub trait Rng: Send + Sync {
    /// Liefert die nächste vorzeichenlose 64-Bit-Ganzzahl.
    fn next_u64(&mut self) -> u64;
    /// Liefert eine Fließkommazahl im halboffenen Intervall `[0.0, 1.0)`.
    fn next_f64(&mut self) -> f64 {
        let v = (self.next_u64() >> 11) as f64;
        v / ((1u64 << 53) as f64)
    }
}

/// Trait für Regler mit Anti-Windup und Stellgrößenbegrenzung.
pub trait AdaptiveController: Send + Sync {
    /// Aktualisiert den Reglerzustand und berechnet das Stellsignal.
    fn update(&mut self, dt: Duration, measurement: f64) -> f64;
    /// Setzt den internen Akkumulator zurück.
    fn reset(&mut self);
}

/// Trait für kontextuelle Bandit-Arm-Auswahl und Regressions-Updates.
pub trait BanditArmSelector: Send + Sync {
    /// Wählt einen Arm basierend auf dem Kontextvektor aus.
    fn select_arm(&mut self, context: &[f32]) -> Result<usize>;
    /// Aktualisiert den Zustand für den gewählten Arm mit beobachteter Belohnung.
    fn update(&mut self, arm: usize, context: &[f32], reward: f64) -> Result<()>;
}

/// Serialisierbarer Zustandsschnappschuss für Regler und Bandit-Profile.
pub trait StateSnapshot {
    /// Serialisiert den Zustand in einen defensiven Byte-Puffer.
    fn to_snapshot_bytes(&self) -> Result<Vec<u8>>;
    /// Stellt den Zustand aus einem Byte-Puffer wieder her.
    fn from_snapshot_bytes(bytes: &[u8]) -> Result<Self>
    where
        Self: Sized;
}

#[doc(hidden)]
/// Konformitätsprüfung für Contract-Traits.
pub fn _assert_contract_conformance<T: StateSnapshot>(item: &T, bytes: &[u8]) -> Result<()> {
    let _ = item.to_snapshot_bytes()?;
    let _ = T::from_snapshot_bytes(bytes)?;
    Ok(())
}
