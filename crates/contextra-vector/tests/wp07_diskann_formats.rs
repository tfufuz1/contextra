// FILE-CONTEXT
// ZWECK: Bit-Flip- und Trunkierungsprüfungen für DANN/PWAL/TWAL-Formate gegen Golden-Dateien (WP-VEC-07).
// INVARIANTEN: Zero-Panic bei korrupten oder verkürzten Headern; sichere Fehlerbehandlung oder Fallback.
// STAND: 2026-11-11

#![cfg(feature = "experimental-diskann")]

use contextra_vector::diskann::{DiskAnnConfig, DiskAnnFallbackPolicy, DiskAnnIndex};
use std::path::PathBuf;
use tempfile::tempdir;

const GOLDEN_DANN: &str = "tests/golden/diskann.dann";
const GOLDEN_PWAL: &str = "tests/golden/diskann.pending.wal";
const GOLDEN_TWAL: &str = "tests/golden/diskann.tombstone.wal";

fn golden_path(rel: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(rel)
}

#[tokio::test]
async fn test_golden_dann_load_succeeds_without_panic() {
    let src = golden_path(GOLDEN_DANN);
    assert!(src.exists(), "Golden DANN must exist: {}", src.display());

    let dir = tempdir().expect("tempdir");
    let test_file = dir.path().join("index.dann");
    std::fs::copy(&src, &test_file).expect("copy golden dann");

    let config = DiskAnnConfig {
        index_path: test_file,
        dimension: 4,
        max_degree: 8,
        sector_size: 512,
        fallback_policy: DiskAnnFallbackPolicy::StrictError,
        ..Default::default()
    };

    let idx = DiskAnnIndex::try_new(config).expect("try_new");
    let load_res = idx.load().await;
    assert!(load_res.is_ok(), "Golden DANN must load cleanly");
}

#[tokio::test]
async fn test_golden_dann_truncations_never_panic() {
    let src = golden_path(GOLDEN_DANN);
    let orig_bytes = std::fs::read(&src).expect("read golden dann");

    let dir = tempdir().expect("tempdir");

    // Test truncations at various critical boundaries: 0, 1, 4, 10, 39, 40, 100, half, len-1
    let cutoffs = [
        0,
        1,
        4,
        10,
        39,
        40,
        100,
        orig_bytes.len() / 2,
        orig_bytes.len().saturating_sub(1),
    ];

    for cutoff in cutoffs {
        let truncated = &orig_bytes[..cutoff];
        let test_file = dir.path().join(format!("trunc_{cutoff}.dann"));
        std::fs::write(&test_file, truncated).expect("write truncated");

        let config = DiskAnnConfig {
            index_path: test_file,
            dimension: 4,
            max_degree: 8,
            sector_size: 512,
            fallback_policy: DiskAnnFallbackPolicy::StrictError,
            ..Default::default()
        };

        match DiskAnnIndex::try_new(config) {
            Ok(idx) => {
                // If try_new succeeded, load must return Err, never panic
                let res = idx.load().await;
                assert!(res.is_err(), "Truncated DANN at {} must fail load", cutoff);
            }
            Err(_) => {
                // Expected immediate rejection at try_new for < 40 bytes
            }
        }
    }
}

#[tokio::test]
async fn test_golden_dann_bitflips_never_panic() {
    let src = golden_path(GOLDEN_DANN);
    let orig_bytes = std::fs::read(&src).expect("read golden dann");

    let dir = tempdir().expect("tempdir");

    // Flip bits across header (bytes 0..40) and footer (last 36 bytes)
    let header_indices: Vec<usize> = (0..40.min(orig_bytes.len())).collect();
    let footer_start = orig_bytes.len().saturating_sub(36);
    let footer_indices: Vec<usize> = (footer_start..orig_bytes.len()).collect();

    for &idx in header_indices.iter().chain(footer_indices.iter()) {
        let mut corrupted = orig_bytes.clone();
        corrupted[idx] ^= 0xFF; // Flip all 8 bits

        let test_file = dir.path().join(format!("flip_{idx}.dann"));
        std::fs::write(&test_file, &corrupted).expect("write bitflipped");

        let config = DiskAnnConfig {
            index_path: test_file,
            dimension: 4,
            max_degree: 8,
            sector_size: 512,
            fallback_policy: DiskAnnFallbackPolicy::StrictError,
            ..Default::default()
        };

        match DiskAnnIndex::try_new(config) {
            Ok(index) => {
                let load_res = index.load().await;
                // It must fail or recover via fallback, never panic
                assert!(
                    load_res.is_err(),
                    "Bit-flipped index at byte {} must reject loading",
                    idx
                );
            }
            Err(_) => {
                // Corrupt magic or size rejected in try_new
            }
        }
    }
}

#[tokio::test]
async fn test_golden_pwal_truncations_and_bitflips_never_panic() {
    let src = golden_path(GOLDEN_PWAL);
    let orig_bytes = std::fs::read(&src).expect("read golden pwal");

    let dir = tempdir().expect("tempdir");

    // Truncations
    for cutoff in [
        0,
        1,
        4,
        5,
        orig_bytes.len() / 2,
        orig_bytes.len().saturating_sub(1),
    ] {
        let truncated = &orig_bytes[..cutoff];
        let test_file = dir.path().join(format!("trunc_{cutoff}.pending.wal"));
        std::fs::write(&test_file, truncated).expect("write truncated pwal");

        let res = DiskAnnIndex::read_pending_wal(&test_file);
        // Either empty, Err, or partial without panic
        let _ = res;
    }

    // Bit flips in magic, version, payload, and hmac
    for idx in 0..orig_bytes.len() {
        let mut corrupted = orig_bytes.clone();
        corrupted[idx] ^= 0xAA;

        let test_file = dir.path().join(format!("flip_{idx}.pending.wal"));
        std::fs::write(&test_file, &corrupted).expect("write flipped pwal");

        let res = DiskAnnIndex::read_pending_wal(&test_file);
        // Never panic: returns Err or skips invalid HMAC record
        let _ = res;
    }
}

#[tokio::test]
async fn test_golden_twal_truncations_and_bitflips_never_panic() {
    let src = golden_path(GOLDEN_TWAL);
    let orig_bytes = std::fs::read(&src).expect("read golden twal");

    let dir = tempdir().expect("tempdir");

    // Truncations
    for cutoff in [
        0,
        1,
        4,
        5,
        orig_bytes.len() / 2,
        orig_bytes.len().saturating_sub(1),
    ] {
        let truncated = &orig_bytes[..cutoff];
        let test_file = dir.path().join(format!("trunc_{cutoff}.tombstone.wal"));
        std::fs::write(&test_file, truncated).expect("write truncated twal");

        let res = DiskAnnIndex::read_tombstone_wal(&test_file);
        let _ = res;
    }

    // Bit flips
    for idx in 0..orig_bytes.len() {
        let mut corrupted = orig_bytes.clone();
        corrupted[idx] ^= 0x55;

        let test_file = dir.path().join(format!("flip_{idx}.tombstone.wal"));
        std::fs::write(&test_file, &corrupted).expect("write flipped twal");

        let res = DiskAnnIndex::read_tombstone_wal(&test_file);
        let _ = res;
    }
}
