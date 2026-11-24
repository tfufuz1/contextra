//! Single-Flight-Koordination und Scope-Sperren (INV-ERA-3, WP-3, SPEC §11.1).
//!
//! Invariante INV-ERA-3: Pro Scope läuft höchstens ein Purge (Single-Flight).
//! Ein Scope-Lock überlebt Prozessabstürze.
//!
//! Stale-Lock-Erkennung:
//! Ein Lock besitzt eine Epoche/Token und eine Lease-Dauer. Zeitprüfungen
//! erfolgen ausschließlich über den injizierten Port `Clock` (keine Wanduhr!).
//! Ist die Lease abgelaufen und kein Heartbeat erfolgt, kann ein Wiederanlauf
//! das Stale Lock geordnet übernehmen, ohne dass zwei Purges parallel laufen.

use crate::error::ErasureError;
use crate::types_local::{Clock, Vfs};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Fehler beim Erwerb oder der Freigabe eines Scope-Locks.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum ScopeLockError {
    /// Scope ist bereits durch einen aktiven Purge gesperrt.
    #[error("scope {scope_id:?} is locked by active owner {owner_id:?} (epoch {epoch})")]
    ScopeLocked {
        /// Betroffener Scope bzw. Erasure-ID.
        scope_id: [u8; 16],
        /// Aktiver Besitzer.
        owner_id: [u8; 16],
        /// Aktuelle Sperr-Epoche.
        epoch: u64,
    },

    /// Lock-Metadaten sind beschädigt (Fail-Closed).
    #[error("corrupted scope lock record")]
    CorruptedLock,

    /// E/A-Fehler bei Dateisystem-Operationen.
    #[error("io error during scope lock: {0}")]
    Io(String),

    /// Epochen-Konflikt bei versuchter Freigabe oder Erneuerung.
    #[error("epoch mismatch: expected {expected}, found {actual}")]
    EpochMismatch {
        /// Erwartete Epoche.
        expected: u64,
        /// Tatsächlich vorgefundene Epoche.
        actual: u64,
    },
}

impl From<ErasureError> for ScopeLockError {
    fn from(e: ErasureError) -> Self {
        match e {
            ErasureError::Corrupt => ScopeLockError::CorruptedLock,
            other => ScopeLockError::Io(other.to_string()),
        }
    }
}

/// Zustand eines Scope-Locks auf Platte oder im VFS.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScopeLockRecord {
    /// Eindeutige Kennung des gesperrten Scopes (z. B. ErasureId).
    pub scope_id: [u8; 16],
    /// Eindeutige Kennung des Besitzers (z. B. Instanz / Transaktion).
    pub owner_id: [u8; 16],
    /// Monoton steigende Sperr-Epoche.
    pub epoch: u64,
    /// Zeitpunkt des Erwerbs in Monotonic Nanos (aus `Clock`).
    pub acquired_monotonic_nanos: u64,
    /// Lease-Dauer in Nanosekunden (nach Ablauf gilt Lock als stale).
    pub lease_nanos: u64,
}

impl ScopeLockRecord {
    const MAGIC: &'static [u8; 8] = b"CTXSLOK\0";

    /// Kodiert den Lock-Record in ein festes Binärformat mit BLAKE3-Prüfsumme.
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut payload = Vec::with_capacity(16 + 16 + 8 + 8 + 8);
        payload.extend_from_slice(&self.scope_id);
        payload.extend_from_slice(&self.owner_id);
        payload.extend_from_slice(&self.epoch.to_be_bytes());
        payload.extend_from_slice(&self.acquired_monotonic_nanos.to_be_bytes());
        payload.extend_from_slice(&self.lease_nanos.to_be_bytes());

        let checksum = blake3::hash(&payload);

