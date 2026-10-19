// FILE-CONTEXT
// STAND: 2026-10-13
// ZWECK: Tests für CopyStatus und B11 Kopien-Geltungsbereich Verifikation.
// INVARIANTEN: Fail-Closed, Standard CopyStatus ist Unknown.

use contextra_checkpoint::copy_registry::CopyStatus;

#[test]
fn test_copy_status_defaults_and_predicates() {
    let status = CopyStatus::default();
    assert_eq!(status, CopyStatus::Unknown);
    assert!(status.is_unknown());
    assert!(!status.is_deleted());

    let deleted = CopyStatus::Deleted;
    assert!(deleted.is_deleted());
    assert!(!deleted.is_unknown());
}

#[test]
fn test_gdpr_art19_and_art28_out_of_scope_notices() {
    let art19 = CopyStatus::art19_recipient("Empfänger X");
    let art28 = CopyStatus::art28_processor("Auftragsverarbeiter Y");

    match art19 {
        CopyStatus::OutOfScope { reason } => {
            assert!(reason.contains("Art. 19 DSGVO"));
            assert!(reason.contains("Empfänger X"));
        }
        _ => panic!("Expected OutOfScope status"),
    }

    match art28 {
        CopyStatus::OutOfScope { reason } => {
            assert!(reason.contains("Art. 28 DSGVO"));
            assert!(reason.contains("Auftragsverarbeiter Y"));
        }
        _ => panic!("Expected OutOfScope status"),
    }
}
