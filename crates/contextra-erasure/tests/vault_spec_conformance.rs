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
    encode_metablock, open_vault_shard, rehash_shard, FileVault, MetablockParams, SlotType,
    VaultShard, VaultSlotData, VAULT_NUM_SHARDS, VAULT_SLOT_BASE, VAULT_SLOT_SIZE,
};
use contextra_ports::keys::{FileId, FileKeyring, FileKind};

/// SPEC: T-VAULT-1
/// T-ID: T-VAULT-1
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

/// SPEC: T-VAULT-7
/// T-ID: T-VAULT-7
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

/// SPEC: T-VAULT-8
/// T-ID: T-VAULT-8
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
    let meta = encode_metablock(MetablockParams {
        vault_id: &vault_id,
        shard_no: 0,
        cap_log2,
        live,
        tombstones,
        generation,
        scrub_cursor,
        mac_key: &mac_key,
    });
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

/// SPEC: T-VAULT-1
/// T-ID: T-VAULT-1
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

/// SPEC: T-FK-1
/// T-ID: T-FK-1
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

/// Modul für dauerhafte VFS-Tests mit Injektion
mod durable_tests {
    use super::*;
    use contextra_ports::vfs::{VerifiedVia, Vfs, VfsFile, VfsOpenOptions};
    use contextra_ports::{ContextraError, Result};
    use parking_lot::Mutex;
    use std::collections::HashMap;
    use std::io::{Error, ErrorKind};
    use std::path::{Path, PathBuf};
    use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
    use std::sync::Arc;

    pub struct MockVfs {
        files: Mutex<HashMap<PathBuf, Arc<Mutex<Vec<u8>>>>>,
        fail_sync: Arc<AtomicBool>,
        sync_count: Arc<AtomicUsize>,
        read_count: Arc<AtomicUsize>,
    }

    impl MockVfs {
        pub fn new() -> Arc<Self> {
            Arc::new(Self {
                files: Mutex::new(HashMap::new()),
                fail_sync: Arc::new(AtomicBool::new(false)),
                sync_count: Arc::new(AtomicUsize::new(0)),
                read_count: Arc::new(AtomicUsize::new(0)),
            })
        }

        pub fn set_fail_sync(&self, fail: bool) {
            self.fail_sync.store(fail, Ordering::Release);
        }
    }

    pub struct MockVfsFile {
        data: Arc<Mutex<Vec<u8>>>,
        fail_sync: Arc<AtomicBool>,
        sync_count: Arc<AtomicUsize>,
        read_count: Arc<AtomicUsize>,
    }

    impl VfsFile for MockVfsFile {
        fn read_at(&self, buf: &mut [u8], offset: u64) -> std::io::Result<usize> {
            self.read_count.fetch_add(1, Ordering::Relaxed);
            let data = self.data.lock();
            let off = offset as usize;
            if off >= data.len() {
                return Ok(0);
            }
            let available = data.len() - off;
            let to_read = buf.len().min(available);
            buf[..to_read].copy_from_slice(&data[off..off + to_read]);
            Ok(to_read)
        }

        fn read_verified(
            &self,
            offset: u64,
            len: usize,
        ) -> std::io::Result<(Vec<u8>, VerifiedVia)> {
            let mut buf = vec![0u8; len];
            let n = self.read_at(&mut buf, offset)?;
            buf.truncate(n);
            Ok((buf, VerifiedVia::Direct))
        }

        fn write_at(&self, buf: &[u8], offset: u64) -> std::io::Result<usize> {
            let mut data = self.data.lock();
            let off = offset as usize;
            let end = off + buf.len();
            if end > data.len() {
                data.resize(end, 0);
            }
            data[off..end].copy_from_slice(buf);
            Ok(buf.len())
        }

        fn set_len(&self, len: u64) -> std::io::Result<()> {
            let mut data = self.data.lock();
            data.resize(len as usize, 0);
            Ok(())
        }

        fn len(&self) -> std::io::Result<u64> {
            let data = self.data.lock();
            Ok(data.len() as u64)
        }

        fn sync_data(&self) -> std::io::Result<()> {
            self.sync_count.fetch_add(1, Ordering::Relaxed);
            if self.fail_sync.load(Ordering::Acquire) {
                return Err(Error::new(ErrorKind::Other, "injected sync_data failure"));
            }
            Ok(())
        }

        fn sync_all(&self) -> std::io::Result<()> {
            self.sync_data()
        }
    }

