#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]

// FILE-CONTEXT
// ZWECK: Integration tests for CchSecureBuf zeroize behavior and safe memory inspection.
// STAND: TS:2026-10-12T00:00:00Z

#![forbid(unsafe_code)]

use contextra_crypto::CchSecureBuf;

#[test]
fn test_secure_buf_basic_operations() {
    let raw = vec![0xAB; 32];
    let buf = CchSecureBuf::from_vec(raw);

    assert_eq!(buf.len(), 32);
    assert!(!buf.is_empty());
    assert_eq!(buf.as_slice(), &[0xAB; 32]);

    let debug_repr = format!("{buf:?}");
    assert_eq!(debug_repr, "CchSecureBuf { len: 32 }");
    assert!(!debug_repr.contains("171")); // 0xAB redacting check
}

#[test]
fn test_secure_buf_zeroize_wipes_memory_safely() {
    let raw = vec![0xDE, 0xAD, 0xBE, 0xEF];
    let mut buf = CchSecureBuf::from_vec(raw);

    assert_eq!(buf.as_slice(), &[0xDE, 0xAD, 0xBE, 0xEF]);

    // Explicitly zeroize buffer
    buf.zeroize();

    assert_eq!(
        buf.as_slice(),
        &[0x00, 0x00, 0x00, 0x00],
        "Buffer memory MUST be zeroed after zeroize call"
    );
}

#[test]
fn test_secure_buf_into_zeroizing() {
    let raw = vec![1, 2, 3, 4, 5];
    let buf = CchSecureBuf::from_vec(raw);

    let z = buf.into_zeroizing();
    assert_eq!(z.as_slice(), &[1, 2, 3, 4, 5]);
}
