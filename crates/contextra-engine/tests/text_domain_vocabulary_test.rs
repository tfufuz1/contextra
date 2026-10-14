#![cfg(not(loom))]
use contextra_text::tokenizer::{GermanMorphTokenizer, Tokenizer};

#[test]
fn test_legal_domain_vocabulary_tokenization() {
    let tokenizer = GermanMorphTokenizer::new();

    // Legal compound term: "Bundesverfassungsgericht" should decompose into "bundes", "verfassungs", "gericht"
    let tokens = tokenizer.tokenize("Bundesverfassungsgericht");

    assert!(
        tokens.contains(&"bundesverfassungsgericht".to_string()),
        "Tokens must contain full compound word: {:?}",
        tokens
    );
    assert!(
        tokens.contains(&"gericht".to_string()),
        "Tokens must contain decomposed legal component 'gericht': {:?}",
        tokens
    );

    // Verify panic-freedom on arbitrary legal text
    let legal_text = "Gemäß § 12 Abs. 2 DSGVO i.V.m. Art. 6 Abs. 1 lit. f DS-GVO ist die Datenverarbeitung zulässig.";
    let legal_tokens = tokenizer.tokenize(legal_text);
    assert!(!legal_tokens.is_empty(), "Legal text tokenization should return tokens");
}
