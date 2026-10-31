//! # Fuzz-Target für FlatBuffers IPC Deserialisierung
//!
//! **Ring & Klasse:** Ring 0 (Fuzzing / REQ-QA-003)
//!
//! ## Zweck
//! Dieses Fuzz-Target speist zufällige Byte-Arrays in die FlatBuffers-Deserialisierungsroutinen
//! von `contextra-wire` ein, um Abstürze, Speicherlecks oder Panics (REQ-QA-010 / P7 Zero-Panic)
//! proaktiv aufzudecken.

#![no_main]

use libfuzzer_sys::fuzz_target;
use contextra_wire::{
    root_as_search_response, root_as_search_response_with_opts,
    size_prefixed_root_as_search_response,
};

// ANCHOR[contextra-wire:FUZ-001]
// Fuzz-Test-Einsprungpunkt für libFuzzer.
// Ablauf:
// 1. Root-Search-Response Parsing & Verifikation mit Standard-Optionen.
// 2. Traversieren aller Felder (Hits, Processing Time, Scored Documents, Embeddings), falls gültig.
// 3. Size-prefixed Parsing mit zufälligen Daten.
// 4. Verifier-Durchlauf mit benutzerdefinierten Tiefen- und Größengrenzen.
fuzz_target!(|data: &[u8]| {
    // 1. Root search response parsing & verification
    if let Ok(resp) = root_as_search_response(data) {
        let _ = resp.total_hits();
        let _ = resp.processing_time_ms();
        if let Some(results) = resp.results() {
            for doc in results.iter() {
                let _ = doc.id();
                let _ = doc.score();
                let _ = doc.metadata();
                if let Some(emb) = doc.embedding() {
                    let _ = emb.data();
                    let _ = emb.quantized_u8();
                    let _ = emb.metric();
                }
            }
        }
    }

    // 2. Size-prefixed parsing
    let _ = size_prefixed_root_as_search_response(data);

    // 3. Verifier with custom options
    let opts = flatbuffers::VerifierOptions {
        max_depth: 64,
        max_tables: 1000,
        max_apparent_size: 1_000_000,
        ..Default::default()
    };
    let _ = root_as_search_response_with_opts(&opts, data);
});
