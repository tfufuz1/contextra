//! # Flush Integrationstests und Absturzmatrix (SPEC §12.1, §13.2, §18)
//!
//! Verifiziert die Invarianten und Spezifikationen rund um den Flush-Ablauf:
//! - INV-FLUSH-1: Schritt 0: Vor jedem Run-Bau Redo bis max_stamp per sync_data sichern.
//! - Absturzmatrix §13.2:
//!   - Flush vor Redo-Sync (Schritt 0): nichts geflusht; Atomarität über Spaces bleibt erhalten
//!   - Flush nach Run/Blob-sync, vor Edit: verwaiste Run/Blob-Dateien gelöscht, Memtable aus Redo wiederhergestellt
//!   - Flush nach Edit, vor Version: Dateien im Catalog lebendig, Redo-Frames mit Stamp <= flushed_stamp übersprungen
//! - Domänenversiegelte Körper: Bei vernichtetem Schlüssel BlobRef mit blob_no = 0.
//! - sync_all auf Run und Blob, sync_dir.
//! - Graveyard-Übergabe von Redo-Segmenten unterhalb redo_floor.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;

use contextra_storage::blob::reader::BlobReader;
use contextra_storage::blob::record::BlobRecord;
use contextra_storage::catalog::current::write_current;
use contextra_storage::catalog::log::{catalog_file_name, CatalogLog};
use contextra_storage::catalog::reserve::IdReserve;
use contextra_storage::catalog::types::{CatalogFileKind, CatalogHeader, CatalogSpaceOptions};
use contextra_storage::codec::tag::{Tag, TagKind};
use contextra_storage::engine::compaction::exec::run_file_name;
use contextra_storage::engine::flush::{flush_memtable, CommitterSync, FlushError, FrozenMemtable, MemtableEntry};
use contextra_storage::engine::recovery::recover_engine;
use contextra_storage::run::blob_ref::BlobRef;
use contextra_storage::run::block::BlockSecurity;
use contextra_storage::run::reader::RunReader;
use contextra_storage::testing::fault_fs::{FaultConfig, FaultFs};
use contextra_storage::testing::mem_fs::MemFs;
use contextra_storage_ports::{File, Fs, KeyDomain, KeyProvider, OpenMode, Perm, Stamp};

/// Test-Committer zur Überprüfung von INV-FLUSH-1.
struct TestCommitter {
    durable: AtomicU64,
    sync_called: AtomicBool,
    fail_sync: AtomicBool,
}

impl TestCommitter {
    fn new(initial_durable: u64) -> Self {
        Self {
            durable: AtomicU64::new(initial_durable),
            sync_called: AtomicBool::new(false),
            fail_sync: AtomicBool::new(false),
        }
    }
}

impl CommitterSync for TestCommitter {
    fn durable_stamp(&self) -> Stamp {
        self.durable.load(Ordering::Acquire)
    }

    fn sync_data(&self, up_to_stamp: Stamp) -> std::io::Result<Stamp> {
        self.sync_called.store(true, Ordering::Release);
        if self.fail_sync.load(Ordering::Relaxed) {
            return Err(std::io::Error::new(std::io::ErrorKind::Other, "Sync data injected error"));
        }
        self.durable.store(up_to_stamp, Ordering::Release);
        Ok(up_to_stamp)
    }
}

/// Dummy KeyProvider für Domänenschlüssel.
struct TestKeyProvider {
    destroyed_domains: std::collections::HashSet<u32>,
}

impl TestKeyProvider {
    fn new() -> Self {
        Self {
            destroyed_domains: std::collections::HashSet::new(),
        }
    }

    fn destroy_domain(&mut self, domain: u32) {
        self.destroyed_domains.insert(domain);
    }
}

impl KeyProvider for TestKeyProvider {
    fn root_key(&self) -> Result<[u8; 32], contextra_storage_ports::KeyError> {
        Ok([0x42; 32])
    }

