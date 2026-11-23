//! # Zentrale Nonce-Tabelle
//!
//! Normative Regeln (SPEC §15.2, Golden A.6, SD-50):
//! Nonces haben genau 12 Bytes.
//! Jedes Präfix ist disjunkt (`0x01` bis `0x05`).
//! Dieses Modul ist die einzige Stelle im gesamten Kern, die Nonces bildet.

use contextra_ports::persistence::Entropy;
use std::io;

/// Erstellt die 12-Byte-Nonce für einen Run-Block.
///
/// Format: `0x01 || block_offset u64 || 0u24`
pub fn run_block_nonce(block_offset: u64) -> [u8; 12] {
    let mut nonce = [0u8; 12];
    if let Some(first) = nonce.first_mut() {
        *first = 0x01;
    }
    if let Some(target) = nonce.get_mut(1..9) {
        target.copy_from_slice(&block_offset.to_le_bytes());
    }
    // Bytes 9..12 bleiben 0
    nonce
}

/// Erstellt die 12-Byte-Nonce für einen Redo-Frame.
///
/// Format: `0x02 || frame_index u32 || 0u56`
pub fn redo_frame_nonce(frame_index: u32) -> [u8; 12] {
    let mut nonce = [0u8; 12];
    if let Some(first) = nonce.first_mut() {
        *first = 0x02;
    }
    if let Some(target) = nonce.get_mut(1..5) {
        target.copy_from_slice(&frame_index.to_le_bytes());
    }
    // Bytes 5..12 bleiben 0
    nonce
}

/// Erstellt die 12-Byte-Nonce für einen domänenversiegelten Redo-Op-Körper.
///
/// Format: `0x03 || segment_id u32 || frame_index u32 || op_index u24`
pub fn redo_op_body_nonce(segment_id: u32, frame_index: u32, op_index: u32) -> [u8; 12] {
    let mut nonce = [0u8; 12];
    if let Some(first) = nonce.first_mut() {
        *first = 0x03;
    }
    if let Some(target) = nonce.get_mut(1..5) {
        target.copy_from_slice(&segment_id.to_le_bytes());
    }
    if let Some(target) = nonce.get_mut(5..9) {
        target.copy_from_slice(&frame_index.to_le_bytes());
    }
    let op_bytes = op_index.to_le_bytes();
    if let (Some(target), Some(src)) = (nonce.get_mut(9..12), op_bytes.get(..3)) {
        target.copy_from_slice(src);
    }
    nonce
}

/// Erstellt die 12-Byte-Nonce für einen Blob-Record.
///
/// Format: `0x04 || offset u64 || 0u24`
pub fn blob_record_nonce(offset: u64) -> [u8; 12] {
    let mut nonce = [0u8; 12];
    if let Some(first) = nonce.first_mut() {
        *first = 0x04;
    }
    if let Some(target) = nonce.get_mut(1..9) {
        target.copy_from_slice(&offset.to_le_bytes());
    }
    // Bytes 9..12 bleiben 0
    nonce
}

/// Erstellt die 12-Byte-Nonce für einen domänenversiegelten Blob-Körper.
///
/// Format: `0x05 || blob_no u56 || offset u32`
pub fn blob_body_nonce(blob_no: u64, offset: u32) -> [u8; 12] {
    let mut nonce = [0u8; 12];
    if let Some(first) = nonce.first_mut() {
        *first = 0x05;
    }
    let b_bytes = blob_no.to_le_bytes();
    if let (Some(target), Some(src)) = (nonce.get_mut(1..8), b_bytes.get(..7)) {
        target.copy_from_slice(src);
    }
    if let Some(target) = nonce.get_mut(8..12) {
        target.copy_from_slice(&offset.to_le_bytes());
    }
    nonce
}

/// Erstellt die 12-Byte-Nonce für einen Catalog-Record.
///
/// Format: `record_index u64 || 0u32`
pub fn catalog_record_nonce(record_index: u64) -> [u8; 12] {
    let mut nonce = [0u8; 12];
    if let Some(target) = nonce.get_mut(0..8) {
        target.copy_from_slice(&record_index.to_le_bytes());
    }
    // Bytes 8..12 bleiben 0
    nonce
}

/// Erstellt die 12-Byte-Nonce für die `KEYS`-Datei aus vorgegebenen Zufallsbytes.
///
/// Typisierter Konstruktor für KEYS-Nonces (SPEC §15.2).
pub fn keys_nonce(random_bytes: [u8; 12]) -> [u8; 12] {
    random_bytes
}

/// Erstellt eine frische 12-Byte-Nonce für die `KEYS`-Datei aus dem Entropie-Port.
pub fn keys_nonce_from_entropy(entropy: &dyn Entropy) -> io::Result<[u8; 12]> {
    let mut nonce = [0u8; 12];
    entropy.fill(&mut nonce)?;
    Ok(nonce)
}

/// Weitet eine 12-Byte-Nonce auf die 24 Byte des Ports `Cipher` (UE-10).
///
/// Format: `n12 || 0u96`. Die Abbildung ist injektiv, daher bleibt die Eindeutigkeit der
/// Nonce-Tabelle (Präfixe `0x01` bis `0x05`) erhalten. Das ist die Brücke bis zur
/// Umstellung auf 24-Byte-Zufallsnonces (SD-14), siehe PR.
pub fn port_nonce(n12: &[u8; 12]) -> [u8; 24] {
    let mut out = [0u8; 24];
    if let Some(head) = out.get_mut(0..12) {
        head.copy_from_slice(n12);
    }
    out
}

#[cfg(test)]
mod port_nonce_tests {
    use super::*;

    #[test]
    fn pads_with_zero_tail_and_keeps_head() {
        let n12 = [7u8; 12];
        let n24 = port_nonce(&n12);
        assert_eq!(n24.get(0..12), Some(&n12[..]));
        assert_eq!(n24.get(12..24), Some(&[0u8; 12][..]));
    }

    #[test]
    fn distinct_table_nonces_stay_distinct() {
        let a = port_nonce(&run_block_nonce(1));
        let b = port_nonce(&redo_frame_nonce(1));
        let c = port_nonce(&catalog_record_nonce(1));
        assert_ne!(a, b);
        assert_ne!(a, c);
        assert_ne!(b, c);
    }
}
