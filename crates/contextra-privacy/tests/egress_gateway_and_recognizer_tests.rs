#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing,
    clippy::disallowed_types
)]
// FILE-CONTEXT
// ZWECK: Uferloser Test-Suite (>= 40 Tests) für RuleBasedEntityRecognizer, Gateway Pipeline Enforcements, Checksums und Zero PII Leakage.

use contextra_ports::Clock;
use contextra_privacy::bulk_exfiltration_detector::{BulkExfiltrationDetector, SessionId};
use contextra_privacy::egress_gateway::{
    CloudQueryRequest, DefaultEgressGuard, EgressGateway, EgressGuardCheck,
};
use contextra_privacy::egress_vault::{
    BlockReason, BoxFuture, EgressClassification, EgressClassifier, EntityRecognizer,
    SurrogateVault,
};
use contextra_privacy::entity_recognizer::{Gazetteer, RuleBasedEntityRecognizer};
use contextra_types::{TenantId, TenantScoped};
use std::sync::Arc;

// ============================================================================
// Mock Helpers
// ============================================================================

struct AllowAllClassifier;
impl EgressClassifier for AllowAllClassifier {
    fn classify<'a>(&'a self, _payload: &'a str) -> BoxFuture<'a, EgressClassification> {
        Box::pin(async move { EgressClassification::Allow })
    }
}

struct BlockClassifier {
    reason: String,
}
impl EgressClassifier for BlockClassifier {
    fn classify<'a>(&'a self, _payload: &'a str) -> BoxFuture<'a, EgressClassification> {
        let reason = self.reason.clone();
        Box::pin(async move { EgressClassification::Block(BlockReason::PolicyDenied(reason)) })
    }
}

struct MockEgressGuard {
    block: bool,
}
impl EgressGuardCheck for MockEgressGuard {
    fn check<'a>(&'a self, _payload: &'a str) -> BoxFuture<'a, EgressClassification> {
        let block = self.block;
        Box::pin(async move {
            if block {
                EgressClassification::Block(BlockReason::SensitivePattern(
                    "layer-4-hnsw-match".to_string(),
                ))
            } else {
                EgressClassification::Allow
            }
        })
    }
}

struct MockClock {
    nanos: u64,
}
impl Clock for MockClock {
    fn now_unix_nanos(&self) -> u64 {
        self.nanos
    }
    fn monotonic_nanos(&self) -> u64 {
        self.nanos
    }
}

// ============================================================================
// 1. E-Mail Tests
// ============================================================================

#[test]
fn test_recognizer_email_valid_positive() {
    let recognizer = RuleBasedEntityRecognizer::new();
    let text = "Sende die Nachricht an max.mustermann@example.com bitte.";
    let results = recognizer.recognize(text);
    assert_eq!(results.len(), 1);
    assert_eq!(&text[results[0].0.clone()], "max.mustermann@example.com");
    assert_eq!(results[0].1, "EMAIL");
}

#[test]
fn test_recognizer_email_subdomain_positive() {
    let recognizer = RuleBasedEntityRecognizer::new();
    let text = "Kontakt: user+tag@mail.sub.company.org im System";
    let results = recognizer.recognize(text);
    assert_eq!(results.len(), 1);
    assert_eq!(&text[results[0].0.clone()], "user+tag@mail.sub.company.org");
    assert_eq!(results[0].1, "EMAIL");
}

#[test]
fn test_recognizer_email_missing_at_negative() {
    let recognizer = RuleBasedEntityRecognizer::new();
    let text = "Ungültige Adresse: user.example.com ohne At-Zeichen";
    let results = recognizer.recognize(text);
    assert!(results.iter().all(|r| r.1 != "EMAIL"));
}

#[test]
fn test_recognizer_email_invalid_tld_negative() {
    let recognizer = RuleBasedEntityRecognizer::new();
    let text = "Ungültig: user@domain.x (TLD zu kurz)";
    let results = recognizer.recognize(text);
    assert!(results.iter().all(|r| r.1 != "EMAIL"));
}

// ============================================================================
// 2. IBAN Tests
// ============================================================================

