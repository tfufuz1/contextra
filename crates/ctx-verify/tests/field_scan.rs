//! Field scanner tests for ctx-verify scan command.
//!
//! SPEC: T-RAW-1, D-72, MK1, MK2, MK3, HR-4

use ctx_verify::codes::InvalidCode;
use ctx_verify::scan::execute_field_scan;
use std::fs::{self, File};
use std::io::Write;
use std::os::fd::AsRawFd;
use tempfile::tempdir;

/// SPEC: T-RAW-1
#[test]
fn t_raw_1_field_scan_clean_directory_passes() {
    let dir = tempdir().unwrap();
    let data_dir = dir.path().join("data");
    fs::create_dir_all(&data_dir).unwrap();

    // Create some unrelated data files
    fs::write(data_dir.join("segment_001.dat"), b"clean unrelated data").unwrap();

    // Create a temporary file for passphrase
    let mut pass_file = tempfile::NamedTempFile::new().unwrap();
    pass_file.write_all(b"super_secret_passphrase").unwrap();
    pass_file.flush().unwrap();

    let file_handle = File::open(pass_file.path()).unwrap();
    let fd = file_handle.as_raw_fd();

    let target_hex = "0102030405060708090a0b0c0d0e0f10";
    let res = execute_field_scan(&data_dir, target_hex, fd);
    assert_eq!(res, Ok(()));
}

/// SPEC: T-RAW-1
#[test]
fn t_raw_1_field_scan_mk2_raw_residue_detected() {
    let dir = tempdir().unwrap();
    let data_dir = dir.path().join("data");
    fs::create_dir_all(&data_dir).unwrap();

    let target_bytes = [
        0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0a, 0x0b, 0x0c, 0x0d, 0x0e, 0x0f,
        0x10,
    ];
    let target_hex = hex::encode(target_bytes);

    // Create file containing raw target needle
    let mut dirty_data = b"prefix_bytes_".to_vec();
    dirty_data.extend_from_slice(&target_bytes);
    dirty_data.extend_from_slice(b"_suffix_bytes");
    fs::write(data_dir.join("segment_dirty.dat"), dirty_data).unwrap();

    let mut pass_file = tempfile::NamedTempFile::new().unwrap();
    pass_file.write_all(b"super_secret_passphrase").unwrap();
    pass_file.flush().unwrap();

    let file_handle = File::open(pass_file.path()).unwrap();
    let fd = file_handle.as_raw_fd();

    let res = execute_field_scan(&data_dir, &target_hex, fd);
    assert_eq!(res, Err(InvalidCode::Mk2RawResidue));
}

/// SPEC: T-RAW-1
#[test]
fn t_raw_1_field_scan_mk3_key_present_in_vault_detected() {
    let dir = tempdir().unwrap();
    let data_dir = dir.path().join("data");
    let vault_dir = data_dir.join("vault");
    fs::create_dir_all(&vault_dir).unwrap();

    let target_bytes = [
        0xaa, 0xbb, 0xcc, 0xdd, 0xee, 0xff, 0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88,
        0x99,
    ];
    let target_hex = hex::encode(target_bytes);

    // Create mock shard file (00.vlt) per SPEC F.3 / D-111:
    // header (64) + metablock (64) = 128 bytes base
    let mut vault_data = vec![0u8; 128];
    // Add 64-byte slot:
    // 0: slot_type (0x04 D), 1: status (0x01 active), 2..8: gen, 8..24: tag, 24..56: payload, 56..64: mac
    let mut slot = vec![0u8; 64];
    slot[0] = 0x04; // slot_type D
    slot[1] = 0x01; // status aktiv! (not tombstoned 0x02)
    slot[8..24].copy_from_slice(&target_bytes); // tag
    vault_data.extend_from_slice(&slot);

    fs::write(vault_dir.join("00.vlt"), vault_data).unwrap();

    let mut pass_file = tempfile::NamedTempFile::new().unwrap();
    pass_file.write_all(b"super_secret_passphrase").unwrap();
    pass_file.flush().unwrap();

    let file_handle = File::open(pass_file.path()).unwrap();
    let fd = file_handle.as_raw_fd();

    let res = execute_field_scan(&data_dir, &target_hex, fd);
    assert_eq!(res, Err(InvalidCode::Mk3KeyPresent));
}

/// SPEC: T-RAW-1
#[test]
fn t_raw_1_field_scan_invalid_fd_rejected() {
    let dir = tempdir().unwrap();
    let data_dir = dir.path().join("data");
    fs::create_dir_all(&data_dir).unwrap();

    let target_hex = "0102030405060708090a0b0c0d0e0f10";
    let res = execute_field_scan(&data_dir, target_hex, -1);
    assert_eq!(res, Err(InvalidCode::PassphraseFdMissing));
}

