//! Ports der Löschkette. Konkrete Implementierungen liegen in `adapters` oder im Aufrufer.

use std::sync::atomic::{AtomicU64, Ordering};

use contextra_ids::ErasureId;
use parking_lot::{RwLock, RwLockReadGuard, RwLockWriteGuard};

use crate::error::ErasureError;
use crate::gate::saga_gate::GateDecision;
use crate::types_local::LayerWitness;

use super::journal::JournalRecord;
use super::retention::RetentionAttestation;
use super::target::EraseTarget;

/// Gate-Modus, mit dem ein Ziel gesperrt wird.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GateMode {
    /// Vollständige Sperre (`Erased`).
    Erased,
    /// Art.-18-Einschränkung (`Restricted`).
    Restricted,
}

/// Durabler Journal-Port. `append` kehrt erst nach fsync zurück.
pub trait Journal: Send + Sync {
    /// Hängt einen Datensatz an und synchronisiert ihn auf Datenträger.
    fn append(&self, record: &JournalRecord) -> Result<(), ErasureError>;
    /// Liest alle vollständig bestätigten Datensätze in Reihenfolge.
    fn load(&self) -> Result<Vec<JournalRecord>, ErasureError>;
}

/// Port für das ReadGate: Sperre setzen, aufheben und abfragen.
pub trait GateSink: Send + Sync {
    /// Setzt die Sperre für das Ziel.
    fn activate(&self, target: &EraseTarget, mode: GateMode) -> Result<(), ErasureError>;
    /// Hebt eine Einschränkung auf (nur `Restricted → Lifted`).
    fn lift_restricted(&self, target: &EraseTarget) -> Result<(), ErasureError>;
    /// Aktuelle Entscheidung für das Ziel. Unbekannt bedeutet `Open` und ist nie ein Nachweis für `Erased`.
    fn decision(&self, target: &EraseTarget) -> GateDecision;
}

/// Port für die Schlüsselvernichtung und deren Verifikation.
pub trait KeyVault: Send + Sync {
    /// Vernichtet den Schlüssel der Ebene des Ziels. Idempotent. Durabel bei Rückkehr.
    fn destroy(&self, target: &EraseTarget) -> Result<(), ErasureError>;
    /// `true`, wenn der Schlüssel nicht mehr vorhanden ist (Constant-Time-Prüfung im Adapter).
    fn verify_absent(&self, target: &EraseTarget) -> Result<bool, ErasureError>;
}

/// Plattenreserve (SPEC §5.3 Schritt 4).
pub trait DiskReserve: Send + Sync {
    /// Verfügbare Bytes.
    fn available_bytes(&self) -> u64;
}

/// Positivkontrolle für die Messung (SPEC §5.3 Schritt 5.2).
pub trait PositiveControl: Send + Sync {
    /// Führt die Canary-Suche durch. `true` nur, wenn der Canary gefunden wurde.
    fn canary_observed(&self) -> Result<bool, ErasureError>;
}

/// Bericht und Ledger (SPEC §5.3 Schritt 6). Signatur und fsync des Ledgers liegen beim Implementierer.
pub trait ReportSink: Send + Sync {
    /// Signiert den Bericht und schreibt den Ledger-Eintrag durabel.
    fn publish(
        &self,
        erasure_id: ErasureId,
        target: &EraseTarget,
        attestation: &RetentionAttestation,
        witnesses: &[LayerWitness],
    ) -> Result<(), ErasureError>;
}

/// Commit-Barriere (SPEC §5.3 Schritt 2.1). Writer halten `shared`, das Gate setzt `exclusive`.
#[derive(Default)]
pub struct CommitBarrier {
    lock: RwLock<()>,
    generation: AtomicU64,
}

impl CommitBarrier {
    /// Neue Barriere.
    pub fn new() -> Self {
        Self::default()
    }

    /// Gemeinsamer Zugriff für Writer.
    pub fn shared(&self) -> RwLockReadGuard<'_, ()> {
        self.lock.read()
    }

    /// Exklusiver Zugriff: blockiert alle Writer bis zur Freigabe.
    pub fn exclusive(&self) -> BarrierExclusive<'_> {
        let guard = self.lock.write();
        BarrierExclusive {
            _guard: guard,
            barrier: self,
        }
    }

    /// Anzahl der exklusiven Sperren seit Erstellung (Diagnose).
    pub fn generation(&self) -> u64 {
        self.generation.load(Ordering::SeqCst)
    }
}

/// Exklusiv-Guard der Commit-Barriere.
pub struct BarrierExclusive<'a> {
    _guard: RwLockWriteGuard<'a, ()>,
    barrier: &'a CommitBarrier,
}

impl Drop for BarrierExclusive<'_> {
    fn drop(&mut self) {
        self.barrier.generation.fetch_add(1, Ordering::SeqCst);
    }
}