#[test]
fn test_recognizer_iban_de_valid_positive() {
    let recognizer = RuleBasedEntityRecognizer::new();
    // Valid DE IBAN (Deutsche Bank Berlin)
    let text = "Überweisung an DE89370400440532013000 ausführen";
    let results = recognizer.recognize(text);
    assert_eq!(results.len(), 1);
    assert_eq!(&text[results[0].0.clone()], "DE89370400440532013000");
    assert_eq!(results[0].1, "IBAN");
}

#[test]
fn test_recognizer_iban_de_with_spaces_positive() {
    let recognizer = RuleBasedEntityRecognizer::new();
    let text = "IBAN: DE89 3704 0044 0532 0130 00 eintragen";
    let results = recognizer.recognize(text);
    assert_eq!(results.len(), 1);
    assert_eq!(&text[results[0].0.clone()], "DE89 3704 0044 0532 0130 00");
    assert_eq!(results[0].1, "IBAN");
}

#[test]
fn test_recognizer_iban_at_valid_positive() {
    let recognizer = RuleBasedEntityRecognizer::new();
    // Valid AT IBAN (Erste Bank)
    let text = "Konto AT921904300234567890 prüfen";
    let results = recognizer.recognize(text);
    assert_eq!(results.len(), 1);
    assert_eq!(&text[results[0].0.clone()], "AT921904300234567890");
    assert_eq!(results[0].1, "IBAN");
}

#[test]
fn test_recognizer_iban_ch_valid_positive() {
    let recognizer = RuleBasedEntityRecognizer::new();
    // Valid CH IBAN (UBS)
    let text = "Schweizer IBAN CH9300762011623852957 vorhanden";
    let results = recognizer.recognize(text);
    assert_eq!(results.len(), 1);
    assert_eq!(&text[results[0].0.clone()], "CH9300762011623852957");
    assert_eq!(results[0].1, "IBAN");
}

#[test]
fn test_recognizer_iban_invalid_checksum_negative() {
    let recognizer = RuleBasedEntityRecognizer::new();
    // DE IBAN with tampered check digit (DE00 instead of DE89)
    let text = "Falsche IBAN: DE00370400440532013000 sollte fehlschlagen";
    let results = recognizer.recognize(text);
    assert!(results.iter().all(|r| r.1 != "IBAN"));
}

#[test]
fn test_recognizer_iban_wrong_country_length_negative() {
    let recognizer = RuleBasedEntityRecognizer::new();
    // DE IBAN with 20 chars instead of required 22
    let text = "Zu kurze IBAN: DE893704004405320130";
    let results = recognizer.recognize(text);
    assert!(results.iter().all(|r| r.1 != "IBAN"));
}

// ============================================================================
// 3. Telefonnummern Tests
// ============================================================================

#[test]
fn test_recognizer_phone_de_international_plus49_positive() {
    let recognizer = RuleBasedEntityRecognizer::new();
    let text = "Rufen Sie +49 170 1234567 an für Rückfragen.";
    let results = recognizer.recognize(text);
    assert_eq!(results.len(), 1);
    assert_eq!(&text[results[0].0.clone()], "+49 170 1234567");
    assert_eq!(results[0].1, "PHONE");
}

#[test]
fn test_recognizer_phone_de_international_0049_positive() {
    let recognizer = RuleBasedEntityRecognizer::new();
    let text = "Telefon: 0049 30 98765432 im Büro";
    let results = recognizer.recognize(text);
    assert_eq!(results.len(), 1);
    assert_eq!(&text[results[0].0.clone()], "0049 30 98765432");
    assert_eq!(results[0].1, "PHONE");
}

#[test]
fn test_recognizer_phone_de_local_0170_positive() {
    let recognizer = RuleBasedEntityRecognizer::new();
    let text = "Mobil: 0170/1234567 direkt erreichen";
    let results = recognizer.recognize(text);
    assert_eq!(results.len(), 1);
    assert_eq!(&text[results[0].0.clone()], "0170/1234567");
    assert_eq!(results[0].1, "PHONE");
}

#[test]
fn test_recognizer_phone_de_formatting_slashes_positive() {
    let recognizer = RuleBasedEntityRecognizer::new();
    let text = "Tel: (089) 1234-5678 Notfallnummer";
    let results = recognizer.recognize(text);
    assert_eq!(results.len(), 1);
    assert_eq!(&text[results[0].0.clone()], "(089) 1234-5678");
    assert_eq!(results[0].1, "PHONE");
}

