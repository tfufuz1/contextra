//! # Testsuite: scrub_refusal_matrix.rs
//!
//! Zweck: Vollständige Matrix-Tests für alle `ScrubRefusal`-Varianten und Readback-Nullenverifikation.

// FILE-CONTEXT
// STAND: 2026-10-13
// ZWECK: Integrationstests fuer die Scrubbing Refusal-Matrix und Readback-Nullenverifikation.
// INVARIANTEN: Zero-Panic, pure safe Rust, fail-closed Refusal-Pfade.

use contextra_durable_fs::{
    scrub_if_safe, scrub_if_safe_with_injector, FaultInjector, ScrubOutcome, ScrubPolicy,
    ScrubRefusal,
};
use std::fs::{File, OpenOptions};
use std::io::Write;
use tempfile::tempdir;

// ANCHOR[contextra-durable-fs:TEST-MATRIX-NORMAL-READBACK]
/// Prüft das Nullen mit Readback-Verifikation für reguläre Dateien.
#[test]
fn test_scrub_matrix_normal_file_scrubbed_with_readback() -> std::io::Result<()> {
    let dir = tempdir()?;
    let path = dir.path().join("test_normal.dat");

    {
        let mut file = File::create(&path)?;
        file.write_all(b"SECRET DATA TO BE SCRUBBED 1234567890")?;
    }

    let outcome = scrub_if_safe(&path, ScrubPolicy::OverwriteZeros, false, false)?;

    if let ScrubOutcome::Scrubbed(receipt) = outcome {
        assert_eq!(receipt.bytes_scrubbed, 37);
    } else {
        assert!(false, "expected ScrubOutcome::Scrubbed, got {:?}", outcome);
    }

    assert!(!path.exists(), "File should be durably removed");
    Ok(())
}

// ANCHOR[contextra-durable-fs:TEST-MATRIX-HARDLINK-REFUSED]
/// Prüft die Verweigerung MultipleHardlinks in der Refusal-Matrix.
#[test]
fn test_scrub_matrix_hardlink_refused() -> std::io::Result<()> {
    let dir = tempdir()?;
    let path1 = dir.path().join("hardlink1.dat");
    let path2 = dir.path().join("hardlink2.dat");

    {
        let mut file = File::create(&path1)?;
        file.write_all(b"SHARED HARDLINK DATA")?;
    }

    std::fs::hard_link(&path1, &path2)?;

    let outcome = scrub_if_safe(&path1, ScrubPolicy::OverwriteZeros, false, false)?;

    assert_eq!(
        outcome,
        ScrubOutcome::Refused(ScrubRefusal::MultipleHardlinks)
    );
    assert!(path1.exists(), "File must remain intact upon refusal");
    assert!(path2.exists(), "Hardlink must remain intact upon refusal");

    Ok(())
}

// ANCHOR[contextra-durable-fs:TEST-MATRIX-READER-REFUSED]
/// Prüft die Verweigerung ReaderAlive bei aktiven Lesern.
#[test]
fn test_scrub_matrix_reader_alive_refused() -> std::io::Result<()> {
    let dir = tempdir()?;
    let path = dir.path().join("reader_alive.dat");

    {
        let mut file = File::create(&path)?;
        file.write_all(b"ACTIVE READER DATA")?;
    }

    let outcome = scrub_if_safe(&path, ScrubPolicy::OverwriteZeros, true, false)?;

    assert_eq!(outcome, ScrubOutcome::Refused(ScrubRefusal::ReaderAlive));
    assert!(path.exists(), "File must remain intact upon refusal");

    Ok(())
}

// ANCHOR[contextra-durable-fs:TEST-MATRIX-NOT-OBSOLETE-REFUSED]
/// Prüft die Verweigerung NotObsolete bei nicht freigegebenen Dateien.
#[test]
fn test_scrub_matrix_not_obsolete_refused() -> std::io::Result<()> {
    let dir = tempdir()?;
    let path = dir.path().join("not_obsolete.dat");

    {
        let mut file = File::create(&path)?;
        file.write_all(b"NOT OBSOLETE DATA")?;
    }

    let outcome = scrub_if_safe(&path, ScrubPolicy::OverwriteZeros, false, true)?;

    assert_eq!(outcome, ScrubOutcome::Refused(ScrubRefusal::NotObsolete));
    assert!(path.exists(), "File must remain intact upon refusal");

    Ok(())
}

// ANCHOR[contextra-durable-fs:TEST-MATRIX-POLICY-DISABLED]
/// Prüft RemovedUnscrubbed bei deaktivierter ScrubPolicy.
#[test]
fn test_scrub_matrix_policy_disabled_removed_unscrubbed() -> std::io::Result<()> {
    let dir = tempdir()?;
    let path = dir.path().join("disabled.dat");

    {
        let mut file = File::create(&path)?;
        file.write_all(b"DISABLED POLICY DATA")?;
    }

    let outcome = scrub_if_safe(&path, ScrubPolicy::Disabled, false, false)?;

    assert_eq!(outcome, ScrubOutcome::RemovedUnscrubbed);
    assert!(!path.exists(), "File should be removed unscrubbed");

    Ok(())
}

// ANCHOR[contextra-durable-fs:TEST-MATRIX-READBACK-FAILURE]
/// Prüft den Fehlerabbruch bei nicht übereinstimmender Readback-Nullenprüfung.
#[test]
fn test_scrub_matrix_readback_failure_returns_err() -> std::io::Result<()> {
    let dir = tempdir()?;
    let path = dir.path().join("corrupt_readback.dat");

    {
        let mut file = File::create(&path)?;
        file.write_all(b"DATA TO BE CORRUPTED DURING SCRUB")?;
    }

    // Corrupt data after overwrite or test readback check by manually writing non-zeros
    // To directly test verify_zero_readback returning Err on non-zero bytes:
    let mut options = OpenOptions::new();
    options.read(true).write(true);
    let mut file = options.open(&path)?;

    // Zero out part of file but leave non-zero at end
    file.write_all(&[0u8; 10])?;
    file.write_all(b"NONZERO")?;

    // Now test if scrub_if_safe fails when non-zero bytes persist before deletion
    // We simulate a custom injector run where we corrupt file between write and flush
    // or verify scrub_if_safe fails on non-zero readback if write_all were incomplete/corrupted.
    // Here we verify readback detection by invoking scrub_if_safe_with_injector on a file
    // that gets corrupted via a custom injector.
    struct CorruptingInjector;
    impl FaultInjector for CorruptingInjector {
        fn on_fsync(&self, path: Option<&std::path::Path>) -> std::io::Result<()> {
            if let Some(p) = path {
                let mut f = OpenOptions::new().write(true).open(p)?;
                use std::io::Seek;
                f.seek(std::io::SeekFrom::Start(0))?;
                f.write_all(b"CORRUPT_NON_ZERO")?;
            }
            Ok(())
        }
    }

    let res = scrub_if_safe_with_injector(
        &path,
        ScrubPolicy::OverwriteZeros,
        false,
        false,
        &CorruptingInjector,
    );

    assert!(
        res.is_err(),
        "Readback verification must return Err when non-zero bytes are detected"
    );
    let err = res.unwrap_err();
    assert!(
        err.to_string()
            .contains("readback zero verification failed"),
        "Error message should mention readback zero verification failure, got: {}",
        err
    );

    Ok(())
}
