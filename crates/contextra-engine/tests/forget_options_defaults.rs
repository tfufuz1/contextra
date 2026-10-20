#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::path::PathBuf;
use std::time::Duration;

use contextra_engine::deletion::forget_options::{
    Assurance, FonLogicalHandle, ForgetOptions, ForgetOutcome, PurgeHandle, PurgeMode, VerifyDepth,
    Wait, WitAssurance,
};

#[test]
fn test_forget_options_default_values_and_validation() {
    let opts = ForgetOptions::default();

    assert_eq!(opts.assurance, WitAssurance::S1);
    assert_eq!(opts.purge, PurgeMode::None);
    assert_eq!(opts.purge_path, None);
    assert_eq!(opts.wait, Wait::No);
    assert_eq!(opts.verify_depth, VerifyDepth::Footprint);

    // Default configuration must validate Ok
    assert!(
        opts.validate().is_ok(),
        "Default ForgetOptions must validate Ok"
    );
}

#[test]
fn test_forget_options_builder_chaining() {
    let opts = ForgetOptions::new()
        .with_assurance(WitAssurance::S2)
        .with_purge(PurgeMode::Purge)
        .with_purge_path(PathBuf::from("/tmp/test_purge"))
        .with_wait(Wait::Forever)
        .with_verify_depth(VerifyDepth::FullSweep);

    assert_eq!(opts.assurance, Assurance::S2);
    assert_eq!(opts.purge, PurgeMode::Purge);
    assert_eq!(opts.purge_path, Some(PathBuf::from("/tmp/test_purge")));
    assert_eq!(opts.wait, Wait::Forever);
    assert_eq!(opts.verify_depth, VerifyDepth::FullSweep);

    assert!(
        opts.validate().is_ok(),
        "Configured options must validate Ok"
    );
}

#[test]
fn test_forget_options_validation_s2_requires_purge() {
    let opts = ForgetOptions::default().with_assurance(WitAssurance::S2);
    let err = opts.validate().unwrap_err();
    assert!(
        err.to_string()
            .contains("Assurance >= S2 requires a purge mode"),
        "Got: {err}"
    );

    let opts3 = ForgetOptions::default().with_assurance(WitAssurance::S3);
    let err3 = opts3.validate().unwrap_err();
    assert!(
        err3.to_string()
            .contains("Assurance >= S2 requires a purge mode"),
        "Got: {err3}"
    );
}

#[test]
fn test_forget_options_validation_purge_with_scrub_requires_path() {
    let opts = ForgetOptions::default().with_purge(PurgeMode::PurgeWithScrub);
    let err = opts.validate().unwrap_err();
    assert!(
        err.to_string()
            .contains("PurgeWithScrub requires a purge path"),
        "Got: {err}"
    );

    let opts_valid = ForgetOptions::default()
        .with_purge(PurgeMode::PurgeWithScrub)
        .with_purge_path("/var/data/scrub");
    assert!(opts_valid.validate().is_ok());
}

#[test]
fn test_forget_options_validation_s4_unavailable() {
    let opts = ForgetOptions::default().with_assurance(WitAssurance::S4);
    let err = opts.validate().unwrap_err();
    assert!(
        err.to_string()
            .contains("S4 requires keyring, not available"),
        "Got: {err}"
    );
}

#[test]
fn test_forget_options_validation_until_zero_invalid() {
    let opts = ForgetOptions::default().with_wait(Wait::Until(Duration::ZERO));
    let err = opts.validate().unwrap_err();
    assert!(
        err.to_string()
            .contains("Wait::Until(0) duration is invalid"),
        "Got: {err}"
    );
}

#[test]
fn test_logical_and_purge_handle_debug_redaction() {
    let logical = FonLogicalHandle {
        proof_id: [0x42; 16],
        proof_path: PathBuf::from("/secret/vault/keys/proof_42.bin"),
        full_proof_id: [0x42; 32],
    };

    let dbg_str = format!("{logical:?}");
    assert!(
        dbg_str.contains("proof_id"),
        "Debug output must contain proof_id: {dbg_str}"
    );
    assert!(
        dbg_str.contains("[REDACTED_PATH]"),
        "Debug output must redact secret path: {dbg_str}"
    );
    assert!(
        !dbg_str.contains("/secret/vault/keys"),
        "Debug output MUST NOT leak secret path: {dbg_str}"
    );

    let purge = PurgeHandle::Final(logical.clone());
    let purge_dbg = format!("{purge:?}");
    assert!(
        purge_dbg.contains("[REDACTED_PATH]"),
        "PurgeHandle debug must redact path: {purge_dbg}"
    );
    assert!(
        !purge_dbg.contains("/secret/vault/keys"),
        "PurgeHandle debug MUST NOT leak secret path: {purge_dbg}"
    );

    let outcome = ForgetOutcome {
        logical,
        physical: Some(purge),
        purge_report: None,
    };
    assert_eq!(outcome.logical.proof_id, [0x42; 16]);
}
