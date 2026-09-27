use contextra_text::morphology::{normalize_umlauts, GermanCompoundSplitter, MorphologicalTokenizer};
use contextra_text::tokenizer::{GermanMorphTokenizer, Tokenizer};

#[test]
fn inspect_20_compounds() {
    let splitter = GermanCompoundSplitter::new();
    let tokenizer = GermanMorphTokenizer::new();

    let words = vec![
        "Datenschutzgrundverordnung",
        "Bundesdatenschutzgesetz",
        "Softwarearchitektur",
        "Computerprogramm",
        "Kraftfahrzeugsteuer",
        "Bundesverfassungsgericht",
        "Informationstechnologie",
        "Datenbankmanagement",
        "Künstliche Intelligenz",
        "Maschinelles Lernen",
        "Arbeitnehmerüberlassungsgesetz",
        "Telekommunikationsgesetz",
        "Finanzdienstleistungsaufsicht",
        "Umweltschutzorganisation",
        "Qualitätsmanagementsystem",
        "Kundenbeziehungsmanagement",
        "Lieferkettensorgfaltspflichtengeschäft",
        "Kraftfahrzeug-Haftpflichtversicherung",
        "Betriebsratsvorsitzender",
        "Urheberrechtsreform",
    ];

    println!("\n=== 20 GERMAN COMPOUNDS EVALUATION ===");
    for word in words {
        let norm = normalize_umlauts(&word.to_lowercase());
        let decomp = splitter.decompose(&norm);
        let tokens = tokenizer.tokenize(word);
        println!("Word: '{}' | norm: '{}'", word, norm);
        println!("  Splitter decomp: {:?}", decomp);
        println!("  Tokenizer tokens: {:?}", tokens);
    }
}

#[test]
fn inspect_umlaut_bimap() {
    let tokenizer = GermanMorphTokenizer::new();

    let pairs = vec![
        ("Bär", "Baer"),
        ("Öl", "Oel"),
        ("Über", "Ueber"),
        ("Straße", "Strasse"),
    ];

    println!("\n=== UMLAUT BIMAP EVALUATION ===");
    for (umlaut_word, ae_word) in pairs {
        let tok_u = tokenizer.tokenize(umlaut_word);
        let tok_a = tokenizer.tokenize(ae_word);
        println!("Word: '{}' -> Tokens: {:?}", umlaut_word, tok_u);
        println!("Word: '{}' -> Tokens: {:?}", ae_word, tok_a);

        // Check if query for tok_a matches index of tok_u
        let match_u_in_a = tok_a.iter().any(|t| tok_u.contains(t));
        // Check if query for tok_u matches index of tok_a
        let match_a_in_u = tok_u.iter().any(|t| tok_a.contains(t));

        println!("  Match '{}' in indexed '{}': {}", ae_word, umlaut_word, match_u_in_a);
        println!("  Match '{}' in indexed '{}': {}", umlaut_word, ae_word, match_a_in_u);
    }
}
