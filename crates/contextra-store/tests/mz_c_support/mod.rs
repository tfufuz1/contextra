//! Hilfsmodul für MZ_C Integrationstests (Manifest und Memtable)
//!
//! Stellt Test-Helfer für Byte-Identität und Shard-Verifikation bereit.

use contextra_store::manifest::ManifestEntry;
use std::path::PathBuf;

/// Erzeugt eine deterministische Menge von Manifest-Einträgen für Roundtrip-Tests.
pub fn generate_sample_manifest_entries() -> Vec<ManifestEntry> {
    vec![
        ManifestEntry::Add {
            path: PathBuf::from("sst_001.sst"),
            max_tx: 10,
        },
        ManifestEntry::Add {
            path: PathBuf::from("sst_002.sst"),
            max_tx: 20,
        },
        ManifestEntry::WalCheckpoint { hmac: [0x42; 32] },
        ManifestEntry::Replace {
            removed: vec![PathBuf::from("sst_001.sst")],
            added: PathBuf::from("sst_003.sst"),
            added_max_tx: 30,
            rank: 0,
        },
        ManifestEntry::Remove {
            path: PathBuf::from("sst_002.sst"),
        },
        ManifestEntry::RollbackComplete { target_tx: 100 },
    ]
}
