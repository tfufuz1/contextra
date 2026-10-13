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

/// Gibt die numerischen IDs aller Pflicht-Exclusions für eine bestimmte Assurance-Stufe zurück.
pub fn mandatory_ids_for(assurance: WitAssurance) -> Vec<u16> {
    mandatory_exclusions_for(assurance)
        .iter()
        .map(|e| e.id())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mandatory_ids_non_empty_and_matching() {
        let ids = mandatory_ids();
        assert_eq!(ids.len(), WIT_MANDATORY_EXCLUSIONS.len());
        assert!(ids.contains(&3)); // StorageMediaRemanence is ID 3

        let ids_s4 = mandatory_ids_for(WitAssurance::S4);
        assert_eq!(ids_s4.len(), WIT_MANDATORY_EXCLUSIONS.len() - 1);
        assert!(!ids_s4.contains(&3)); // StorageMediaRemanence (id 3) removed in S4
    }
}
