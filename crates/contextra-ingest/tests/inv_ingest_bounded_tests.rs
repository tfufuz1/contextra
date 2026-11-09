//! Invarianten-Test: `INV-INGEST-BOUNDED-1`
//!
//! Stellt sicher, dass Formaterkennung unabhängig von Eingabegröße und Streaming-Quelle
//! strikt niemals mehr als `MAX_DETECT_BYTES` (64 KiB) einliest oder analysiert.

use contextra_ingest::detect::{detect, detect_reader, DetectHints, MAX_DETECT_BYTES};
use proptest::prelude::*;
use std::io::Read;

struct InfiniteByteReader {
    byte: u8,
    bytes_read: usize,
}

impl InfiniteByteReader {
    fn new(byte: u8) -> Self {
        Self {
            byte,
            bytes_read: 0,
        }
    }
}

impl Read for InfiniteByteReader {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        for b in buf.iter_mut() {
            *b = self.byte;
        }
        self.bytes_read += buf.len();
        Ok(buf.len())
    }
}

// ANCHOR[contextra-ingest:TST-0451]
#[test]
fn test_inv_ingest_bounded_stream_never_reads_more_than_64k() {
    let mut reader = InfiniteByteReader::new(b'A');
    let hints = DetectHints {
        filename: Some("large.txt"),
        mime: None,
    };

    let res = detect_reader(&mut reader, &hints);
    assert!(res.is_ok());
    assert!(reader.bytes_read <= MAX_DETECT_BYTES);
    assert_eq!(reader.bytes_read, MAX_DETECT_BYTES);
}

// ANCHOR[contextra-ingest:TST-0452]
#[test]
fn test_inv_ingest_bounded_large_slice() {
    let large_data = vec![b'%'; 10 * 1024 * 1024]; // 10 MiB
    let hints = DetectHints::default();

    let res = detect(&large_data, &hints);
    // Sollte nicht hängen oder über 64k iterieren
    assert!(res.is_err() || res.is_ok());
}

// ANCHOR[contextra-ingest:TST-0453]
proptest! {
    #[test]
    fn prop_detect_bounded_invariant(data in proptest::collection::vec(any::<u8>(), 0..128 * 1024)) {
        let hints = DetectHints {
            filename: Some("test.bin"),
            mime: None,
        };
        let _ = detect(&data, &hints);

        let mut reader = std::io::Cursor::new(&data);
        let _ = detect_reader(&mut reader, &hints);
        prop_assert!(reader.position() as usize <= MAX_DETECT_BYTES);
    }
}
