// FILE-CONTEXT
// ZWECK: Fuzz-Target für DiskANN Formatprüfungen (DANN, PWAL, TWAL)
// INVARIANTEN: Reines fuzz_one ohne direkte libfuzzer-Laufzeitabhängigkeit für Smoke-Treiber
// HOTSPOTS: 15-40
// STAND: 2026-11-11
// BEREICH: AGENTS §8, SPEC §11.4 — DiskANN Delta-Recovery und Format-Robustheit

// ANCHOR[contextra-vector:W11-FUZZ-002]

#![no_main]

use libfuzzer_sys::fuzz_target;

/// Reines Fuzz-Primitiv zur Entkopplung für Smoke-Treiber ohne libFuzzer.
pub fn fuzz_one(data: &[u8]) {
    if data.len() < 5 {
        return;
    }

    if let Ok(temp_dir) = tempfile::tempdir() {
        let wal_pwal = temp_dir.path().join("fuzz.pending.wal");
        let wal_twal = temp_dir.path().join("fuzz.tombstone.wal");

        let _ = std::fs::write(&wal_pwal, data);
        let _ = std::fs::write(&wal_twal, data);

        let config = contextra_vector::diskann::DiskAnnConfig {
            dimension: 16,
            max_degree: 8,
            beam_width: 16,
            index_path: temp_dir.path().join("fuzz.dann"),
            ..Default::default()
        };
        if let Ok(index) = contextra_vector::diskann::DiskAnnIndex::try_new(config) {
            let _ = index.recover_pending_delta_sync();
        }
    }
}

fuzz_target!(|data: &[u8]| {
    fuzz_one(data);
});