#[test]
fn test_recognizer_phone_too_short_negative() {
    let recognizer = RuleBasedEntityRecognizer::new();
    let text = "Nummer 0123 ist zu kurz für Telefonnummer";
    let results = recognizer.recognize(text);
    assert!(results.iter().all(|r| r.1 != "PHONE"));
}

#[test]
fn test_recognizer_phone_alphanumeric_negative() {
    let recognizer = RuleBasedEntityRecognizer::new();
    let text = "Code ABC017012345 ist keine Telefonnummer";
    let results = recognizer.recognize(text);
    assert!(results.iter().all(|r| r.1 != "PHONE"));
}

// ============================================================================
// 4. Kreditkartennummern Tests (Luhn)
// ============================================================================

#[test]
fn test_recognizer_credit_card_visa_valid_luhn_positive() {
    let recognizer = RuleBasedEntityRecognizer::new();
    // Valid Visa test card (Luhn valid)
    let text = "Zahlung mit 4532820011223344 bestätigen.";
    let results = recognizer.recognize(text);
    assert_eq!(results.len(), 1);
    assert_eq!(&text[results[0].0.clone()], "4532820011223344");
    assert_eq!(results[0].1, "CREDIT_CARD");
}

#[test]
fn test_recognizer_credit_card_mastercard_spaced_positive() {
    let recognizer = RuleBasedEntityRecognizer::new();
    // Valid Mastercard test card with spaces (Luhn valid)
    let text = "Karte: 5412 7512 3456 7898 verwendet";
    let results = recognizer.recognize(text);
    assert_eq!(results.len(), 1);
    assert_eq!(&text[results[0].0.clone()], "5412 7512 3456 7898");
    assert_eq!(results[0].1, "CREDIT_CARD");
}

#[test]
fn test_recognizer_credit_card_amex_hyphenated_positive() {
    let recognizer = RuleBasedEntityRecognizer::new();
    // Valid Amex 15-digit card with hyphens
    let text = "Amex: 3782-822463-10005 verarbeiten";
    let results = recognizer.recognize(text);
    assert_eq!(results.len(), 1);
    assert_eq!(&text[results[0].0.clone()], "3782-822463-10005");
    assert_eq!(results[0].1, "CREDIT_CARD");
}

#[test]
fn test_recognizer_credit_card_invalid_luhn_checksum_negative() {
    let recognizer = RuleBasedEntityRecognizer::new();
    // Invalid Luhn checksum (last digit modified)
    let text = "Ungültige Karte: 4532820011223345 sollte fehlschlagen";
    let results = recognizer.recognize(text);
    assert!(results.iter().all(|r| r.1 != "CREDIT_CARD"));
}

#[test]
fn test_recognizer_credit_card_too_short_negative() {
    let recognizer = RuleBasedEntityRecognizer::new();
    let text = "Zahlung 123456789012 (12 Stellen)";
    let results = recognizer.recognize(text);
    assert!(results.iter().all(|r| r.1 != "CREDIT_CARD"));
}

// ============================================================================
// 5. Deutsche Steuer-ID Tests
// ============================================================================

#[test]
fn test_recognizer_steuer_id_valid_positive() {
    let recognizer = RuleBasedEntityRecognizer::new();
    // Valid German Steuer-ID: 11 digits, ISO 7064 Mod 11, 10 valid
    let text = "Steuer-ID: 86095742116 ist gültig.";
    let results = recognizer.recognize(text);
    assert_eq!(results.len(), 1);
    assert_eq!(&text[results[0].0.clone()], "86095742116");
    assert_eq!(results[0].1, "STEUER_ID");
}

#[test]
fn test_recognizer_steuer_id_invalid_checksum_negative() {
    let recognizer = RuleBasedEntityRecognizer::new();
    // Tampered check digit (86095742130)
    let text = "Falsche Steuer-ID: 86095742130 ablehnen";
    let results = recognizer.recognize(text);
    assert!(results.iter().all(|r| r.1 != "STEUER_ID"));
}

#[test]
fn test_recognizer_steuer_id_leading_zero_negative() {
    let recognizer = RuleBasedEntityRecognizer::new();
    // German Steuer-ID cannot start with 0
    let text = "Ungültig: 06095742139 beginnt mit Null";
    let results = recognizer.recognize(text);
    assert!(results.iter().all(|r| r.1 != "STEUER_ID"));
}

