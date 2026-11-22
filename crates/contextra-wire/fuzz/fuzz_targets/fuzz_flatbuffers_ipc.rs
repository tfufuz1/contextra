#![no_main]

use libfuzzer_sys::fuzz_target;
use contextra_wire::{
    root_as_search_response, root_as_search_response_with_opts,
    size_prefixed_root_as_search_response,
};

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
