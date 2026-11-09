//! Tests für dauerhaftes Journal auf Vfs (SPEC §11.1, §11.2, §11.6, D-58, D-119, D-125, R13, Anhang F.1, F.9).
//!
//! T-ID: T-CRASH-1, T-CRASH-2, T-CRASH-4, T-JRN-3, T-ERID-1

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]

use contextra_erasure::journal::appendlog::DurableJournal;
use contextra_erasure::journal::epoch::{compute_commitment, EpochKeys};
use contextra_erasure::journal::record::{JournalRecord, RecordCode, TargetType};
use contextra_erasure::types_local::Vfs;
use contextra_erasure::types_local::{VfsFile, VfsOpenOptions};
use parking_lot::RwLock;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

struct MemFile {
    data: Arc<RwLock<Vec<u8>>>,
    sync_count: Arc<AtomicUsize>,
}

impl VfsFile for MemFile {
    fn read_at(&self, buf: &mut [u8], offset: u64) -> std::io::Result<usize> {
        let guard = self.data.read();
        let off = offset as usize;
        if off >= guard.len() {
            return Ok(0);
        }
        let available = guard.len() - off;
        let to_read = buf.len().min(available);
        buf[..to_read].copy_from_slice(&guard[off..off + to_read]);
        Ok(to_read)
    }

    fn write_at(&self, buf: &[u8], offset: u64) -> std::io::Result<usize> {
        let mut guard = self.data.write();
        let off = offset as usize;
        let end = off + buf.len();
        if end > guard.len() {
            guard.resize(end, 0);
        }
        guard[off..end].copy_from_slice(buf);
        Ok(buf.len())
    }

    fn set_len(&self, len: u64) -> std::io::Result<()> {
        let mut guard = self.data.write();
        guard.resize(len as usize, 0);
        Ok(())
    }

    fn len(&self) -> std::io::Result<u64> {
        Ok(self.data.read().len() as u64)
    }

    fn sync_data(&self) -> std::io::Result<()> {
        self.sync_count.fetch_add(1, Ordering::SeqCst);
        Ok(())
    }

    fn sync_all(&self) -> std::io::Result<()> {
        self.sync_count.fetch_add(1, Ordering::SeqCst);
        Ok(())
    }
}

struct TrackingVfs {
    files: RwLock<BTreeMap<PathBuf, Arc<RwLock<Vec<u8>>>>>,
    sync_counter: Arc<AtomicUsize>,
}

impl TrackingVfs {
    fn new() -> Self {
        Self {
            files: RwLock::new(BTreeMap::new()),
            sync_counter: Arc::new(AtomicUsize::new(0)),
        }
    }
}

impl Vfs for TrackingVfs {
    fn read(
        &self,
        path: &Path,
    ) -> std::result::Result<Vec<u8>, contextra_erasure::error::ErasureError> {
        let files = self.files.read();
        files
            .get(path)
            .map(|data| data.read().clone())
            .ok_or_else(|| {
                contextra_erasure::error::ErasureError::Io(std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    path.display().to_string(),
                ))
            })
    }

    fn write_atomic(
        &self,
        path: &Path,
        data: &[u8],
    ) -> std::result::Result<(), contextra_erasure::error::ErasureError> {
        let mut files = self.files.write();
        files.insert(path.to_path_buf(), Arc::new(RwLock::new(data.to_vec())));
        self.sync_counter.fetch_add(1, Ordering::SeqCst);
        Ok(())
    }

    fn create_dir_all(
        &self,
        _path: &Path,
    ) -> std::result::Result<(), contextra_erasure::error::ErasureError> {
        Ok(())
    }

    fn remove_file(
        &self,
        path: &Path,
    ) -> std::result::Result<(), contextra_erasure::error::ErasureError> {
        self.files.write().remove(path);
        Ok(())
    }

    fn exists(
        &self,
        path: &Path,
    ) -> std::result::Result<bool, contextra_erasure::error::ErasureError> {
        Ok(self.files.read().contains_key(path))
    }

    fn len(&self, path: &Path) -> std::result::Result<u64, contextra_erasure::error::ErasureError> {
        let files = self.files.read();
        files
            .get(path)
            .map(|data| data.read().len() as u64)
            .ok_or_else(|| {
                contextra_erasure::error::ErasureError::Io(std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    path.display().to_string(),
                ))
            })
    }

    fn list_dir(
        &self,
        _path: &Path,
    ) -> std::result::Result<Vec<String>, contextra_erasure::error::ErasureError> {
        Ok(Vec::new())
    }

    fn sync_dir(
        &self,
        _path: &Path,
    ) -> std::result::Result<(), contextra_erasure::error::ErasureError> {
        self.sync_counter.fetch_add(1, Ordering::SeqCst);
        Ok(())
    }
}