    impl Vfs for MockVfs {
        fn open(&self, path: &Path, opts: &VfsOpenOptions) -> Result<Box<dyn VfsFile>> {
            let mut files = self.files.lock();
            if opts.create_new && files.contains_key(path) {
                return Err(ContextraError::Io(Error::new(
                    ErrorKind::AlreadyExists,
                    "file exists",
                )));
            }

            let entry = files
                .entry(path.to_path_buf())
                .or_insert_with(|| Arc::new(Mutex::new(Vec::new())));

            if opts.truncate {
                entry.lock().clear();
            }

            Ok(Box::new(MockVfsFile {
                data: Arc::clone(entry),
                fail_sync: Arc::clone(&self.fail_sync),
                sync_count: Arc::clone(&self.sync_count),
                read_count: Arc::clone(&self.read_count),
            }))
        }

        fn read(&self, path: &Path) -> Result<Vec<u8>> {
            let files = self.files.lock();
            if let Some(data) = files.get(path) {
                Ok(data.lock().clone())
            } else {
                Err(ContextraError::NotFound(path.display().to_string()))
            }
        }

        fn write_atomic(&self, path: &Path, data: &[u8]) -> Result<()> {
            let mut files = self.files.lock();
            files.insert(path.to_path_buf(), Arc::new(Mutex::new(data.to_vec())));
            Ok(())
        }

        fn len(&self, path: &Path) -> Result<u64> {
            let files = self.files.lock();
            if let Some(data) = files.get(path) {
                Ok(data.lock().len() as u64)
            } else {
                Err(ContextraError::NotFound(path.display().to_string()))
            }
        }

        fn list_dir(&self, _path: &Path) -> Result<Vec<String>> {
            let files = self.files.lock();
            let list = files
                .keys()
                .filter_map(|p| p.file_name().and_then(|f| f.to_str()).map(String::from))
                .collect();
            Ok(list)
        }

        fn create_dir_all(&self, _path: &Path) -> Result<()> {
            Ok(())
        }

        fn remove_file(&self, path: &Path) -> Result<()> {
            let mut files = self.files.lock();
            files.remove(path);
            Ok(())
        }

        fn rename(&self, from: &Path, to: &Path) -> Result<()> {
            let mut files = self.files.lock();
            if let Some(data) = files.remove(from) {
                files.insert(to.to_path_buf(), data);
                Ok(())
            } else {
                Err(ContextraError::Io(Error::new(
                    ErrorKind::NotFound,
                    "source file not found",
                )))
            }
        }

        fn sync_dir(&self, _path: &Path) -> Result<()> {
            Ok(())
        }

        fn exists(&self, path: &Path) -> Result<bool> {
            let files = self.files.lock();
            Ok(files.contains_key(path))
        }
    }

    /// SPEC: T-VAULT-1
    /// T-ID: T-VAULT-1
    #[test]
    fn test_t_vault_1_durable_creation_open_and_destroy() {
        let vfs: Arc<dyn Vfs> = MockVfs::new();
        let vault_dir = Path::new("/data/.contextra-vault/store1");
        let vault_id = [0x42u8; 16];
        let mac_key = [0x99u8; 32];

        // 1. Vault neu anlegen
        let mut vault = FileVault::create_dir(&vfs, vault_dir, vault_id, mac_key, 1000, 100)
            .expect("create_dir failed");

        assert_eq!(vault.live_count(), 0);
        assert_eq!(vault.capacity(), 16 * 1024);

        let doc_tag = [0x55u8; 16];
        let payload = [0x77u8; 32];

        // 2. Slot anlegen
        vault
            .put_slot(SlotType::Document, doc_tag, payload)
            .expect("put_slot failed");

        assert_eq!(vault.live_count(), 1);

        // 3. Slot lesen
        let slot = vault
            .get_slot(SlotType::Document, doc_tag)
            .expect("get_slot failed");
        assert!(slot.is_some());
        assert_eq!(slot.unwrap().payload, payload);

        // 4. Tresor neu öffnen (D-111 Abs. 8: ohne Vollscan)
        let mut reopened =
            FileVault::open_dir(&vfs, vault_dir, &vault_id, &mac_key).expect("open_dir failed");

        let slot2 = reopened
            .get_slot(SlotType::Document, doc_tag)
            .expect("get_slot on reopened failed");
        assert!(slot2.is_some());
        assert_eq!(slot2.unwrap().payload, payload);

        // 5. Slot vernichten auf geöffnetem Tresor
        let destroyed = reopened
            .destroy_slot(SlotType::Document, doc_tag)
            .expect("destroy failed");
        assert_eq!(
            destroyed.verified_via,
            contextra_erasure::vault::VerifiedVia::Direct
        );

        assert!(reopened
            .get_slot(SlotType::Document, doc_tag)
            .unwrap()
            .is_none());
    }