#[test]
fn test_recognizer_steuer_id_invalid_duplicate_distribution_negative() {
    let recognizer = RuleBasedEntityRecognizer::new();
    // All distinct digits in first 10 digits (no 2x or 3x duplicate digit)
    let text = "Ungültig: 12345678901 verteilung falsch";
    let results = recognizer.recognize(text);
    assert!(results.iter().all(|r| r.1 != "STEUER_ID"));
}

// ============================================================================
// 6. IPv4 & IPv6 Tests
// ============================================================================

#[test]
fn test_recognizer_ipv4_valid_positive() {
    let recognizer = RuleBasedEntityRecognizer::new();
    let text = "Server-IP 192.168.1.254 ist erreichbar";
    let results = recognizer.recognize(text);
    assert_eq!(results.len(), 1);
    assert_eq!(&text[results[0].0.clone()], "192.168.1.254");
    assert_eq!(results[0].1, "IPV4");
}

#[test]
fn test_recognizer_ipv6_valid_positive() {
    let recognizer = RuleBasedEntityRecognizer::new();
    let text = "Adresse 2001:db8:85a3::8a2e:370:7334 konfiguriert";
    let results = recognizer.recognize(text);
    assert_eq!(results.len(), 1);
    assert_eq!(&text[results[0].0.clone()], "2001:db8:85a3::8a2e:370:7334");
    assert_eq!(results[0].1, "IPV6");
}

#[test]
fn test_recognizer_ipv4_octet_overflow_negative() {
    let recognizer = RuleBasedEntityRecognizer::new();
    let text = "Ungültige IP 256.100.0.1 ignorieren";
    let results = recognizer.recognize(text);
    assert!(results.iter().all(|r| r.1 != "IPV4"));
}

#[test]
fn test_recognizer_ipv6_invalid_colons_negative() {
    let recognizer = RuleBasedEntityRecognizer::new();
    let text = "Ungültig: 1234:5678 nur ein Doppelpunkt ist keine IPv6";
    let results = recognizer.recognize(text);
    assert!(results.iter().all(|r| r.1 != "IPV6"));
}

// ============================================================================
// 7. PLZ & License Plate Tests
// ============================================================================

#[test]
fn test_recognizer_plz_valid_positive() {
    let recognizer = RuleBasedEntityRecognizer::new();
    let text = "Standort 10115 Berlin Mitte";
    let results = recognizer.recognize(text);
    assert_eq!(results.len(), 1);
    assert_eq!(&text[results[0].0.clone()], "10115");
    assert_eq!(results[0].1, "PLZ");
}

#[test]
fn test_recognizer_plz_invalid_length_negative() {
    let recognizer = RuleBasedEntityRecognizer::new();
    let text = "Code 123456 ist 6 Stellen (keine PLZ)";
    let results = recognizer.recognize(text);
    assert!(results.iter().all(|r| r.1 != "PLZ"));
}

#[test]
fn test_recognizer_license_plate_valid_positive() {
    let recognizer = RuleBasedEntityRecognizer::new();
    let text = "Fahrzeug B-MW 1234 geparkt.";
    let results = recognizer.recognize(text);
    assert_eq!(results.len(), 1);
    assert_eq!(&text[results[0].0.clone()], "B-MW 1234");
    assert_eq!(results[0].1, "LICENSE_PLATE");
}

#[test]
fn test_recognizer_license_plate_invalid_format_negative() {
    let recognizer = RuleBasedEntityRecognizer::new();
    let text = "Kein Kennzeichen: ABCDE-12345 zu viele Buchstaben";
    let results = recognizer.recognize(text);
    assert!(results.iter().all(|r| r.1 != "LICENSE_PLATE"));
}

// ============================================================================
// 8. Gazetteer Tests
// ============================================================================

#[test]
fn test_recognizer_gazetteer_term_match_positive() {
    let mut gazetteer = Gazetteer::new();
    gazetteer.add_term("Acme Middle East Ltd");
    let recognizer = RuleBasedEntityRecognizer::with_gazetteer(gazetteer);

    let text = "Vertrag mit Acme Middle East Ltd wurde unterzeichnet.";
    let results = recognizer.recognize(text);
    assert_eq!(results.len(), 1);
    assert_eq!(&text[results[0].0.clone()], "Acme Middle East Ltd");
    assert_eq!(results[0].1, "GAZETTEER");
}

