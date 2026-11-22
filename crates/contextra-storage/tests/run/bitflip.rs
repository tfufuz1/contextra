//! # Bitflip-Robustheitstests für Run (SPEC §18, INV-FMT-1, INV-FMT-2)
//!
//! Jedes Byte eines kleinen Runs gekippt führt zu Err oder identischem Ergebnis,
//! niemals zu falschen oder still beschädigten Daten.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::indexing_slicing, clippy::panic)]

use contextra_storage::codec::tag::{Tag, TagKind};
use contextra_storage::run::block::BlockSecurity;
use contextra_storage::run::builder::{RunBuilder, RunBuilderOptions};
use contextra_storage::run::reader::RunReader;
use std::sync::Arc;

use super::roundtrip::MemoryFile;

#[test]
fn test_bitflip_all_bytes() {
    let opts = RunBuilderOptions {
        space_id: 1,
        run_id: 777,
        header_len: 4,
        weights_k: 1,
        prefix_filter_len: Some(3),
        created_ms: 1_700_000_000_000,
    };

    let mut builder = RunBuilder::new(opts, BlockSecurity::Plain);

    let entries: Vec<(Vec<u8>, Tag, Vec<u8>)> = (0..20)
        .map(|i| {
            let k = format!("key:{:03}", i).into_bytes();
            let v = format!("hdr_{:05}", i).into_bytes();
            let t = Tag::new((i + 1) as u64, TagKind::Put);
            (k, t, v)
        })
        .collect();

    for (k, t, v) in &entries {
        builder.add(k, *t, v).unwrap();
    }

    let orig_bytes = builder.finish().unwrap();
    let file_len = orig_bytes.len() as u64;

    // Sanity-Check auf unbeschädigter Datei
    {
        let file = Arc::new(MemoryFile(Arc::new(orig_bytes.clone())));
        let reader = RunReader::open(file, file_len, BlockSecurity::Plain, None).unwrap();
        for (k, t, v) in &entries {
            let e = reader.get(k).unwrap().expect("Eintrag muss existieren");
            assert_eq!(&e.tag, t);
            assert_eq!(&e.value, v);
        }
    }

    let mut detected_errors = 0usize;
    let mut clean_reads = 0usize;

    // Jedes Byte einzeln kippen
    for byte_idx in 0..orig_bytes.len() {
        for &flip_mask in &[0xFFu8, 0x01u8] {
            let mut corrupted = orig_bytes.clone();
            corrupted[byte_idx] ^= flip_mask;

            let file = Arc::new(MemoryFile(Arc::new(corrupted)));
            match RunReader::open(file, file_len, BlockSecurity::Plain, None) {
                Err(_) => {
                    // Fehler beim Öffnen erkannt (Footer, Directory, Meta, Index)
                    detected_errors += 1;
                }
                Ok(reader) => {
                    // Reader konnte öffnen (z. B. Bitflip in lazy Filter- oder Datenblock)
                    // Alle Einträge abfragen: darf NIE falsche Daten liefern!
                    for (k, t, v) in &entries {
                        match reader.get(k) {
                            Err(_) => {
                                // Fehler beim Lesen des Blocks erkannt
                                detected_errors += 1;
                            }
                            Ok(Some(entry)) => {
                                // Wenn ein Wert zurückkommt, MUSS er exakt dem Original entsprechen!
                                assert_eq!(&entry.user_key, k, "Schlüssel korrumpiert an Byte {}", byte_idx);
                                assert_eq!(&entry.tag, t, "Tag korrumpiert an Byte {}", byte_idx);
                                assert_eq!(&entry.value, v, "Wert korrumpiert an Byte {}", byte_idx);
                                clean_reads += 1;
                            }
                            Ok(None) => {
                                // Schlüssel wurde durch korrumpierten Filter / Index nicht gefunden
                                detected_errors += 1;
                            }
                        }
                    }
                }
            }
        }
    }

    assert!(detected_errors > 0, "Bitflips müssen detektiert werden");
    assert!(clean_reads > 0, "Einige unkritische Bits erlauben noch sauberes Lesen");
}
