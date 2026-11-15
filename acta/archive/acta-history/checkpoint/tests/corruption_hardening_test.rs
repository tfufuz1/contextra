//! # Härtungstests: Bitflip, Truncation und Fehlertoleranz
//!
//! Prüft, dass manipulierte oder abgeschnittene Checkpoint-Dateien deterministisch
//! mit Err abgelehnt werden und niemals zu Panic oder unvollständigen Mutationen führen.

#![allow(clippy::expect_used, clippy::unwrap_used)]

use acta_checkpoint::{CheckpointManifest, CheckpointMeta, OrphanState};
use std::fs;
use std::path::PathBuf;

fn golden_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/golden")
}

#[test]
fn test_meta_bitflip_resilience() {
    let path = golden_dir().join("meta_golden.json");
    let mut bytes = fs::read(&path).expect("read meta");

    // Invert bit at index 20 (inside JSON structure)
    if let Some(b) = bytes.get_mut(20) {
        *b ^= 0xFF;
    }

    let res: Result<CheckpointMeta, _> = serde_json::from_slice(&bytes);
    assert!(
        res.is_err(),
        "Corrupted JSON with bitflip must return Err, not panic"
    );
}

#[test]
fn test_meta_truncation_resilience() {
    let path = golden_dir().join("meta_golden.json");
    let content = fs::read_to_string(&path).expect("read meta");
    let trimmed = content.trim();

    for cut in [0, 5, 20, trimmed.len() / 2, trimmed.len() - 1] {
        let truncated = &trimmed[..cut];
        let res: Result<CheckpointMeta, _> = serde_json::from_str(truncated);
        assert!(
            res.is_err(),
            "Truncated JSON at {} must return Err, not panic",
            cut
        );
    }
}

#[test]
fn test_manifest_bitflip_and_verification_failure() {
    let path = golden_dir().join("manifest_golden.json");
    let bytes = fs::read(&path).expect("read manifest");

    // Deserialization of valid manifest
    let mut manifest: CheckpointManifest =
        serde_json::from_slice(&bytes).expect("deserialize valid manifest");

    // Corrupt one component
    manifest.components.push("tampered_component".to_string());
    assert!(
        manifest.verify().is_err(),
        "Modified manifest components must fail verification"
    );
}

#[test]
fn test_orphan_state_truncation_resilience() {
    let path = golden_dir().join("orphan_state_golden.json");
    let content = fs::read_to_string(&path).expect("read orphan");
    let trimmed = content.trim();

    for cut in [10, 50, trimmed.len() - 2] {
        let truncated = &trimmed[..cut];
        let res: Result<OrphanState, _> = serde_json::from_str(truncated);
        assert!(
            res.is_err(),
            "Truncated orphan state must fail deserialization safely"
        );
    }
}
