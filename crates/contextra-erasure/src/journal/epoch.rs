//! Journal-Epochenrotation (SPEC §11.6, D-48, D-125, ER-29).
//!
//! Nach `commit_epoch` wird ein neuer Epochenschlüssel erzeugt.

/// Trait zur Bereitstellung von Epochenschlüsseln (SPEC D-125).
///
/// Rotation vernichtet den alten Epochenschlüssel (Slot E im Tresor).
/// Danach sind alte Commitments nicht mehr verknüpfbar.
pub trait EpochKeys: Send + Sync {
    /// Liefert den 32-Byte-Epochenschlüssel `K_commit(epoch)`.
    fn commit_key(&self, epoch_id: u32) -> [u8; 32];
}

/// Berechnet das kryptografische Commitment für Journal und Bericht nach SPEC §11.6 und D-125:
/// `BLAKE3-keyed(K_commit(epoch), "ctx/commit/v1" ‖ subject ‖ tenant)`
pub fn compute_commitment(commit_key: &[u8; 32], subject: &[u8], tenant: &[u8]) -> [u8; 16] {
    let mut hasher = blake3::Hasher::new_keyed(commit_key);
    hasher.update(b"ctx/commit/v1");
    hasher.update(subject);
    hasher.update(tenant);
    let hash = hasher.finalize();

    let mut out = [0u8; 16];
    out.copy_from_slice(&hash.as_bytes()[0..16]);
    out
}

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