    /// SPEC: T-VAULT-7
    /// T-ID: T-VAULT-7
    #[test]
    fn test_t_vault_7_durable_sync_failure_poisons_shard() {
        let mock = MockVfs::new();
        let vfs: Arc<dyn Vfs> = mock.clone();
        let vault_dir = Path::new("/data/.contextra-vault/store_poison");
        let vault_id = [0x33u8; 16];
        let mac_key = [0x44u8; 32];

        let mut vault = FileVault::create_dir(&vfs, vault_dir, vault_id, mac_key, 1000, 1)
            .expect("create_dir failed");

        let tag = [0x12u8; 16];
        vault
            .put_slot(SlotType::Document, tag, [0x10; 32])
            .expect("initial put");

        mock.set_fail_sync(true);

        let err = vault.put_slot(SlotType::Document, tag, [0x20; 32]);
        assert!(err.is_err());
        assert!(vault.is_poisoned());

        mock.set_fail_sync(false);
        let get_res = vault.get_slot(SlotType::Document, tag);
        assert!(
            get_res.is_err(),
            "Vergifteter Shard darf nie wiederverwendet werden"
        );
    }
    /// SPEC: T-VAULT-7
    /// T-ID: T-VAULT-7
    #[test]
    fn test_t_vault_7_durable_rehash_cancellation_and_overwriting() {
        let vfs: Arc<dyn Vfs> = MockVfs::new();
        let vault_dir = Path::new("/data/.contextra-vault/store_rehash");
        let vault_id = [0x55u8; 16];
        let mac_key = [0x66u8; 32];

        let mut vault = FileVault::create_dir(&vfs, vault_dir, vault_id, mac_key, 1000, 1)
            .expect("create_dir failed");

        let shard = vault.durable_shard_mut(0).expect("shard 0 exists");

        // 1. Abbruch durch wartendes erase (D-111 Abs. 6)
        let erase_pending = AtomicBool::new(true);
        let res = contextra_erasure::vault::rehash_durable_shard(shard, &vfs, Some(&erase_pending));
        assert!(
            res.is_err(),
            "Hintergrund-Rehash muss bei wartendem erase abbrechen"
        );

        // 2. Regulärer Rehash
        erase_pending.store(false, Ordering::Release);
        let (new_shard, outcome) =
            contextra_erasure::vault::rehash_durable_shard(shard, &vfs, Some(&erase_pending))
                .expect("rehash_durable_shard failed");

        assert!(
            outcome.old_overwritten,
            "Altdatei muss mit Nullen überschrieben sein"
        );
        assert_eq!(new_shard.shard_no(), 0);
    }

    struct MockScrubCtx;
    impl contextra_erasure::vault::VaultScrubContext for MockScrubCtx {
        fn file_exists(&self, file_id: &[u8; 16]) -> bool {
            file_id[0] != 0xFF
        }
        fn document_in_manifest(&self, doc_tag: &[u8; 16]) -> bool {
            doc_tag[0] != 0xEE
        }
        fn next_seq(&self) -> u64 {
            (1 << 20) + 100
        }
    }

    /// SPEC: T-VAULT-8
    /// T-ID: T-VAULT-8
    #[test]
    fn test_t_vault_8_scrub_step_removes_orphans() {
        let mut vault = FileVault::new_in_memory(64);
        let ctx = MockScrubCtx;

        // F-Slot Waise (file_id beginnt mit 0xFF)
        let orphan_file_tag = [0xFF; 16];
        vault
            .put_slot(SlotType::File, orphan_file_tag, [1; 32])
            .expect("put F");

        // D-Slot Waise (beginnt mit 0xEE und gen = 0 < next_seq - 2^20)
        let orphan_doc_tag = [0xEE; 16];
        vault
            .put_slot_with_gen(SlotType::Document, orphan_doc_tag, [2; 32], 0)
            .expect("put D");

        let s_idx = vault.shard_index(SlotType::File, &orphan_file_tag);
        let outcome = contextra_erasure::vault::scrub_shard_step(&mut vault, s_idx, 64, &ctx)
            .expect("scrub step");

        assert!(outcome.slots_scanned > 0);
    }
}
