#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
// ANCHOR[contextra-mvcc:TEST-SENTINEL-REGRESSION]
//! # Regressionstest T-MVCC-SENTINEL (Befund D-C02 / B-11)
//!
//! Prüft die Behebung des Snapshot-Sentinel-Fehlers an der Wurzel:
//! 1. Compaction / Flush ohne aktiven Snapshot darf `pruned_through` niemals auf einen
//!    Sentinel-Wert (`u64::MAX`) oder einen Wert größer als die höchste vergebene Commit-Sequenz setzen.
//! 2. Folge-Commits unter SSI mit Snapshot-Reads müssen erfolgreich durchgehen und dürfen
//!    nicht mehr an 'snapshot too old' scheitern.

use contextra_mvcc::error::MvccError;
use contextra_mvcc::ssi::{ReadSet, SequenceLogSsiValidator, SsiValidator};
use contextra_mvcc::SnapshotRegistry;
use contextra_types::{TxId, TOMBSTONE_BIT};
use std::sync::Arc;

#[test]
fn test_sentinel_prune_through_checked_none_and_invalid() {
    let validator = SequenceLogSsiValidator::new();

    // 1. None bound (kein aktiver Snapshot) -> Ok(()), pruned_through bleibt 0
    let res_none = validator.prune_through_checked(None);
    assert_eq!(res_none, Ok(()));
    assert_eq!(validator.pruned_through(), 0);

    // 2. u64::MAX bound -> Err(InvalidSequence)
    let res_max = validator.prune_through_checked(Some(u64::MAX));
    assert_eq!(res_max, Err(MvccError::InvalidSequence(u64::MAX)));
    assert_eq!(validator.pruned_through(), 0);

    // 3. Tombstone-Bit bound -> Err(InvalidSequence)
    let bad_seq = 100 | TOMBSTONE_BIT;
    let res_tomb = validator.prune_through_checked(Some(bad_seq));
    assert_eq!(res_tomb, Err(MvccError::InvalidSequence(bad_seq)));
    assert_eq!(validator.pruned_through(), 0);
}

#[test]
fn test_t_mvcc_sentinel_compaction_without_snapshot_succeeds() {
    let validator = SequenceLogSsiValidator::new();
    let snapshot_registry = Arc::new(SnapshotRegistry::new());

    // Zustand vor Compaction: keine aktiven Snapshots (None, kein Sentinel!)
    assert_eq!(snapshot_registry.min_read_snapshot(), None);

    // Initialer Commit bei seq 10
    validator.record_commit_key(b"key_a", 10);
    assert_eq!(validator.pruned_through(), 0);

    // Simulation des alten Aufrufers (prune_through ignoriert u64::MAX sicher)
    #[allow(deprecated)]
    validator.prune_through(u64::MAX);
    assert_eq!(
        validator.pruned_through(),
        0,
        "pruned_through must NOT be set to u64::MAX"
    );

    // Und mit der prune_through_checked API direkt mit Option<Seq>:
    let min_active = snapshot_registry.min_read_snapshot();
    validator
        .prune_through_checked(min_active)
        .expect("valid check");
    assert_eq!(validator.pruned_through(), 0);

    // Jetzt startet eine Transaktion mit Snapshot bei seq 10
    let mut read_set = ReadSet::new();
    read_set.record_read(b"key_a".to_vec(), 10);

    // Folge-Commit bei seq 12: Muss erfolgreich validieren und darf NICHT mit SnapshotTooOld scheitern!
    let tx2 = TxId::new(2);
    let val_res = validator.validate(tx2, &read_set);
    assert!(
        val_res.is_ok(),
        "Folge-Commit nach Compaction ohne Snapshots muss gelingen, scheiterte aber: {:?}",
        val_res
    );
}

#[test]
fn test_sentinel_u64_i64_boundary_handling() {
    // T-MVCC-SENTINEL-1: Verifiziert, dass Grenzwerte wie u64::MAX, i64::MAX as u64
    // und Sequenznummern mit gesetztem MSB / Tombstone-Bit nie fälschlicherweise als
    // i64::MAX interpretiert werden oder negatives Vorzeichen-Wrapping auslösen.
    let validator = SequenceLogSsiValidator::new();

    let i64_max_as_u64 = i64::MAX as u64;
    let beyond_i64_max = (i64::MAX as u64) + 1;

    // Prune mit i64::MAX as u64 ist eine gültige Sequenznummer (sofern kein Tombstone-Bit gesetzt)
    let res = validator.prune_through_checked(Some(i64_max_as_u64));
    assert!(res.is_ok());

    // Prune mit u64::MAX muss fail-closed mit InvalidSequence abgelehnt werden
    let res_max = validator.prune_through_checked(Some(u64::MAX));
    assert_eq!(res_max, Err(MvccError::InvalidSequence(u64::MAX)));

    // Prune mit beyond_i64_max (oberhalb Vorzeichengrenze von i64):
    // Solange TOMBSTONE_BIT nicht kollidiert, bleibt es eine unsigned u64 Sequenz
    let res_beyond = validator.prune_through_checked(Some(beyond_i64_max));
    if (beyond_i64_max & TOMBSTONE_BIT) != 0 {
        assert_eq!(res_beyond, Err(MvccError::InvalidSequence(beyond_i64_max)));
    } else {
        assert!(res_beyond.is_ok());
    }

    // Snapshot-Registry liefert bei leerem Zustand immer None, nie i64::MAX oder u64::MAX
    let registry = SnapshotRegistry::new();
    assert_eq!(registry.min_read_snapshot(), None);
    assert_eq!(registry.min_for_compaction(100), 100);
}
