//! # Golden Vektoren für Run und Anhang A.6
//!
//! SPEC §9.5, Anhang A.6, A.8 (Run in Plain).

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::indexing_slicing, clippy::panic)]

use contextra_storage::codec::tag::{Tag, TagKind};
use contextra_storage::run::blob_ref::BlobRef;
use contextra_storage::run::block::BlockSecurity;
use contextra_storage::run::builder::{RunBuilder, RunBuilderOptions};
use contextra_storage::run::footer::RUN_MAGIC;
use contextra_storage::run::reader::RunReader;
use std::sync::Arc;

use super::roundtrip::MemoryFile;

#[test]
fn test_golden_a6_blobref_vector() {
    // BlobRef (28 B): blob_no = 0x71, offset = 4096, len = 1000, crc32c = 0xDEADBEEF, domain = 42
    let bref = BlobRef::new(0x71, 4096, 1000, 0xDEADBEEF, 42);
    let bytes = bref.encode();
    assert_eq!(
        bytes,
        [
            0x71, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // blob_no
            0x00, 0x10, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // offset = 4096
            0xe8, 0x03, 0x00, 0x00,                         // len = 1000
            0xef, 0xbe, 0xad, 0xde,                         // crc32c = 0xDEADBEEF
            0x2a, 0x00, 0x00, 0x00,                         // domain = 42
        ]
    );
}

#[test]
fn test_golden_run_plain_layout() {
    // Run mit einem Eintrag in Plain (SPEC §9.5, Anhang A.8)
    let opts = RunBuilderOptions {
        space_id: 1,
        run_id: 0x1000,
        header_len: 4,
        weights_k: 1,
        prefix_filter_len: Some(3),
        created_ms: 1_700_000_000_000,
    };
    let mut builder = RunBuilder::new(opts, BlockSecurity::Plain);
    builder.add(b"key1", Tag::new(1, TagKind::Put), b"hdr_val1").unwrap();
    let bytes = builder.finish().unwrap();

    let file_len = bytes.len() as u64;
    assert!(file_len > 28);

    // Footer prüfen (letzte 28 B)
    let footer_slice = &bytes[bytes.len() - 28..];
    assert_eq!(&footer_slice[0..4], RUN_MAGIC);
    assert_eq!(&footer_slice[24..28], RUN_MAGIC);

    // Reader öffnen und prüfen
    let file = Arc::new(MemoryFile(Arc::new(bytes)));
    let reader = RunReader::open(file, file_len, BlockSecurity::Plain, None).unwrap();

    assert_eq!(reader.meta().space_id, 1);
    assert_eq!(reader.meta().run_id, 0x1000);
    assert_eq!(reader.meta().entries, 1);

    // Directory-Prüfung
    assert!(reader.index().entries.len() == 1);
    let entry = reader.get(b"key1").unwrap().unwrap();
    assert_eq!(entry.value, b"hdr_val1");
}