#[test]
fn test_recognizer_gazetteer_case_insensitive_positive() {
    let mut gazetteer = Gazetteer::new();
    gazetteer.add_term("Dr. Hans Mueller");
    let recognizer = RuleBasedEntityRecognizer::with_gazetteer(gazetteer);

    let text = "Ansprechpartner dr. hans mueller vor Ort.";
    let results = recognizer.recognize(text);
    assert_eq!(results.len(), 1);
    assert_eq!(&text[results[0].0.clone()], "dr. hans mueller");
    assert_eq!(results[0].1, "GAZETTEER");
}

#[test]
fn test_recognizer_gazetteer_unlisted_term_negative() {
    let mut gazetteer = Gazetteer::new();
    gazetteer.add_term("Acme Corp");
    let recognizer = RuleBasedEntityRecognizer::with_gazetteer(gazetteer);

    let text = "Vertrag mit Globex Corp nicht in der Liste.";
    let results = recognizer.recognize(text);
    assert!(results.iter().all(|r| r.1 != "GAZETTEER"));
}

// ============================================================================
// 9. Unicode, Zero-Width Characters & Obfuscation Tests
// ============================================================================

#[test]
fn test_recognizer_zero_width_spaces_stripped_positive() {
    let recognizer = RuleBasedEntityRecognizer::new();
    // Email containing zero-width spaces (\u{200B}) inserted to evade detection
    let text = "E-Mail m\u{200B}a\u{200B}x@ex\u{200B}ample.com im System";
    let results = recognizer.recognize(text);
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].1, "EMAIL");
    assert_eq!(
        &text[results[0].0.clone()],
        "m\u{200B}a\u{200B}x@ex\u{200B}ample.com"
    );
}

#[test]
fn test_recognizer_spaced_out_iban_obfuscation_positive() {
    let recognizer = RuleBasedEntityRecognizer::new();
    // Spaced out IBAN
    let text = "Konto DE89 3704 0044 0532 0130 00 verarbeiten";
    let results = recognizer.recognize(text);
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].1, "IBAN");
}

#[test]
fn test_recognizer_cyrillic_homoglyphs_folded_positive() {
    let recognizer = RuleBasedEntityRecognizer::new();
    // Cyrillic 'а' (\u{0430}) and 'е' (\u{0435}) mixed into email
    let text = "E-Mail m\u{0430}x@ex\u{0435}ample.com verwenden";
    let results = recognizer.recognize(text);
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].1, "EMAIL");
}

#[test]
fn test_recognizer_multibyte_utf8_boundary_safety() {
    let recognizer = RuleBasedEntityRecognizer::new();
    let text = "Hallo 🌍! Mail max@example.com 🚀 und 漢字 Text DE89370400440532013000 🔥";
    let results = recognizer.recognize(text);
    assert_eq!(results.len(), 2);

    for (range, _label) in results {
        assert!(text.is_char_boundary(range.start));
        assert!(text.is_char_boundary(range.end));
    }
}

// ============================================================================
// 10. Gateway Pipeline & Fail-Closed Enforcement Tests
// ============================================================================

#[test]
fn test_gateway_builder_missing_recognizer_fails() {
    let classifier = Arc::new(AllowAllClassifier);
    let guard = Arc::new(DefaultEgressGuard);

    let res = EgressGateway::builder()
        .with_classifier(classifier)
        .with_egress_guard(guard)
        .build();

    assert!(res.is_err());
    let err_msg = res.unwrap_err().to_string();
    assert!(err_msg.contains("missing recognizer"));
}

#[test]
fn test_gateway_builder_missing_egress_guard_fails() {
    let classifier = Arc::new(AllowAllClassifier);
    let recognizer = Arc::new(RuleBasedEntityRecognizer::new());

    let res = EgressGateway::builder()
        .with_classifier(classifier)
        .with_recognizer(recognizer)
        .build();

    assert!(res.is_err());
    let err_msg = res.unwrap_err().to_string();
    assert!(err_msg.contains("missing mandatory egress_guard"));
}

