//! Eigenschafts- und Vektortests für den regelbasierten Entitäten-Erkenner.
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing,
    clippy::disallowed_types,
    clippy::cast_possible_truncation
)]
//! Synthetic test suite and property-based panic-freedom tests for PatternRecognizer,
//! IBAN mod 97, BIC, Steuer-ID mod 11, Luhn credit cards, PLZ+Ort, overlong inputs,
//! and Surrogation -> Re-Identification roundtrips.

use contextra_privacy::{EgressVault, PatternRecognizer};

// ============================================================================
// 1. IBAN Mod 97 Tests
// ============================================================================

#[test]
// ANCHOR[contextra-privacy:TEST-PROP-001]
// Prüft: Prüft synthetische IBAN-Testvektoren mit Modulo-97-Validierung.
fn test_iban_mod_97_valid_and_invalid_synthetic_vectors() {
    let recognizer = PatternRecognizer::new();

    // Valid synthetic DE IBAN
    let valid_de = "DE89370400440532013000"; // Mod 97 valid
    let text = format!("Konto {valid_de} überweisen");
    let results = recognizer.recognize(&text);
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].1, "IBAN");
    assert_eq!(&text[results[0].0.clone()], valid_de);

    // Invalid DE IBAN (tampered check digit: DE88 instead of DE89)
    let invalid_de = "DE88370400440532013000";
    let bad_text = format!("Konto {invalid_de} fehlerhaft");
    let bad_results = recognizer.recognize(&bad_text);
    assert!(bad_results.iter().all(|r| r.1 != "IBAN"));
}

// ============================================================================
// 2. BIC / SWIFT Code Tests
// ============================================================================

#[test]
// ANCHOR[contextra-privacy:TEST-PROP-002]
// Prüft: Prüft synthetische BIC/SWIFT-Vektoren auf ISO-Konformität.
fn test_bic_valid_and_invalid_synthetic_vectors() {
    let recognizer = PatternRecognizer::new();

    // 11-character valid BIC
    let text11 = "BIC Code DEUTDED1800 verarbeiten";
    let res11 = recognizer.recognize(text11);
    assert_eq!(res11.len(), 1);
    assert_eq!(res11[0].1, "BIC");
    assert_eq!(&text11[res11[0].0.clone()], "DEUTDED1800");

    // 8-character valid BIC
    let text8 = "Bank BIC DEUTDED1 verwenden";
    let res8 = recognizer.recognize(text8);
    assert_eq!(res8.len(), 1);
    assert_eq!(res8[0].1, "BIC");
    assert_eq!(&text8[res8[0].0.clone()], "DEUTDED1");

    // Invalid BIC (invalid country code / length)
    let text_bad = "Code DEUT1234 invalid";
    let res_bad = recognizer.recognize(text_bad);
    assert!(res_bad.iter().all(|r| r.1 != "BIC"));
}

// ============================================================================
// 3. German Steuer-ID Mod 11 Tests
// ============================================================================

#[test]
// ANCHOR[contextra-privacy:TEST-PROP-004]
// Prüft: Prüft deutsche Steuer-ID-Vektoren inklusive Ziffernhäufigkeit und Prüfziffer.
fn test_steuer_id_valid_and_invalid_synthetic_vectors() {
    let recognizer = PatternRecognizer::new();

    // Valid synthetic Steuer-ID (86095742116)
    let text_valid = "Steuer-ID 86095742116 eintragen";
    let res_valid = recognizer.recognize(text_valid);
    assert_eq!(res_valid.len(), 1);
    assert_eq!(res_valid[0].1, "STEUER_ID");
    assert_eq!(&text_valid[res_valid[0].0.clone()], "86095742116");

    // Invalid Steuer-ID (tampered checksum)
    let text_invalid = "Steuer-ID 86095742117 falsch";
    let res_invalid = recognizer.recognize(text_invalid);
    assert!(res_invalid.iter().all(|r| r.1 != "STEUER_ID"));

    // Invalid Steuer-ID (leading zero)
    let text_zero = "Steuer-ID 06095742116 mit Null";
    let res_zero = recognizer.recognize(text_zero);
    assert!(res_zero.iter().all(|r| r.1 != "STEUER_ID"));
}

// ============================================================================
// 4. Credit Card Luhn Algorithm Tests
// ============================================================================

#[test]
// ANCHOR[contextra-privacy:TEST-PROP-003]
// Prüft: Prüft Kreditkarten-Prüfsummen mittels Luhn-Algorithmus.
fn test_credit_card_luhn_valid_and_invalid_synthetic_vectors() {
    let recognizer = PatternRecognizer::new();

    // Valid Visa test card
    let text_valid = "Kreditkarte 4532820011223344 abrechnen";
    let res_valid = recognizer.recognize(text_valid);
    assert_eq!(res_valid.len(), 1);
    assert_eq!(res_valid[0].1, "CREDIT_CARD");
    assert_eq!(&text_valid[res_valid[0].0.clone()], "4532820011223344");

    // Invalid Luhn checksum
    let text_invalid = "Kreditkarte 4532820011223345 fehlerhaft";
    let res_invalid = recognizer.recognize(text_invalid);
    assert!(res_invalid.iter().all(|r| r.1 != "CREDIT_CARD"));
}

// ============================================================================
// 5. DE PLZ + Ort Tests
// ============================================================================

