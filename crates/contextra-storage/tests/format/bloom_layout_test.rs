//! # Layout des Bloom-Filters (SPEC §10.5.1)
//!
//! `k = 7` fest; `nbits = max(64, n·10)` aufgerundet auf ein Vielfaches von 64.
//! Abweichende Dekodierung ist `Corrupt`, nie stilles Akzeptieren (P-4, INV-FMT-1).

use contextra_storage::codec::bloom::{BloomError, BloomFilter};

fn encoded(k: u8, nbits: u64, bit_bytes: usize) -> Vec<u8> {
    let mut v = vec![k];
    v.extend_from_slice(&nbits.to_le_bytes());
    v.extend(std::iter::repeat_n(0u8, bit_bytes));
    v
}

#[test]
fn minimale_groesse_64_bits_wird_akzeptiert() {
    assert!(BloomFilter::decode(&encoded(7, 64, 8)).is_ok());
}

#[test]
fn nbits_null_ist_corrupt() {
    assert_eq!(
        BloomFilter::decode(&encoded(7, 0, 0)),
        Err(BloomError::Corrupt)
    );
}

#[test]
fn nbits_unter_64_ist_corrupt() {
    assert_eq!(
        BloomFilter::decode(&encoded(7, 8, 1)),
        Err(BloomError::Corrupt)
    );
}

#[test]
fn nbits_kein_vielfaches_von_64_ist_corrupt() {
    assert_eq!(
        BloomFilter::decode(&encoded(7, 65, 9)),
        Err(BloomError::Corrupt)
    );
}

#[test]
fn k_abweichend_von_7_ist_corrupt() {
    assert_eq!(
        BloomFilter::decode(&encoded(6, 64, 8)),
        Err(BloomError::Corrupt)
    );
    assert_eq!(
        BloomFilter::decode(&encoded(0, 64, 8)),
        Err(BloomError::Corrupt)
    );
}

#[test]
fn insert_in_leeren_filter_paniert_nicht() {
    let mut f = BloomFilter::with_explicit_bits(7, 0, Vec::new());
    f.insert(b"k");
    assert!(!f.may_contain(b"k"));
}

#[test]
fn neuer_filter_hat_layout_nach_spec() {
    // n = 3 Schlüssel, 10 Bits je Schlüssel: 30 -> min 64 -> 64 Bit = 8 Byte.
    let f = BloomFilter::new(3, 10);
    assert_eq!(f.k(), 7);
    assert_eq!(f.nbits(), 64);
    assert_eq!(f.as_bytes().len(), 8);
    // n = 100: 1000 Bit -> aufgerundet auf 1024 Bit = 128 Byte.
    let g = BloomFilter::new(100, 10);
    assert_eq!(g.nbits(), 1024);
}
