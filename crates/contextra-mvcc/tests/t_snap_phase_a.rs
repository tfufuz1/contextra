//! # Phase A Tests: T-SNAP-1, T-VER-1, T-LIMIT-1, und Property-Test "Kein Sentinel"
//!
//! Prüft:
//! - Property-Test „kein Sentinel“: Für beliebige Folgen aus open und drop ist min_read_snapshot bei leerer Registry None, und die Kompaktionsgrenze ist current_seq (MV-01, MV-02, I-12).
//! - T-SNAP-1 mit zwei Mandanten: Widerruf wirkt nur auf Snapshots des Mandanten des Ziels mit seq <= gated_seq (MV-03, D-61). Snapshots anderer Mandanten bleiben unberührt, auch ihre Zählung.
//! - T-VER-1: nach Widerruf liefert SnapshotHandle::seq() SnapshotRevoked; laufende Iteration prüft vor jedem Block.
//! - T-LIMIT-1: 1024 Snapshots je Mandant, Überschreitung => Limit::Snapshots (MV-04).

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use contextra_mvcc::error::MvccError;
use contextra_mvcc::snapshot::{SnapshotRegistry, TenantTag, MAX_SNAPSHOTS_PER_TENANT};
use proptest::prelude::*;
use std::sync::Arc;

const TENANT_A: TenantTag = [1u8; 16];
const TENANT_B: TenantTag = [2u8; 16];

#[test]
fn t_snap_1_two_tenants_revocation_isolation() {
    let registry = Arc::new(SnapshotRegistry::new());

    // Mandant A: 2 Snapshots (seq 50, 150)
    let s_a1 = registry.open(TENANT_A, 50).expect("open A1");
    let s_a2 = registry.open(TENANT_A, 150).expect("open A2");

    // Mandant B: 2 Snapshots (seq 50, 150)
    let s_b1 = registry.open(TENANT_B, 50).expect("open B1");
    let s_b2 = registry.open(TENANT_B, 150).expect("open B2");

    assert_eq!(registry.min_read_snapshot(), Some(50));

    // Widerruf nur für Mandant A mit gated_seq = 100
    let revoked_count = registry.revoke_covering(TENANT_A, &(), 100);
    assert_eq!(revoked_count, 1, "Nur s_a1 sollte widerrufen werden");

    // T-VER-1: s_a1 liefert jetzt Err(SnapshotRevoked)
    assert_eq!(s_a1.seq(), Err(MvccError::SnapshotRevoked(50)));
    // s_a2 (seq 150 > 100) bleibt gültig
    assert_eq!(s_a2.seq(), Ok(150));

    // Mandant B bleibt vollständig unberührt!
    assert_eq!(s_b1.seq(), Ok(50));
    assert_eq!(s_b2.seq(), Ok(150));
    assert!(!registry.is_revoked_tenant(TENANT_B, 50));
    assert!(!registry.is_revoked_tenant(TENANT_B, 150));
}

#[test]
fn t_ver_1_immediate_revocation_checks_before_block() {
    let registry = Arc::new(SnapshotRegistry::new());
    let handle = registry.open(TENANT_A, 100).expect("open");

    assert_eq!(handle.seq(), Ok(100));

    // Simulierte Iteration: vor Block 0 ist alles ok
    assert!(handle.seq().is_ok());

    // Externer Widerruf trifft ein
    registry.revoke_covering(TENANT_A, &(), 100);

    // Nächster Block: sofort SnapshotRevoked
    let err = handle.seq().expect_err("muss widerrufen sein");
    assert_eq!(err, MvccError::SnapshotRevoked(100));
}

#[test]
fn t_limit_1_max_1024_snapshots_per_tenant() {
    let registry = Arc::new(SnapshotRegistry::new());
    let mut handles_a = Vec::with_capacity(MAX_SNAPSHOTS_PER_TENANT);

    for i in 0..MAX_SNAPSHOTS_PER_TENANT {
        let h = registry
            .open(TENANT_A, i as u64 + 1)
            .expect("innerhalb des Limits");
        handles_a.push(h);
    }

    // 1025. Snapshot für Mandant A muss mit LimitSnapshots fehlschlagen
    let overflow = registry.open(TENANT_A, 2000);
    assert_eq!(overflow.map(|_| ()), Err(MvccError::LimitSnapshots));

    // Mandant B ist davon unberührt und kann weiterhin Snapshots öffnen
    let handle_b = registry.open(TENANT_B, 1);
    assert!(handle_b.is_ok(), "Mandant B darf nicht blockiert sein");

    // Nach Freigabe eines Snapshots bei A kann A wieder öffnen
    handles_a.pop();
    let re_open = registry.open(TENANT_A, 2000);
    assert!(re_open.is_ok(), "Nach Freigabe wieder Platz frei");
}

proptest! {
    #[test]
    fn prop_kein_sentinel_empty_registry_is_none_compaction_is_current(
        seqs in proptest::collection::vec(1..1000u64, 0..30),
        current_seq in 1..2000u64
    ) {
        let registry = Arc::new(SnapshotRegistry::new());

        // Bei leerer Registry: min_read_snapshot == None, min_for_compaction == current_seq (nie u64::MAX)
        prop_assert_eq!(registry.min_read_snapshot(), None);
        prop_assert_eq!(registry.min_for_compaction(current_seq), current_seq);

        let mut handles = Vec::new();
        for &s in &seqs {
            let h = registry.open(TENANT_A, s).expect("open");
            handles.push(h);
        }

        if seqs.is_empty() {
            prop_assert_eq!(registry.min_read_snapshot(), None);
            prop_assert_eq!(registry.min_for_compaction(current_seq), current_seq);
        } else {
            let min_expected = *seqs.iter().min().unwrap();
            prop_assert_eq!(registry.min_read_snapshot(), Some(min_expected));
            let expected_compaction = min_expected.min(current_seq);
            prop_assert_eq!(registry.min_for_compaction(current_seq), expected_compaction);
        }

        // Alle schließen
        drop(handles);
        prop_assert_eq!(registry.min_read_snapshot(), None);
        prop_assert_eq!(registry.min_for_compaction(current_seq), current_seq);
    }
}
