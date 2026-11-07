//! Tests für kanonische Löschberichte nach SPEC §11.7, §12.5, F.10, Anhang D.
//!
//! SPEC: T-REPORT-1, T-PROOF-1, T-CLASS-1, T-LINT-1

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use contextra_erasure::report::build::build_report;
use contextra_erasure::report::canonical::{to_canonical_json_string, verify_canonical_nfc};
use contextra_erasure::report::signing::{DerivedReportSigner, ReportSigner};
use contextra_erasure::types_local::ErasureId;
use contextra_erasure::types_local::ReportSigner as ReportSignerPort;
use ed25519_dalek::SigningKey;
use futures_util::FutureExt;

/// T-ID: T-REPORT-1
/// SPEC: T-REPORT-1
#[test]
fn test_t_report_1_canonical_json_spec_rules() {
    let erasure_id = ErasureId::new([0x42; 16]);
    let report = build_report(erasure_id, 1000000, 2000000, vec![], 5, 10240, true);

    let canonical_json = to_canonical_json_string(&report).expect("canonical json string");

    // F.10: Keine Zeilenumbrüche
    assert!(!canonical_json.contains('\n'));

    // F.10: Keine Leerzeichen außerhalb von Strings
    let mut in_string = false;
    let mut escaped = false;
    for b in canonical_json.bytes() {
        if in_string {
            if escaped {
                escaped = false;
            } else if b == b'\\' {
                escaped = true;
            } else if b == b'"' {
                in_string = false;
            }
        } else if b == b'"' {
            in_string = true;
        } else {
            // Außerhalb von Strings darf kein Whitespace vorkommen
            assert!(
                b != b' ' && b != b'\t' && b != b'\r' && b != b'\n',
                "Whitespace außerhalb von Strings gefunden"
            );
        }
    }

    // F.10 Regel 3: Keine Gleitkommazahlen (Zahlen sind Ganzzahlen)
    assert!(canonical_json.contains("\"report_version\":1"));
}

/// T-ID: T-REPORT-1
/// SPEC: T-REPORT-1
#[test]
fn test_t_report_1_nfc_validation_rejects_decomposed() {
    // Normalisierte Eingabe: "é" als U+00E9
    let nfc_valid = "Kompaktierung abgeschlossen: Café";
    assert!(verify_canonical_nfc(nfc_valid).is_ok());

    // Decomposed NFD Eingabe: 'e' + U+0301 (combining acute accent)
    let nfd_invalid = "Kompaktierung abgeschlossen: Cafe\u{0301}";
    assert!(verify_canonical_nfc(nfd_invalid).is_err());
}

/// T-ID: T-PROOF-1
/// SPEC: T-PROOF-1
#[test]
fn test_t_proof_1_field_tampering_breaks_signature() {
    let erasure_id = ErasureId::new([0x12; 16]);
    let report = build_report(erasure_id, 1000, 2000, vec![], 0, 0, true);
    let signing_key = SigningKey::from_bytes(&[0x55; 32]);

    let signer = DerivedReportSigner::new(signing_key.clone());
    assert_eq!(signer.signer_id(), 0);

    let signed = ReportSigner::sign_report(&report, &signing_key).expect("signing must succeed");

    // Verifikation der unveränderten Version muss gelingen
    assert!(signed.verify_with_key(&signing_key.verifying_key()).is_ok());

    // Manipuliere canonical body
    let mut manipulated = signed.clone();
    manipulated.canonical_body = manipulated
        .canonical_body
        .replace("\"completed_at\":2000", "\"completed_at\":2001");

    // Verifikation der manipulierten Version muss fehlschlagen
    assert!(manipulated
        .verify_with_key(&signing_key.verifying_key())
        .is_err());
}