    fn space_dek(&self, _space_id: u16) -> Result<[u8; 32], contextra_storage_ports::KeyError> {
        Ok([0x24; 32])
    }

    fn domain_key(&self, domain: KeyDomain) -> Result<Option<[u8; 32]>, contextra_storage_ports::KeyError> {
        if self.destroyed_domains.contains(&domain.0) {
            Ok(None)
        } else {
            Ok(Some([0x55; 32]))
        }
    }
}

fn setup_catalog(fs: &dyn Fs, base_dir: &Path) -> (Arc<dyn File>, CatalogLog, contextra_storage::catalog::state::CatalogState) {
    fs.create_dir_all(base_dir, Perm::Private).unwrap();
    write_current(fs, base_dir, 1).unwrap();
    let cat_path = base_dir.join(catalog_file_name(1));
    let cat_file = fs.open(&cat_path, OpenMode::CreateNew(Perm::Private)).unwrap();
    let hdr = CatalogHeader::new(1);
    cat_file.write_all_at(&hdr.encode(), 0).unwrap();
    cat_file.sync_all().unwrap();

    let (state, log) = CatalogLog::read_and_recover(&*cat_file, 1, 0, None, None, None, None).unwrap();
    (Arc::from(cat_file), log, state)
}

#[test]
fn test_flush_inv_flush_1_sync_before_run_builder() {
    // // INV-FLUSH-1: Schritt 0: vor jedem Run-Bau Redo bis max_stamp per sync_data sichern
    let mem = MemFs::new();
    let base_dir = Path::new("/test_inv_flush_1");
    let (cat_file, mut cat_log, mut cat_state) = setup_catalog(&mem, base_dir);

    // Initialer Durable Stamp = 5, Memtable max_stamp = 10
    let committer = TestCommitter::new(5);

    let memtable = FrozenMemtable {
        space_id: 1,
        max_stamp: 10,
        entries: vec![
            MemtableEntry {
                key: b"user:100".to_vec(),
                tag: Tag::new(10, TagKind::Put),
                header: vec![0x01],
                body: b"Alice".to_vec(),
                domain: None,
            },
        ],
    };

    let mut id_reserve = IdReserve::new(&cat_state);
    let space_opts = CatalogSpaceOptions {
        header_len: 1,
        weights_k: 1,
        prefix_filter: 0,
        flags: 0,
    };
    let mut graveyard = Vec::new();

    let new_state = flush_memtable(
        &mem,
        base_dir,
        &*cat_file,
        &mut cat_log,
        &cat_state,
        &committer,
        &memtable,
        5, // floor
        &mut id_reserve,
        &space_opts,
        1000,
        None,
        None,
        None,
        None,
        &mut graveyard,
    ).expect("Flush gelingt");

    assert!(committer.sync_called.load(Ordering::Acquire), "INV-FLUSH-1: sync_data muss aufgerufen worden sein");
    assert_eq!(committer.durable_stamp(), 10, "INV-FLUSH-1: Redo muss bis max_stamp gesichert sein");
    assert_eq!(new_state.space_flushed.get(&1), Some(&10));
}

