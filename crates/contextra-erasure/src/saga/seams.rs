//! Asynchrone Nähte (Seams) für den generischen `ErasureCore` (SPEC §11.2, AUFTRAG J-07).
//!
//! Alle Traits sind asynchron, objekt-sicher (`Send + Sync`) und fehlerfähig.
//! Sie entkoppeln den Orchestrator von konkreten persistenten Modulen,
//! sodass der Kern unabhängig kompiliert und gegen Test-Doubles verifiziert werden kann.

use crate::error::ErasureError;
use crate::saga::machine::{EraseSaga, SagaState};
use contextra_ports::erasure::{EraseTarget, LayerId, LayerWitness};
use contextra_ports::gate::GateDecision;
use contextra_ports::BoxFuture;
use contextra_types::erasure::{PendingReason, RetentionAttestation};
use contextra_types::ids::ErasureId;

/// Naht für das Intent-Journal (SPEC §11.6, ER-29).
pub trait SagaJournal: Send + Sync {
    /// Hängt den initialen `Requested+Intent`-Eintrag mit eingebettetem Umfang an (fsync #1).
    fn append_intent<'a>(
        &'a self,
        erasure_id: ErasureId,
        target: &'a EraseTarget,
        attestation: &'a RetentionAttestation,
    ) -> BoxFuture<'a, Result<(), ErasureError>>;

    /// Hängt einen Zustandsübergang an.
    fn append_state<'a>(
        &'a self,
        erasure_id: ErasureId,
        state: SagaState,
        reason: Option<&'a PendingReason>,
    ) -> BoxFuture<'a, Result<(), ErasureError>>;

    /// Rekonstruiert noch offene Sagas aus dem Journal.
    fn recover_open_sagas<'a>(&'a self) -> BoxFuture<'a, Result<Vec<EraseSaga>, ErasureError>>;
}

/// Naht für das Gate-Subsystem (SPEC §11.3, ER-11, ER-12, ER-13).
pub trait SagaGate: Send + Sync {
    /// Trägt eine Lesesperre ein und persistiert das Gate-Log (fsync #2).
    /// Liefert die `gated_seq`.
    fn add_gate<'a>(
        &'a self,
        erasure_id: ErasureId,
        target: &'a EraseTarget,
    ) -> BoxFuture<'a, Result<u64, ErasureError>>;

    /// Kompaktiert das Gate nach Abschluss der Löschung (SPEC §11.2 Schritt 6).
    fn compact_gate<'a>(&'a self, erasure_id: ErasureId)
        -> BoxFuture<'a, Result<(), ErasureError>>;

    /// Liefert die Gate-Entscheidung (`Erased`, `Restricted`, `Open`) für ein Ziel.
    fn decision_for_target<'a>(
        &'a self,
        target: &'a EraseTarget,
    ) -> BoxFuture<'a, Result<GateDecision, ErasureError>>;
}

/// Naht für den Schlüsseltresor (SPEC §11.5, ER-05, ER-06).
pub trait SagaVault: Send + Sync {
    /// Vernichtet den Slot der höchsten Ebene (`T`/`C`/`S`/`D`) und verifiziert das Nullen (fsync #3).
    fn destroy_highest_key<'a>(
        &'a self,
        target: &'a EraseTarget,
    ) -> BoxFuture<'a, Result<(), ErasureError>>;

    /// Prüft lesend, dass der Schlüssel nicht mehr existiert.
    fn verify_key_absent<'a>(
        &'a self,
        target: &'a EraseTarget,
    ) -> BoxFuture<'a, Result<bool, ErasureError>>;
}

/// Naht für die Messung und Zeugenprüfung (SPEC §12, ER-25, ER-26).
pub trait SagaMeasure: Send + Sync {
    /// Führt die Positivkontrolle durch und verifiziert Residuen für eine Schicht.
    fn witness_layer<'a>(
        &'a self,
        layer_id: LayerId,
        target: &'a EraseTarget,
    ) -> BoxFuture<'a, Result<LayerWitness, ErasureError>>;
}

/// Naht für die Berichterstellung und kryptografische Signatur (SPEC §11.7, ER-27).
pub trait SagaReport: Send + Sync {
    /// Erstellt den kanonischen, signierten Löschbericht.
    fn sign_report<'a>(
        &'a self,
        erasure_id: ErasureId,
        target: &'a EraseTarget,
        attestation: &'a RetentionAttestation,
        witnesses: &'a [LayerWitness],
    ) -> BoxFuture<'a, Result<Vec<u8>, ErasureError>>;
}

/// Naht für das Ledger und Restore-Absicherung (SPEC §11.8, ER-30).
pub trait SagaLedger: Send + Sync {
    /// Schreibt den Eintrag nach `Completed` in das Ledger außerhalb des Datenverzeichnisses.
    fn record_completed<'a>(
        &'a self,
        erasure_id: ErasureId,
        target: &'a EraseTarget,
        gated_seq: u64,
        completed_at: u64,
    ) -> BoxFuture<'a, Result<(), ErasureError>>;
}

/// Naht für den Snapshot-Widerruf (SPEC §11.2 Schritt 5a, D-61, L-21).
pub trait SnapshotRevoker: Send + Sync {
    /// Widerruft Snapshots des Mandanten für den betroffenen Bereich bis `gated_seq`.
    fn revoke_covering<'a>(
        &'a self,
        target: &'a EraseTarget,
        gated_seq: u64,
    ) -> BoxFuture<'a, Result<u64, ErasureError>>;
}
