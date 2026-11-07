//! Domänenlogik für Sonderziel Ersetzung (`replace_erasing`, SPEC §11.2, D-115, Zieltyp 6).
//!
//! # Normative Referenzen (D-115, I-25)
//! - **Vorbedingungen:** `doc_tag(new_ref) != doc_tag(old)`, sonst `SameIdentity`.
//! - **Protokoll (Zieltyp 6, DocumentReplace):**
//!   - R1: Journal `Requested+Intent` mit Umfang des alten Dokuments und `new_doc_tag`.
//!   - R2: `PreGate::run`: neues Dokument dauerhaft anlegen (LSM).
//!   - R3: Gate des alten Dokuments.
//!   - R4: Vernichtung des `D`-Slots, Journal `KeyDestroyed`, danach Purge.
//! - **Absturz / Recovery:** Wenn neues Dokument vor R3 fehlt, bleibt das alte Dokument
//!   unberührt und der Status ist `Pending { phase: Replace, reason: ReplacementMissing }`.
//! - **I-25:** Bei `replace_erasing` ist nie alter Inhalt gesperrt, während neuer Inhalt
//!   nicht dauerhaft ist.
//! - **Bericht:** Führt `target = Document`, `replaced = true` und `replacement_tag`.

use crate::error::ErasureError;
use crate::types_local::ErasureId;
use serde::{Deserialize, Serialize};

/// Einzelschritte des Ersetzungsprotokolls (SPEC D-115).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReplaceStep {
    /// R1: Initialer Zustand nach Eingang des Ersetzungsantrags.
    R1Requested,
    /// R2: Absicht registriert, PreGate-Ausführung für Neudokument aktiv.
    R2PreGate,
    /// R3: Neudokument dauerhaft bestätigt, Altdokument gegated.
    R3GatedOld,
    /// R4: D-Slot des Altdokuments vernichtet, Übergang zum physischen Purge.
    R4KeyDestroyed,
}

/// Plan und Zustandsverwaltung für die Ersetzung eines Mehr-Betroffenen-Dokuments (SPEC D-115).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReplacePlan {
    erasure_id: ErasureId,
    old_doc_tag: [u8; 16],
    new_doc_tag: [u8; 16],
    step: ReplaceStep,
}

impl ReplacePlan {
    /// Erstellt einen neuen Ersetzungsplan.
    ///
    /// # Fehler
    /// Liefert `ErasureError::SameIdentity`, wenn altes und neues Dokument denselben Tag tragen.
    pub fn new(
        erasure_id: ErasureId,
        old_doc_tag: [u8; 16],
        new_doc_tag: [u8; 16],
    ) -> Result<Self, ErasureError> {
        if old_doc_tag == new_doc_tag {
            return Err(ErasureError::SameIdentity);
        }
        Ok(Self {
            erasure_id,
            old_doc_tag,
            new_doc_tag,
            step: ReplaceStep::R1Requested,
        })
    }

    /// Liefert die zugehörige Lösch-ID.
    #[inline]
    pub fn erasure_id(&self) -> ErasureId {
        self.erasure_id
    }

    /// Liefert den Tag des alten Dokuments.
    #[inline]
    pub fn old_doc_tag(&self) -> &[u8; 16] {
        &self.old_doc_tag
    }

    /// Liefert den Ersatz-Tag (`replacement_tag`, SPEC D-115) für den Löschbericht.
    #[inline]
    pub fn replacement_tag(&self) -> &[u8; 16] {
        &self.new_doc_tag
    }

    /// Liefert den aktuellen Protokollschritt (R1 bis R4).
    #[inline]
    pub fn step(&self) -> ReplaceStep {
        self.step
    }

    /// Schaltet idempotent auf R2 (PreGate Intent) vor (D-115 R2).
    pub fn advance_to_r2_intent(&mut self) -> Result<(), ErasureError> {
        match self.step {
            ReplaceStep::R1Requested | ReplaceStep::R2PreGate => {
                self.step = ReplaceStep::R2PreGate;
                Ok(())
            }
            _ => Err(ErasureError::InvalidTransition {
                from: format!("{:?}", self.step),
                to: format!("{:?}", ReplaceStep::R2PreGate),
            }),
        }
    }

    /// Prüft vor dem Gate-Schritt (R3), ob das Ersatzdokument dauerhaft vorhanden ist (I-25).
    ///
    /// Liefert `ErasureError::ReplacementMissing` (FM-19), falls das Ersatzdokument fehlt.
    pub fn verify_replacement_present(&self, is_present: bool) -> Result<(), ErasureError> {
        if !is_present {
            return Err(ErasureError::ReplacementMissing);
        }
        Ok(())
    }

    /// Schaltet nach Bestätigung des Neudokuments auf R3 (Gate altes Dokument) vor (D-115 R3).
    pub fn advance_to_r3_gate_old(&mut self) -> Result<(), ErasureError> {
        match self.step {
            ReplaceStep::R2PreGate | ReplaceStep::R3GatedOld => {
                self.step = ReplaceStep::R3GatedOld;
                Ok(())
            }
            _ => Err(ErasureError::InvalidTransition {
                from: format!("{:?}", self.step),
                to: format!("{:?}", ReplaceStep::R3GatedOld),
            }),
        }
    }

    /// Schaltet auf R4 (D-Slot des Altdokuments vernichtet) vor (D-115 R4).
    pub fn advance_to_r4_key_destroyed(&mut self) -> Result<(), ErasureError> {
        match self.step {
            ReplaceStep::R3GatedOld | ReplaceStep::R4KeyDestroyed => {
                self.step = ReplaceStep::R4KeyDestroyed;
                Ok(())
            }
            _ => Err(ErasureError::InvalidTransition {
                from: format!("{:?}", self.step),
                to: format!("{:?}", ReplaceStep::R4KeyDestroyed),
            }),
        }
    }
}