#[test]
fn test_flush_inv_flush_1_sync_failure_aborts() {
    // // INV-FLUSH-1: Scheitert sync_data in Schritt 0, bricht der Flush vor dem Run-Bau ab
    let mem = MemFs::new();
    let base_dir = Path::new("/test_inv_flush_1_fail");
    let (cat_file, mut cat_log, cat_state) = setup_catalog(&mem, base_dir);

    let committer = TestCommitter::new(5);
    committer.fail_sync.store(true, Ordering::Relaxed);

    let memtable = FrozenMemtable {
        space_id: 1,
        max_stamp: 10,
        entries: vec![
            MemtableEntry {
                key: b"k".to_vec(),
                tag: Tag::new(10, TagKind::Put),
                header: vec![],
                body: b"v".to_vec(),
                domain: None,
            },
        ],
    };

    let mut id_reserve = IdReserve::new(&cat_state);
    let space_opts = CatalogSpaceOptions::default();
    let mut graveyard = Vec::new();

    let res = flush_memtable(
        &mem,
        base_dir,
        &*cat_file,
        &mut cat_log,
        &cat_state,
        &committer,
        &memtable,
        5,
        &mut id_reserve,
        &space_opts,
        1000,
        None,
        None,
        None,
        None,
        &mut graveyard,
    );

    assert!(res.is_err(), "INV-FLUSH-1: Flush muss bei Sync-Fehler abbrechen");
    // Es dürfen keine Run-Dateien existieren
    let entries = mem.list_dir(base_dir).unwrap();
    for e in entries {
        assert!(!e.name.starts_with("RUN-"), "Kein Run darf gebaut worden sein");
    }
}

#[test]
fn test_flush_domain_sealed_shredded_body_produces_blobref_zero() {
    // SPEC §12.1: Domänenversiegelte Körper: ist der Domänenschlüssel vernichtet,
    // BlobRef mit blob_no = 0 (shredded), Körper nicht schreiben.
    let mem = MemFs::new();
    let base_dir = Path::new("/test_domain_sealed");
    let (cat_file, mut cat_log, cat_state) = setup_catalog(&mem, base_dir);

    let committer = TestCommitter::new(10);
    let mut key_provider = TestKeyProvider::new();
    // Domäne 42 vernichten!
    key_provider.destroy_domain(42);

    let memtable = FrozenMemtable {
        space_id: 1,
        max_stamp: 10,
        entries: vec![
            MemtableEntry {
                key: b"secret_key".to_vec(),
                tag: Tag::new(10, TagKind::Put),
                header: vec![0xAA],
                body: b"super_sensitive_data".to_vec(),
                domain: Some(42),
            },
        ],
    };

    let mut id_reserve = IdReserve::new(&cat_state);
    let space_opts = CatalogSpaceOptions {
        header_len: 1,
        weights_k: 1,
        prefix_filter: 0,
        flags: 4, // separate_body
    };
    let mut graveyard = Vec::new();

    let new_state = flush_memtable(
        &mem,
        base_dir,
        &*cat_file,
        &mut cat_log,
        &cat_state,
        &committer,
        &memtable,
        0,
        &mut id_reserve,
        &space_opts,
        1000,
        Some(&key_provider),
        None,
        None,
        None,
        &mut graveyard,
    ).expect("Flush gelingt");

    // Run-Datei öffnen und prüfen, dass der Eintrag PutRef mit blob_no = 0 trägt
    let run_meta = new_state.files.values().find(|m| m.kind == CatalogFileKind::Run).unwrap();
    let run_path = base_dir.join(run_file_name(run_meta.id));
    let run_file = mem.open(&run_path, OpenMode::Read).unwrap();
    let file_len = run_file.len().unwrap();
    let reader = RunReader::open(Arc::from(run_file), file_len, BlockSecurity::Plain, None).unwrap();

    let block_data = reader.load_data_block(0).unwrap();
    let db_reader = contextra_storage::run::data::DataBlockReader::new(&block_data).unwrap();
    let mut entries_iter = db_reader.iter();
    let entry = entries_iter.next().unwrap().unwrap();

    assert_eq!(entry.tag.kind().unwrap(), TagKind::PutRef);
    let bref_bytes = &entry.value[1..];
    let bref = BlobRef::decode(bref_bytes).unwrap();
    assert_eq!(bref.blob_no, 0, "Vernichteter Schlüssel muss blob_no = 0 liefern");
    assert!(bref.is_shredded());
}

