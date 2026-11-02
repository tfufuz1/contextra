//! Auswertung und Erfassung von Restrisiken gemäß Katalog (SPEC §11.7, Anhang D, ER-28).
//!
//! Die vollständige Liste der Restrisiken wird immer bewertet.
//! `WrappedKeyRemanence`, `FlashRemanence`, `FsJournalOrCow` sind immer present
//! solange `FileVault` der Anker ist.

use contextra_types::erasure::{ResidualRisk, RiskCondition};

/// Bewertet alle Restrisiken für einen abgeschlossenen Löschvorgang (ER-28).
///
/// Gibt immer die vollständige Liste aller zutreffenden Restrisiken zurück.
pub fn evaluate_residual_risks(purge_complete: bool) -> Vec<ResidualRisk> {
    let mut risks = Vec::new();

    // Immer present (solange FileVault der Anker ist):
    risks.push(ResidualRisk::FlashRemanence);
    risks.push(ResidualRisk::FsJournalOrCow);
    risks.push(ResidualRisk::WrappedKeyRemanence);
    risks.push(ResidualRisk::FreedBlockRemanence);
    risks.push(ResidualRisk::OldKeyslotRemanence);
    risks.push(ResidualRisk::ProcessMemory);
    risks.push(ResidualRisk::IndexTopologyInference);
    risks.push(ResidualRisk::ReplicaWindow);
    risks.push(ResidualRisk::ExternalCopies);
    risks.push(ResidualRisk::ExternalProcessors);
    risks.push(ResidualRisk::PseudonymCommitment);
    risks.push(ResidualRisk::PurgeScalesWithDb);
    risks.push(ResidualRisk::KeyCompromise);

    // Bis Purge abgeschlossen:
    if !purge_complete {
        risks.push(ResidualRisk::TextIndexUntilPurge);
        risks.push(ResidualRisk::PurgeDeferred);
    }

    risks
}

/// Serialisiert ein Restrisiko als JSON-Objekt.
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct ResidualRiskEntry {
    /// Risiko-Identifikator (SPEC Anhang D).
    pub id: &'static str,
    /// Bedingung (always / cow_or_journaling / until_purge_complete).
    pub condition: &'static str,
}

impl ResidualRiskEntry {
    /// Erstellt einen Eintrag aus einem `ResidualRisk`.
    pub fn from_risk(risk: &ResidualRisk) -> Self {
        let condition = match risk.condition() {
            RiskCondition::Always => "always",
            RiskCondition::CowOrJournaling => "cow_or_journaling",
            RiskCondition::UntilPurgeComplete => "until_purge_complete",
        };
        Self {
            id: risk.id(),
            condition,
        }
    }
}
