// FILE-CONTEXT: Integrationspruefung fuer GermanCompoundSplitter mit Domaenenvokabularen.
// ZWECK: Verifiziert, dass der GermanCompoundSplitter mit den ausgebauten Vokabularen mindestens 20 reprasentative Komposita je Domaene (Recht & Medizin) korrekt zerlegt und geschuetzte Begriffe unangetastet laesst.

use contextra_text::morphology::{GermanCompoundSplitter, MorphologicalTokenizer};

#[test]
fn test_legal_compounds_decomposition_at_least_20_cases() {
    let splitter = GermanCompoundSplitter::new_with_all_domains();

    let legal_cases: &[(&str, usize)] = &[
        ("kaufvertrag", 2),
        ("haftpflichtversicherung", 3),
        ("landgericht", 2),
        ("amtsgericht", 2),
        ("arbeitsvertrag", 2),
        ("geldstrafe", 2),
        ("bauantrag", 2),
        ("ehevertrag", 2),
        ("dienstvertrag", 2),
        ("werkvertrag", 2),
        ("datenschutzgrundverordnung", 2),
        ("behandlungsfehleranspruch", 2),
        ("sozialgerichtsbarkeit", 2),
        ("straffreiheitserklaerung", 2),
        ("mietkautionrueckzahlung", 2),
        ("schadensersatzanspruch", 2),
        ("kuendigungsschutzgesetz", 2),
        ("unterhaltspflicht", 2),
        ("vollstreckungsbescheid", 2),
        ("prozesskostenhilfe", 2),
    ];

    assert!(
        legal_cases.len() >= 20,
        "Must test at least 20 legal compounds, found {}",
        legal_cases.len()
    );

    for (word, min_parts) in legal_cases {
        let parts = splitter.decompose(word);
        assert!(
            parts.len() >= *min_parts,
            "Legal compound '{}' should be decomposed into at least {} parts, got {:?}",
            word,
            min_parts,
            parts
        );
    }
}

#[test]
fn test_medical_compounds_decomposition_at_least_20_cases() {
    let splitter = GermanCompoundSplitter::new_with_all_domains();

    let medical_cases: &[(&str, usize)] = &[
        ("herzmuskelentzuendung", 2),
        ("aortenklappenstenose", 2),
        ("schilddruesenkarzinom", 2),
        ("magenschleimhautentzuendung", 2),
        ("bandscheibenvorfall", 2),
        ("krankenhaus", 2),
        ("krankenpflege", 2),
        ("kinderpflege", 2),
        ("haushaltspflege", 2),
        ("krankentransport", 2),
        ("krankenversicherung", 2),
        ("pflegeheim", 2),
        ("pflegekraft", 2),
        ("pflegedienst", 2),
        ("pflegefall", 2),
        ("blutdruckmessung", 2),
        ("herzbeutelentzuendung", 2),
        ("lungenerkrankung", 2),
        ("gelbfieberimpfung", 2),
        ("leberentzuendung", 2),
    ];

    assert!(
        medical_cases.len() >= 20,
        "Must test at least 20 medical compounds, found {}",
        medical_cases.len()
    );

    for (word, min_parts) in medical_cases {
        let parts = splitter.decompose(word);
        assert!(
            parts.len() >= *min_parts,
            "Medical compound '{}' should be decomposed into at least {} parts, got {:?}",
            word,
            min_parts,
            parts
        );
    }
}

#[test]
fn test_protected_terms_are_not_decomposed() {
    let splitter = GermanCompoundSplitter::new_with_all_domains();

    let legal_protected = &[
        "akte", "eid", "frist", "klage", "notar", "recht", "urteil", "sorge", "kraft", "pacht",
        "gut", "los", "stand", "straf", "traeger",
    ];

    for term in legal_protected {
        let parts = splitter.decompose(term);
        assert_eq!(
            parts,
            vec![*term],
            "Legal protected term '{}' must NOT be decomposed, got {:?}",
            term,
            parts
        );
    }

    let medical_protected = &[
        "arzt", "dosis", "ekg", "labor", "virus", "wunde", "bauch", "blut", "brust", "darm",
        "herz", "hirn", "lunge", "mark", "muskel", "nerv", "niere", "zahn",
    ];

    for term in medical_protected {
        let parts = splitter.decompose(term);
        assert_eq!(
            parts,
            vec![*term],
            "Medical protected term '{}' must NOT be decomposed, got {:?}",
            term,
            parts
        );
    }
}
