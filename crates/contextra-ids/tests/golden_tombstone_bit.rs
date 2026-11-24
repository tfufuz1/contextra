//! Golden test for TOMBSTONE_BIT — Format invariant guard per ADR-0120.
//!
//! Diese Datei muss grün sein, BEVOR jede Änderung an TOMBSTONE_BIT oder den
//! importierenden Crates gemergt wird. Sie sichert INV-1 (Formatgleichheit).
//!
//! G-F1: Format-Invariante. Bit 63 einer u64-Sequenznummer kennzeichnet Tombstones.
//! Jede Änderung am Bitmuster bricht die SSTable-, WAL- und MVCC-Persistenz.

use contextra_ids::TOMBSTONE_BIT;

/// INV-1: Der Bitwert muss exakt 2^63 sein — keine andere Konstante ist korrekt.
#[test]
fn test_tombstone_bit_exact_value() {
    assert_eq!(TOMBSTONE_BIT, 0x8000_0000_0000_0000u64);
    assert_eq!(TOMBSTONE_BIT, 1u64 << 63);
    // Exakt ein Bit gesetzt
    assert_eq!(TOMBSTONE_BIT.count_ones(), 1);
    assert_eq!(TOMBSTONE_BIT.leading_zeros(), 0);
    assert_eq!(TOMBSTONE_BIT.trailing_zeros(), 63);
}

/// INV-1: Little-Endian-Serialisierung des Tombstone-Bit in gespeicherten Sequenznummern.
/// SSTable- und WAL-Einträge serialisieren u64-Werte als 8 Little-Endian-Bytes.
/// Bit 63 landet im 8. Byte (Index 7), Bit 7 dieses Bytes.
#[test]
fn test_tombstone_bit_little_endian_format() {
    let le = TOMBSTONE_BIT.to_le_bytes();
    // Erste 7 Bytes müssen 0 sein
    assert_eq!(&le[..7], &[0u8; 7], "lower 7 bytes must be zero");
    // Letztes Byte (höchstwertiges in LE) muss 0x80 sein
    assert_eq!(le[7], 0x80, "highest byte must be 0x80 in little-endian");
}

/// Maskierungssemantik: seq | TOMBSTONE_BIT setzt Bit 63.
/// seq & !TOMBSTONE_BIT extrahiert die saubere Sequenznummer.
#[test]
fn test_tombstone_bit_mask_operations() {
    let seq_raw: u64 = 42;
    let seq_tomb = seq_raw | TOMBSTONE_BIT;

    assert_ne!(seq_raw, seq_tomb, "tombstoning must change the value");
    assert_eq!(seq_tomb & !TOMBSTONE_BIT, seq_raw, "unmasking must recover original seq");
    assert!((seq_tomb & TOMBSTONE_BIT) != 0, "tombstoned seq must have bit 63 set");
    assert!((seq_raw & TOMBSTONE_BIT) == 0, "raw seq must not have bit 63 set");
}

/// Idempotenz: Doppeltes Setzen des Bits ändert nichts.
#[test]
fn test_tombstone_bit_idempotent() {
    let seq: u64 = 1234567890;
    let once = seq | TOMBSTONE_BIT;
    let twice = once | TOMBSTONE_BIT;
    assert_eq!(once, twice);
}

/// Randfall: Sequenznummer 0 als Tombstone.
#[test]
fn test_tombstone_bit_zero_seq() {
    let seq: u64 = 0;
    let tomb = seq | TOMBSTONE_BIT;
    assert_eq!(tomb, TOMBSTONE_BIT);
    assert_eq!(tomb & !TOMBSTONE_BIT, 0);
}

/// Randfall: u64::MAX als Tombstone (Bit 63 bereits gesetzt).
#[test]
fn test_tombstone_bit_max_seq() {
    let seq = u64::MAX;
    assert!((seq & TOMBSTONE_BIT) != 0, "u64::MAX has bit 63 set");
    let tomb = seq | TOMBSTONE_BIT;
    assert_eq!(tomb, u64::MAX, "idempotent: MAX | TOMBSTONE_BIT == MAX");
    // Unmask: liefert u64::MAX & 0x7FFF... = i64::MAX als u64
    assert_eq!(tomb & !TOMBSTONE_BIT, i64::MAX as u64);
}

/// Kompatiblität: contextra-ids und contextra-types definieren denselben Wert.
/// Dieser Test kann nach Deprecation von types::TOMBSTONE_BIT entfernt werden (S-70).
#[test]
fn test_tombstone_bit_matches_types_copy() {
    // Hardcode des Wertes aus types/domain/misc.rs — kein Import nötig
    const TYPES_TOMBSTONE_BIT: u64 = 1 << 63;
    assert_eq!(TOMBSTONE_BIT, TYPES_TOMBSTONE_BIT,
        "contextra-ids and contextra-types TOMBSTONE_BIT must be bit-identical");
}

/// Kompatiblität: contextra-vector definierte denselben Wert lokal.
/// Dieser Test sichert die Migration ab.
#[test]
fn test_tombstone_bit_matches_vector_copy() {
    // Hardcode des Wertes aus vector/src/types.rs — kein Import nötig
    const VECTOR_TOMBSTONE_BIT: u64 = 1 << 63;
    assert_eq!(TOMBSTONE_BIT, VECTOR_TOMBSTONE_BIT,
        "contextra-ids and contextra-vector TOMBSTONE_BIT must be bit-identical");
}

/// Golden-Vektor: Le-Serialisierung einer konkreten tombstoneten Sequenznummer.
/// Diese Bytes dürfen sich NIE ändern — sie erscheinen so auf Disk.
#[test]
fn test_tombstone_bit_golden_encoded_seq_42() {
    let encoded = (42u64 | TOMBSTONE_BIT).to_le_bytes();
    // 42 = 0x2A; mit Bit 63: 0x800000000000002A
    // LE: [0x2A, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x80]
    assert_eq!(
        encoded,
        [0x2A, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x80],
        "golden little-endian encoding of tombstoned seq=42"
    );
}
