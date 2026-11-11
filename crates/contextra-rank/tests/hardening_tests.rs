//! Härtungstests für contextra-rank (RK3.5).
//!
//! Prüft Fehlerabbildung, Bitflip/Truncation-Sicherheit auf Golden-Dateien und Zero-Panic-Invariante.

#![allow(clippy::indexing_slicing, clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use contextra_rank::RankError;
use std::fs;
use std::path::Path;

#[test]
fn test_table_driven_error_conversion() {
    let cases = vec![
        (
            RankError::InvalidInput("test input".to_string()),
            "test input",
        ),
        (
            RankError::Calibration("cal error".to_string()),
            "cal error",
        ),
        (
            RankError::FusionFailed("fusion error".to_string()),
            "fusion error",
        ),
    ];

    #[cfg(feature = "workspace")]
    for (err, msg) in cases {
        let converted: contextra_types::ContextraError = err.into();
        match converted {
            contextra_types::ContextraError::InvalidInput(m) => {
                assert_eq!(m, msg);
            }
            other => panic!("Unexpected conversion result: {other:?}"),
        }
    }

    #[cfg(not(feature = "workspace"))]
    for (err, msg) in cases {
        assert!(err.to_string().contains(msg));
    }
}

#[test]
fn test_golden_bitflip_and_truncation_robustness() {
    let golden_dir = Path::new("tests/golden");
    if !golden_dir.exists() {
        return;
    }

    for entry in fs::read_dir(golden_dir).expect("read golden dir") {
        let entry = entry.expect("entry");
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("json") {
            continue;
        }

        let content = fs::read(&path).expect("read raw bytes");
        if content.is_empty() {
            continue;
        }

        for cut in [1, content.len() / 4, content.len() / 2, content.len() - 1] {
            let truncated = &content[..cut];
            let res: Result<serde_json::Value, _> = serde_json::from_slice(truncated);
            assert!(res.is_err(), "Truncated JSON at {} must fail", cut);
        }

        for &pos in &[0, content.len() / 4, content.len() / 2, content.len() - 1] {
            let mut corrupted = content.clone();
            corrupted[pos] ^= 0xFF;
            let _ = serde_json::from_slice::<serde_json::Value>(&corrupted);
        }
    }
}