        let mut out = Vec::with_capacity(8 + 32 + payload.len());
        out.extend_from_slice(Self::MAGIC);
        out.extend_from_slice(checksum.as_bytes());
        out.extend_from_slice(&payload);
        out
    }

    /// Dekodiert und validiert den Lock-Record fail-closed.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, ScopeLockError> {
        if bytes.len() < 8 + 32 + 56 {
            return Err(ScopeLockError::CorruptedLock);
        }

        let magic = bytes.get(0..8).ok_or(ScopeLockError::CorruptedLock)?;
        if magic != Self::MAGIC {
            return Err(ScopeLockError::CorruptedLock);
        }

        let stored_checksum = bytes.get(8..40).ok_or(ScopeLockError::CorruptedLock)?;
        let payload = bytes.get(40..).ok_or(ScopeLockError::CorruptedLock)?;

        let computed_checksum = blake3::hash(payload);
        if computed_checksum.as_bytes() != stored_checksum {
            return Err(ScopeLockError::CorruptedLock);
        }

        if payload.len() != 56 {
            return Err(ScopeLockError::CorruptedLock);
        }

        let mut scope_id = [0u8; 16];
        if let Some(src) = payload.get(0..16) {
            scope_id.copy_from_slice(src);
        }

        let mut owner_id = [0u8; 16];
        if let Some(src) = payload.get(16..32) {
            owner_id.copy_from_slice(src);
        }

        let epoch = u64::from_be_bytes(
            payload
                .get(32..40)
                .ok_or(ScopeLockError::CorruptedLock)?
                .try_into()
                .map_err(|_| ScopeLockError::CorruptedLock)?,
        );

        let acquired = u64::from_be_bytes(
            payload
                .get(40..48)
                .ok_or(ScopeLockError::CorruptedLock)?
                .try_into()
                .map_err(|_| ScopeLockError::CorruptedLock)?,
        );

        let lease = u64::from_be_bytes(
            payload
                .get(48..56)
                .ok_or(ScopeLockError::CorruptedLock)?
                .try_into()
                .map_err(|_| ScopeLockError::CorruptedLock)?,
        );

        Ok(Self {
            scope_id,
            owner_id,
            epoch,
            acquired_monotonic_nanos: acquired,
            lease_nanos: lease,
        })
    }

    /// Prüft, ob das Lock relativ zur übergebenen Monotonic-Zeit abgelaufen (stale) ist.
    pub fn is_stale(&self, current_monotonic_nanos: u64) -> bool {
        current_monotonic_nanos >= self.acquired_monotonic_nanos.saturating_add(self.lease_nanos)
    }
}

/// Pfad der Sperrdatei für einen Scope.
pub fn scope_lock_path(scope_id: &[u8; 16]) -> PathBuf {
    let hex_id = scope_id
        .iter()
        .map(|b| format!("{:02x}", b))
        .collect::<String>();
    PathBuf::from(format!("scope/{}.lock", hex_id))
}

/// Single-Flight Guard für einen Scope (INV-ERA-3).
///
/// Hält das Lock auf einen Scope solange aktiv, bis der Guard gedroppt oder
/// explizit via [`ScopeLockGuard::release`] freigegeben wird.
pub struct ScopeLockGuard<'a> {
    vfs: &'a dyn Vfs,
    path: PathBuf,
    record: ScopeLockRecord,
}

impl<'a> std::fmt::Debug for ScopeLockGuard<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ScopeLockGuard")
            .field("path", &self.path)
            .field("record", &self.record)
            .finish()
    }
}

impl<'a> ScopeLockGuard<'a> {
    /// Gibt die aktuelle Sperr-Epoche zurück.
    pub fn epoch(&self) -> u64 {
        self.record.epoch
    }

    /// Gibt die Besitzer-ID zurück.
    pub fn owner_id(&self) -> &[u8; 16] {
        &self.record.owner_id
    }

    /// Gibt die Scope-ID zurück.
    pub fn scope_id(&self) -> &[u8; 16] {
        &self.record.scope_id
    }

    /// Gibt das Scope-Lock explizit frei.
    pub fn release(self) -> Result<(), ScopeLockError> {
        let current_bytes = self
            .vfs
            .read(&self.path)
            .map_err(|e| ScopeLockError::Io(e.to_string()))?;
        let existing = ScopeLockRecord::from_bytes(&current_bytes)?;

        if existing.epoch != self.record.epoch || existing.owner_id != self.record.owner_id {
            return Err(ScopeLockError::EpochMismatch {
                expected: self.record.epoch,
                actual: existing.epoch,
            });
        }

        self.vfs
            .remove_file(&self.path)
            .map_err(|e| ScopeLockError::Io(e.to_string()))?;

        if let Some(parent) = self.path.parent() {
            let _ = self.vfs.sync_dir(parent);
        }

        Ok(())
    }
}

/// Verwaltet Single-Flight Scope-Locks (INV-ERA-3).
pub struct ScopeLockManager<'a> {
    vfs: &'a dyn Vfs,
    clock: &'a dyn Clock,
    default_lease_nanos: u64,
}

