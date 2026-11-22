#![allow(clippy::unwrap_used, clippy::expect_used, clippy::indexing_slicing)]

use contextra_crypto_ref::{Blake3Kdf, Blake3Mac, ChaChaCipher};
use contextra_storage::blob::reader::{BlobReadError, BlobReader};
use contextra_storage::blob::record::BlobRecord;
use contextra_storage::blob::writer::BlobWriter;
use contextra_storage::testing::MemFs;
use contextra_storage_ports::{Fs, OpenMode, Perm};
use std::path::Path;

#[test]
fn test_blob_writer_reader_plain() {
    let fs = MemFs::new();
    let path = Path::new("blob_01.ctxb");
    let file = fs.open(path, OpenMode::CreateNew(Perm::Private)).unwrap();

    let mut writer = BlobWriter::new(1, 1, 0, 0); // blob_no=1, gen=1, space=0, Plain=0
    let rec1 = BlobRecord::new_body(0, 10, b"key1".to_vec(), b"body_data_one".to_vec());
    let rec2 = BlobRecord::new_body(0, 11, b"key2".to_vec(), b"body_data_two".to_vec());

    // INV-BLOB-2: Kein Teilen: ein BlobRef <-> ein Record
    let ref1 = writer.append(&*file, &rec1, None, None, None, None, None).unwrap();
    let ref2 = writer.append(&*file, &rec2, None, None, None, None, None).unwrap();
    assert_ne!(ref1.offset, ref2.offset);

    let total_bytes = writer.finish(&*file).unwrap();
    assert!(total_bytes > 0);

    // Jetzt lesen und verifizieren
    let reader = BlobReader::open(&*file, 0).unwrap();
    assert_eq!(reader.header().blob_no, 1);
    assert_eq!(reader.header().generation, 1);

    // INV-BLOB-1: Jeder BlobRef löst auf genau einen Record gleichen Schlüssels/Stamps auf
    let body1 = reader.read(&*file, &ref1, b"key1", 10, None, None, None, None, None).unwrap();
    assert_eq!(body1, b"body_data_one");

    let body2 = reader.read(&*file, &ref2, b"key2", 11, None, None, None, None, None).unwrap();
    assert_eq!(body2, b"body_data_two");

    // Falscher Schlüssel oder falscher Stamp führt zwingend zu Fehler (INV-BLOB-1)
    let err_key = reader.read(&*file, &ref1, b"wrong_key", 10, None, None, None, None, None);
    assert!(matches!(err_key, Err(BlobReadError::Corrupt(_))));

    let err_stamp = reader.read(&*file, &ref1, b"key1", 999, None, None, None, None, None);
    assert!(matches!(err_stamp, Err(BlobReadError::Corrupt(_))));

    // Footer lesen
    let footer = reader.read_footer(&*file).unwrap();
    assert_eq!(footer.record_count, 2);
}

#[test]
fn test_blob_writer_reader_authenticated() {
    let fs = MemFs::new();
    let path = Path::new("blob_auth.ctxb");
    let file = fs.open(path, OpenMode::CreateNew(Perm::Private)).unwrap();

    let mac = Blake3Mac;
    let space_dek = [0x55u8; 32];

    let mut writer = BlobWriter::new(2, 1, 1, 1); // Authenticated=1
    let rec = BlobRecord::new_body(0, 42, b"auth_key".to_vec(), b"authenticated_payload".to_vec());

    let b_ref = writer.append(&*file, &rec, Some(&space_dek), None, None, Some(&mac), None).unwrap();
    writer.finish(&*file).unwrap();

    let reader = BlobReader::open(&*file, 1).unwrap();
    // INV-BLOB-1
    let body = reader.read(&*file, &b_ref, b"auth_key", 42, Some(&space_dek), None, None, Some(&mac), None).unwrap();
    assert_eq!(body, b"authenticated_payload");

    // Falscher Schlüssel bei Authentifizierung schlägt fehl
    let wrong_dek = [0x99u8; 32];
    let err = reader.read(&*file, &b_ref, b"auth_key", 42, Some(&wrong_dek), None, None, Some(&mac), None);
    assert!(matches!(err, Err(BlobReadError::Corrupt(_))));
}

#[test]
fn test_blob_writer_reader_encrypted() {
    let fs = MemFs::new();
    let path = Path::new("blob_enc.ctxb");
    let file = fs.open(path, OpenMode::CreateNew(Perm::Private)).unwrap();

    let cipher = ChaChaCipher;
    let mac = Blake3Mac;
    let kdf = Blake3Kdf;
    let space_dek = [0x77u8; 32];

    let mut writer = BlobWriter::new(3, 1, 1, 2); // Encrypted=2
    let rec = BlobRecord::new_body(0, 100, b"secret_key".to_vec(), b"top_secret_body".to_vec());

    let b_ref = writer.append(
        &*file,
        &rec,
        Some(&space_dek),
        None,
        Some(&cipher),
        Some(&mac),
        Some(&kdf),
    ).unwrap();
    writer.finish(&*file).unwrap();

    let reader = BlobReader::open(&*file, 2).unwrap();
    // INV-BLOB-1
    let body = reader.read(
        &*file,
        &b_ref,
        b"secret_key",
        100,
        Some(&space_dek),
        None,
        Some(&cipher),
        Some(&mac),
        Some(&kdf),
    ).unwrap();
    assert_eq!(body, b"top_secret_body");
}
