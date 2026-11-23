#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::panic
)]

//! # Tests des Seal-Rahmens (SPEC §10.10, §10.1, §10.4, §10.5, §10.7, §16.3)
//!
//! Golden: `tests/format/golden/seal_frame.txt`, erzeugt von `tools/reference/generator.py`.
//! Die Tests lesen die Golden-Datei zur Laufzeit; Werte werden nicht aus dem Kern abgeleitet.

use super::seal_cipher::XChaCha20Poly1305Ref;
use super::*;
use std::io;
use std::sync::Mutex;

use contextra_ports::persistence::{Entropy, Kdf};

const GOLDEN: &str = include_str!("golden/seal_frame.txt");

fn hex(s: &str) -> Vec<u8> {
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16).unwrap())
        .collect()
}

fn key32(s: &str) -> [u8; 32] {
    hex(s).try_into().unwrap()
}

fn nonce24(s: &str) -> SealNonce {
    hex(s).try_into().unwrap()
}

fn seq16(start: u8) -> [u8; 16] {
    let mut out = [0u8; 16];
    for (i, b) in out.iter_mut().enumerate() {
        *b = start.wrapping_add(u8::try_from(i).unwrap());
    }
    out
}

struct FrameCase {
    name: String,
    key: [u8; 32],
    nonce: SealNonce,
    aad: Vec<u8>,
    plain: Vec<u8>,
    sealed: Vec<u8>,
}

fn frame_cases() -> Vec<FrameCase> {
    GOLDEN
        .lines()
        .filter(|l| l.starts_with("frame|"))
        .map(|line| {
            let f: Vec<&str> = line.split('|').collect();
            assert_eq!(f.len(), 7, "Golden-Zeile hat 7 Felder: {line}");
            FrameCase {
                name: f[1].to_string(),
                key: key32(f[2]),
                nonce: nonce24(f[3]),
                aad: hex(f[4]),
                plain: hex(f[5]),
                sealed: hex(f[6]),
            }
        })
        .collect()
}

/// Typisierte AAD der Golden-Fälle; `None` für den rohen Vektor aus dem Entwurf.
fn typed_aad(name: &str) -> Option<SealAad> {
    match name {
        "blob_record" => Some(SealAad::Blob {
            space: 1,
            cluster: seq16(0x10),
            blob_id: 0x0102_0304_0506_0708,
            record_uid: seq16(0x30),
        }),
        "redo_op" => Some(SealAad::RedoOp {
            space: 2,
            cluster: seq16(0x10),
            op_uid: seq16(0x50),
            op_index: 7,
        }),
        "block" => Some(SealAad::Block {
            space: 3,
            cluster: seq16(0x10),
            block_uid: seq16(0x70),
        }),
        "block_empty_plain" => Some(SealAad::Block {
            space: 0,
            cluster: [0; 16],
            block_uid: [0; 16],
        }),
        "block_odd_97" => Some(SealAad::Block {
            space: 4,
            cluster: seq16(0x10),
            block_uid: seq16(0xE0),
        }),
        "blob_multi_block_1000" => Some(SealAad::Blob {
            space: 5,
            cluster: seq16(0x10),
            blob_id: 0xFFEE_DDCC_BBAA_9988,
            record_uid: seq16(0x40),
        }),
        _ => None,
    }
}

#[test]
fn golden_seal_frame_matches_generator() {
    let cases = frame_cases();
    assert_eq!(cases.len(), 7, "alle sieben Frame-Goldens vorhanden");
    let cipher = XChaCha20Poly1305Ref;
    for c in &cases {
        if let Some(aad) = typed_aad(&c.name) {
            assert_eq!(aad.encode(), c.aad, "AAD-Kodierung {}", c.name);
        }
        let got = seal_frame(&cipher, &c.key, &c.nonce, &c.aad, &c.plain);
        assert_eq!(got, c.sealed, "Golden seal_frame {}", c.name);
        assert_eq!(
            c.sealed.len(),
            SEAL_NONCE_LEN + c.plain.len() + SEAL_TAG_LEN
        );
        let opened = open_frame(&cipher, &c.key, &c.aad, &c.sealed).unwrap();
        assert_eq!(opened, c.plain, "Roundtrip {}", c.name);
    }
}