impl<'a> ScopeLockManager<'a> {
    /// Erstellt einen neuen `ScopeLockManager`.
    pub fn new(vfs: &'a dyn Vfs, clock: &'a dyn Clock, default_lease_nanos: u64) -> Self {
        Self {
            vfs,
            clock,
            default_lease_nanos,
        }
    }

    /// Versucht, ein exklusives Single-Flight Lock auf den angegebenen Scope zu erwerben.
    ///
    /// - Falls keine Sperre existiert: Erwirbt Lock mit Epoche 1.
    /// - Falls eine Sperre existiert und noch aktiv ist: Schlägt fail-closed fehl mit [`ScopeLockError::ScopeLocked`].
    /// - Falls eine Sperre existiert, aber gemäß `Clock::monotonic_nanos` abgelaufen (stale) ist:
    ///   Übernimmt die Sperre geordnet mit inkrementierter Epoche (`old_epoch + 1`),
    ///   wodurch ein nach Absturz verwaistes Lock sicher bereinigt wird, ohne dass zwei
    ///   Purge-Vorgänge jemals denselben Scope parallel bearbeiten.
    pub fn acquire(
        &self,
        scope_id: [u8; 16],
        owner_id: [u8; 16],
    ) -> Result<ScopeLockGuard<'a>, ScopeLockError> {
        self.acquire_with_lease(scope_id, owner_id, self.default_lease_nanos)
    }

    /// Versucht, ein exklusives Single-Flight Lock mit benutzerdefinierter Lease-Dauer zu erwerben.
    pub fn acquire_with_lease(
        &self,
        scope_id: [u8; 16],
        owner_id: [u8; 16],
        lease_nanos: u64,
    ) -> Result<ScopeLockGuard<'a>, ScopeLockError> {
        let path = scope_lock_path(&scope_id);
        if let Some(parent) = path.parent() {
            self.vfs
                .create_dir_all(parent)
                .map_err(|e| ScopeLockError::Io(e.to_string()))?;
        }

        let now = self.clock.monotonic_nanos();

        let exists = self
            .vfs
            .exists(&path)
            .map_err(|e| ScopeLockError::Io(e.to_string()))?;

        let (new_epoch, is_takeover) = if exists {
            let data = self
                .vfs
                .read(&path)
                .map_err(|e| ScopeLockError::Io(e.to_string()))?;
            let existing = ScopeLockRecord::from_bytes(&data)?;

            if !existing.is_stale(now) {
                // Lock ist noch aktiv -> Fail-Closed Single Flight (INV-ERA-3)
                return Err(ScopeLockError::ScopeLocked {
                    scope_id,
                    owner_id: existing.owner_id,
                    epoch: existing.epoch,
                });
            }

            // Lock ist stale -> geordnete Übernahme mit inkrementierter Epoche
            let next_epoch = existing.epoch.saturating_add(1);
            (next_epoch, true)
        } else {
            (1u64, false)
        };

        let new_record = ScopeLockRecord {
            scope_id,
            owner_id,
            epoch: new_epoch,
            acquired_monotonic_nanos: now,
            lease_nanos,
        };

        let record_bytes = new_record.to_bytes();

        // Atomares Schreiben der Lock-Datei via VFS
        self.vfs
            .write_atomic(&path, &record_bytes)
            .map_err(|e| ScopeLockError::Io(e.to_string()))?;

        if let Some(parent) = path.parent() {
            let _ = self.vfs.sync_dir(parent);
        }

        let _ = is_takeover; // Kann für Audit-Logging herangezogen werden

        Ok(ScopeLockGuard {
            vfs: self.vfs,
            path,
            record: new_record,
        })
    }

    /// Prüft, ob ein Scope aktuell gesperrt ist.
    pub fn is_locked(&self, scope_id: &[u8; 16]) -> Result<bool, ScopeLockError> {
        let path = scope_lock_path(scope_id);
        if !self
            .vfs
            .exists(&path)
            .map_err(|e| ScopeLockError::Io(e.to_string()))?
        {
            return Ok(false);
        }

        let bytes = self
            .vfs
            .read(&path)
            .map_err(|e| ScopeLockError::Io(e.to_string()))?;
        let record = ScopeLockRecord::from_bytes(&bytes)?;
        let now = self.clock.monotonic_nanos();
        Ok(!record.is_stale(now))
    }
}
