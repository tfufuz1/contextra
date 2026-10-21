// FILE-CONTEXT
// STAND: 2026-10-21
// ZWECK: Mandatory taxonomy and phase unit tests (§WP-1, §WP-2).

use contextra_ports::erasure::{PurgeBlocked, PurgeError};
use contextra_store::lsm::purge::plan::PurgePhase;
use contextra_ports::strum::IntoEnumIterator;

#[test]
fn purge_error_classification() {
    for variant in PurgeError::iter() {
        match variant {
            PurgeError::Blocked(b) => {
                let err_str = b.to_string();
                assert!(!err_str.is_empty());
                assert!(!b.retry_hint().is_empty());
            }
            PurgeError::Read { .. } => {}
            PurgeError::Write { .. } => {}
            PurgeError::Manifest(..) => {}
            PurgeError::Journal(..) => {}
            PurgeError::Fence(..) => {}
            PurgeError::Intent(..) => {}
            PurgeError::ScopeUnavailable { .. } => {}
            PurgeError::ScopeMismatch { .. } => {}
            PurgeError::ResidueFound { .. } => {}
            PurgeError::DisposeRefused(..) => {}
            #[cfg(any(test, feature = "fault-injection"))]
            PurgeError::Failpoint(..) => {}
        }
    }
}

#[test]
fn hard_io_error_never_maps_to_blocked() {
    let write_err = PurgeError::Write {
        file: "test.sst".to_string(),
        source_details: "Disk I/O error".to_string(),
    };

    assert!(!matches!(write_err, PurgeError::Blocked(_)));
}

#[test]
fn phase_codes_roundtrip() {
    let phases = [
        PurgePhase::Admitted,
        PurgePhase::Fenced,
        PurgePhase::Flushed,
        PurgePhase::Rewriting,
        PurgePhase::Disposing,
        PurgePhase::WalRetiring,
        PurgePhase::Witnessing,
        PurgePhase::Finalizing,
        PurgePhase::Released,
    ];

    for phase in phases {
        let code = phase.to_u8();
        let parsed = PurgePhase::try_from(code).unwrap();
        assert_eq!(phase, parsed);
    }
}

#[test]
fn phase_code_zero_rejected() {
    let res = PurgePhase::try_from(0);
    assert!(res.is_err());
    match res.unwrap_err() {
        PurgeError::Intent(msg) => {
            assert!(msg.contains("Phase code 0 is invalid"));
        }
        other => panic!("Expected PurgeError::Intent, got {:?}", other),
    }
}

#[test]
fn phase_order_total() {
    assert!(PurgePhase::Admitted < PurgePhase::Fenced);
    assert!(PurgePhase::Fenced < PurgePhase::Flushed);
    assert!(PurgePhase::Flushed < PurgePhase::Rewriting);
    assert!(PurgePhase::Rewriting < PurgePhase::Disposing);
    assert!(PurgePhase::Disposing < PurgePhase::WalRetiring);
    assert!(PurgePhase::WalRetiring < PurgePhase::Witnessing);
    assert!(PurgePhase::Witnessing < PurgePhase::Finalizing);
    assert!(PurgePhase::Finalizing < PurgePhase::Released);

    assert!(PurgePhase::Admitted.must_run(PurgePhase::Fenced));
    assert!(!PurgePhase::Fenced.must_run(PurgePhase::Fenced));
}

#[test]
fn legacy_phase_code_rejected_fail_closed() {
    let legacy_codes = [0u8, 10u8, 11u8, 255u8];
    for code in legacy_codes {
        let res = PurgePhase::try_from(code);
        assert!(res.is_err(), "Code {} must be rejected fail-closed", code);
    }
}
