//! Domänenspezifische Vokabularpakete für die erweiterte Morphologie.
//!
//! **Hinweis zur Verdrahtung**: Diese Pakete sind aktuell NICHT automatisch in
//! `GermanCompoundSplitter` oder `get_german_stopwords` eingehängt (kein automatischer
//! Seiteneffekt auf bestehendes Verhalten). Die Verdrahtung in den eigentlichen
//! Splitter/Stopwort-Pfad ist ein bewusst separater Folge-Task, weil er die bestehende,
//! bereits gehärtete Morphologie-Logik verändern würde und damit außerhalb des isolierten
//! Scopes dieses Tasks liegt.

pub mod legal_de;
pub mod medical_de;

pub use legal_de::LegalDomainVocabulary;
pub use medical_de::MedicalDomainVocabulary;

/// Trait für domänenspezifische Vokabularpakete.
pub trait DomainVocabulary {
    /// Domänenspezifische Komposita-Bausteine (Ergänzung, kein Ersatz für das KMU-Basiswörterbuch).
    fn compound_stems(&self) -> &'static [&'static str];
    /// Fachbegriffe, die NICHT als Stopwort behandelt werden dürfen, selbst wenn sie kurz/häufig sind
    /// (z. B. "Klage", "Akte" im juristischen Kontext).
    fn protected_terms(&self) -> &'static [&'static str];
    /// Kurzname für Logging/Konfiguration, z. B. "legal_de", "medical_de".
    fn domain_id(&self) -> &'static str;
}

#[cfg(test)]
mod tests {
    use super::*;

    fn verify_vocabulary_invariants<V: DomainVocabulary>(vocab: &V, expected_id: &str) {
        assert_eq!(vocab.domain_id(), expected_id);

        let stems = vocab.compound_stems();
        assert!(
            stems.len() >= 40,
            "Domain {} must have at least 40 compound stems, found {}",
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
