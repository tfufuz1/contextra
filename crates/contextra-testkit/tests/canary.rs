//! Selbsttests des Canary-Scanners.
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]

use contextra_testkit::canary::*;
use std::fs;

#[test]
fn needles_are_deterministic_and_distinct() {
    let a = Canary::from_seed(1, 0x0102_0304_0506_0708);
    assert_eq!(a, Canary::from_seed(1, 0x0102_0304_0506_0708));
    assert_ne!(a.value_needle, Canary::from_seed(2, 1).value_needle);
    assert_eq!(a.value_needle.len(), VALUE_NEEDLE_LEN);
    assert_eq!(a.doc_id_needles[0], [8, 7, 6, 5, 4, 3, 2, 1]);
    assert_eq!(a.doc_id_needles[1], [1, 2, 3, 4, 5, 6, 7, 8]);
    assert_eq!(a.doc_id_needles[2], b"72623859790382856");
    assert_eq!(a.doc_id_needles[3], b"0102030405060708");
}

#[test]
fn finds_content_across_chunk_boundaries_once_and_in_names() {
    let dir = tempfile::tempdir().unwrap();
    let c = Canary::from_seed(3, 77_777_777);
    let n = c.value_needle.as_slice();
    fs::create_dir(dir.path().join("sub")).unwrap();
    let offsets = [0usize, 65_536 - 20, 65_536, 200_000 - n.len()];
    let mut data = vec![0u8; 200_000];
    for o in [offsets[0], offsets[1], offsets[3]] {
        data[o..o + n.len()].copy_from_slice(n);
    }
    fs::write(dir.path().join("sub/blob.bin"), &data).unwrap();
    let hits = scan_tree(dir.path(), &[n], ScanOpts::default()).unwrap();
    let got: Vec<u64> = hits.iter().filter_map(|h| h.offset).collect();
    assert_eq!(got, [0, 65_516, 199_952]);

    // Kurze Nadel neben langer: Treffer im Übertrag darf nicht doppelt zählen.
    let mut data = vec![0u8; 70_000];
    data[65_530..65_533].copy_from_slice(b"zzq");
    fs::write(dir.path().join("sub/blob.bin"), &data).unwrap();
    let hits = scan_tree(dir.path(), &[n, b"zzq"], ScanOpts::default()).unwrap();
    assert_eq!(hits.len(), 1);

    let named = dir
        .path()
        .join(String::from_utf8(c.key_needle.clone()).unwrap());
    fs::write(&named, b"").unwrap();
    let hits = scan_tree(dir.path(), &[&c.key_needle], ScanOpts::default()).unwrap();
    assert_eq!((hits.len(), hits[0].offset), (1, None));
    let excluded = ScanOpts {
        exclude: vec![named],
        ..Default::default()
    };
    assert!(scan_tree(dir.path(), &[&c.key_needle], excluded)
        .unwrap()
        .is_empty());
}

#[test]
fn scan_is_fail_closed() {
    let dir = tempfile::tempdir().unwrap();
    assert!(scan_tree(&dir.path().join("missing"), &[b"x"], ScanOpts::default()).is_err());
    assert!(scan_tree(dir.path(), &[], ScanOpts::default()).is_err());
    assert!(scan_tree(dir.path(), &[b""], ScanOpts::default()).is_err());
}