/// SPEC: T-RAW-1, ZERO-TRUST positive control check
#[test]
fn t_raw_1_field_scan_positive_control_enforced() {
    let dir = tempdir().unwrap();
    let data_dir = dir.path().join("data");
    fs::create_dir_all(&data_dir).unwrap();

    // Data directory with unrelated data and positive control canary
    fs::write(data_dir.join("segment_001.dat"), b"clean unrelated data").unwrap();
    fs::write(
        data_dir.join("control.dat"),
        b"canary_positive_control_pattern_ok",
    )
    .unwrap();

    let mut pass_file = tempfile::NamedTempFile::new().unwrap();
    pass_file.write_all(b"super_secret_passphrase").unwrap();
    pass_file.flush().unwrap();
    let file_handle = File::open(pass_file.path()).unwrap();
    let fd = file_handle.as_raw_fd();

    let target_hex = "0102030405060708090a0b0c0d0e0f10";
    let control_pattern = b"canary_positive_control_pattern_ok";

    // 1. Positive control present -> scan succeeds
    let res_ok = ctx_verify::scan::execute_field_scan_with_control(
        &data_dir,
        target_hex,
        fd,
        Some(control_pattern),
    );
    assert_eq!(res_ok, Ok(()));

    // 2. Positive control missing (cannot be proven) -> scan fails closed with MissingPositiveControl
    let missing_pattern = b"non_existent_control_canary_pattern";
    let res_fail = ctx_verify::scan::execute_field_scan_with_control(
        &data_dir,
        target_hex,
        fd,
        Some(missing_pattern),
    );
    assert_eq!(res_fail, Err(InvalidCode::MissingPositiveControl));
}

/// SPEC: T-MUT-1, T-RAW-2
/// Test planted residues across MK1, MK2, and MK3, ensuring each triggers the appropriate error code.
#[test]
fn t_mut_1_planted_residues_trigger_findings() {
    let target_bytes = [
        0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88, 0x99, 0xaa, 0xbb, 0xcc, 0xdd, 0xee, 0xff,
        0x00,
    ];
    let target_hex = hex::encode(target_bytes);

    // 1. Mutant MK1 / MK2: Planted raw needle inside a file
    {
        let dir = tempdir().unwrap();
        let data_dir = dir.path().join("data");
        fs::create_dir_all(&data_dir).unwrap();

        fs::write(data_dir.join("clean.dat"), b"clean_data").unwrap();
        let mut dirty = b"header_data_".to_vec();
        dirty.extend_from_slice(&target_bytes);
        dirty.extend_from_slice(b"_trailer");
        fs::write(data_dir.join("polluted.dat"), dirty).unwrap();

        let mut pass_file = tempfile::NamedTempFile::new().unwrap();
        pass_file.write_all(b"secret").unwrap();
        let handle = File::open(pass_file.path()).unwrap();
        let fd = handle.as_raw_fd();

        let res = execute_field_scan(&data_dir, &target_hex, fd);
        assert_eq!(
            res,
            Err(InvalidCode::Mk2RawResidue),
            "Planted raw needle mutant must be detected by MK2"
        );
    }

    // 2. Mutant MK3: Planted active key in vault shard 00.vlt
    {
        let dir = tempdir().unwrap();
        let data_dir = dir.path().join("data");
        fs::create_dir_all(&data_dir).unwrap();

        let mut vault_data = vec![0u8; 128];
        let mut slot = vec![0u8; 64];
        slot[0] = 0x01; // slot type T
        slot[1] = 0x01; // active slot status
        slot[8..24].copy_from_slice(&target_bytes);
        vault_data.extend_from_slice(&slot);
        fs::write(data_dir.join("00.vlt"), vault_data).unwrap();

        let mut pass_file = tempfile::NamedTempFile::new().unwrap();
        pass_file.write_all(b"secret").unwrap();
        let handle = File::open(pass_file.path()).unwrap();
        let fd = handle.as_raw_fd();

        let res = execute_field_scan(&data_dir, &target_hex, fd);
        assert_eq!(
            res,
            Err(InvalidCode::Mk3KeyPresent),
            "Planted active vault slot mutant must be detected by MK3"
        );
    }
}

/// SPEC: T-MUT-1, T-RAW-2
/// Ensure a negative finding without positive control is invalid.
#[test]
fn t_mut_1_negative_finding_without_passed_positive_control_is_invalid() {
    let dir = tempdir().unwrap();
    let empty_data_dir = dir.path().join("data_empty");
    fs::create_dir_all(&empty_data_dir).unwrap();

    let mut pass_file = tempfile::NamedTempFile::new().unwrap();
    pass_file.write_all(b"secret").unwrap();
    let handle = File::open(pass_file.path()).unwrap();
    let fd = handle.as_raw_fd();

    let target_hex = "112233445566778899aabbccddeeff00";

    // In an empty directory, even if no target needle is found, there is NO positive control passed!
    // Therefore, scan MUST fail closed with MissingPositiveControl, rather than returning Ok(()).
    let res = execute_field_scan(&empty_data_dir, target_hex, fd);
    assert_eq!(
        res,
        Err(InvalidCode::MissingPositiveControl),
        "A negative finding on empty data without positive control must be invalid"
    );
}