/// T-ID: T-PROOF-1
/// SPEC: T-PROOF-1
#[test]
fn test_t_proof_1_port_signer_conformance() {
    let signing_key = SigningKey::from_bytes(&[0x77; 32]);
    let signer = DerivedReportSigner::new(signing_key.clone());
    assert_eq!(signer.signer_id(), 0);

    let payload = b"{\"erasure_id\":\"test\"}";
    let signature = signer
        .sign_report(payload)
        .now_or_never()
        .unwrap()
        .expect("signing");
    assert_eq!(signature.signer_id, 0);
    assert!(!signature.signature_base64.is_empty());
    assert_eq!(signature.pub_fingerprint.len(), 32);
}

/// T-ID: T-CLASS-1
/// SPEC: T-CLASS-1
#[test]
fn test_t_class_1_lattice_independent_eval() {
    let erasure_id = ErasureId::new([0x34; 16]);
    let report = build_report(erasure_id, 1000, 2000, vec![], 0, 0, false);

    // G, K, P einzeln als reached/not_reached, keine Gesamtnote
    assert!(report.classes.contains_key("G"));
    assert!(report.classes.contains_key("K"));
    assert!(report.classes.contains_key("P"));

    // Fehlende Schicht wird als "nicht erfasst" ausgewiesen (P17)
    for (_layer, status) in &report.key_coverage {
        assert!(
            status == "full"
                || status == "unlinked"
                || status == "none"
                || status == "nicht erfasst"
        );
    }
}

/// T-ID: T-CLASS-1
/// SPEC: T-CLASS-1
#[test]
fn test_t_class_1_unrecorded_layer_marked_not_recorded() {
    let erasure_id = ErasureId::new([0x35; 16]);
    let w = contextra_erasure::types_local::Scanner::new(
        contextra_erasure::types_local::LayerId::new([1; 16]),
        contextra_erasure::types_local::MeasureClass::Mk1,
        1,
    )
    .arm(contextra_erasure::types_local::ControlEvidence::observed())
    .finish(0, std::time::Duration::from_millis(5), [0u8; 32])
    .expect("witness");

    let report = build_report(erasure_id, 1000, 2000, vec![w], 0, 0, false);
    // doc_store ist erfasst
    assert_ne!(
        report.key_coverage.get("doc_store").unwrap(),
        "nicht erfasst"
    );
    // die anderen Schichten sind "nicht erfasst" (P17)
    assert_eq!(
        report.key_coverage.get("vector_payload").unwrap(),
        "nicht erfasst"
    );
    assert_eq!(
        report.key_coverage.get("text_postings").unwrap(),
        "nicht erfasst"
    );
}

/// T-ID: T-LINT-1
/// SPEC: T-LINT-1
#[test]
fn test_t_lint_1_wording_mandatory_texts() {
    let erasure_id = ErasureId::new([0x77; 16]);
    let report = build_report(erasure_id, 1000, 2000, vec![], 1, 100, true);

    // E-L11 Signaturumfang
    assert!(report.signature_scope_notice.contains(
        "Die Signatur beweist Integrität und Herkunft aus dieser Installation, nicht die Wahrheit des Inhalts."
    ));

    // Keine Rechtsberatung
    assert!(report
        .legal_notice
        .contains("Keine Rechtsberatung; die rechtliche Wertung obliegt dem Betreiber."));

    // Außenaussage nach 6.4
    assert!(report
        .scope_notice
        .contains("Contextra entfernt die Daten eines Löschziels aus dem Datenverzeichnis"));
}

/// T-ID: T-REPORT-1
/// SPEC: T-REPORT-1
#[test]
fn test_t_report_1_byte_scan_no_content_or_keys() {
    let erasure_id = ErasureId::new([0x99; 16]);
    let report = build_report(erasure_id, 1000, 2000, vec![], 0, 0, true);
    let json_bytes = to_canonical_json_string(&report).unwrap().into_bytes();

    // Byte-Scan auf P13-Konformität: Keine vertraulichen Wörter / Klartext-Schlüssel
    let forbidden_payloads = [
        b"secret_key".as_slice(),
        b"passphrase".as_slice(),
        b"user_content".as_slice(),
    ];
    for payload in &forbidden_payloads {
        assert!(!json_bytes.windows(payload.len()).any(|w| w == *payload));
    }
}
