//! # Epochen Tests
//!
//! SPEC §15.2, SD-63, §18 (INV-EPOCH-1):
//! Verifiziert Schlüsselepoch-Inkrementierung und Gültigkeitsprüfungen.

use contextra_storage::keys::epoch::{EpochTracker, KeyEpoch};

#[test]
fn test_epoch_initial_and_next() {
    let e0 = KeyEpoch::INITIAL;
    assert_eq!(e0.as_u32(), 0);

    // // INV-EPOCH-1: Epochensprung beim Klonen
    let e1 = e0.next_clone_epoch();
    assert_eq!(e1.as_u32(), 1);

    let e2 = e1.next_clone_epoch();
    assert_eq!(e2.as_u32(), 2);
}

#[test]
fn test_epoch_tracker_validations() {
    let mut tracker = EpochTracker::new(KeyEpoch::new(1));
    assert_eq!(tracker.current_epoch(), KeyEpoch::new(1));

    // Lesezugriffe: Alles <= current_epoch ist gültig
    assert!(tracker.is_valid_read_epoch(KeyEpoch::new(0)));
    assert!(tracker.is_valid_read_epoch(KeyEpoch::new(1)));
    assert!(!tracker.is_valid_read_epoch(KeyEpoch::new(2)));

    // Schreibzugriffe: Nur aktuelle Epoche ist gültig
    assert!(!tracker.is_valid_write_epoch(KeyEpoch::new(0)));
    assert!(tracker.is_valid_write_epoch(KeyEpoch::new(1)));

    // Klonen erhöht die Epoche
    let new_epoch = tracker.advance_for_clone();
    assert_eq!(new_epoch, KeyEpoch::new(2));
    assert_eq!(tracker.current_epoch(), KeyEpoch::new(2));

    assert!(tracker.is_valid_read_epoch(KeyEpoch::new(2)));
    assert!(tracker.is_valid_write_epoch(KeyEpoch::new(2)));
    assert!(!tracker.is_valid_write_epoch(KeyEpoch::new(1)));
}
