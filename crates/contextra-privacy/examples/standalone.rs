//! Minimales Standalone-Beispiel für `contextra-privacy` (<= 30 Zeilen).
//! Demonstriert PII-Erkennung und Egress-Vaulting mit Surrogat-Ersetzung.

use contextra_privacy::{EgressVault, RuleBasedEntityRecognizer};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let recognizer = RuleBasedEntityRecognizer::new();
    let vault = EgressVault::try_default()?;

    let text = "Mitarbeiter Max (max@corp.de) nutzt IBAN DE89370400440532013000.";
    let (sanitized, count) = vault.sanitize_and_vault(text, &recognizer)?;

    println!("Ersetzungen: {count}");
    println!("Sanitized: {sanitized}");
    Ok(())
}
