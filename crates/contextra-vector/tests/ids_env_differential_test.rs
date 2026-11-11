//! Differenztest für IDs und Environment (WP6 / SPEC §5.4).

use contextra_vector::types::{Clock, DocId, Rng, SeededRng, SystemClock, TxId};

#[test]
fn test_doc_id_differential() {
    let id1 = DocId::from(42u64);
    assert_eq!(id1.inner(), 42);

    let key = "document-12345";
    let hash = blake3::hash(key.as_bytes());
    let mut expected_bytes = [0u8; 8];
    expected_bytes.copy_from_slice(&hash.as_bytes()[..8]);
    let expected_u64 = u64::from_le_bytes(expected_bytes);

    let from_key_id = DocId::from_key(key).expect("from_key");
    assert_eq!(from_key_id.as_u64(), expected_u64);
}

#[test]
fn test_tx_id_differential() {
    let tx = TxId::new(100);
    assert_eq!(tx.as_u64(), 100);
    assert_eq!(tx.inner(), 100);
}

#[test]
fn test_env_differential() {
    let rng = SeededRng::new(12345);
    let val1 = rng.next_u64();
    let val2 = rng.next_u64();
    assert_ne!(val1, val2);

    let clock = SystemClock::new();
    assert!(clock.now_unix_nanos() > 0);
}
