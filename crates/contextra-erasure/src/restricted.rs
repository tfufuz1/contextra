//! Domänenlogik für eingeschränkte Verarbeitung (Art. 18 DSGVO, SPEC §11.2, D-116).
//!
//! # Normative Referenzen (D-116, Art. 18 Abs. 2)
//! - `Restricted` sperrt reguläres Lesen, Suchen und Schreiben.
//! - Expliziter, auditierter Lesepfad nach Art. 18 Abs. 2:
//!   - `RestrictedPurpose`: `LegalClaims` oder `DataSubjectConsent`.
//!   - `RestrictedAccess`: Zweckbindung + Betreiber-Attestierung.
//! - **Audit vor Auslieferung (I-26):** Journal-Eintrag Code 9 `RestrictedRead`
//!   mit `erasure_id`, `purpose`, `time`, `doc_count`, `case_commitment`.
//!   Dauerhaft vor Rückgabe von Klartextdaten.
//! - `case_commitment = BLAKE3-keyed(K_commit(epoch), case_ref)`. `case_ref` selbst
//!   wird nie persistent gespeichert (P13).
//! - **Stapelgrenze:** Höchstens 1 000 Dokumente je Aufruf/Audit-Eintrag (T-RESTR-4).
//! - **Aufhebung:** `Restricted -> Lifted` nur mit Journal-Eintrag; `Erase` auf
//!   `Restricted` ist zulässig und hebt Restriktion durch Löschung auf (Journal `SupersededByErasure`).

use crate::error::ErasureError;
use contextra_types::ids::ErasureId;
use serde::{Deserialize, Serialize};

/// Gesetzlicher Ausnahmezweck nach Art. 18 Abs. 2 DSGVO (SPEC D-116).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RestrictedPurpose {
    /// Geltendmachung, Ausübung oder Verteidigung von Rechtsansprüchen.
    LegalClaims,
    /// Einwilligung der betroffenen Person.
    DataSubjectConsent,
}

/// Attestierung des beauftragten Betreibers für Art. 18 Abs. 2 Zugriff (SPEC D-116).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OperatorAttestation {
    operator: String,
    case_commitment: [u8; 32],
    attested_at: u64,
}

impl OperatorAttestation {
    /// Erstellt eine neue Betreiber-Attestierung mit gehashtem Fallbezug.
    ///
    /// `case_ref` wird kryptografisch mit dem Epochenschlüssel gehasht (`case_commitment`),
    /// sodass kein Personenbezug im Klartext persistiert wird (P13, D-116).
    pub fn new(operator: &str, case_ref: &[u8], attested_at: u64, epoch_key: &[u8; 32]) -> Self {
        let commitment = blake3::keyed_hash(epoch_key, case_ref);
        Self {
            operator: operator.to_string(),
            case_commitment: *commitment.as_bytes(),
            attested_at,
        }
    }

    /// Liefert den Betreiber-Identifikator.
    #[inline]
    pub fn operator(&self) -> &str {
        &self.operator
    }

    /// Liefert das kryptografische Commitment des Fallbezugs.
    #[inline]
    pub fn case_commitment(&self) -> &[u8; 32] {
        &self.case_commitment
    }

    /// Liefert den Zeitstempel der Attestierung.
    #[inline]
    pub fn attested_at(&self) -> u64 {
        self.attested_at
    }
}

/// Autorisierungskontext für den Zugriff auf eingeschränkte Daten (SPEC D-116).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RestrictedAccess {
    purpose: RestrictedPurpose,
    attestation: OperatorAttestation,
}

impl RestrictedAccess {
    /// Erstellt ein neues Zugriffsgesuch nach Art. 18 Abs. 2 DSGVO.
    pub fn new(purpose: RestrictedPurpose, attestation: OperatorAttestation) -> Self {
        Self {
            purpose,
            attestation,
        }
    }

    /// Liefert den rechtlichen Zweck.
    #[inline]
    pub fn purpose(&self) -> RestrictedPurpose {
        self.purpose
    }

    /// Liefert die zugehörige Betreiber-Attestierung.
    #[inline]
    pub fn attestation(&self) -> &OperatorAttestation {
        &self.attestation
    }