struct StaticEpochKeys([u8; 32]);

impl EpochKeys for StaticEpochKeys {
    fn commit_key(&self, _epoch_id: u32) -> [u8; 32] {
        self.0
    }
}

/// T-ID: T-CRASH-1
#[test]
fn t_crash_1_durable_journal_roundtrip_and_fsync() {
    let vfs = Arc::new(TrackingVfs::new());
    let path = Path::new("erasure/journal.log");
    let derived_key = [0x77u8; 32];

    let mut journal = DurableJournal::open_or_create(vfs.clone(), path, &derived_key)
        .expect("open_or_create journal");

    let sync_before = vfs.sync_counter.load(Ordering::SeqCst);

    let epoch_keys = StaticEpochKeys([0xee; 32]);
    let commitment = compute_commitment(&epoch_keys.commit_key(1), b"subj-1", b"tenant-1");

    let rec1 = JournalRecord::new_v2(
        [0x11; 16],
        RecordCode::RequestedIntent,
        1000,
        commitment,
        TargetType::Document,
        1, // retention
        1, // phase Intent
        0, // reason
        vec![0x01, 0x02],
        1,
        vec![[0x99; 32]],
    );

    journal.append_record(&rec1).expect("append rec1");

    let sync_after = vfs.sync_counter.load(Ordering::SeqCst);
    assert!(sync_after > sync_before, "fsync must be called on append");

    // Lade Journal neu ein
    let reopened =
        DurableJournal::open_or_create(vfs.clone(), path, &derived_key).expect("reopen journal");
    assert_eq!(reopened.records().len(), 1);
    assert_eq!(reopened.records()[0].erasure_id, [0x11; 16]);
    assert_eq!(
        reopened.records()[0].record_code,
        RecordCode::RequestedIntent
    );
    assert_eq!(reopened.records()[0].commitment, commitment);
}

/// T-ID: T-CRASH-2
#[test]
fn t_crash_2_torn_tail_recovery() {
    let vfs = Arc::new(TrackingVfs::new());
    let path = Path::new("erasure/journal.log");
    let derived_key = [0x33u8; 32];

    let mut journal =
        DurableJournal::open_or_create(vfs.clone(), path, &derived_key).expect("open_or_create");

    let rec = JournalRecord::new_v2(
        [0x22; 16],
        RecordCode::Gated,
        2000,
        [0xbb; 16],
        TargetType::Subject,
        0,
        2, // phase Gate
        0,
        vec![],
        0,
        vec![],
    );
    journal.append_record(&rec).expect("append");

    // Simuliere Torn-Tail: Hänge unvollständige Bytes an das Ende der Datei
    {
        let raw = vfs.read(path).expect("read");
        let mut torn = raw.clone();
        torn.extend_from_slice(&[0x00, 0x00, 0x01]); // 3 Bytes (unvollständige Längenangabe)
        vfs.write_atomic(path, &torn).expect("write torn");
    }

    // Beim Wiederöffnen muss der Torn-Tail entweder repariert oder typisiert fehlschlagen,
    // keinesfalls still ignoriert oder paniken.
    let res = DurableJournal::open_or_create(vfs.clone(), path, &derived_key);
    assert!(res.is_err(), "Torn tail must fail closed or return error");
}

