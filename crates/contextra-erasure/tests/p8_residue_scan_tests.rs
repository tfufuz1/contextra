//! Testsuite für WP-12 (P8): Port `ResidueScanner` und Restspuren-Messung.
//!
//! Verifiziert die geforderten Tests und Invarianten aus Spezifikation und Prompt:
//! - positive_control_finds_known_residue
//! - negative_control_after_erasure_finds_nothing
//! - scan_without_working_positive_control_is_invalid
//! - scanner_error_never_reported_as_clean
//! - Invarianten:
//!   - INV-CLASS-MEASURED-1: Kein Zeuge ohne funktionierende Positivkontrolle und Restmenge 0
//!   - INV-NO-RESURRECTION-1: Gelöschte Einträge tauchen nach erfolgreicher Messung nicht wieder auf
//!   - INV-VAULT-FAILCLOSED-1: Scanner-Fehler führen fail-closed zum Abbruch
//!   - INV-CLAIM-BIND-1: Keine rechtsverbindlichen Garantiezusagen im Code/Bericht

use contextra_erasure::measure::residue::{
    IndexCorruptionKind, InMemoryVectorIndexStub, ResidueMeasurementCoordinator,
    ResidueMeasurementError, ResiduePhase, ResidueQuery, ScannerFaultKind,
};
use contextra_erasure::types_local::{DocId, LayerId, SystemClock};

const TEST_LAYER: LayerId = LayerId([0x12; 16]);

// =========================================================================
// Pflicht-Tests gemäß Aufgabenstellung
// =========================================================================

#[test]
fn positive_control_finds_known_residue() {
    let mut index = InMemoryVectorIndexStub::new(TEST_LAYER);
    let canary = b"ctx-canary-known-test-value-v1".to_vec();
    index.register_canary(canary.clone());

    let clock = SystemClock::new();
    let coordinator = ResidueMeasurementCoordinator::new(&index, &clock);

    // Positivkontrolle muss für das registrierte Muster erfolgreich sein
    let control_res = coordinator.verify_positive_control(&canary);
    assert!(control_res.is_ok(), "Positivkontrolle mit bekanntem Muster muss gelingen");
}

#[test]
fn negative_control_after_erasure_finds_nothing() {
    let mut index = InMemoryVectorIndexStub::new(TEST_LAYER);
    let doc_id = DocId::new(42);
    let canary = b"ctx-canary-valid".to_vec();
    index.register_canary(canary.clone());

    // Dokument vorher mit Backlinks anlegen
    index.insert_document(doc_id, vec![101, 102, 103]);

    let clock = SystemClock::new();
    let query = ResidueQuery::new(doc_id, [0xaa; 32]);

    // Vor der Löschung: Scan muss Treffer (Vektorknoten + Backlinks + Quantized) anzeigen
    let pre_erase_res = index.active_docs_count_for_test(&query);
    assert!(pre_erase_res > 0, "Vor der Löschung müssen Spuren vorhanden sein");

    // Physische Löschung ausführen
    index.physical_erase(doc_id);

    // Negativkontrolle nach Löschung: Alles genullt/entfernt, 0 Spuren
    let coordinator = ResidueMeasurementCoordinator::new(&index, &clock);
    let report = coordinator
        .measure_after_erasure(&query, &canary, pre_erase_res)
        .expect("Messung nach vollständiger Löschung muss erfolgreich sein");

    assert_eq!(report.post_erase_matches, 0);
    assert!(report.positive_control_passed);
    assert_eq!(report.witness.layer, TEST_LAYER);
    assert_ne!(report.witness.digest, [0u8; 32]);
}

#[test]
fn scan_without_working_positive_control_is_invalid() {
    let mut index = InMemoryVectorIndexStub::new(TEST_LAYER);
    let doc_id = DocId::new(99);
    // Kein Canary registriert
    let unregistered_canary = b"unknown-canary-value";

    index.insert_document(doc_id, vec![]);
    index.physical_erase(doc_id);

    let clock = SystemClock::new();
    let coordinator = ResidueMeasurementCoordinator::new(&index, &clock);
    let query = ResidueQuery::new(doc_id, [0xbb; 32]);

    // Positivkontrolle schlägt fehl -> Messung MUSS ungültig sein (kein "sauber")
    let err = coordinator
        .measure_after_erasure(&query, unregistered_canary, 1)
        .expect_err("Ohne Positivkontrolle darf kein sauberer Bericht ausgestellt werden");

    match err {
        ResidueMeasurementError::PositiveControlFailed { phase } => {
            assert_eq!(phase, ResiduePhase::PositiveControl);
        }
        other => panic!("Erwartete PositiveControlFailed, erhielt: {:?}", other),
    }
}

#[test]
fn scanner_error_never_reported_as_clean() {
    let mut index = InMemoryVectorIndexStub::new(TEST_LAYER);
    let doc_id = DocId::new(777);
    let canary = b"ctx-canary-valid".to_vec();
    index.register_canary(canary.clone());
    index.insert_document(doc_id, vec![]);
    index.physical_erase(doc_id);

    // Simulation eines Fehlers im Scanner (z. B. I/O-Fehler)
    index.set_simulate_fault(Some(ScannerFaultKind::IoFailure));

    let clock = SystemClock::new();
    let coordinator = ResidueMeasurementCoordinator::new(&index, &clock);
    let query = ResidueQuery::new(doc_id, [0xcc; 32]);

    let err = coordinator
        .measure_after_erasure(&query, &canary, 1)
        .expect_err("Scanner-Fehler darf NIEMALS als sauber interpretiert werden");

    match err {
        ResidueMeasurementError::ScannerFault { phase, detail } => {
            assert_eq!(detail, ScannerFaultKind::IoFailure);
            // Fehler tritt bereits in Positivkontrolle oder PostEraseScan auf
            assert!(
                phase == ResiduePhase::PositiveControl || phase == ResiduePhase::PostEraseScan
            );
        }
        other => panic!("Erwartete ScannerFault, erhielt: {:?}", other),
    }
}