#[test]
// ANCHOR[contextra-privacy:TEST-PROP-005]
// Prüft: Prüft die Erkennung von Postleitzahlen und Ortszusätzen.
fn test_plz_and_plz_ort_detection() {
    let recognizer = PatternRecognizer::new();

    // Standalone PLZ
    let text_plz = "PLZ 10115 in Deutschland";
    let res_plz = recognizer.recognize(text_plz);
    assert_eq!(res_plz.len(), 1);
    assert_eq!(res_plz[0].1, "PLZ");
    assert_eq!(&text_plz[res_plz[0].0.clone()], "10115");

    // PLZ + Ort (Berlin)
    let text_berlin = "Adresse 10115 Berlin Mitte";
    let res_berlin = recognizer.recognize(text_berlin);
    assert_eq!(res_berlin.len(), 1);
    assert_eq!(res_berlin[0].1, "PLZ_ORT");
    assert_eq!(&text_berlin[res_berlin[0].0.clone()], "10115 Berlin Mitte");

    // PLZ + Ort with Umlaut (München)
    let text_muc = "Adresse 80331 München Zentrum";
    let res_muc = recognizer.recognize(text_muc);
    assert_eq!(res_muc.len(), 1);
    assert_eq!(res_muc[0].1, "PLZ_ORT");
    assert_eq!(&text_muc[res_muc[0].0.clone()], "80331 München Zentrum");
}

#[test]
// ANCHOR[contextra-privacy:TEST-PROP-006]
// Prüft: Stellt sicher, dass nachfolgende Satzteile nicht versehentlich in Ortsnamen geschluckt werden.
fn test_plz_ort_does_not_swallow_subsequent_sentence_text() {
    let recognizer = PatternRecognizer::new();
    let text = "Der Hauptsitz ist in 10115 Berlin und die Filiale ist in 80331 München.";
    let results = recognizer.recognize(text);

    assert_eq!(results.len(), 2);
    assert_eq!(&text[results[0].0.clone()], "10115 Berlin");
    assert_eq!(results[0].1, "PLZ_ORT");

    assert_eq!(&text[results[1].0.clone()], "80331 München");
    assert_eq!(results[1].1, "PLZ_ORT");
}

// ============================================================================
// 6. Overlong Input and Boundary Tests
// ============================================================================

#[test]
// ANCHOR[contextra-privacy:TEST-PROP-007]
// Prüft: Prüft die Begrenzung überlanger Eingaben auf 64 KiB.
fn test_overlong_input_handling_and_bounded_error() {
    let small_limit_recognizer = PatternRecognizer::new().with_max_input_length(100);
    assert_eq!(small_limit_recognizer.max_input_length(), 100);

    let overlong_text = "a".repeat(101);
    let recognize_res = small_limit_recognizer.recognize(&overlong_text);
    assert!(recognize_res.is_empty(), "returns empty on overlong input");

    let bounded_res = small_limit_recognizer.recognize_bounded(&overlong_text);
    assert!(bounded_res.is_err());
    let err_msg = bounded_res.unwrap_err().to_string();
    assert!(err_msg.contains("Payload size exceeds limit"));

    let short_text = "Mail user@example.com im System";
    let bounded_ok = small_limit_recognizer.recognize_bounded(short_text);
    assert!(bounded_ok.is_ok());
    assert_eq!(bounded_ok.unwrap().len(), 1);
}

// ============================================================================
// 7. Surrogation -> Re-Identification Roundtrip
// ============================================================================

#[test]
// ANCHOR[contextra-privacy:TEST-PROP-008]
// Prüft: Prüft den vollständigen Zyklus von Pseudonymisierung und anschließender Rehydrierung.
fn test_surrogation_to_reidentification_roundtrip() {
    let vault = EgressVault::try_default().expect("vault creates");
    let recognizer = PatternRecognizer::new();

    let raw_email = "max.mustermann@example.com";
    let raw_iban = "DE89370400440532013000";
    let original_text = format!("Kontakt: {raw_email} oder Konto {raw_iban}");

    let (sanitized_text, count) = vault
        .sanitize_and_vault(&original_text, &recognizer)
        .expect("sanitization succeeds");

    assert_eq!(count, 2);
    assert!(!sanitized_text.contains(raw_email));
    assert!(!sanitized_text.contains(raw_iban));

    // Extract surrogate tokens from sanitized text
    let surrogate_vault = vault.surrogate_vault();

    // Re-identification: lookup each surrogate token in the vault
    let mut reidentified = sanitized_text.clone();
    for _ in 0..10 {
        if let Some(start) = reidentified.find("[USER_ENTITY_") {
            let end = start + "[USER_ENTITY_".len() + 16 + 1;
            let token = &reidentified[start..end];
            let original_entity = surrogate_vault
                .get_entity(token)
                .expect("surrogate token exists in vault");
            reidentified.replace_range(start..end, &original_entity);
        } else {
            break;
        }
    }

    assert_eq!(reidentified, original_text);
}

// ============================================================================
// 8. Property Test: Erkennung paniert nie
// ============================================================================

#[test]
// ANCHOR[contextra-privacy:TEST-PROP-009]
// Prüft: Eigenschaftstest: Erkenner gerät bei beliebigen Zufallsstrings niemals in Panik.
fn test_property_recognizer_never_panics_on_arbitrary_inputs() {
    let recognizer = PatternRecognizer::new();

    let mut state = 0x1234_5678_u64;

    for len in 0..500 {
        let mut bytes = Vec::with_capacity(len);
        for _ in 0..len {
            state = state.wrapping_mul(6364136223846793005).wrapping_add(1);
            bytes.push((state >> 24) as u8);
        }

        let lossy_str = String::from_utf8_lossy(&bytes);
        let _ = recognizer.recognize(&lossy_str);
        let _ = recognizer.recognize_bounded(&lossy_str);

        let char_str: String = bytes
            .iter()
            .filter_map(|&b| char::from_u32(b as u32))
            .collect();
        let _ = recognizer.recognize(&char_str);
        let _ = recognizer.recognize_bounded(&char_str);
    }
}
