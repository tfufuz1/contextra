#![no_main]

use arbitrary::Arbitrary;
use libfuzzer_sys::fuzz_target;
use contextra_crypto::wal_crypto::{IntegrityVerifier, WalEntrySnapshot, WalHmac};

#[derive(Arbitrary, Debug)]
pub struct FuzzWalEntry {
    pub tx_id: u64,
    pub seq_no: u64,
    pub op_type: u8,
    pub key: Vec<u8>,
    pub value: Vec<u8>,
    pub checksum: [u8; 32],
    pub prev_hmac: [u8; 32],
    pub offset: u64,
    pub tamper_checksum: bool,
    pub tamper_prev_hmac: bool,
    pub verify_mode: u8,
}

#[derive(Arbitrary, Debug)]
pub struct FuzzWalHmacChainInput {
    pub integrity_key: Vec<u8>,
    pub entries: Vec<FuzzWalEntry>,
    pub standalone_hmac_key: Vec<u8>,
    pub standalone_hmac_data: Vec<Vec<u8>>,
    pub initial_last_hmac: Option<[u8; 32]>,
}

fuzz_target!(|input: FuzzWalHmacChainInput| {
    // 1. Standalone WalHmac constructor and update testing with arbitrary keys
    if let Ok(mut hmac) = WalHmac::new(&input.standalone_hmac_key) {
        for chunk in &input.standalone_hmac_data {
            hmac.update(chunk);
        }
        let _digest = hmac.finalize();
    }

    // 2. IntegrityVerifier hash chain verification over arbitrary sequence
    let mut verifier = IntegrityVerifier::new(&input.integrity_key);

    if let Some(initial_hmac) = input.initial_last_hmac {
        verifier.set_last_hmac(initial_hmac);
        let snapshot = verifier.last_hmac_snapshot();
        let _ = snapshot;
    }

    // Process arbitrary entries
    for fuzz_entry in &input.entries {
        let mut entry = WalEntrySnapshot {
            tx_id: fuzz_entry.tx_id,
            seq_no: fuzz_entry.seq_no,
            op_type: fuzz_entry.op_type,
            key: fuzz_entry.key.clone(),
            value: fuzz_entry.value.clone(),
            checksum: fuzz_entry.checksum,
            prev_hmac: fuzz_entry.prev_hmac,
        };

        if fuzz_entry.tamper_checksum {
            entry.checksum[0] ^= 0xFF;
        }
        if fuzz_entry.tamper_prev_hmac {
            entry.prev_hmac[0] ^= 0xFF;
        }

        match fuzz_entry.verify_mode % 4 {
            0 => {
                let _ = verifier.verify_and_update(&entry, fuzz_entry.offset);
            }
            1 => {
                let _ = verifier.verify_and_update_v3(&entry, fuzz_entry.offset);
            }
            2 => {
                let _ = verifier.verify_and_update_v2(&entry, fuzz_entry.offset);
            }
            _ => {
                verifier.skip_hmac_verify_legacy(&entry);
            }        }
    }

    // 3. Construct a strictly valid chain and verify behavior under single-bit mutations
    if input.integrity_key.is_empty() || input.integrity_key.len() > 10_000 {
        return;
    }

    let mut valid_verifier = IntegrityVerifier::new(&input.integrity_key);
    let mut prev_hmac = [0u8; 32];

    for (idx, fuzz_entry) in input.entries.iter().take(10).enumerate() {
        let seq_no = (idx as u64) + 1;
        let tx_id = seq_no;

        // Compute valid checksum
        let mut hmac = match WalHmac::new(&input.integrity_key) {
            Ok(h) => h,
            Err(_) => return,
        };
        hmac.update(&prev_hmac);
        hmac.update(&seq_no.to_le_bytes());
        hmac.update(&tx_id.to_le_bytes());

        let op_type = fuzz_entry.op_type % 3;
        if op_type == 0 {
            hmac.update(&[0u8]);
            hmac.update(&(fuzz_entry.key.len() as u32).to_le_bytes());
            hmac.update(&fuzz_entry.key);
            hmac.update(&(fuzz_entry.value.len() as u32).to_le_bytes());
            hmac.update(&fuzz_entry.value);
        } else if op_type == 1 {
            hmac.update(&[1u8]);
            hmac.update(&(fuzz_entry.key.len() as u32).to_le_bytes());
            hmac.update(&fuzz_entry.key);
        } else {
            hmac.update(&[2u8]);
            let committed = if fuzz_entry.value.first().copied().unwrap_or(0) != 0 {
                1u8
            } else {
                0u8
            };
            hmac.update(&[committed]);
        }

        let checksum = hmac.finalize();

        let valid_entry = WalEntrySnapshot {
            tx_id,
            seq_no,
            op_type,
            key: fuzz_entry.key.clone(),
            value: fuzz_entry.value.clone(),
            checksum,
            prev_hmac,
        };

        if fuzz_entry.tamper_checksum {
            let mut tampered = valid_entry.clone();
            tampered.checksum[0] ^= 0x01;
            let res = valid_verifier.verify_and_update(&tampered, seq_no * 100);
            assert!(
                res.is_err(),
                "Tampered checksum MUST fail verification with Err(WalCorruption)"
            );
            break;
        } else {
            let res = valid_verifier.verify_and_update(&valid_entry, seq_no * 100);
            if res.is_ok() {
                prev_hmac = checksum;
            } else {
                break;
            }
        }
    }
});
