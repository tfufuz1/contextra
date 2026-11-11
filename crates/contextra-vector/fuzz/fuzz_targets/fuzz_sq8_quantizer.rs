// FILE-CONTEXT
// ZWECK: Fuzz-Target für SQ8 Skalar-Quantisierer (Training, Quantize, Dequantize)
// INVARIANTEN: Reines fuzz_one ohne direkte libfuzzer-Laufzeitabhängigkeit für Smoke-Treiber
// HOTSPOTS: 20-55
// STAND: 2026-11-11
// BEREICH: AGENTS §8, SPEC §11.4 — SQ8-Quantisierung und Outlier-Stabilität

// ANCHOR[contextra-vector:W11-FUZZ-005]

#![no_main]

use contextra_vector::quantize::ScalarQuantizer;
use libfuzzer_sys::fuzz_target;

/// Reines Fuzz-Primitiv zur Entkopplung für Smoke-Treiber ohne libFuzzer.
pub fn fuzz_one(data: &[u8]) {
    if data.len() < 16 {
        return;
    }
    let dim = (data[0] as usize % 64) + 1;
    let floats: Vec<f32> = data[1..]
        .chunks_exact(4)
        .map(|chunk| {
            let mut arr = [0u8; 4];
            arr.copy_from_slice(chunk);
            f32::from_le_bytes(arr)
        })
        .collect();

    if floats.len() >= dim {
        let refs: Vec<&[f32]> = floats.chunks_exact(dim).collect();
        if !refs.is_empty() {
            let quantizer = ScalarQuantizer::train(&refs, dim);
            if let Ok(quantized) = quantizer.quantize(refs[0]) {
                let _ = quantizer.dequantize(&quantized);
            }
        }
    }
}

fuzz_target!(|data: &[u8]| {
    fuzz_one(data);
});
