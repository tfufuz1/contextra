//! Journal-Epochenrotation (SPEC §11.6, D-48, ER-29).
//!
//! Nach `commit_epoch` wird ein neuer Epochenschlüssel erzeugt.

/// Zustand einer Journal-Epoche.
#[derive(Debug, Clone)]
pub struct Epoch {
    /// Epochenindex (monoton steigend).
    pub index: u32,
    /// Zeitstempel des Epochenbeginns (Mikrosekunden UTC).
    pub started_at: u64,
}

impl Epoch {
    /// Erstellt die erste Epoche.
    pub fn first(started_at: u64) -> Self {
        Self {
            index: 0,
            started_at,
        }
    }

    /// Rotiert zur nächsten Epoche.
    pub fn next(&self, now: u64) -> Self {
        Self {
            index: self.index + 1,
            started_at: now,
        }
    }
}