    /// Prüft, ob der Pfad ausschließlich auf `Restricted`-Bereiche angewandt wird (SPEC D-116, T-RESTR-3).
    ///
    /// Gegen `Erased`-Bereiche ist kein Zugriff möglich; liefert `false`.
    pub fn allows_access_to_restricted(&self, is_target_restricted: bool) -> bool {
        if !is_target_restricted {
            return false;
        }
        matches!(
            self.purpose,
            RestrictedPurpose::LegalClaims | RestrictedPurpose::DataSubjectConsent
        )
    }

    /// Validiert die Stapelgröße (SPEC D-116: höchstens 1 000 Dokumente).
    pub fn validate_batch_size(&self, batch_size: usize) -> Result<(), ErasureError> {
        if batch_size > 1000 {
            return Err(ErasureError::LimitExceeded(
                "Restricted-Zugriffsstapel überschreitet Obergrenze von 1000 Dokumenten"
                    .to_string(),
            ));
        }
        Ok(())
    }
}

/// Audit-Datensatz für Journal Code 9 `RestrictedRead` (SPEC D-116, F.9).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RestrictedReadAudit {
    /// Zugehörige Löschungs-/Restriktions-ID.
    pub erasure_id: ErasureId,
    /// Record-Code nach F.9 (Code 9).
    pub record_code: u8,
    /// Rechtsgrund / Zweck.
    pub purpose: RestrictedPurpose,
    /// Zeitstempel des Zugriffs.
    pub timestamp: u64,
    /// Anzahl gelesener Dokumente in diesem Batch (max. 1000).
    pub doc_count: u32,
    /// BLAKE3-keyed Commitment des Fallbezugs.
    pub case_commitment: [u8; 32],
}

impl RestrictedReadAudit {
    /// Erstellt einen auditierbaren Journal-Eintrag für Art. 18 Abs. 2 Zugriff.
    pub fn create_entry(
        erasure_id: ErasureId,
        access: &RestrictedAccess,
        doc_count: u32,
        timestamp: u64,
    ) -> Result<Self, ErasureError> {
        access.validate_batch_size(doc_count as usize)?;

        Ok(Self {
            erasure_id,
            record_code: 9,
            purpose: access.purpose(),
            timestamp,
            doc_count,
            case_commitment: *access.attestation().case_commitment(),
        })
    }
}

/// Zustandsmodell für eingeschränkte Daten (Art. 18 DSGVO, SPEC §11.2, D-116, D-119).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RestrictedState {
    /// Einschränkung der Verarbeitung aktiv.
    Restricted,
    /// Einschränkung durch formalen Journal-Eintrag aufgehoben.
    Lifted,
    /// Restriktion durch nachfolgenden Löschauftrag abgelöst (D-119).
    SupersededByErasure,
}

impl RestrictedState {
    /// Prüft, ob die Einschränkung aufgehoben ist.
    pub fn is_lifted(&self) -> bool {
        matches!(self, Self::Lifted)
    }

    /// Prüft, ob es sich um einen Endzustand handelt.
    pub fn is_terminal(&self) -> bool {
        matches!(self, Self::Lifted | Self::SupersededByErasure)
    }

    /// Hebt die Restriktion mit einer Begründung auf (Journal-Eintrag erforderlich).
    pub fn lift(&mut self, _reason: &str) -> Result<(), ErasureError> {
        match self {
            Self::Restricted => {
                *self = Self::Lifted;
                Ok(())
            }
            _ => Err(ErasureError::InvalidTransition {
                from: format!("{:?}", self),
                to: format!("{:?}", Self::Lifted),
            }),
        }
    }

    /// Ersetzt die Restriktion durch einen nachfolgenden regulären Erase-Vorgang (D-119).
    pub fn supersede_by_erasure(&mut self) -> Result<(), ErasureError> {
        match self {
            Self::Restricted => {
                *self = Self::SupersededByErasure;
                Ok(())
            }
            _ => Err(ErasureError::InvalidTransition {
                from: format!("{:?}", self),
                to: format!("{:?}", Self::SupersededByErasure),
            }),
        }
    }
}
