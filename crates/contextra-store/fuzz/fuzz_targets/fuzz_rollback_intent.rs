#![no_main]

use libfuzzer_sys::fuzz_target;

const INTENT_MAGIC: &[u8] = b"MFRLBK\0\0";

fuzz_target!(|data: &[u8]| {
    // 1. Validierung des 16-Byte Intent-Formats (8B Magic + 8B TxId little-endian)
    if data.len() < 16 {
        return;
    }

    if &data[0..8] == INTENT_MAGIC {
        if let Ok(raw_tx) = data[8..16].try_into() {
            let tx_num = u64::from_le_bytes(raw_tx);
            // Intent tx_id in recovery.rs ist immer valid
            let _tx = contextra_types::TxId::new(tx_num);
        }
    }
});
