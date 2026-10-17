use contextra_text::tokenizer::{DefaultTokenizer, GermanMorphTokenizer, Tokenizer};

#[test]
fn test_protected_regex_german_compounds_integration() {
    let input = "Senden Sie Ihre Anträge für Software-Entwicklung an dev-team@beispiel.de oder besuchen Sie https://beispiel.de/datenschutz-richtlinien.";

    let default_tok = DefaultTokenizer;
    let def_tokens = default_tok.tokenize(input);

    assert!(
        def_tokens.contains(&"dev-team@beispiel.de".to_string()),
        "DefaultTokenizer should protect email. Got: {:?}",
        def_tokens
    );
    assert!(
        def_tokens.contains(&"https://beispiel.de/datenschutz-richtlinien".to_string()),
        "DefaultTokenizer should protect URL. Got: {:?}",
        def_tokens
    );

    let german_tok = GermanMorphTokenizer::new();
    let ger_tokens = german_tok.tokenize(input);

    assert!(
        ger_tokens.contains(&"dev-team@beispiel.de".to_string()),
        "GermanMorphTokenizer should protect email. Got: {:?}",
        ger_tokens
    );
    assert!(
        ger_tokens.contains(&"https://beispiel.de/datenschutz-richtlinien".to_string()),
        "GermanMorphTokenizer should protect URL. Got: {:?}",
        ger_tokens
    );

    // Verify German hyphenated compound decomposing
    assert!(
        ger_tokens
            .iter()
            .any(|t| t.contains("software") || t.contains("entwicklung")),
        "GermanMorphTokenizer should decompose Software-Entwicklung. Got: {:?}",
        ger_tokens
    );
}
