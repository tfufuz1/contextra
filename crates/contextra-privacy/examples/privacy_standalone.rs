//! Standalone-Beispiel für `contextra-privacy` (M10).
//! Demonstriert PII-Erkennung und Egress-Vaulting mit Surrogat-Ersetzung.

use contextra_privacy::{EgressVault, NoOpRecognizer, RuleBasedEntityRecognizer};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. PII-Erkennung mit regelbasiertem Erkenner
    let recognizer = RuleBasedEntityRecognizer::new();
    let text =
        "Mitarbeiter Max (max@example.com) nutzt IBAN DE89370400440532013000 und IP 192.168.1.1.";
    let recognized = recognizer.recognize(text);
    println!("Erkannte PII-Bereiche: {}", recognized.len());
    for (range, category) in &recognized {
        println!(
            " - Kategorie: {}, Wert: '{}'",
            category,
            &text[range.clone()]
        );
    }

    // 2. Initialisierung des EgressVault
    let vault = EgressVault::try_default()?;

    // 3. Surrogat-Ersetzung und Vaulting
    let (sanitized_text, count) = vault.sanitize_and_vault(text, &recognizer)?;
    println!("\nErsetzungen: {}", count);
    println!("Bereinigter Text:\n{}", sanitized_text);

    // 4. Null-Objekt-Erkenner (NoOp)
    let (noop_text, noop_count) = vault.sanitize_and_vault(text, &NoOpRecognizer)?;
    assert_eq!(noop_count, 0);
    assert_eq!(noop_text, text);

    Ok(())
}
