//! # Integrationstests für bedingtes Scrubbing (DF-07, T-CFG-3)
//!
//! Prüft die D-83 Wahrheitstabelle (`rotational == true && !is_cow`) und Sicherheitsverweigerungen.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::disallowed_methods
)]

use contextra_durable_fs::scrub::{scrub_before_unlink, ScrubOutcome, ScrubReceipt, ScrubRefusal};
use std::fs;
use tempfile::TempDir;

#[test]
fn test_df_07_scrub_truth_table() {
    let cases = [
        // (rotational, is_cow, expected_scrubbed)
        (true, false, true), // Fall 1: HDD / Rotational ohne CoW -> Scrubbing aktiv (D-83)
        (true, true, false), // Fall 2: CoW-Dateisystem -> No-Op (kein Überschreiben)
        (false, false, false), // Fall 3: Flash/SSD -> No-Op (Wear-Leveling macht Nullen wirkungslos)
        (false, true, false),  // Fall 4: Flash/SSD mit CoW -> No-Op
    ];

    for (rotational, is_cow, should_scrub) in cases {
        let temp = TempDir::new().expect("TempDir failed");
        let file_path = temp.path().join("sensitive.bin");
        let initial_data = b"TOP_SECRET_RECORDS_ABCDEF_0123456789";
        fs::write(&file_path, initial_data).expect("Initial write failed");

        let outcome = scrub_before_unlink(&file_path, rotational, is_cow)
            .expect("scrub_before_unlink failed");

        if should_scrub {
            assert_eq!(
                outcome,
                ScrubOutcome::Scrubbed(ScrubReceipt {
                    bytes_scrubbed: initial_data.len() as u64
                }),
                "rotational=true && !is_cow must result in Scrubbed"
            );

            // Dateiinhalt muss vollständig genullt sein
            let content = fs::read(&file_path).expect("Read scrubbed file failed");
            assert!(
                content.iter().all(|&b| b == 0),
                "Scrubbed file must consist purely of zeros"
            );
        } else {
            assert_eq!(
                outcome,
                ScrubOutcome::NoOp,
                "rotational={rotational}, is_cow={is_cow} must be a NoOp"
            );

            // Dateiinhalt darf bei No-Op nicht überschrieben worden sein
            let content = fs::read(&file_path).expect("Read untouched file failed");
            assert_eq!(
                content, initial_data,
                "File content must remain unchanged when scrubbing is bypassed"
            );
        }
    }
}

#[test]
fn test_df_07_t_cfg_3_hardlinks_refused() {
    let temp = TempDir::new().expect("TempDir failed");
    let file1 = temp.path().join("original.bin");
    let file2 = temp.path().join("hardlink.bin");

    fs::write(&file1, b"MULTIPLE_HARDLINKS_DATA").expect("Write file1 failed");

    #[cfg(unix)]
    {
        fs::hard_link(&file1, &file2).expect("Hard link creation failed");

        // Wenn Hardlinks existieren, muss auch bei rotational=true und !is_cow verweigert werden
        let outcome = scrub_before_unlink(&file1, true, false).expect("scrub_before_unlink failed");

        assert_eq!(
            outcome,
            ScrubOutcome::Refused(ScrubRefusal::MultipleHardlinks),
            "Hardlinked file must refuse scrubbing to protect secondary links"
        );

        // Daten dürfen nicht beschädigt sein
        assert_eq!(
            fs::read(&file2).expect("Read hardlink failed"),
            b"MULTIPLE_HARDLINKS_DATA"
        );
    }
}