#[test]
fn golden_label_matches_generator() {
    struct ContextKdf;
    impl Kdf for ContextKdf {
        fn derive(&self, _key: &[u8; 32], context: &[&[u8]]) -> [u8; 32] {
            let mut out = [0u8; 32];
            let mut i = 0;
            for part in context {
                for b in part.iter() {
                    if let Some(slot) = out.get_mut(i) {
                        *slot = *b;
                    }
                    i += 1;
                }
            }
            out
        }
    }
    let labels: Vec<&str> = GOLDEN.lines().filter(|l| l.starts_with("label|")).collect();
    assert_eq!(labels.len(), 2);
    for line in labels {
        let f: Vec<&str> = line.split('|').collect();
        let epoch: u32 = f[1].parse().unwrap();
        let want = hex(f[2]);
        let got = derive_seal_key(&ContextKdf, &[0; 32], epoch);
        assert_eq!(&got[..want.len()], want.as_slice(), "Label epoch {epoch}");
    }
}

#[test]
fn manipulating_any_frame_byte_is_rejected() {
    let cipher = XChaCha20Poly1305Ref;
    for c in frame_cases() {
        for i in 0..c.sealed.len() {
            for mask in [0x01u8, 0x80, 0xFF] {
                let mut bad = c.sealed.clone();
                bad[i] ^= mask;
                assert_eq!(
                    open_frame(&cipher, &c.key, &c.aad, &bad),
                    Err(SealError::Corrupt),
                    "{}: Byte {i} mit {mask:#04x} gekippt wird akzeptiert",
                    c.name
                );
            }
        }
    }
}

#[test]
fn manipulating_any_aad_byte_is_rejected() {
    let cipher = XChaCha20Poly1305Ref;
    for c in frame_cases() {
        for j in 0..c.aad.len() {
            for mask in [0x01u8, 0x80, 0xFF] {
                let mut bad = c.aad.clone();
                bad[j] ^= mask;
                assert_eq!(
                    open_frame(&cipher, &c.key, &bad, &c.sealed),
                    Err(SealError::Corrupt),
                    "{}: AAD-Byte {j} mit {mask:#04x} gekippt wird akzeptiert",
                    c.name
                );
            }
        }
    }
}

#[test]
fn wrong_key_and_truncation_are_rejected() {
    let cipher = XChaCha20Poly1305Ref;
    let c = &frame_cases()[0];
    let mut other = c.key;
    other[0] ^= 1;
    assert_eq!(
        open_frame(&cipher, &other, &c.aad, &c.sealed),
        Err(SealError::Corrupt)
    );
    // Zu kurz für Nonce und Tag, dann gerade ohne Klartext-Teil, dann verlängert.
    for n in [0usize, 23, 24, 39] {
        assert_eq!(
            open_frame(&cipher, &c.key, &c.aad, &c.sealed[..n]),
            Err(SealError::Corrupt),
            "Länge {n}"
        );
    }
    let mut longer = c.sealed.clone();
    longer.push(0);
    assert_eq!(
        open_frame(&cipher, &c.key, &c.aad, &longer),
        Err(SealError::Corrupt)
    );
}

#[test]
fn aad_kind_is_bound() {
    // Dieselben Felder als Block-AAD öffnen einen Blob-Rahmen nicht: das Art-Byte trennt.
    let cipher = XChaCha20Poly1305Ref;
    let blob = frame_cases()
        .into_iter()
        .find(|c| c.name == "blob_record")
        .unwrap();
    let as_block = SealAad::Block {
        space: 1,
        cluster: seq16(0x10),
        block_uid: seq16(0x30),
    }
    .encode();
    assert_eq!(
        open_frame(&cipher, &blob.key, &as_block, &blob.sealed),
        Err(SealError::Corrupt)
    );
}

#[test]
fn aad_lengths_are_fixed_per_kind() {
    assert_eq!(typed_aad("redo_op").unwrap().encode().len(), 37);
    assert_eq!(typed_aad("block").unwrap().encode().len(), 35);
    assert_eq!(typed_aad("blob_record").unwrap().encode().len(), 43);
    assert_eq!(SEAL_AAD_MAX_LEN, 43);
}

