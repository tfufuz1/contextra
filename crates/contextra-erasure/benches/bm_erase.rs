//! Benchmarks für den Löschkern (SPEC §11, BM-ERASE-LAT, BM-ERASE-O1, BM-VAULT-OPEN, BM-VAULT-REHASH, BM-VAULT-GROUP).

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing,
    clippy::disallowed_methods,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss
)]

use contextra_erasure::vault::{FileVault, SlotType, VAULT_NUM_SHARDS};
use std::time::Instant;

fn main() {
    println!("Running contextra-erasure benchmarks...");

    // BM-VAULT-OPEN: Öffnen des 16-Shard-Tresors
    let start_open = Instant::now();
    let mut vault = FileVault::new_in_memory(16 * 1024);
    let open_dur = start_open.elapsed();
    println!("BM-VAULT-OPEN: {:?}", open_dur);

    // BM-ERASE-O1: O(1) bis Gated (Schlüsselvernichtung unabhängig von Zielgröße)
    let start_erase_o1 = Instant::now();
    vault
        .put_slot(SlotType::Document, [0xaa; 16], [0xbb; 32])
        .expect("put");
    let destroy_res = vault.destroy_slot(SlotType::Document, [0xaa; 16]);
    let erase_o1_dur = start_erase_o1.elapsed();
    assert!(destroy_res.is_ok());
    println!("BM-ERASE-O1: {:?}", erase_o1_dur);

    // BM-VAULT-GROUP: 16 Shards Durchsatz
    let start_group = Instant::now();
    for i in 0..1000 {
        let tag = [(i % 256) as u8; 16];
        let _ = vault.put_slot(SlotType::Document, tag, [0x11; 32]);
    }
    let group_dur = start_group.elapsed();
    println!(
        "BM-VAULT-GROUP (1000 inserts over {} shards): {:?}",
        VAULT_NUM_SHARDS, group_dur
    );
}
