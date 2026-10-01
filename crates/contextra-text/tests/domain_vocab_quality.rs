// FILE-CONTEXT: Qualitaetstest fuer domaenenspezifische Vokabulare (Medizin & Recht).
// ZWECK: Verifiziert Invarianten (keine Duplikate, Kleinschreibung, Disjunktheit, Mindestgroessen, exakte Sortierung) und Zerlegungsverhalten.

use contextra_text::domain::{
    legal_de::{LEGAL_COMPOUND_STEMS, LEGAL_PROTECTED_TERMS},
    medical_de::{MEDICAL_COMPOUND_STEMS, MEDICAL_PROTECTED_TERMS},
    DomainVocabulary, LegalDomainVocabulary, MedicalDomainVocabulary,
};
use contextra_text::morphology::{GermanCompoundSplitter, MorphologicalTokenizer};
use std::collections::HashSet;
use std::time::Instant;

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

    check_list("MEDICAL_COMPOUND_STEMS", *MEDICAL_COMPOUND_STEMS);
    check_list("MEDICAL_PROTECTED_TERMS", *MEDICAL_PROTECTED_TERMS);
    check_list("LEGAL_COMPOUND_STEMS", *LEGAL_COMPOUND_STEMS);
    check_list("LEGAL_PROTECTED_TERMS", *LEGAL_PROTECTED_TERMS);
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
        medical.compound_stems().len() >= 2000,
        "Medical stems count {} < 2000",
        medical.compound_stems().len()
    );
    assert!(
        medical.protected_terms().len() >= 100,
        "Medical protected terms count {} < 100",
        medical.protected_terms().len()
    );
    assert!(
        legal.compound_stems().len() >= 2000,
        "Legal stems count {} < 2000",
        legal.compound_stems().len()
    );
    assert!(
        legal.protected_terms().len() >= 100,
        "Legal protected terms count {} < 100",
        legal.protected_terms().len()
    );
}

#[test]
fn test_domain_vocabulary_sample_term_recognition() {
    let medical = MedicalDomainVocabulary;
    let legal = LegalDomainVocabulary;

    let med_stems_set: HashSet<_> = medical.compound_stems().iter().copied().collect();
    let med_prot_set: HashSet<_> = medical.protected_terms().iter().copied().collect();

    let sample_med_stems = ["acetylsalicylsaeure", "aorten", "entzuendung", "adipositas"];

    for stem in &sample_med_stems {
        assert!(
            med_stems_set.contains(stem),
            "Expected medical stem '{}' to be recognized in medical_de",
            stem
        );
    }

    let sample_med_prot = [
        "muskel", "herz", "arzt", "dosis", "ekg", "labor", "virus", "wunde",
    ];
    for prot in &sample_med_prot {
        assert!(
            med_prot_set.contains(prot),
            "Expected medical protected term '{}' to be recognized in medical_de",
            prot
        );
    }

    let leg_stems_set: HashSet<_> = legal.compound_stems().iter().copied().collect();
    let leg_prot_set: HashSet<_> = legal.protected_terms().iter().copied().collect();

    let sample_leg_stems = [
        "agb-recht",
        "bundesverfassungsgericht",
        "daten",
        "sozial",
        "behandlungs",
    ];

    for stem in &sample_leg_stems {
        assert!(
            leg_stems_set.contains(stem),
            "Expected legal stem '{}' to be recognized in legal_de",
            stem
        );
    }

    let sample_leg_prot = ["akte", "eid", "frist", "klage", "notar", "urteil"];
    for prot in &sample_leg_prot {
        assert!(
            leg_prot_set.contains(prot),
            "Expected legal protected term '{}' to be recognized in legal_de",
            prot
        );
    }
}

#[test]
fn test_domain_vocabulary_cold_start_loading_performance() {
    let start = Instant::now();

    let legal = LegalDomainVocabulary;
    let leg_stems = legal.compound_stems();
    let leg_prot = legal.protected_terms();

    let medical = MedicalDomainVocabulary;
    let med_stems = medical.compound_stems();
    let med_prot = medical.protected_terms();

    let elapsed = start.elapsed();

    println!(
        "Cold-start vocabulary parse & load time: {:?} (legal: {} stems, medical: {} stems)",
        elapsed,
        leg_stems.len(),
        med_stems.len()
    );

    assert!(
        elapsed.as_millis() < 100,
        "Cold-start loading time {:?} exceeded 100ms threshold",
        elapsed
    );
    assert!(!leg_stems.is_empty() && !leg_prot.is_empty());
    assert!(!med_stems.is_empty() && !med_prot.is_empty());
}

#[test]
fn test_compound_splitter_before_after_domain_expansion_comparison() {
    // Vorher: Basis-Splitter ohne Domänenvokabular (nur allgemeines KMU-Wörterbuch)
    let old_splitter = GermanCompoundSplitter::new();

    // Nachher: Erweiterter Splitter mit beiden Domänenvokabularen (Legal & Medical)
    let expanded_splitter = GermanCompoundSplitter::new_with_all_domains();

    // Medizinische Fachbegriff-Komposita (mindestens 5 konstruierte Testfälle)
    let med_test_cases = [
        "herzmuskelentzuendung",
        "aortenklappenstenose",
        "schilddruesenkarzinom",
        "magenschleimhautentzuendung",
        "bandscheibenvorfall",
    ];

    println!("\n=== VORHER / NACHHER VERGLEICH - MEDIZINISCHE KOMPOSITA ===");
    for &word in &med_test_cases {
        let old_res = old_splitter.decompose(word);
        let new_res = expanded_splitter.decompose(word);

        println!(
            "Wort: {:<30} | Vorher: {:<35} | Nachher: {:?}",
            word,
            format!("{:?}", old_res),
            new_res
        );

        // Nachher MUSS eine echtere / tiefere Zerlegung liefern als das unzerlegte Fallback des Basis-Splitters
        assert!(
            new_res.len() >= 2,
            "Expanded splitter failed to decompose medical compound '{}': got {:?}",
            word,
            new_res
        );
        assert!(
            new_res.len() >= old_res.len(),
            "Expanded splitter decomposition should be deeper or equal for '{}'",
            word
        );
    }

    // Juristische Fachbegriff-Komposita (mindestens 5 konstruierte Testfälle)
    let leg_test_cases = [
        "datenschutzgrundverordnung",
        "behandlungsfehleranspruch",
        "sozialgerichtsbarkeit",
        "straffreiheitserklaerung",
        "mietkautionrueckzahlung",
    ];

    println!("\n=== VORHER / NACHHER VERGLEICH - JURISTISCHE KOMPOSITA ===");
    for &word in &leg_test_cases {
        let old_res = old_splitter.decompose(word);
        let new_res = expanded_splitter.decompose(word);

        println!(
            "Wort: {:<30} | Vorher: {:<35} | Nachher: {:?}",
            word,
            format!("{:?}", old_res),
            new_res
        );

        assert!(
            new_res.len() >= 2,
            "Expanded splitter failed to decompose legal compound '{}': got {:?}",
            word,
            new_res
        );
        assert!(
            new_res.len() >= old_res.len(),
            "Expanded splitter decomposition should be deeper or equal for '{}'",
            word
        );
    }
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
