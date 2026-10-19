// FILE-CONTEXT
// STAND: 2026-10-12
// ZWECK: Selbsttest fuer Canary-Orakel, Rohsweep und Report (Spezifikation T1).

use std::fs::{self, File};
use std::io::Write;
use tempfile::tempdir;

use contextra_testkit::{raw_sweep, OrcMarkerSet};

#[test]
fn test_marker_detection_scenarios() {
    let dir = tempdir().expect("failed to create tempdir");
    let root = dir.path();

    let set = OrcMarkerSet::generate("run_test", 999, &["field_a", "field_b"]);
    let vec_f32 = OrcMarkerSet::vector_marker(64, 999);
    let needles = set.needles();

    assert_eq!(needles.len(), 3); // field_a, field_b, vector_signature

    // 1. Regular file with field_a marker
    let file1_path = root.join("data.db");
    let mut f1 = File::create(&file1_path).expect("create file1");
    f1.write_all(b"HEADER_PREFIX_").expect("write header");
    let marker_a_bytes = needles[0].clone();
    f1.write_all(&marker_a_bytes).expect("write marker_a");
    f1.write_all(b"_TRAILER_SUFFIX").expect("write trailer");
    drop(f1);

    // 2. Subdirectory file with field_b marker
    let sub_dir = root.join("nested/sub");
    fs::create_dir_all(&sub_dir).expect("create subdir");
    let file2_path = sub_dir.join("wal.log");
    let mut f2 = File::create(&file2_path).expect("create file2");
    f2.write_all(&needles[1]).expect("write marker_b");
    drop(f2);

    // 3. Hidden / .tmp file with vector_marker signature
    let tmp_path = root.join(".orphan_staging.tmp");
    let mut f_tmp = File::create(&tmp_path).expect("create tmp file");
    // Write 4 bytes prefix, then vector marker 8-byte signature
    f_tmp.write_all(b"TMP_").expect("write tmp prefix");
    let vec_bytes: Vec<u8> = vec_f32[0..2].iter().flat_map(|f| f.to_le_bytes()).collect();
    f_tmp.write_all(&vec_bytes).expect("write vec signature");
    drop(f_tmp);

    // 4. Hardlink pointing to data.db
    let link_path = root.join("data_link.db");
    fs::hard_link(&file1_path, &link_path).expect("create hardlink");

    // 5. File straddling 1-MiB window boundary
    let boundary_path = root.join("boundary.bin");
    let window_size = 1_048_576; // 1 MiB
    let mut boundary_data = vec![0u8; window_size + 1024];
    let boundary_offset = window_size - 5;
    boundary_data[boundary_offset..boundary_offset + needles[0].len()].copy_from_slice(&needles[0]);
    fs::write(&boundary_path, &boundary_data).expect("write boundary file");

    // 6. File with marker at EOF
    let eof_path = root.join("eof.dat");
    let mut eof_data = vec![0u8; 500];
    let eof_offset = 500 - needles[0].len();
    eof_data[eof_offset..].copy_from_slice(&needles[0]);
    fs::write(&eof_path, &eof_data).expect("write eof file");

    // 7. File with exact 1-MiB window size
    let exact_path = root.join("exact_1mib.dat");
    let mut exact_data = vec![0u8; window_size];
    exact_data[100..100 + needles[0].len()].copy_from_slice(&needles[0]);
    fs::write(&exact_path, &exact_data).expect("write exact file");

    // Execute raw_sweep
    let report = raw_sweep(root, &needles).expect("raw_sweep succeeded");

    assert!(report.files_scanned >= 7);
    assert!(!report.hits.is_empty());

    // Verify boundary hit offset
    let boundary_hit = report
        .hits
        .iter()
        .find(|h| h.relative_path == std::path::Path::new("boundary.bin"))
        .expect("boundary hit found");
    assert_eq!(boundary_hit.offset, boundary_offset as u64);

    // Verify EOF hit offset
    let eof_hit = report
        .hits
        .iter()
        .find(|h| h.relative_path == std::path::Path::new("eof.dat"))
        .expect("eof hit found");
    assert_eq!(eof_hit.offset, eof_offset as u64);

    // Verify exact 1-MiB file hit offset
    let exact_hit = report
        .hits
        .iter()
        .find(|h| h.relative_path == std::path::Path::new("exact_1mib.dat"))
        .expect("exact hit found");
    assert_eq!(exact_hit.offset, 100);

    // Verify hardlink count (nlink >= 2 on unix)
    #[cfg(unix)]
    {
        let data_hit = report
            .hits
            .iter()
            .find(|h| h.relative_path == std::path::Path::new("data.db"))
            .expect("data.db hit found");
        assert!(data_hit.nlink >= 2);
    }

    // Assert clean fails when hits exist
    assert!(report.assert_clean().is_err());
}