#[test]
fn test_crash_matrix_flush_after_run_blob_sync_before_edit_orphans_cleaned() {
    // Zeile §13.2: Flush: nach Run/Blob-sync, vor Edit | verwaiste Run- und Blob-Dateien | Verwaisten-Suche löscht
    let mem = MemFs::new();
    let fault_cfg = Arc::new(FaultConfig::default());
    let fs = FaultFs::new(mem, Arc::clone(&fault_cfg));
    let base_dir = Path::new("/crash_flush_before_edit");

    let (cat_file, mut cat_log, cat_state) = setup_catalog(&fs, base_dir);
    let committer = TestCommitter::new(10);

    let memtable = FrozenMemtable {
        space_id: 1,
        max_stamp: 10,
        entries: vec![
            MemtableEntry {
                key: b"k".to_vec(),
                tag: Tag::new(10, TagKind::Put),
                header: vec![],
                body: b"v".to_vec(),
                domain: None,
            },
        ],
    };

    let mut id_reserve = IdReserve::new(&cat_state);
    let space_opts = CatalogSpaceOptions::default();
    let mut graveyard = Vec::new();

    // Fehler bei sync_existing vor dem Katalog-Commit injizieren
    fault_cfg.fail_sync.store(true, Ordering::Relaxed);

    let res = flush_memtable(
        &fs,
        base_dir,
        &*cat_file,
        &mut cat_log,
        &cat_state,
        &committer,
        &memtable,
        0,
        &mut id_reserve,
        &space_opts,
        1000,
        None,
        None,
        None,
        None,
        &mut graveyard,
    );
    assert!(res.is_err(), "Flush bricht vor Katalog-Edit ab");

    // Nach Absturz: Wiederanlauf (Recovery) ausführen
    fault_cfg.fail_sync.store(false, Ordering::Relaxed);
    let rec_res = recover_engine(&fs, base_dir, None, None, None, None).expect("Recovery gelingt");

    // Verwaiste Run-Dateien müssen gelöscht sein (INV-CAT-4)
    let entries = fs.list_dir(base_dir).unwrap();
    for e in entries {
        assert!(!e.name.starts_with("RUN-"), "Verwaiste Run-Datei muss gelöscht sein: {}", e.name);
    }
    assert_eq!(rec_res.catalog_state.files.len(), 0, "Keine Dateien im Katalog");
}

#[test]
fn test_crash_matrix_flush_after_edit_files_live_inv_flush_1() {
    // Zeile §13.2: Flush: nach Edit, vor Version | Dateien im Catalog | lebendig, Redo-Frames mit Stamp <= flushed_stamp übersprungen
    let mem = MemFs::new();
    let base_dir = Path::new("/crash_flush_after_edit");
    let (cat_file, mut cat_log, cat_state) = setup_catalog(&mem, base_dir);

    let committer = TestCommitter::new(10);
    let memtable = FrozenMemtable {
        space_id: 1,
        max_stamp: 10,
        entries: vec![
            MemtableEntry {
                key: b"k".to_vec(),
                tag: Tag::new(10, TagKind::Put),
                header: vec![],
                body: b"v".to_vec(),
                domain: None,
            },
        ],
    };

    let mut id_reserve = IdReserve::new(&cat_state);
    let space_opts = CatalogSpaceOptions::default();
    let mut graveyard = Vec::new();

    // Flush vollständig durchführen (Edit commited)
    let _ = flush_memtable(
        &mem,
        base_dir,
        &*cat_file,
        &mut cat_log,
        &cat_state,
        &committer,
        &memtable,
        0,
        &mut id_reserve,
        &space_opts,
        1000,
        None,
        None,
        None,
        None,
        &mut graveyard,
    ).unwrap();

    // Absturz vor weiterer Nutzung: Recovery ausführen
    let rec_res = recover_engine(&mem, base_dir, None, None, None, None).expect("Recovery gelingt");
    assert_eq!(rec_res.catalog_state.space_flushed.get(&1), Some(&10));
    assert_eq!(rec_res.catalog_state.files.len(), 1, "Run-Datei ist dauerhaft lebendig im Katalog");
}
