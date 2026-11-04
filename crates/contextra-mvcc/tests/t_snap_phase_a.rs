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
use contextra_testmacros::ctx_test;
use proptest::prelude::*;
use std::sync::Arc;

const TENANT_A: TenantTag = [1u8; 16];
const TENANT_B: TenantTag = [2u8; 16];

#[ctx_test(id = "T-SNAP-1")]
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

#[ctx_test(id = "T-CONC-1")]
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

#[ctx_test(id = "T-LIMIT-1")]
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
#[ctx_test(id = "T-VER-1")]
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

// T-SNAP-PURGE-1: Test „Purge unter laufendem Snapshot“ laut SPEC §9.5, D-11, D-51, D-127.
// Leser auf einem aktiven Snapshot sehen entweder den konsistenten Altzustand
// oder werden bei Purge-Kollision fail-closed widerrufen (`SnapshotRevoked`),
// aber niemals einen unvollständigen/zerrissenen Zustand.
#[ctx_test(id = "T-SNAP-1")]
fn test_snap_purge_under_active_snapshot() {
    let registry = Arc::new(SnapshotRegistry::new());

    // 1. Snapshot für Tenant A bei seq = 50 öffnen
    let snap_50 = registry.open(TENANT_A, 50).expect("open snapshot 50");
    assert_eq!(snap_50.seq(), Ok(50));

    // 2. Snapshot für Tenant B bei seq = 50 öffnen
    let snap_b_50 = registry.open(TENANT_B, 50).expect("open snapshot B 50");
    assert_eq!(snap_b_50.seq(), Ok(50));

    // 3. Purge-Operation auf Tenant A für Daten bis einschließlich gated_seq = 60
    // Laut D-61 / D-127 widerruft dies nur Snapshots von Tenant A mit seq <= 60.
    let count = registry.revoke_covering(TENANT_A, &(), 60);
    assert_eq!(count, 1);

    // 4. Prüfung der fail-closed Semantik:
    // Snapshot von Tenant A muss als widerrufen fail-closed abgebrochen werden
    assert_eq!(snap_50.seq(), Err(MvccError::SnapshotRevoked(50)));

    // Snapshot von Tenant B darf unberührt bleiben (Mandantenisolation T-SNAP-1)
    assert_eq!(snap_b_50.seq(), Ok(50));

    // 5. Neuer Snapshot für Tenant A NACH dem Purge (seq = 70) ist aktiv und nicht widerrufen
    let snap_70 = registry.open(TENANT_A, 70).expect("open snapshot 70");
    assert_eq!(snap_70.seq(), Ok(70));
}