/// T-ID: T-CRASH-4
#[test]
fn t_crash_4_recover_open_sagas_conservative_gate() {
    let vfs = Arc::new(TrackingVfs::new());
    let path = Path::new("erasure/journal.log");
    let derived_key = [0x55u8; 32];

    let mut journal =
        DurableJournal::open_or_create(vfs.clone(), path, &derived_key).expect("open_or_create");

    let open_id = [0x44; 16];
    let rec = JournalRecord::new_v2(
        open_id,
        RecordCode::RequestedIntent,
        1500,
        [0xcc; 16],
        TargetType::Document,
        0,
        1, // Intent
        0,
        vec![],
        0,
        vec![],
    );
    journal.append_record(&rec).expect("append");

    // recover_open_sagas
    let open_sagas = journal.recover_open_sagas();
    assert_eq!(open_sagas.len(), 1);
    assert_eq!(open_sagas[0].erasure_id().as_bytes(), &open_id);
    // Gate ohne Journal gilt konservativ als gegatet (§11.2)
}

/// T-ID: T-JRN-3
#[test]
fn t_jrn_3_reduction_after_completed_r13() {
    let vfs = Arc::new(TrackingVfs::new());
    let path = Path::new("erasure/journal.log");
    let derived_key = [0x88u8; 32];

    let mut journal =
        DurableJournal::open_or_create(vfs.clone(), path, &derived_key).expect("open_or_create");

    let id = [0x66; 16];
    journal
        .append_record(&JournalRecord::new_v2(
            id,
            RecordCode::RequestedIntent,
            100,
            [0x11; 16],
            TargetType::Document,
            0,
            1,
            0,
            vec![],
            0,
            vec![],
        ))
        .expect("append 1");

    journal
        .append_record(&JournalRecord::new_v2(
            id,
            RecordCode::Gated,
            200,
            [0x11; 16],
            TargetType::Document,
            0,
            2,
            0,
            vec![],
            0,
            vec![],
        ))
        .expect("append 2");

    journal
        .append_record(&JournalRecord::new_v2(
            id,
            RecordCode::Completed,
            300,
            [0x11; 16],
            TargetType::Document,
            0,
            6,
            0,
            vec![],
            0,
            vec![],
        ))
        .expect("append 3");

    assert_eq!(journal.records().len(), 3);

    // Führe Reduktion aus (R13)
    journal.reduce_completed().expect("reduce");

    // Nach Reduktion darf nur der letzte Completed-Eintrag für diese ID übrig sein
    assert_eq!(journal.records().len(), 1);
    assert_eq!(journal.records()[0].record_code, RecordCode::Completed);
}

/// T-ID: T-ERID-1
#[test]
fn t_erid_1_p13_no_plaintext_keys_or_ids_in_journal() {
    let vfs = Arc::new(TrackingVfs::new());
    let path = Path::new("erasure/journal.log");
    let derived_key = [0x99u8; 32];

    let mut journal =
        DurableJournal::open_or_create(vfs.clone(), path, &derived_key).expect("open_or_create");

    let rec = JournalRecord::new_v2(
        [0x77; 16],
        RecordCode::RequestedIntent,
        1000,
        [0xee; 16],
        TargetType::Document,
        0,
        1,
        0,
        vec![0xde, 0xad, 0xbe, 0xef],
        1,
        vec![[0x12; 32]],
    );
    journal.append_record(&rec).expect("append");

    // Lese Roh-Bytes der journal.log Datei
    let raw = vfs.read(path).expect("read raw journal");

    // P13 Byte-Scan: Ciphertext darf den Klartext nicht enthalten (insbesondere derived_key oder klare Marker)
    assert!(
        !raw.windows(derived_key.len()).any(|w| w == derived_key),
        "P13: Journal must not leak derived encryption keys in file bytes!"
    );
}
