//! # Integrationstests für `adapter.rs` (`WireBuffer`)
//!
//! Direkte, umfassende Verifikation aller öffentlichen Methoden und Eigenschaften von `WireBuffer`:
//! 1. Allokations- und Kopiervorgänge (Zero-Copy).
//! 2. Typzustand `Unverified` -> `Verified<T>` gemäß `INV-WIRE-BOUNDS-1`.
//! 3. Thread-Sicherheit bei geteiltem Zugriff (`Arc<WireBuffer>`).
//! 4. Dereferenzierung und Konvertierung nach `&[u8]` und `Bytes`.
//! 5. Verhalten bei leeren und großen Puffern.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]

use bytes::Bytes;
use contextra_wire::adapter::{Unverified, Verified, WireBuffer};
use contextra_wire::contextra_generated::contextra::ipc::{Embedding, EmbeddingArgs};
use contextra_wire::verification::FlatBufferConfig;
use std::sync::Arc;
use std::thread;

#[test]
fn test_wire_buffer_new_and_as_slice() {
    let raw = b"hello_world";
    let bytes_obj = Bytes::from_static(raw);
    let buffer: WireBuffer<Unverified> = WireBuffer::new(bytes_obj);

    assert_eq!(buffer.as_slice(), raw);
    assert_eq!(buffer.len(), raw.len());
    assert!(!buffer.is_empty());
}

#[test]
fn test_wire_buffer_into_inner() {
    let raw = b"zero_copy_payload";
    let bytes_obj = Bytes::from_static(raw);
    let buffer = WireBuffer::new(bytes_obj.clone());

    let inner = buffer.into_inner();
    assert_eq!(inner, bytes_obj);
    assert_eq!(inner.as_ref(), raw);
}

#[test]
fn test_wire_buffer_as_ref() {
    let raw = b"as_ref_test";
    let buffer = WireBuffer::new(Bytes::from_static(raw));

    fn check_ref<T: AsRef<[u8]>>(val: &T, expected: &[u8]) {
        assert_eq!(val.as_ref(), expected);
    }

    check_ref(&buffer, raw);
}

#[test]
fn test_wire_buffer_empty() {
    let empty_bytes = Bytes::new();
    let buffer = WireBuffer::new(empty_bytes);

    assert!(buffer.is_empty());
    assert_eq!(buffer.len(), 0);
    assert_eq!(buffer.as_slice(), b"");
}

#[test]
fn test_wire_buffer_clone_and_reference_counting() {
    let payload = vec![0xAA; 1024];
    let original = WireBuffer::new(Bytes::copy_from_slice(&payload));

    let cloned = original.clone();
    assert_eq!(original.as_slice(), cloned.as_slice());
    assert_eq!(original.len(), cloned.len());

    let inner_orig = original.into_inner();
    let inner_clone = cloned.into_inner();

    assert_eq!(inner_orig, inner_clone);
}

#[test]
fn test_wire_buffer_debug_formatting() {
    let buffer = WireBuffer::new(Bytes::from_static(b"debug"));
    let debug_str = format!("{buffer:?}");
    assert!(debug_str.contains("WireBuffer"));
}

#[test]
fn test_wire_buffer_multithreaded_sharing() {
    let payload = vec![0x55; 4096];
    let buffer = Arc::new(WireBuffer::new(Bytes::copy_from_slice(&payload)));

    let mut handles = Vec::new();
    for thread_idx in 0..8 {
        let buf_clone = Arc::clone(&buffer);
        handles.push(thread::spawn(move || {
            let slice = buf_clone.as_slice();
            assert_eq!(slice.len(), 4096);
            for (byte_idx, &byte) in slice.iter().enumerate() {
                assert_eq!(
                    byte, 0x55,
                    "Mismatch at thread {thread_idx}, byte {byte_idx}"
                );
            }
        }));
    }

    for handle in handles {
        handle.join().expect("thread join succeeded");
    }
}

/// SPEC: INV-WIRE-BOUNDS-1
/// Verifiziert, dass ein WireBuffer den Typzustand Unverified -> Verified<T> durchläuft
/// und unzulässige Puffer abgefangen werden.
#[test]
fn test_inv_wire_bounds_1_typestate_transition() {
    let mut builder = flatbuffers::FlatBufferBuilder::new();
    let emb_args = EmbeddingArgs {
        metric: 1,
        ..Default::default()
    };
    let emb_offset = Embedding::create(&mut builder, &emb_args);
    builder.finish(emb_offset, None);
    let valid_data = Bytes::copy_from_slice(builder.finished_data());

    let unverified: WireBuffer<Unverified> = WireBuffer::new(valid_data);
    let config = FlatBufferConfig::default();

    let verified: WireBuffer<Verified<Embedding>> = unverified
        .verify::<Embedding>(&config)
        .expect("Valid FlatBuffer verifies successfully");

    let root = verified.root().expect("Root extraction succeeds");
    assert_eq!(root.metric(), 1);

    // Ungültiger Puffer (abgeschnitten)
    let bad_data = Bytes::from_static(&[0x01, 0x02]);
    let unverified_bad = WireBuffer::new(bad_data);
    let err = unverified_bad.verify::<Embedding>(&config);
    assert!(err.is_err(), "Corrupt buffer must fail verification");
}