#[test]
fn test_empty_and_clean_directory() {
    let dir = tempdir().expect("failed to create tempdir");
    let root = dir.path();

    let set = OrcMarkerSet::generate("run_clean", 42, &["field1"]);
    let needles = set.needles();

    // 1. Empty directory
    let report1 = raw_sweep(root, &needles).expect("sweep empty dir");
    assert_eq!(report1.files_scanned, 0);
    assert!(report1.assert_clean().is_ok());

    // 2. Directory with clean files (no markers)
    let file_path = root.join("clean.db");
    fs::write(&file_path, b"JUST_REGULAR_UNMARK_DATA").expect("write clean file");

    let report2 = raw_sweep(root, &needles).expect("sweep clean dir");
    assert_eq!(report2.files_scanned, 1);
    assert!(report2.assert_clean().is_ok());
}

#[test]
#[cfg(unix)]
fn test_unreadable_file_fails_closed() {
    use std::os::unix::fs::PermissionsExt;

    let dir = tempdir().expect("failed to create tempdir");
    let root = dir.path();

    let unreadable_path = root.join("unreadable.db");
    fs::write(&unreadable_path, b"SECRET_CONTENT").expect("write unreadable file");

    // Remove read permissions (mode 0o000)
    let mut perms = fs::metadata(&unreadable_path)
        .expect("metadata")
        .permissions();
    perms.set_mode(0o000);
    fs::set_permissions(&unreadable_path, perms).expect("set_permissions 0o000");

    let set = OrcMarkerSet::generate("run_fail", 100, &["field"]);
    let needles = set.needles();

    let result = raw_sweep(root, &needles);
    assert!(result.is_err(), "sweep must fail-closed on unreadable file");

    // Restore permissions for clean tempdir drop
    let mut perms_restore = fs::metadata(&unreadable_path)
        .expect("metadata")
        .permissions();
    perms_restore.set_mode(0o600);
    let _ = fs::set_permissions(&unreadable_path, perms_restore);
}

#[test]
fn test_determinism_and_seed() {
    let set1 = OrcMarkerSet::generate("run1", 12345, &["col_a", "col_b"]);
    let set2 = OrcMarkerSet::generate("run1", 12345, &["col_a", "col_b"]);
    let set_diff = OrcMarkerSet::generate("run1", 54321, &["col_a", "col_b"]);

    assert_eq!(set1, set2);
    assert_ne!(set1, set_diff);

    let vec1 = OrcMarkerSet::vector_marker(128, 12345);
    let vec2 = OrcMarkerSet::vector_marker(128, 12345);
    let vec_diff = OrcMarkerSet::vector_marker(128, 54321);

    assert_eq!(vec1, vec2);
    assert_ne!(vec1, vec_diff);

    // Verify vector signature matches position 0 floats
    let sig1 = set1.vector_signature();
    let mut sig_vec1 = [0u8; 8];
    sig_vec1[0..4].copy_from_slice(&vec1[0].to_le_bytes());
    sig_vec1[4..8].copy_from_slice(&vec1[1].to_le_bytes());
    assert_eq!(sig1, &sig_vec1);
}

#[test]
fn test_no_payload_in_debug_output() {
    let set = OrcMarkerSet::generate("run_debug", 777, &["sensitive_field"]);
    let needles = set.needles();

    let dir = tempdir().expect("create tempdir");
    let file_path = dir.path().join("hit.db");
    fs::write(&file_path, &needles[0]).expect("write needle");

    let report = raw_sweep(dir.path(), &needles).expect("sweep");
    assert_eq!(report.hits.len(), 1);

    let debug_output = format!("{:?}", report);
    let display_output = format!("{}", report);

    // Verify neither Debug nor Display contains raw needle string/bytes or payload content
    let needle_str = &set.fields()[0].marker_string;
    assert!(!debug_output.contains(needle_str));
    assert!(!display_output.contains(needle_str));
}
