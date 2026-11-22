#![allow(clippy::unwrap_used, clippy::expect_used, clippy::indexing_slicing)]

use contextra_storage::blob::reader::{BlobReadError, BlobReader};
use contextra_storage::blob::record::BlobRecord;
use contextra_storage::blob::sweep::sweep_blob;
use contextra_storage::blob::writer::BlobWriter;
use contextra_storage::testing::MemFs;
use contextra_storage_ports::{Fs, OpenMode, Perm};
use std::collections::HashSet;
use std::path::Path;

#[test]
fn test_blob_sweep_generation_exchange_and_holes() {
    let fs = MemFs::new();
    let src_path = Path::new("blob_g1.ctxb");
    let src_file = fs.open(src_path, OpenMode::CreateNew(Perm::Private)).unwrap();

    let mut writer = BlobWriter::new(10, 1, 0, 0); // blob_no=10, gen=1, Plain=0
    let rec1 = BlobRecord::new_body(0, 1, b"key1".to_vec(), b"body1_keep_alive".to_vec());
    let rec2 = BlobRecord::new_body(0, 2, b"key2".to_vec(), b"body2_delete_me".to_vec());
    let rec3 = BlobRecord::new_body(0, 3, b"key3".to_vec(), b"body3_keep_alive".to_vec());

    let ref1 = writer.append(&*src_file, &rec1, None, None, None, None, None).unwrap();
    let ref2 = writer.append(&*src_file, &rec2, None, None, None, None, None).unwrap();
    let ref3 = writer.append(&*src_file, &rec3, None, None, None, None, None).unwrap();
    writer.finish(&*src_file).unwrap();

    // Sweep vorbereiten: rec1 und rec3 sind lebendig, rec2 wird gelöscht (Hole)
    let mut live_offsets = HashSet::new();
    live_offsets.insert(ref1.offset);
    live_offsets.insert(ref3.offset);

    let dst_path = Path::new("blob_g2.ctxb");
    let dst_file = fs.open(dst_path, OpenMode::CreateNew(Perm::Private)).unwrap();

    let new_header = sweep_blob(&*src_file, &*dst_file, &live_offsets, 0, None, None).unwrap();
    assert_eq!(new_header.generation, 2);
    assert_eq!(new_header.blob_no, 10);

    // Ziel-Datei prüfen
    let reader_g2 = BlobReader::open(&*dst_file, 0).unwrap();
    assert_eq!(reader_g2.header().generation, 2);

    // 1. Lebendige Records behalten ihre Offsets und Bytes (INV-BLOB-1, INV-BLOB-2)
    let body1 = reader_g2.read(&*dst_file, &ref1, b"key1", 1, None, None, None, None, None).unwrap();
    assert_eq!(body1, b"body1_keep_alive");

    let body3 = reader_g2.read(&*dst_file, &ref3, b"key3", 3, None, None, None, None, None).unwrap();
    assert_eq!(body3, b"body3_keep_alive");

    // 2. Gelöschter Record wurde durch unverschlüsseltes Hole ersetzt (INV-BLOB-3)
    // INV-BLOB-3: ein BlobRef auf ein Hole liefert HoleRef
    let err_hole = reader_g2.read(&*dst_file, &ref2, b"key2", 2, None, None, None, None, None);
    assert!(matches!(err_hole, Err(BlobReadError::HoleRef)));

    // Dateigröße bleibt identisch (gleiche Offsets)
    assert_eq!(src_file.len().unwrap(), dst_file.len().unwrap());
}
