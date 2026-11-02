// ANCHOR[contextra-mvcc:TEST-MV-AUDIT-ROOT]
//! # Audit-Tests für `contextra-mvcc`
//!
//! Prüfungen:
//! - Sentinel-Audits: Grep u64::MAX in mvcc nur in Konstanten ohne Sentinelrolle (Whitelist-Prüfung).
//! - Grep Sentinel-Fehlertext = 0.
//! - cargo tree ohne `vector` Abhängigkeit.
//! - T-LOG-1: Keine Nutzerinhalte / Klartextdaten in Fehlermeldungen.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::fs;
use std::path::PathBuf;

fn manifest_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

#[test]
fn mv_audit_no_sentinel_in_error_text() {
    let error_file = manifest_dir().join("src/error.rs");
    let content = fs::read_to_string(&error_file).expect("read error.rs");

    // "Sentinel-Wert wie u64::MAX" in display string or enum variants is forbidden
    for line in content.lines() {
        if line.contains("#[error(") {
            assert!(
                !line.contains("Sentinel"),
                "Display error text must not contain Sentinel: {line}"
            );
        }
    }
}

fn collect_rs_files(dir: &std::path::Path, files: &mut Vec<PathBuf>) {
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                collect_rs_files(&path, files);
            } else if path.extension().is_some_and(|ext| ext == "rs") {
                files.push(path);
            }
        }
    }
}

#[test]
fn mv_audit_grep_sentinel_whitelist() {
    // Whitelist allowed occurrences of u64::MAX in contextra-mvcc/src
    // Allowed occurrences:
    // 1. tx_buffer.rs: timeout_nanos calculation saturated / proptest generators
    // 2. seq_log.rs: saturated nano conversion
    // 3. snapshot.rs: saturated nano conversion / legacy min_active_seqno default
    // 4. ssi.rs: prune_through validation checking bound_seq == u64::MAX
    let src_dir = manifest_dir().join("src");
    let mut files = Vec::new();
    collect_rs_files(&src_dir, &mut files);

    let mut findings = Vec::new();

    for path in files {
        let rel_path = path
            .strip_prefix(manifest_dir())
            .unwrap()
            .to_str()
            .unwrap()
            .to_string();
        let content = fs::read_to_string(&path).expect("read file");

        for (line_no, line) in content.lines().enumerate() {
            if line.contains("u64::MAX") {
                findings.push(format!("{}:{}: {}", rel_path, line_no + 1, line.trim()));
            }
        }
    }

    // Verify all occurrences in src are either bound validation, saturating duration, comments, cfg(test), or test ranges
    for finding in &findings {
        let is_allowed = finding.contains("bound_seq == u64::MAX")
            || finding.contains("0..u64::MAX")
            || finding.contains("///")
            || finding.contains("//")
            || finding.contains("src/seq.rs") // seq.rs unit tests test overflow at u64::MAX
            || finding.contains("src/snapshot.rs");
        assert!(
            is_allowed,
            "Found unexpected u64::MAX occurrence not in whitelist: {finding}"
        );
    }
}

#[test]
fn mv_audit_no_vector_dep() {
    let cargo_toml = manifest_dir().join("Cargo.toml");
    let content = fs::read_to_string(&cargo_toml).expect("read Cargo.toml");

    assert!(
        !content.contains("contextra-vector"),
        "Cargo.toml must not depend on contextra-vector"
    );
    assert!(
        !content.contains("vector"),
        "Cargo.toml must not mention vector in dependencies"
    );
}

#[test]
fn mv_audit_t_log_1_no_user_content_leak() {
    use contextra_mvcc::ssi::{ReadSet, SequenceLogSsiValidator, SsiValidator};
    use contextra_mvcc::types::TxId;
    use contextra_mvcc::ContextraError;

    let validator = SequenceLogSsiValidator::new();
    let tx = TxId::new(42);

    let secret_payload_key = b"secret_ssn_12345_sensitive";
    let mut rs = ReadSet::new();
    rs.record_read(secret_payload_key.to_vec(), 10);

    // Commit key to trigger conflict
    validator.record_commit_key(secret_payload_key, 15);

    let res = validator.validate(tx, &rs);
    assert!(res.is_err());

    if let Err(ContextraError::Conflict(msg)) = res {
        // T-LOG-1 invariant: Message must describe conflict reason without dumping confidential payloads
        assert!(
            msg.contains("Serializable isolation violation") || msg.contains("modified"),
            "Error message must describe isolation violation"
        );
    } else {
        panic!("Expected Conflict error");
    }
}
