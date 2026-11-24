//! Tests für Evasion-Angriffsvektoren (Zalgo, Bidi-Controls, erweiterte Homoglyphen).
#![forbid(unsafe_code)]
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]

use contextra_privacy::RuleBasedEntityRecognizer;

#[test]
fn test_evasion_zalgo_combining_characters() {
    let recognizer = RuleBasedEntityRecognizer::new();
    // "admin@example.com" verunreinigt mit Combining Diacritical Marks (Zalgo)
    let zalgo_email = "a\u{0300}\u{0301}d\u{0302}m\u{0303}i\u{0304}n\u{0305}@\u{0306}e\u{0307}x\u{0308}a\u{0309}m\u{030A}p\u{030B}l\u{030C}e\u{030D}.\u{030E}c\u{030F}o\u{0310}m\u{0311}";
    let results = recognizer.recognize(zalgo_email);

    assert_eq!(
        results.len(),
        1,
        "Zalgo-obfuskierte E-Mail muss erkannt werden"
    );
    assert_eq!(results[0].1, "EMAIL");
    assert_eq!(&zalgo_email[results[0].0.clone()], zalgo_email);
}

#[test]
fn test_evasion_bidi_isolate_and_override_controls() {
    let recognizer = RuleBasedEntityRecognizer::new();
    // E-Mail durchsetzt mit Bidi-Controls (\u{202E} RLO, \u{2066} LRI, \u{2069} PDI)
    let bidi_email = "a\u{202E}d\u{2066}m\u{2069}in@example.com";
    let results = recognizer.recognize(bidi_email);

    assert_eq!(
        results.len(),
        1,
        "Bidi-obfuskierte E-Mail muss erkannt werden"
    );
    assert_eq!(results[0].1, "EMAIL");
}

#[test]
fn test_evasion_math_bold_homoglyphs() {
    let recognizer = RuleBasedEntityRecognizer::new();
    // "DE" in Mathematical Bold (\u{1D403}\u{1D404}) + IBAN Ziffern
    // DE89 3704 0044 0532 0130 00 -> gültige IBAN
    let math_iban = "🇩🇪 \u{1D403}\u{1D404}89370400440532013000";
    let results = recognizer.recognize(math_iban);

    assert_eq!(results.len(), 1, "Math-Bold-IBAN muss erkannt werden");
    assert_eq!(results[0].1, "IBAN");
}
