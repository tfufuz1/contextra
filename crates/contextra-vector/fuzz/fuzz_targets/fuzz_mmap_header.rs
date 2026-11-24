// FILE-CONTEXT
// ZWECK: Fuzz-Target für Mmap-Header-Deserialisierung
// INVARIANTEN: Reines fuzz_one ohne direkte libfuzzer-Laufzeitabhängigkeit für Smoke-Treiber
// HOTSPOTS: 15-30
// STAND: 2026-11-11
// BEREICH: AGENTS §8, SPEC §11.4 — Mmap-Header-Robustheit gegen In-Place-Korruption

// ANCHOR[contextra-vector:W11-FUZZ-004]

#![no_main]

use contextra_vector::persistence::HnswHeader;
use libfuzzer_sys::fuzz_target;

/// Reines Fuzz-Primitiv zur Entkopplung für Smoke-Treiber ohne libFuzzer.
pub fn fuzz_one(data: &[u8]) {
    if let Ok(header) = HnswHeader::try_from_bytes(data) {
        let bytes = header.to_bytes();
        let _ = header.checksum();
        let _ = HnswHeader::try_from_bytes(&bytes);
    }
}

fuzz_target!(|data: &[u8]| {
    fuzz_one(data);
});
