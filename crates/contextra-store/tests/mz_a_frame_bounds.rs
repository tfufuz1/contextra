//! Additive Integrationstests für Framing-Grenzwerte und Header-Konstanten (Auftrag STORE-MOD-A).

use contextra_store::wal::{MAX_WAL_ENTRY_SIZE, MAX_WAL_SIZE, WAL_V2_HEADER, WAL_V3_HEADER};

#[test]
fn test_wal_frame_constants() {
    assert_eq!(WAL_V2_HEADER, *b"MFW2");
    assert_eq!(WAL_V3_HEADER, *b"MFW3");
    assert_eq!(MAX_WAL_SIZE, 128 * 1024 * 1024);
    assert_eq!(MAX_WAL_ENTRY_SIZE, 64 * 1024 * 1024);
}
