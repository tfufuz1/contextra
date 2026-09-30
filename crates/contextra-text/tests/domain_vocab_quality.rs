// FILE-CONTEXT: Qualitaetstest fuer domaenenspezifische Vokabulare (Medizin & Recht).
// ZWECK: Verifiziert Invarianten (keine Duplikate, Kleinschreibung, Disjunktheit, Mindestgroessen, exakte Sortierung) und Zerlegungsverhalten.

use contextra_text::domain::{
    legal_de::{LEGAL_COMPOUND_STEMS, LEGAL_PROTECTED_TERMS},
    medical_de::{MEDICAL_COMPOUND_STEMS, MEDICAL_PROTECTED_TERMS},
    DomainVocabulary, LegalDomainVocabulary, MedicalDomainVocabulary,
};
use contextra_text::morphology::{GermanCompoundSplitter, MorphologicalTokenizer};
use std::collections::HashSet;

#[test]
fn test_no_duplicates_and_valid_formatting() {
    let check_list = |name: &str, list: &[&str]| {
        let mut seen = HashSet::new();
        for &entry in list {
            assert!(!entry.is_empty(), "Entry in {} must not be empty", name);
            assert_eq!(
                entry,
                entry.trim(),
                "Entry '{}' in {} has leading or trailing whitespace",
                entry,
                name
            );
            assert_eq!(
                entry,
                entry.to_lowercase(),
                "Entry '{}' in {} must be lowercase",
                entry,
                name
            );
            assert!(
                seen.insert(entry),
                "Duplicate entry '{}' found in {}",
                entry,
                name
            );
        }

        // Verify strict alphabetical order
        for window in list.windows(2) {
            assert!(
                window[0] < window[1],
                "List {} is not strictly sorted alphabetically: '{}' >= '{}'",
                name,
                window[0],
                window[1]
            );
        }
    };

    let medical = MedicalDomainVocabulary;
    let legal = LegalDomainVocabulary;

    check_list("medical compound stems", medical.compound_stems());
    check_list("medical protected terms", medical.protected_terms());
    check_list("legal compound stems", legal.compound_stems());
    check_list("legal protected terms", legal.protected_terms());

    check_list("MEDICAL_COMPOUND_STEMS", MEDICAL_COMPOUND_STEMS);
    check_list("MEDICAL_PROTECTED_TERMS", MEDICAL_PROTECTED_TERMS);
    check_list("LEGAL_COMPOUND_STEMS", LEGAL_COMPOUND_STEMS);
    check_list("LEGAL_PROTECTED_TERMS", LEGAL_PROTECTED_TERMS);
}

#[test]
fn test_stems_and_protected_terms_disjoint() {
    let medical = MedicalDomainVocabulary;
    let legal = LegalDomainVocabulary;

    let med_stems: HashSet<_> = medical.compound_stems().iter().copied().collect();
    let med_prot: HashSet<_> = medical.protected_terms().iter().copied().collect();
    let overlap_med: Vec<_> = med_stems.intersection(&med_prot).collect();
    assert!(
        overlap_med.is_empty(),
        "Medical stems and protected terms overlap: {:?}",
        overlap_med
    );

    let leg_stems: HashSet<_> = legal.compound_stems().iter().copied().collect();
    let leg_prot: HashSet<_> = legal.protected_terms().iter().copied().collect();
    let overlap_leg: Vec<_> = leg_stems.intersection(&leg_prot).collect();
    assert!(
        overlap_leg.is_empty(),
        "Legal stems and protected terms overlap: {:?}",
        overlap_leg
    );
}

#[test]
fn test_minimum_list_sizes() {
    let medical = MedicalDomainVocabulary;
    let legal = LegalDomainVocabulary;

    assert!(
        medical.compound_stems().len() >= 300,
        "Medical stems count {} < 300",
        medical.compound_stems().len()
    );
    assert!(
        medical.protected_terms().len() >= 150,
        "Medical protected terms count {} < 150",
        medical.protected_terms().len()
    );
    assert!(
        legal.compound_stems().len() >= 300,
        "Legal stems count {} < 300",
        legal.compound_stems().len()
    );
    assert!(
        legal.protected_terms().len() >= 150,
        "Legal protected terms count {} < 150",
        legal.protected_terms().len()
    );
}

#[test]
fn test_compound_splitter_sample_decomp() {
    let splitter = GermanCompoundSplitter::new();

    // Medical compounds (at least 10 sample compounds)
    let med_compounds: &[(&str, &[&str])] = &[
        ("krankenhaus", &["kranken", "haus"]),
        ("krankenpflege", &["kranken", "pflege"]),
        ("kinderpflege", &["kinder", "pflege"]),
        ("haushaltspflege", &["haus", "halts", "pflege"]),
        ("krankentransport", &["kranken", "transport"]),
        ("krankenversicherung", &["kranken", "versicherung"]),
        ("pflegeheim", &["pflege", "heim"]),
        ("pflegekraft", &["pflege", "kraft"]),
        ("pflegedienst", &["pflege", "dienst"]),
        ("pflegefall", &["pflege", "fall"]),
    ];

    for (word, expected) in med_compounds {
        let actual = splitter.decompose(word);
        assert_eq!(
            &actual, expected,
            "Decomposition mismatch for medical compound '{}': actual {:?}, expected {:?}",
            word, actual, expected
        );
    }

    // Medical protected terms (at least 5 terms NOT split)
    let med_protected = &["arzt", "dosis", "ekg", "labor", "virus", "wunde"];
    for term in med_protected {
        let actual = splitter.decompose(term);
        assert_eq!(
            actual,
            vec![*term],
            "Medical protected term '{}' should NOT be split, got {:?}",
            term,
            actual
        );
    }

    // Legal compounds (at least 10 sample compounds)
    let leg_compounds: &[(&str, &[&str])] = &[
        ("kaufvertrag", &["kauf", "vertrag"]),
        (
            "haftpflichtversicherung",
            &["haft", "pflicht", "versicherung"],
        ),
        ("landgericht", &["land", "gericht"]),
        ("amtsgericht", &["amts", "gericht"]),
        ("arbeitsvertrag", &["arbeits", "vertrag"]),
        ("geldstrafe", &["geld", "strafe"]),
        ("bauantrag", &["bau", "antrag"]),
        ("ehevertrag", &["ehe", "vertrag"]),
        ("dienstvertrag", &["dienst", "vertrag"]),
        ("werkvertrag", &["werk", "vertrag"]),
    ];

    for (word, expected) in leg_compounds {
        let actual = splitter.decompose(word);
        assert_eq!(
            &actual, expected,
            "Decomposition mismatch for legal compound '{}': actual {:?}, expected {:?}",
            word, actual, expected
        );
    }

    // Legal protected terms (at least 5 terms NOT split)
    let leg_protected = &["akte", "eid", "frist", "klage", "notar", "recht", "urteil"];
    for term in leg_protected {
        let actual = splitter.decompose(term);
        assert_eq!(
            actual,
            vec![*term],
            "Legal protected term '{}' should NOT be split, got {:?}",
            term,
            actual
        );
    }
}
