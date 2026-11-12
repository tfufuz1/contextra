//! # SSTable-Metadaten
//!
//! Zweck: Metadaten über eine erstellte SSTable (Schlüsselgrenzen, Offsets, Prüfsummen).
// ANCHOR[contextra-store:SST-BLD-101]

use bytes::Bytes;
use std::path::PathBuf;

pub struct SstableMetadata {
    pub first_key: Bytes,
    pub last_key: Bytes,
    pub file_size: u64,
    pub min_tx_id: u64,
    pub max_tx_id: u64,
    pub min_seq: u64,
    pub max_seq: u64,
}