// =========================================================================
// Invarianten-Tests (§4 und Prompt)
// =========================================================================

#[test]
fn inv_class_measured_1_no_witness_without_positive_control_and_zero_residue() {
    let mut index = InMemoryVectorIndexStub::new(TEST_LAYER);
    let doc_id = DocId::new(555);
    let canary = b"valid-canary".to_vec();
    index.register_canary(canary.clone());

    // Dokument existiert noch (nicht gelöscht)
    index.insert_document(doc_id, vec![1, 2]);

    let clock = SystemClock::new();
    let coordinator = ResidueMeasurementCoordinator::new(&index, &clock);
    let query = ResidueQuery::new(doc_id, [0x55; 32]);

    // Positivkontrolle funktioniert, aber Spuren sind vorhanden
    let err = coordinator
        .measure_after_erasure(&query, &canary, 2)
        .expect_err("INV-CLASS-MEASURED-1: Residuen müssen Fehler erzeugen");

    match err {
        ResidueMeasurementError::ResidueDetected { phase, matches } => {
            assert_eq!(phase, ResiduePhase::PostEraseScan);
            assert!(matches > 0);
        }
        other => panic!("Erwartete ResidueDetected, erhielt: {:?}", other),
    }
}

#[test]
fn inv_no_resurrection_1_erased_entry_cannot_reappear() {
    let mut index = InMemoryVectorIndexStub::new(TEST_LAYER);
    let doc_id = DocId::new(1234);
    let canary = b"canary-1234".to_vec();
    index.register_canary(canary.clone());

    index.insert_document(doc_id, vec![10]);
    index.physical_erase(doc_id);

    let clock = SystemClock::new();
    let coordinator = ResidueMeasurementCoordinator::new(&index, &clock);
    let query = ResidueQuery::new(doc_id, [0x12; 32]);

    // 1. Messung sauber
    let report1 = coordinator
        .measure_after_erasure(&query, &canary, 1)
        .expect("Erste Messung muss sauber sein");
    assert_eq!(report1.post_erase_matches, 0);

    // 2. Wiederholter Scan ohne Mutation muss identisch 0 bleiben
    let report2 = coordinator
        .measure_after_erasure(&query, &canary, 1)
        .expect("Zweite Messung muss ebenso sauber sein");
    assert_eq!(report2.post_erase_matches, 0);
}

#[test]
fn inv_vault_failclosed_1_scanner_error_fails_closed() {
    let mut index = InMemoryVectorIndexStub::new(TEST_LAYER);
    let doc_id = DocId::new(888);
    let canary = b"canary-888".to_vec();
    index.register_canary(canary.clone());

    // Korruption im Index simulieren
    index.set_simulate_corrupt(Some(IndexCorruptionKind::DanglingBacklink));

    let clock = SystemClock::new();
    let coordinator = ResidueMeasurementCoordinator::new(&index, &clock);
    let query = ResidueQuery::new(doc_id, [0x88; 32]);

    let err = coordinator
        .measure_after_erasure(&query, &canary, 1)
        .expect_err("INV-VAULT-FAILCLOSED-1: Korruption muss fail-closed abbrechen");

    match err {
        ResidueMeasurementError::CorruptIndexState { reason, .. } => {
            assert_eq!(reason, IndexCorruptionKind::DanglingBacklink);
        }
        other => panic!("Erwartete CorruptIndexState, erhielt: {:?}", other),
    }
}

#[test]
fn inv_claim_bind_1_no_legal_warranty_wording() {
    // Prüft, dass im Fehlermeldungstext und in Typen keine rechtlich bindenden Garantien stehen
    let err = ResidueMeasurementError::ResidueDetected {
        phase: ResiduePhase::PostEraseScan,
        matches: 3,
    };
    let text = err.to_string();
    let lower = text.to_lowercase();

    assert!(!lower.contains("garantiert"), "Darf keine Rechtsgarantie enthalten");
    assert!(!lower.contains("dsgvo"), "Darf keine Rechtsgarantie enthalten");
    assert!(!lower.contains("revisionssicher"), "Darf keine Rechtsgarantie enthalten");
    assert!(!lower.contains("unwiederbringlich"), "Darf keine Rechtsgarantie enthalten");
}

// =========================================================================
// Hilfsfunktion für Test-Assertion
// =========================================================================

trait TestHelper {
    fn active_docs_count_for_test(&self, query: &ResidueQuery) -> u64;
}

impl TestHelper for InMemoryVectorIndexStub {
    fn active_docs_count_for_test(&self, query: &ResidueQuery) -> u64 {
        use contextra_erasure::measure::residue::ResidueScanner;
        self.scan_residue(query)
            .map(|r| r.total_matches)
            .unwrap_or(0)
    }
}