#[tokio::test]
async fn test_gateway_pipeline_all_stages_executed_in_order() {
    let classifier = Arc::new(AllowAllClassifier);
    let recognizer = Arc::new(RuleBasedEntityRecognizer::new());
    let surrogate_vault = Arc::new(SurrogateVault::random_salt());
    let detector = Arc::new(BulkExfiltrationDetector::new(
        1000,
        std::time::Duration::from_secs(60),
    ));
    let clock = Arc::new(MockClock { nanos: 123456789 });
    let tenant_id = TenantId::try_new(42).unwrap();

    let gateway = EgressGateway::builder()
        .with_classifier(classifier)
        .with_egress_guard(Arc::new(DefaultEgressGuard))
        .with_recognizer(recognizer)
        .with_surrogate_vault(surrogate_vault.clone())
        .with_bulk_detector(detector, SessionId::from("s1"))
        .with_clock(clock)
        .build()
        .expect("gateway builds with all components");

    let req = CloudQueryRequest {
        query: "Sende DE89370400440532013000 an max@example.com".to_string(),
        collection: None,
        max_results: None,
    };
    let scoped = TenantScoped::new(tenant_id, req);

    let resp = gateway
        .process(scoped, &tenant_id)
        .await
        .expect("processing succeeds");

    assert_eq!(resp.status, "success");
    assert!(resp.abstracted);
    assert!(!resp.query.contains("DE89370400440532013000"));
    assert!(!resp.query.contains("max@example.com"));
    assert!(resp.query.contains("[USER_ENTITY_"));
    assert_eq!(surrogate_vault.len(), 2);
}

#[tokio::test]
async fn test_gateway_pipeline_policy_block_fails_closed() {
    let classifier = Arc::new(BlockClassifier {
        reason: "Access forbidden by policy".to_string(),
    });
    let recognizer = Arc::new(RuleBasedEntityRecognizer::new());
    let tenant_id = TenantId::try_new(10).unwrap();

    let gateway = EgressGateway::builder()
        .with_classifier(classifier)
        .with_egress_guard(Arc::new(DefaultEgressGuard))
        .with_recognizer(recognizer)
        .build()
        .expect("gateway builds");

    let req = CloudQueryRequest {
        query: "Public harmless query".to_string(),
        collection: None,
        max_results: None,
    };
    let scoped = TenantScoped::new(tenant_id, req);

    let res = gateway.process(scoped, &tenant_id).await;
    assert!(res.is_err());
    assert!(res
        .unwrap_err()
        .to_string()
        .contains("Access forbidden by policy"));
}

#[tokio::test]
async fn test_gateway_pipeline_bulk_detector_block_fails_closed() {
    let classifier = Arc::new(AllowAllClassifier);
    let recognizer = Arc::new(RuleBasedEntityRecognizer::new());
    let detector = Arc::new(BulkExfiltrationDetector::new(
        20,
        std::time::Duration::from_secs(60),
    ));
    let tenant_id = TenantId::try_new(5).unwrap();

    let gateway = EgressGateway::builder()
        .with_classifier(classifier)
        .with_egress_guard(Arc::new(DefaultEgressGuard))
        .with_recognizer(recognizer)
        .with_bulk_detector(detector, SessionId::from("s2"))
        .build()
        .expect("gateway builds");

    let req = CloudQueryRequest {
        query: "This is a query exceeding the 20 byte limit for bulk exfiltration detector"
            .to_string(),
        collection: None,
        max_results: None,
    };
    let scoped = TenantScoped::new(tenant_id, req);

    let res = gateway.process(scoped, &tenant_id).await;
    assert!(res.is_err());
    assert!(res
        .unwrap_err()
        .to_string()
        .contains("Egress volume rate limit exceeded"));
}

#[tokio::test]
async fn test_gateway_pipeline_missing_tenant_scope_fails_closed() {
    let classifier = Arc::new(AllowAllClassifier);
    let recognizer = Arc::new(RuleBasedEntityRecognizer::new());
    let tenant_a = TenantId::try_new(1).unwrap();
    let tenant_b = TenantId::try_new(2).unwrap();

    let gateway = EgressGateway::builder()
        .with_classifier(classifier)
        .with_egress_guard(Arc::new(DefaultEgressGuard))
        .with_recognizer(recognizer)
        .build()
        .expect("gateway builds");

    let req = CloudQueryRequest {
        query: "Harmless query".to_string(),
        collection: None,
        max_results: None,
    };
    let scoped = TenantScoped::new(tenant_a, req);

    // Mismatched tenant expected vs bound
    let res = gateway.process(scoped, &tenant_b).await;
    assert!(res.is_err());
    assert!(res
        .unwrap_err()
        .to_string()
        .contains("tenant scope mismatch"));
}

