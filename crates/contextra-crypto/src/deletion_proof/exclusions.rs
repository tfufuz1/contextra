//! Pflichteinschränkungen / Mandatory Exclusions für Deletion Witnesses (C7).

#![forbid(unsafe_code)]

use crate::deletion_proof::{claim_text::WitAssurance, types::ExcludedScope};

/// Pflicht-Exclusions-Liste aller standardmäßig ausgeschlossenen Scopes.
pub const WIT_MANDATORY_EXCLUSIONS: &[ExcludedScope] = &[
    ExcludedScope::ConsolidatedAndDistilled,
    ExcludedScope::LlmParameterMemory,
    ExcludedScope::StorageMediaRemanence,
    ExcludedScope::UnregisteredBackups,
    ExcludedScope::OsPageCacheSwapCoreDumps,
    ExcludedScope::ClientAndModelContext,
    ExcludedScope::AggregateDerivedStatistics,
    ExcludedScope::LogsAndTelemetryOutsideProcess,
    ExcludedScope::ExportedData,
];

/// Sortiert eine Liste von ExcludedScope-Instanzen deterministisch nach ihrer kanonischen ID.
pub fn sort_exclusions_canonically(exclusions: &mut [ExcludedScope]) {
    exclusions.sort_by_key(|e| e.id());
}

/// Gibt die deterministisch nach ID sortierten Pflicht-Exclusions zurück.
pub fn deterministic_mandatory_exclusions() -> Vec<ExcludedScope> {
    let mut list = WIT_MANDATORY_EXCLUSIONS.to_vec();
    sort_exclusions_canonically(&mut list);
    list
}

/// Gibt die deterministisch sortierten Pflicht-Exclusions für eine bestimmte Assurance-Stufe zurück.
/// `StorageMediaRemanence` wird NUR bei Assurance S4 entfernt.
pub fn mandatory_exclusions_for(assurance: WitAssurance) -> Vec<ExcludedScope> {
    let mut list: Vec<ExcludedScope> = WIT_MANDATORY_EXCLUSIONS
        .iter()
        .filter(|&scope| {
            !(*scope == ExcludedScope::StorageMediaRemanence && assurance == WitAssurance::S4)
        })
        .cloned()
        .collect();

    sort_exclusions_canonically(&mut list);
    list
}

/// Gibt die numerischen IDs aller Pflicht-Exclusions in deterministisch sortierter Reihenfolge zurück.
pub fn mandatory_ids() -> Vec<u16> {
    deterministic_mandatory_exclusions()
        .iter()
        .map(|e| e.id())
        .collect()
}

/// Gibt die numerischen IDs aller Pflicht-Exclusions für eine bestimmte Assurance-Stufe in deterministisch sortierter Reihenfolge zurück.
pub fn mandatory_ids_for(assurance: WitAssurance) -> Vec<u16> {
    mandatory_exclusions_for(assurance)
        .iter()
        .map(|e| e.id())
        .collect()
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::indexing_slicing)]

    use super::*;

    #[test]
    fn test_mandatory_ids_non_empty_and_matching() {
        let ids = mandatory_ids();
        assert_eq!(ids.len(), WIT_MANDATORY_EXCLUSIONS.len());
        assert!(ids.contains(&3)); // StorageMediaRemanence is ID 3

        let ids_s4 = mandatory_ids_for(WitAssurance::S4);
        assert_eq!(ids_s4.len(), WIT_MANDATORY_EXCLUSIONS.len() - 1);
        assert!(!ids_s4.contains(&3)); // StorageMediaRemanence (id 3) removed in S4

        // Verify strictly sorted order by ID
        let mut sorted_ids = ids.clone();
        sorted_ids.sort();
        assert_eq!(ids, sorted_ids);
    }
}