#[test]
fn fresh_nonce_is_drawn_from_entropy() {
    struct Counter;
    impl Entropy for Counter {
        fn fill(&self, dst: &mut [u8]) -> io::Result<()> {
            for (i, b) in dst.iter_mut().enumerate() {
                *b = u8::try_from(i).unwrap();
            }
            Ok(())
        }
    }
    struct Broken;
    impl Entropy for Broken {
        fn fill(&self, _dst: &mut [u8]) -> io::Result<()> {
            Err(io::Error::other("kein Zufall"))
        }
    }
    let n = fresh_nonce(&Counter).unwrap();
    assert_eq!(n.len(), SEAL_NONCE_LEN);
    assert_eq!(n[23], 23);
    assert!(fresh_nonce(&Broken).is_err());
}

#[test]
fn constants_match_spec() {
    assert_eq!(SEAL_NONCE_LEN, 24);
    assert_eq!(SEAL_TAG_LEN, 16);
    assert_eq!(SEAL_NONCE_LIMIT, 1 << 32);
}

/// Hochwassermarke, die jede Persistierung protokolliert und auf Wunsch fehlschlägt.
struct Hw {
    log: Mutex<Vec<u64>>,
    fail: bool,
}

impl Hw {
    fn ok() -> Self {
        Self {
            log: Mutex::new(Vec::new()),
            fail: false,
        }
    }
    fn failing() -> Self {
        Self {
            log: Mutex::new(Vec::new()),
            fail: true,
        }
    }
    fn persisted(&self) -> Vec<u64> {
        self.log.lock().unwrap().clone()
    }
}

impl SealHighWater for Hw {
    fn persist(&self, new_hw: u64) -> io::Result<()> {
        if self.fail {
            return Err(io::Error::other("Katalog nicht erreichbar"));
        }
        self.log.lock().unwrap().push(new_hw);
        Ok(())
    }
}

#[test]
fn budget_reserves_before_issuing_and_stops_at_limit() {
    let hw = Hw::ok();
    let mut b = SealBudget::resume(10, 4, 0).unwrap();
    for _ in 0..10 {
        b.take(&hw).unwrap();
    }
    // Reservierungen: 4, 8, dann auf die Schranke 10 gekappt.
    assert_eq!(hw.persisted(), vec![4, 8, 10]);
    assert_eq!(b.used(), 10);
    assert_eq!(b.take(&hw), Err(SealError::Exhausted));
    assert_eq!(
        hw.persisted().len(),
        3,
        "Erschöpfung reserviert nichts mehr"
    );
}

#[test]
fn budget_after_crash_never_reissues_reserved_values() {
    let hw = Hw::ok();
    let mut before = SealBudget::resume(10, 4, 0).unwrap();
    for _ in 0..5 {
        before.take(&hw).unwrap();
    }
    // Absturz: dauerhaft ist der letzte persistierte Wert (8). Der Rest des Blocks verfällt.
    let durable = *hw.persisted().last().unwrap();
    assert_eq!(durable, 8);
    let hw2 = Hw::ok();
    let mut after = SealBudget::resume(10, 4, durable).unwrap();
    assert_eq!(
        after.used(),
        8,
        "nach Absturz gilt der Hochwert als verbraucht"
    );
    after.take(&hw2).unwrap();
    after.take(&hw2).unwrap();
    assert_eq!(hw2.persisted(), vec![10]);
    assert_eq!(after.take(&hw2), Err(SealError::Exhausted));
    // Gesamtzahl über beide Läufe bleibt unter der Schranke.
    assert!(before.used() + 2 <= 10);
}

#[test]
fn budget_persist_failure_leaves_state_unchanged() {
    let failing = Hw::failing();
    let mut b = SealBudget::resume(10, 4, 0).unwrap();
    assert_eq!(b.take(&failing), Err(SealError::Io));
    assert_eq!(b.used(), 0);
    assert_eq!(b.reserved(), 0);
    let ok = Hw::ok();
    b.take(&ok).unwrap();
    assert_eq!(b.used(), 1);
    assert_eq!(ok.persisted(), vec![4]);
}

#[test]
fn budget_rejects_invalid_resume_state() {
    assert_eq!(
        SealBudget::resume(10, 4, 11).unwrap_err(),
        SealError::Corrupt
    );
    assert_eq!(
        SealBudget::resume(10, 0, 0).unwrap_err(),
        SealError::Corrupt
    );
}

#[test]
fn budget_at_limit_refuses_without_reserving() {
    let hw = Hw::ok();
    let mut b = SealBudget::for_epoch(SEAL_NONCE_LIMIT).unwrap();
    assert_eq!(b.take(&hw), Err(SealError::Exhausted));
    assert!(hw.persisted().is_empty());
}
