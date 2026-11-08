//! # Testsuite: vfs_conformance.rs
//!
//! Verifiziert die Einhaltung des `Vfs`-Vertrags durch `RealFs` mittels
//! der generischen Konformitätssuite aus `contextra_ports::contract_tests::verify_vfs_contract`.

use contextra_durable_fs::RealFs;
use contextra_ports::contract_tests::verify_vfs_contract;
use tempfile::tempdir;

#[test]
fn test_real_fs_vfs_contract_conformance() -> std::io::Result<()> {
    let dir = tempdir()?;
    let real_fs = RealFs;
    verify_vfs_contract(&real_fs, dir.path());
    Ok(())
}
