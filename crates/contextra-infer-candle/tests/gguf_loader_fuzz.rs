// FILE-CONTEXT
// STAND: 2026-11-09T00:00:00Z
// ZWECK: In-process fuzzing suite for GGUF model container parsing.
// INVARIANTEN: Zero panics under arbitrary byte inputs; fail-closed execution.

//! Fuzz-Testsuite für den GGUF-Header- und Metadaten-Parser.
//!
//! # Zweck
//! Speist pseudozufällige Bytes und manipulierte GGUF-Strukturen in `parse_gguf_metadata` ein
//! und verifiziert, dass unter keinen Umständen Panics, OOMs oder Endlosschleifen auftreten.

// ANCHOR[contextra-infer-candle:TEST-GGUF-LOADER-FUZZ]
use contextra_infer_candle::gguf_loader::parse_gguf_metadata;
use std::io::Write;
use tempfile::NamedTempFile;

#[test]
fn test_fuzz_gguf_loader_random_bytes_no_panic() {
    let mut rng_state = 0x123456789abcdef0u64;
    let mut next_u32 = || {
        rng_state = rng_state.wrapping_mul(6364136223846793005).wrapping_add(1);
        (rng_state >> 32) as u32
    };

    for iter in 0..100 {
        let len = (next_u32() % 1024) as usize;
        let mut buffer = vec![0u8; len];
        for byte in &mut buffer {
            *byte = (next_u32() & 0xFF) as u8;
        }

        // Gelegentlich gültige Magic Bytes einstreuen
        if iter % 5 == 0 && buffer.len() >= 4 {
            buffer[0..4].copy_from_slice(b"GGUF");
        }

        let mut tmp_file = NamedTempFile::new().unwrap();
        tmp_file.write_all(&buffer).unwrap();
        tmp_file.flush().unwrap();

        // Parser darf unter keinen Umständen panicken
        let _ = parse_gguf_metadata(tmp_file.path());
    }
}
