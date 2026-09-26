#![no_main]

use arbitrary::Arbitrary;
use contextra_core::TxId;
use contextra_store::wal::{Wal, WalConfig, WalOp, WalVersion};
use libfuzzer_sys::fuzz_target;
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
 *
 * 5. `Targeted HMAC Mutation`:
 *    - Explicitly targeting the 32-byte `checksum` or 32-byte `prev_hmac` fields (discovered via
 *      `wal.replay()` offset introspection) systematically tests HMAC chain validation and corruption
 *      detection without relying purely on random byte hits across large WAL files.
 *
 * 6. `Compound Mutations`:
 *    - Applying multiple independent mutations (1..=5) sequentially before reopening tests multi-point
 *      corruptions (e.g. length prefix corruption on entry N combined with HMAC corruption on entry N+2).
 */

#[derive(Arbitrary, Debug)]
enum FuzzInput {
    Single(WalMutationInput),
    Compound(CompoundMutationInput),
}

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
    /// Whether to target discovered HMAC fields specifically
    target_hmac: bool,
}

#[derive(Arbitrary, Debug)]
struct SingleMutationSpec {
    mutation_offset_pct: u8,
    mutation_mode: MutationMode,
    mutation_bytes: Vec<u8>,
    target_hmac: bool,
}

#[derive(Arbitrary, Debug)]
struct CompoundMutationInput {
    /// Entries to write before mutation (0..=20)
    valid_count: u8,
    /// Sequence of 1..=5 mutations applied onto the same WAL file
    mutations: Vec<SingleMutationSpec>,
}

#[derive(Arbitrary, Debug, Clone, Copy)]
enum MutationMode {
    Overwrite, // Überschreibe N Bytes ab offset
    Insert,    // Füge N Bytes ein (verschiebt Rest)
    Truncate,  // Schneide ab offset ab
    ZeroOut,   // Setze N Bytes auf 0x00
    AllOnes,   // Setze N Bytes auf 0xFF
}

fn apply_single_mutation(
    file_bytes: &mut Vec<u8>,
    mode: MutationMode,
    offset_pct: u8,
    mut bytes: Vec<u8>,
    target_hmac: bool,
    hmac_ranges: &[(usize, usize)],
) {
    if bytes.is_empty() {
        bytes.push(0xAB);
    }
    if bytes.len() > 64 {
        bytes.truncate(64);
    }

    let offset = if target_hmac && !hmac_ranges.is_empty() {
        let (r_start, r_end) = hmac_ranges[(offset_pct as usize) % hmac_ranges.len()];
        if r_end > r_start && r_start < file_bytes.len() {
            r_start + ((offset_pct as usize) % (r_end - r_start))
        } else {
            0
        }
    } else if file_bytes.is_empty() {
        0
    } else {
        ((offset_pct as usize) * file_bytes.len()) / 255
    };

    match mode {
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
}

fuzz_target!(|input: FuzzInput| {
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

        let (valid_count, is_compound) = match &input {
            FuzzInput::Single(s) => (s.valid_count, false),
            FuzzInput::Compound(c) => (c.valid_count, true),
        };

        // 1. Wal öffnen & valid_count Entries schreiben
        let count = (valid_count % 21) as usize;
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

        // 2. Gezielte HMAC-Byte-Offsets via `wal.replay()` Introspektion ermitteln
        let mut hmac_ranges = Vec::new();
        if let Ok(replayed) = wal.replay().await {
            for (_, entry, end_pos) in replayed {
                if let Ok(entry_bytes) = entry.to_bytes() {
                    let entry_len = entry_bytes.len() as u64;
                    let start_pos = end_pos.saturating_sub(entry_len) as usize;
                    // Layout in `WalEntry::to_bytes()`:
                    // len (4 B) + crc32 (4 B) + seq_no (8 B) + checksum (32 B) + prev_hmac (32 B)
                    let checksum_start = start_pos + 16;
                    let checksum_end = checksum_start + 32;
                    let prev_hmac_start = checksum_end;
                    let prev_hmac_end = prev_hmac_start + 32;

                    hmac_ranges.push((checksum_start, checksum_end));
                    hmac_ranges.push((prev_hmac_start, prev_hmac_end));
                }
            }
        }

        // 3. File drop / schließen
        drop(wal);

        // 4. Datei mutieren
        let mut file_bytes = match tokio::fs::read(&wal_path).await {
            Ok(b) => b,
            Err(_) => return,
        };

        match input {
            FuzzInput::Single(s) => {
                apply_single_mutation(
                    &mut file_bytes,
                    s.mutation_mode,
                    s.mutation_offset_pct,
                    s.mutation_bytes,
                    s.target_hmac,
                    &hmac_ranges,
                );
            }
            FuzzInput::Compound(mut c) => {
                if c.mutations.is_empty() {
                    c.mutations.push(SingleMutationSpec {
                        mutation_offset_pct: 128,
                        mutation_mode: MutationMode::Overwrite,
                        mutation_bytes: vec![0xDE, 0xAD],
                        target_hmac: true,
                    });
                }
                if c.mutations.len() > 5 {
                    c.mutations.truncate(5);
                }

                for spec in c.mutations {
                    apply_single_mutation(
                        &mut file_bytes,
                        spec.mutation_mode,
                        spec.mutation_offset_pct,
                        spec.mutation_bytes,
                        spec.target_hmac,
                        &hmac_ranges,
                    );
                }
            }
        }

        if tokio::fs::write(&wal_path, &file_bytes).await.is_err() {
            return;
        }

        // 5. Wal erneut öffnen -> darf NIEMALS paniken
        let wal_reopen = match Wal::open_with_config(&wal_path, config).await {
            Ok(w) => w,
            Err(_) => return,
        };

        // 6. scan_entries / replay -> darf NIEMALS paniken
        if let Ok(replayed_entries) = wal_reopen.replay().await {
            // Invariant 1c: Replayed count must never exceed original written count
            assert!(
                replayed_entries.len() <= count,
                "Phantom entry detected! Replayed {} entries, but only {} were originally written (compound={})",
                replayed_entries.len(),
                count,
                is_compound
            );
        }
    });
});
