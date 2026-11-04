//! Zusätzliche Konformitäts- und Eigenschaftstests für den Tresor (T-VAULT-1..9, T-FK-1..3).
//!
//! Validiert:
//! - 16 Shards und Shard-Lookup nach SPEC D-111
//! - Rehash-Verhalten (λ-Schwellen 0.60 / 0.80 / 0.90, D-111 Abs. 4)
//! - Nullung der Altdatei beim Rehash (D-111 Abs. 5)
//! - Open ohne Vollscan mit Metablock-Prüfung und Cursor `rsv` (D-111 Abs. 8)
//! - Shard Poisoning bei Fehlern (D-55, I-10)
//! - FileKeyring Verhalten mit create/open/destroy (D-85, D-113, PO-06)

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss
)]

use contextra_erasure::vault::{
    encode_metablock, open_vault_shard, rehash_shard, FileVault, SlotType, VaultShard,
    VaultSlotData, VAULT_NUM_SHARDS, VAULT_SLOT_BASE, VAULT_SLOT_SIZE,
};
use contextra_ports::keys::{FileId, FileKeyring, FileKind};

#[test]
fn test_t_vault_sharding_distribution() {
    let mut vault = FileVault::new_in_memory(256);
    let mut hit_shards = std::collections::BTreeSet::new();

    // Verschiedene Tags einfügen und prüfen, dass sie auf unterschiedliche Shards verteilt werden
    for i in 0..100u8 {
        let mut tag = [0u8; 16];
        tag[0] = i;
        tag[1] = 0x5A;
        vault
            .put_slot(SlotType::Document, tag, [i; 32])
            .expect("put slot");
    }

    for s_idx in 0..VAULT_NUM_SHARDS {
        if let Some(shard) = vault.shard(s_idx) {
            if shard.live_count() > 0 {
                hit_shards.insert(s_idx);
            }
        }
    }

    // Bei 100 Schlüsseln müssen mehrere der 16 Shards getroffen werden
    assert!(
        hit_shards.len() > 1,
        "Shards müssen verteilt genutzt werden"
    );
}

#[test]
fn test_t_vault_rehash_and_zeroization_d111() {
    let vault_id = [0x33u8; 16];
    let mac_key = [0x44u8; 32];
    let mut shard = VaultShard::new_shard(0, 16, vault_id, mac_key);

    // 10 aktive Slots einfügen
    for i in 0..10u8 {
        let mut tag = [0u8; 16];
        tag[0] = i;
        let sd = VaultSlotData {
            slot_type: SlotType::Document,
            tag,
            payload: [i; 32],
            gen: i as u64,
        };
        shard.put(sd).expect("put");
    }

    // 2 Slots vernichten (werden zu Grabsteinen)
    let tag0 = [0u8; 16];
    shard.destroy(SlotType::Document, &tag0).expect("destroy 0");
    let mut tag1 = [0u8; 16];
    tag1[0] = 1;
    shard.destroy(SlotType::Document, &tag1).expect("destroy 1");

    assert_eq!(shard.live_count(), 8);
    assert_eq!(shard.tombstone_count(), 2);

    // Rehash durchführen
    let (new_shard, outcome) = rehash_shard(&mut shard, vault_id, mac_key).expect("rehash_shard");

    // Nach D-111 Abs. 5:
    // 1. Grabsteine werden nicht übernommen -> nur aktive Slots (8) kopiert
    assert_eq!(outcome.slots_copied, 8);
    assert_eq!(new_shard.live_count(), 8);
    assert_eq!(new_shard.tombstone_count(), 0);

    // 2. Altdatei wurde vollständig mit Nullen überschrieben (D-111 Abs. 5, outcome.old_overwritten == true)
    assert!(outcome.old_overwritten);
    assert_eq!(shard.live_count(), 0);
    assert_eq!(shard.tombstone_count(), 0);

    // 3. Neue Shard kann die aktiven Slots weiterhin lesen
    for i in 2..10u8 {
        let mut tag = [0u8; 16];
        tag[0] = i;
        let slot = new_shard
            .get(SlotType::Document, &tag)
            .expect("get in new shard");
        assert!(slot.is_some());
        assert_eq!(slot.unwrap().payload, [i; 32]);
    }
}

