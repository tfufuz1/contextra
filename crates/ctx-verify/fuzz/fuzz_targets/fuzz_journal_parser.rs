#![no_main]

use ctx_verify::journal::JournalChainEntry;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    if data.len() >= 32 {
        let mut prev = [0u8; 32];
        prev.copy_from_slice(&data[0..32]);
        let payload = &data[32..];
        let _ = JournalChainEntry::compute_chain_step(&prev, payload);
    }
});