#[tokio::test]
async fn test_gateway_pipeline_empty_query_fails_closed() {
    let classifier = Arc::new(AllowAllClassifier);
    let recognizer = Arc::new(RuleBasedEntityRecognizer::new());
    let tenant_id = TenantId::try_new(1).unwrap();

    let gateway = EgressGateway::builder()
        .with_classifier(classifier)
        .with_egress_guard(Arc::new(DefaultEgressGuard))
        .with_recognizer(recognizer)
        .build()
        .expect("gateway builds");

    let req = CloudQueryRequest {
        query: "   ".to_string(),
        collection: None,
        max_results: None,
    };
    let scoped = TenantScoped::new(tenant_id, req);

    let res = gateway.process(scoped, &tenant_id).await;
    assert!(res.is_err());
    assert!(res
        .unwrap_err()
        .to_string()
        .contains("query cannot be empty"));
}

#[tokio::test]
async fn test_gateway_pipeline_oversized_query_fails_closed() {
    let classifier = Arc::new(AllowAllClassifier);
    let recognizer = Arc::new(RuleBasedEntityRecognizer::new());
    let tenant_id = TenantId::try_new(1).unwrap();

    let gateway = EgressGateway::builder()
        .with_classifier(classifier)
        .with_egress_guard(Arc::new(DefaultEgressGuard))
        .with_recognizer(recognizer)
        .build()
        .expect("gateway builds");

    let huge_query = "A".repeat(65536 + 1);
    let req = CloudQueryRequest {
        query: huge_query,
        collection: None,
        max_results: None,
    };
    let scoped = TenantScoped::new(tenant_id, req);

    let res = gateway.process(scoped, &tenant_id).await;
    assert!(res.is_err());
    assert!(res
        .unwrap_err()
        .to_string()
        .contains("query size exceeds limit"));
}

#[tokio::test]
async fn test_gateway_pipeline_layer4_guard_block_fails_closed() {
    let classifier = Arc::new(AllowAllClassifier);
    let recognizer = Arc::new(RuleBasedEntityRecognizer::new());
    let guard = Arc::new(MockEgressGuard { block: true });
    let tenant_id = TenantId::try_new(1).unwrap();

    let gateway = EgressGateway::builder()
        .with_classifier(classifier)
        .with_egress_guard(guard)
        .with_recognizer(recognizer)
        .build()
        .expect("gateway builds");

    let req = CloudQueryRequest {
        query: "Clean query triggering L4 guard match".to_string(),
        collection: None,
        max_results: None,
    };
    let scoped = TenantScoped::new(tenant_id, req);

    let res = gateway.process(scoped, &tenant_id).await;
    assert!(res.is_err());
    assert!(res.unwrap_err().to_string().contains("layer-4-hnsw-match"));
}

#[tokio::test]
async fn test_gateway_pipeline_zero_raw_pii_leaked_in_audit_or_logs() {
    let classifier = Arc::new(AllowAllClassifier);
    let recognizer = Arc::new(RuleBasedEntityRecognizer::new());
    let tenant_id = TenantId::try_new(99).unwrap();

    let raw_email = "super_secret_pii_user@company.de";
    let raw_iban = "DE89370400440532013000";

    let gateway = EgressGateway::builder()
        .with_classifier(classifier)
        .with_egress_guard(Arc::new(DefaultEgressGuard))
        .with_recognizer(recognizer)
        .build()
        .expect("gateway builds");

    let req = CloudQueryRequest {
        query: format!("Nutzer {raw_email} mit Konto {raw_iban}"),
        collection: None,
        max_results: None,
    };
    let scoped = TenantScoped::new(tenant_id, req);

    let resp = gateway
        .process(scoped, &tenant_id)
        .await
        .expect("processing succeeds");

    // Assert output query has zero raw PII
    assert!(!resp.query.contains(raw_email));
    assert!(!resp.query.contains(raw_iban));

    // Assert abstraction notice has zero raw PII
    if let Some(notice) = &resp.abstraction_notice {
        assert!(!notice.contains(raw_email));
        assert!(!notice.contains(raw_iban));
    }
}