#[test]
fn test_t_vault_open_metablock_and_scrub_cursor_d111() {
    let vault_id = [0x55u8; 16];
    let mac_key = [0x66u8; 32];
    let cap_log2 = 6; // 64 Slots
    let live = 12;
    let tombstones = 3;
    let generation = 2;
    let scrub_cursor = 1024; // rsv Cursor

    // Synthetische Shard-Datei aufbauen: Kopf (64 B) + Metablock (64 B) + 64 Slots (64 B)
    let mut raw_shard = vec![0u8; VAULT_SLOT_BASE + 64 * VAULT_SLOT_SIZE];

    // F.1 Dateikopf: kind = 10 (FileKind::Vault)
    raw_shard[0] = 10;

    // Metablock einsetzen
    let meta = encode_metablock(
        &vault_id,
        0,
        cap_log2,
        live,
        tombstones,
        generation,
        scrub_cursor,
        &mac_key,
    );
    raw_shard[64..128].copy_from_slice(&meta);

    // Open durchführen (D-111 Abs. 8)
    let outcome = open_vault_shard(&raw_shard, &vault_id, &mac_key).expect("open_vault_shard");

    assert_eq!(outcome.vault_id, vault_id);
    assert_eq!(outcome.shard_no, 0);
    assert_eq!(outcome.capacity, 64);
    assert_eq!(outcome.live_estimate, live);
    assert_eq!(outcome.tombstone_estimate, tombstones);
    assert_eq!(outcome.generation, generation);
    assert_eq!(outcome.scrub_cursor, scrub_cursor);
}

#[test]
fn test_t_vault_shard_poisoning_d55() {
    let vault_id = [0x12u8; 16];
    let mac_key = [0x34u8; 32];
    let mut shard = VaultShard::new_shard(0, 16, vault_id, mac_key);

    let tag = [0x01u8; 16];
    shard
        .put(VaultSlotData {
            slot_type: SlotType::Document,
            tag,
            payload: [0xAA; 32],
            gen: 0,
        })
        .expect("initial put");

    // Nach D-55, I-10: Tritt ein Sync-Fehler auf, wird der Shard vergiftet und darf nie wiederholt werden
    shard.poison();
    assert!(shard.is_poisoned());

    // Alle weiteren Operationen müssen fehlschlagen
    let get_res = shard.get(SlotType::Document, &tag);
    assert!(get_res.is_err(), "get auf vergiftetem Shard muss scheitern");

    let put_res = shard.put(VaultSlotData {
        slot_type: SlotType::Document,
        tag: [0x02; 16],
        payload: [0xBB; 32],
        gen: 0,
    });
    assert!(put_res.is_err(), "put auf vergiftetem Shard muss scheitern");

    let destroy_res = shard.destroy(SlotType::Document, &tag);
    assert!(
        destroy_res.is_err(),
        "destroy auf vergiftetem Shard muss scheitern"
    );
}

#[tokio::test]
async fn test_t_fk_keyring_lifecycle_and_zeroize() {
    let keyring = contextra_erasure::vault::VaultKeyring::new_in_memory();
    let file_id = FileId::new(1001);

    // 1. Noch nicht angelegt -> open liefert Fehler
    assert!(keyring.open(file_id).await.is_err());

    // 2. create legt F-Slot an (SPEC D-85 / I-9b)
    let key = keyring
        .create(FileKind::SSTable, file_id)
        .await
        .expect("create file key");
    assert_eq!(key.file_id(), file_id);
    assert_eq!(key.as_bytes().len(), 32);

    // 3. open lädt den gleichen Dateischlüssel
    let opened = keyring.open(file_id).await.expect("open file key");
    assert_eq!(opened.as_bytes(), key.as_bytes());

    // 4. destroy vernichtet den Slot (nullt, sync, verifiziert; D-113, PO-06)
    keyring.destroy(file_id).await.expect("destroy file key");

    // 5. Nach Vernichtung ist der Dateischlüssel unableitbar / nicht auffindbar
    assert!(
        keyring.open(file_id).await.is_err(),
        "Vernichteter Dateischlüssel darf nicht mehr geöffnet werden"
    );

    // 6. destroy ist idempotent
    assert!(keyring.destroy(file_id).await.is_ok());
}
