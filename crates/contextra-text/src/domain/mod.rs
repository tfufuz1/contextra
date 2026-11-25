//! Domänenspezifische Vokabularpakete für die erweiterte Morphologie.
//!
//! Lädt versionierte Wörterbücher aus den Crate-Ressourcendateien (`data/legal_de.txt`,
//! `data/medical_de.txt`) über `include_str!` und stellt sie mit performantem
//! In-Memory-Lookup (einmaliges Parsen via `OnceLock`) zur Verfügung.

pub mod legal_de;
pub mod medical_de;

pub use legal_de::LegalDomainVocabulary;
pub use medical_de::MedicalDomainVocabulary;

use crate::morphology::normalize_umlauts;
use std::collections::HashSet;

/// Trait für domänenspezifische Vokabularpakete.
pub trait DomainVocabulary: Send + Sync {
    /// Domänenspezifische Komposita-Bausteine (Ergänzung, kein Ersatz für das KMU-Basiswörterbuch).
    fn compound_stems(&self) -> &'static [&'static str];
    /// Fachbegriffe, die NICHT als Stopwort behandelt werden dürfen, selbst wenn sie kurz/häufig sind
    /// (z. B. "Klage", "Akte" im juristischen Kontext).
    fn protected_terms(&self) -> &'static [&'static str];
    /// Kurzname für Logging/Konfiguration, z. B. "legal_de", "medical_de".
    fn domain_id(&self) -> &'static str;
}

pub(crate) struct ParsedVocabulary {
    pub compound_stems: &'static [&'static str],
    pub protected_terms: &'static [&'static str],
}

pub(crate) fn parse_domain_resource(raw: &'static str) -> ParsedVocabulary {
    let mut current_section = "stems";
    let mut stems_vec: Vec<&'static str> = Vec::new();
    let mut protected_vec: Vec<&'static str> = Vec::new();

    for line in raw.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        if trimmed == "[compound_stems]" {
            current_section = "stems";
            continue;
        }
        if trimmed == "[protected_terms]" {
            current_section = "protected";
            continue;
        }

        let norm = normalize_umlauts(trimmed);
        if norm.len() < 2 {
            continue;
        }

        let leaked_str: &'static str = Box::leak(norm.into_boxed_str());
        match current_section {
            "stems" => stems_vec.push(leaked_str),
            "protected" => protected_vec.push(leaked_str),
            _ => stems_vec.push(leaked_str),
        }
    }

    stems_vec.sort_unstable();
    stems_vec.dedup();
    protected_vec.sort_unstable();
    protected_vec.dedup();

    let protected_set: HashSet<&'static str> = protected_vec.iter().copied().collect();
    stems_vec.retain(|stem| !protected_set.contains(stem));

    let static_stems: &'static [&'static str] = Box::leak(stems_vec.into_boxed_slice());
    let static_protected: &'static [&'static str] = Box::leak(protected_vec.into_boxed_slice());

    ParsedVocabulary {
        compound_stems: static_stems,
        protected_terms: static_protected,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn verify_vocabulary_invariants<V: DomainVocabulary>(vocab: &V, expected_id: &str) {
        assert_eq!(vocab.domain_id(), expected_id);

        let stems = vocab.compound_stems();
        assert!(
            stems.len() >= 2000,
            "Domain {} must have at least 2000 compound stems, found {}",
            expected_id,
            stems.len()
        );

        // Verify compound stems are alphabetically sorted and duplicate-free
        assert!(
            stems.windows(2).all(|w| w[0] < w[1]),
            "Domain {} compound stems are not strictly sorted or contain duplicates: {:?}",
            expected_id,
            stems
        );

        let protected = vocab.protected_terms();
        assert!(
            !protected.is_empty(),
            "Domain {} protected terms must not be empty",
            expected_id
        );

        // Verify protected terms are alphabetically sorted and duplicate-free
        assert!(
            protected.windows(2).all(|w| w[0] < w[1]),
            "Domain {} protected terms are not strictly sorted or contain duplicates: {:?}",
            expected_id,
            protected
        );
    }

    #[test]
    fn test_legal_domain_vocabulary_invariants() {
        let legal = LegalDomainVocabulary;
        verify_vocabulary_invariants(&legal, "legal_de");
    }

    #[test]
    fn test_medical_domain_vocabulary_invariants() {
        let medical = MedicalDomainVocabulary;
        verify_vocabulary_invariants(&medical, "medical_de");
    }
}
