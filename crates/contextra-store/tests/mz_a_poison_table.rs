//! Additive Integrationstests für die PoisonCause-Wertetabelle (Auftrag STORE-MOD-A).
//!
//! Validiert numerische Stabilität (Discriminant mapping) gemäß INV-FSYNC-1.

use contextra_store::wal::PoisonCause;

#[test]
fn test_poison_cause_discriminants_table() {
    let cases = [
        (PoisonCause::FsyncFailed, 1u8),
        (PoisonCause::WriteFailed, 2u8),
        (PoisonCause::TruncateFailed, 3u8),
        (PoisonCause::HmacChainDiverged, 4u8),
        (PoisonCause::TornWrite, 5u8),
    ];

    for (cause, expected_u8) in cases {
        assert_eq!(cause.to_u8(), expected_u8, "to_u8 mismatch for {:?}", cause);
        assert_eq!(
            PoisonCause::from_u8(expected_u8),
            Some(cause),
            "from_u8 mismatch for {}",
            expected_u8
        );
    }

    assert_eq!(PoisonCause::from_u8(0), None);
    assert_eq!(PoisonCause::from_u8(6), None);
    assert_eq!(PoisonCause::from_u8(255), None);
}
