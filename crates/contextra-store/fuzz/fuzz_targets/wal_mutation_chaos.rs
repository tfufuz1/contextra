#![no_main]

use arbitrary::Arbitrary;
use libfuzzer_sys::fuzz_target;
use contextra_core::TxId;
use contextra_store::wal::{Wal, WalConfig, WalOp, WalVersion};
use tempfile::tempdir;

/*
 * REFLECTION ON MUTATION MODES & PANIC LIKELIHOOD:
 *
 * 1. `Truncate`:
 *    - Truncating a WAL file at arbitrary byte offsets (especially within inner framing,
 *      entry headers, or CRC/length fields) is extremely likely to trigger edge-case handling
 *      for EOF and partial WAL entries. In particular, truncating right after a length header
 *      or mid-entry requires clean tail-truncation logic without panicking or index out-of-bounds.
 *
 * 2. `Overwrite`:
 *    - Overwriting random bytes tests length-prefix boundaries and CRC checksum validation logic.
 *      Corrupting payload lengths to very large numbers (e.g., > MAX_WAL_ENTRY_SIZE) or unexpected
 *      op_type byte values tests bounds checking against file size and deserialization error paths.
 *
 * 3. `Insert`:
 *    - Inserting bytes shifts entry boundaries, breaking internal alignment, header magic, and HMAC / CRC
 *      verification. This verifies that offset calculations and slice index parsing do not underflow/overflow.
 *
 * 4. `ZeroOut` / `AllOnes`:
 *    - Zeroing or setting bytes to 0xFF tests zeroed-header validation, NULL-byte payload handling,
 *      and unexpected flag values without panicking during integer conversions or option unwraps.
 */

#[derive(Arbitrary, Debug)]
struct WalMutationInput {
    /// Entries to write before mutation (0..=20)
    valid_count: u8,
    /// Mutation mode
    mutation_mode: MutationMode,
    /// Byte offset for mutation (relative to file, 0..=255 mapped to 0..=100%)
    mutation_offset_pct: u8,
    /// Bytes to inject/overwrite (1..=64)
    mutation_bytes: Vec<u8>,
}

#[derive(Arbitrary, Debug)]
enum MutationMode {
    Overwrite, // Überschreibe N Bytes ab offset
    Insert,    // Füge N Bytes ein (verschiebt Rest)
    Truncate,  // Schneide ab offset ab
    ZeroOut,   // Setze N Bytes auf 0x00
    AllOnes,   // Setze N Bytes auf 0xFF
}

fuzz_target!(|input: WalMutationInput| {
    let rt = match tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
    {
        Ok(rt) => rt,
        Err(_) => return,
    };

    rt.block_on(async {
        let dir = match tempdir() {
            Ok(d) => d,
            Err(_) => return,
        };
        let wal_path = dir.path().join("fuzz_mutation.wal");

        let config = WalConfig {
            allow_legacy_integrity_key_fallback: true,
            min_wal_version: WalVersion::V1,
            ..Default::default()
        };

        // 1. Wal öffnen & valid_count Entries schreiben
        let count = (input.valid_count % 21) as usize;
        let wal = match Wal::open_with_config(&wal_path, config.clone()).await {
            Ok(w) => w,
            Err(_) => return,
        };

        for i in 0..count {
            let seq = (i + 1) as u64;
            let op = if i % 2 == 0 {
                WalOp::Put {
                    tx_id: TxId::new(seq),
                    key: format!("k_{i}").into_bytes(),
                    value: format!("v_{i}").into_bytes(),
                }
            } else {
                WalOp::Delete {
                    tx_id: TxId::new(seq),
                    key: format!("k_{i}").into_bytes(),
                }
            };

            let (batch, _) = match wal.prepare_batch(vec![(op, seq)]).await {
                Ok(b) => b,
                Err(_) => return,
            };

            if wal.append_batch(batch).await.is_err() {
                return;
            }
        }

        // 2. File drop / schließen
        drop(wal);

        // 3. Datei mutieren
        let mut file_bytes = match tokio::fs::read(&wal_path).await {
            Ok(b) => b,
            Err(_) => return,
        };

        let file_len = file_bytes.len();
        let offset = if file_len == 0 {
            0
        } else {
            ((input.mutation_offset_pct as usize) * file_len) / 255
        };

        let mut bytes = input.mutation_bytes;
        if bytes.is_empty() {
            bytes.push(0xAB);
        }
        if bytes.len() > 64 {
            bytes.truncate(64);
        }

        match input.mutation_mode {
            MutationMode::Overwrite => {
                if !file_bytes.is_empty() {
                    let start = offset.min(file_bytes.len() - 1);
                    let end = (start + bytes.len()).min(file_bytes.len());
                    let mut_len = end - start;
                    file_bytes[start..end].copy_from_slice(&bytes[..mut_len]);
                }
            }
            MutationMode::Insert => {
                let insert_pos = offset.min(file_bytes.len());
                file_bytes.splice(insert_pos..insert_pos, bytes);
            }
            MutationMode::Truncate => {
                let trunc_pos = offset.min(file_bytes.len());
                file_bytes.truncate(trunc_pos);
            }
            MutationMode::ZeroOut => {
                if !file_bytes.is_empty() {
                    let start = offset.min(file_bytes.len() - 1);
                    let end = (start + bytes.len()).min(file_bytes.len());
                    for b in &mut file_bytes[start..end] {
                        *b = 0x00;
                    }
                }
            }
            MutationMode::AllOnes => {
                if !file_bytes.is_empty() {
                    let start = offset.min(file_bytes.len() - 1);
                    let end = (start + bytes.len()).min(file_bytes.len());
                    for b in &mut file_bytes[start..end] {
                        *b = 0xFF;
                    }
                }
            }
        }

        if tokio::fs::write(&wal_path, &file_bytes).await.is_err() {
            return;
        }

        // 4. Wal erneut öffnen -> kein Panic
        let wal_reopen = match Wal::open_with_config(&wal_path, config).await {
            Ok(w) => w,
            Err(_) => return,
        };

        // 5. scan_entries / replay -> kein Panic
        let _ = wal_reopen.replay().await;
    });
});
