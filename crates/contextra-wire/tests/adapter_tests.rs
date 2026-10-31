//! # Integrationstests für `adapter.rs` (`WireBuffer`)
//!
//! **Zweck:**
//! Direkte, umfassende Verifikation aller öffentlichen Methoden und Eigenschaften von `WireBuffer`:
//! `new`, `as_slice`, `into_inner`, `AsRef<[u8]>`, `Clone`, `Debug`, leere Puffer, große Puffer,
//! Referenzzähler-Semantik und multithreaded Sharing ohne Kopiervorgänge.
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]

use bytes::Bytes;
use contextra_wire::WireBuffer;
use std::sync::Arc;
use std::thread;

#[test]
fn test_wire_buffer_new_and_as_slice() {
    let raw = b"hello contextra wire buffer";
    let bytes_obj = Bytes::from_static(raw);
    let buffer = WireBuffer::new(bytes_obj);

    assert_eq!(buffer.as_slice(), raw);
    assert_eq!(buffer.as_slice().len(), raw.len());
}

#[test]
fn test_wire_buffer_into_inner() {
    let raw = vec![1, 2, 3, 4, 5, 42];
    let bytes_obj = Bytes::copy_from_slice(&raw);
    let buffer = WireBuffer::new(bytes_obj.clone());

    let inner = buffer.into_inner();
    assert_eq!(inner, bytes_obj);
    assert_eq!(&inner[..], &raw[..]);
}

#[test]
fn test_wire_buffer_as_ref() {
    let raw = b"as_ref test data";
    let buffer = WireBuffer::new(Bytes::from_static(raw));

    let slice_ref: &[u8] = buffer.as_ref();
    assert_eq!(slice_ref, raw);
}

#[test]
fn test_wire_buffer_empty() {
    let empty_bytes = Bytes::new();
    let buffer = WireBuffer::new(empty_bytes);

    assert!(buffer.as_slice().is_empty());
    assert_eq!(buffer.as_slice().len(), 0);
    assert_eq!(buffer.as_ref(), &[] as &[u8]);

    let inner = buffer.into_inner();
    assert!(inner.is_empty());
}

#[test]
fn test_wire_buffer_clone_and_reference_counting() {
    let payload = vec![0xAA; 1024];
    let original = WireBuffer::new(Bytes::copy_from_slice(&payload));
    let cloned = original.clone();

    assert_eq!(original.as_slice(), cloned.as_slice());
    assert_eq!(original.as_slice().as_ptr(), cloned.as_slice().as_ptr());

    // Beide Puffer zeigen auf denselben Speicherbereich
    drop(original);
    assert_eq!(cloned.as_slice().len(), 1024);
    assert_eq!(cloned.as_slice()[0], 0xAA);
}

#[test]
fn test_wire_buffer_debug_formatting() {
    let buffer = WireBuffer::new(Bytes::from_static(b"debug"));
    let debug_str = format!("{buffer:?}");
    assert!(debug_str.contains("WireBuffer"));
    assert!(debug_str.contains("data"));
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
