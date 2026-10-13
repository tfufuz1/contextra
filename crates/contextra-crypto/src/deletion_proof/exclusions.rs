//! Pflichteinschränkungen / Mandatory Exclusions für Deletion Witnesses (C7).

#![forbid(unsafe_code)]

use crate::deletion_proof::{claim_text::WitAssurance, types::ExcludedScope};

/// Pflicht-Exclusions-Liste aller standardmäßig ausgeschlossenen Scopes.
pub const WIT_MANDATORY_EXCLUSIONS: &[ExcludedScope] = &[
    ExcludedScope::StorageMediaRemanence,
    ExcludedScope::UnregisteredBackups,
    ExcludedScope::OsPageCacheSwapCoreDumps,
    ExcludedScope::ClientAndModelContext,
    ExcludedScope::ConsolidatedAndDistilled,
    ExcludedScope::LlmParameterMemory,
    ExcludedScope::AggregateDerivedStatistics,
    ExcludedScope::LogsAndTelemetryOutsideProcess,
    ExcludedScope::ExportedData,
];

/// Gibt die Liste der Pflicht-Exclusions für eine bestimmte Assurance-Stufe zurück.
/// `StorageMediaRemanence` wird NUR bei Assurance S4 entfernt.
pub fn mandatory_exclusions_for(assurance: WitAssurance) -> Vec<ExcludedScope> {
    WIT_MANDATORY_EXCLUSIONS
        .iter()
        .filter(|&scope| {
            !(*scope == ExcludedScope::StorageMediaRemanence && assurance == WitAssurance::S4)
        })
        .cloned()
        .collect()
}

/// Gibt die numerischen IDs aller Pflicht-Exclusions zurück.
pub fn mandatory_ids() -> Vec<u16> {
    WIT_MANDATORY_EXCLUSIONS.iter().map(|e| e.id()).collect()
}
