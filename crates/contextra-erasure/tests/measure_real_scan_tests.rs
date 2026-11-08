//! Reale Mess- und Verifikationstests mit Positivkontrolle (SPEC 12.1, 12.2, 12.4, D-35, D-72, D-113, T-RAW-1..6, T-MUT-1, T-PROOF-1, T-KEYCOV-1).

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use contextra_erasure::error::ErasureError;
use contextra_erasure::measure::field::{Mk3StatusInput, ScanPlan, StructureDecoder};
use contextra_erasure::measure::scanner_flow::scan_with_control;
use contextra_erasure::types_local::Vfs;
use contextra_erasure::types_local::{LayerId, MeasureClass};
use contextra_erasure::types_local::{VfsFile, VfsOpenOptions};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

#[derive(Default, Clone)]
struct TestMemoryVfs {
    files: Arc<Mutex<BTreeMap<PathBuf, Vec<u8>>>>,
}

struct TestMemoryVfsFile {
    path: PathBuf,
    vfs: TestMemoryVfs,
}

impl VfsFile for TestMemoryVfsFile {
    fn read_at(&self, buf: &mut [u8], offset: u64) -> std::io::Result<usize> {
        let files = self.vfs.files.lock().unwrap();
        let content = files
            .get(&self.path)
            .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::NotFound, "file not found"))?;
        let off = usize::try_from(offset).unwrap_or(0);
        if off >= content.len() {
            return Ok(0);
        }
        let available = content.get(off..).unwrap_or(&[]);
        let to_copy = buf.len().min(available.len());
        if let (Some(dest), Some(src)) = (buf.get_mut(..to_copy), available.get(..to_copy)) {
            dest.copy_from_slice(src);
        }
        Ok(to_copy)
    }

    fn write_at(&self, buf: &[u8], offset: u64) -> std::io::Result<usize> {
        let mut files = self.vfs.files.lock().unwrap();
        let content = files.entry(self.path.clone()).or_default();
        let off = usize::try_from(offset).unwrap_or(0);
        if content.len() < off + buf.len() {
            content.resize(off + buf.len(), 0);
        }
        if let Some(dest) = content.get_mut(off..off + buf.len()) {
            dest.copy_from_slice(buf);
        }
        Ok(buf.len())
    }

    fn set_len(&self, len: u64) -> std::io::Result<()> {
        let mut files = self.vfs.files.lock().unwrap();
        let content = files.entry(self.path.clone()).or_default();
        content.resize(usize::try_from(len).unwrap_or(0), 0);
        Ok(())
    }

    fn len(&self) -> std::io::Result<u64> {
        let files = self.vfs.files.lock().unwrap();
        let content = files
            .get(&self.path)
            .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::NotFound, "file not found"))?;
        Ok(content.len() as u64)
    }

    fn sync_data(&self) -> std::io::Result<()> {
        Ok(())
    }

    fn sync_all(&self) -> std::io::Result<()> {
        Ok(())
    }
}

impl Vfs for TestMemoryVfs {
    fn read(
        &self,
        path: &Path,
    ) -> std::result::Result<Vec<u8>, contextra_erasure::error::ErasureError> {
        let files = self.files.lock().unwrap();
        files.get(path).cloned().ok_or_else(|| {
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
        let mut files = self.files.lock().unwrap();
        files.insert(path.to_path_buf(), data.to_vec());
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
        let mut files = self.files.lock().unwrap();
        files.remove(path);
        Ok(())
    }

    fn exists(
        &self,
        path: &Path,
    ) -> std::result::Result<bool, contextra_erasure::error::ErasureError> {
        let files = self.files.lock().unwrap();
        Ok(files.contains_key(path))
    }

    fn len(&self, path: &Path) -> std::result::Result<u64, contextra_erasure::error::ErasureError> {
        let files = self.files.lock().unwrap();
        let content = files.get(path).ok_or_else(|| {
            contextra_erasure::error::ErasureError::Io(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                path.display().to_string(),
            ))
        })?;
        Ok(content.len() as u64)
    }

    fn list_dir(
        &self,
        path: &Path,
    ) -> std::result::Result<Vec<String>, contextra_erasure::error::ErasureError> {
        let files = self.files.lock().unwrap();
        let mut names = Vec::new();
        for k in files.keys() {
            if let Ok(rel) = k.strip_prefix(path) {
                if let Some(first) = rel.components().next() {
                    let s = first.as_os_str().to_string_lossy().to_string();
                    if !names.contains(&s) {
                        names.push(s);
                    }
                }
            }
        }
        Ok(names)
    }

    fn sync_dir(
        &self,
        _path: &Path,
    ) -> std::result::Result<(), contextra_erasure::error::ErasureError> {
        Ok(())
    }
}

/// Strukturdecoder für Test-SSTable-Indizes
struct MockSstDecoder;

impl StructureDecoder for MockSstDecoder {
    fn name(&self) -> &str {
        "MockSstDecoder"
    }

    fn can_decode(&self, path: &Path) -> bool {
        path.to_string_lossy().ends_with(".sst")
    }

    fn decode_and_check(
        &self,
        _path: &Path,
        content: &[u8],
        target_pattern: &[u8],
    ) -> Result<u64, ErasureError> {
        let mut matches = 0u64;
        if content.len() >= target_pattern.len() && !target_pattern.is_empty() {
            for i in 0..=content.len() - target_pattern.len() {
                if let Some(window) = content.get(i..i + target_pattern.len()) {
                    if window == target_pattern {
                        matches = matches.saturating_add(1);
                    }
                }
            }
        }
        Ok(matches)
    }
}

/// SPEC: T-RAW-2
/// T-ID: T-RAW-2
#[test]
fn t_raw_2_missing_positive_control_fails_closed() {
    let vfs = TestMemoryVfs::default();
    let root = Path::new("db_root");
    vfs.create_dir_all(root).unwrap();

    let layer = LayerId::new([0x02; 16]);
    // Plan mit leerem Kontrollmuster muss zwingend fehlschlagen
    let plan = ScanPlan::new(layer, MeasureClass::Mk2, 2, root, b"");
    let res = scan_with_control(&vfs, &plan);
    assert!(
        res.is_err(),
        "Ohne Positivkontrolle darf kein Zeuge erzeugt werden"
    );

    // Plan mit angegebener, aber fehlender Kontrolldatei muss fehlschlagen
    let plan_missing_file = ScanPlan::new(layer, MeasureClass::Mk2, 2, root, b"canary")
        .with_control_file(root.join("nonexistent_canary.bin"));
    let res_missing = scan_with_control(&vfs, &plan_missing_file);
    assert!(
        res_missing.is_err(),
        "Bei fehlender Kontrolldatei muss der Scan scheitern"
    );
}

/// SPEC: T-RAW-1, T-RAW-4, T-MUT-1
/// T-ID: T-RAW-1
#[test]
fn t_raw_1_and_4_scan_finds_needle_in_tmp_and_old_segments() {
    let vfs = TestMemoryVfs::default();
    let root = Path::new("db_root");
    vfs.create_dir_all(root).unwrap();

    let layer = LayerId::new([0x03; 16]);
    let canary = b"valid-positive-control-canary-sequence";
    let target_needle = b"pii-secret-customer-record";

    // 1. Unbefleckter Zustand
    vfs.write_atomic(&root.join("wal/01.wal"), b"random-noise-no-secrets")
        .unwrap();
    vfs.write_atomic(&root.join("data/01.sst"), b"random-sstable-blocks")
        .unwrap();

    let clean_plan =
        ScanPlan::new(layer, MeasureClass::Mk2, 2, root, canary).with_target_pattern(target_needle);
    let witness = scan_with_control(&vfs, &clean_plan).expect("clean scan produces witness");
    assert_eq!(witness.layer(), layer);

    // 2. Residuum in tmp/ einpflanzen (T-MUT-1)
    vfs.write_atomic(
        &root.join("tmp/unflushed_temp_leak.tmp"),
        b"noise-prefix-pii-secret-customer-record-suffix",
    )
    .unwrap();
    let dirty_tmp = scan_with_control(&vfs, &clean_plan);
    match dirty_tmp {
        Err(ErasureError::Residue(matches)) => {
            assert!(matches > 0, "Residuum in tmp/ muss erkannt werden");
        }
        _ => panic!("Residuum in tmp/ wurde nicht als ErasureError::Residue abgewiesen"),
    }

    // tmp/ bereinigen
    vfs.remove_file(&root.join("tmp/unflushed_temp_leak.tmp"))
        .unwrap();

    // 3. Residuum in altem archivierten Segment einpflanzen (T-MUT-1)
    vfs.write_atomic(
        &root.join("segments/archived_seg_99.sst"),
        b"header-pii-secret-customer-record-trailer",
    )
    .unwrap();
    let dirty_seg = scan_with_control(&vfs, &clean_plan);
    match dirty_seg {
        Err(ErasureError::Residue(matches)) => {
            assert!(matches > 0, "Residuum in altem Segment muss erkannt werden");
        }
        _ => panic!("Residuum in Segment wurde nicht als ErasureError::Residue abgewiesen"),
    }
}

/// SPEC: T-MUT-1, T-KEYCOV-1, T-PROOF-1
/// T-ID: T-MUT-1
#[test]
fn t_mut_1_and_t_keycov_1_mk3_broken_slot_detected() {
    let vfs = TestMemoryVfs::default();
    let root = Path::new("db_root");
    vfs.create_dir_all(root).unwrap();

    let layer = LayerId::new([0x04; 16]);
    let canary = b"canary-mk3-verification";

    // 0. Fehlender Status schlägt fehl (fail-closed)
    let plan_no_status = ScanPlan::new(layer, MeasureClass::Mk3, 3, root, canary);
    assert!(scan_with_control(&vfs, &plan_no_status).is_err());

    // 1. Sauberer Zustand: Top-Slot genullt, DEK scheitert, F-Slots abwesend
    let clean_plan =
        ScanPlan::new(layer, MeasureClass::Mk3, 3, root, canary).with_mk3_status(Mk3StatusInput {
            top_slot_zeroed: true,
            dek_derivation_fails: true,
            file_slots_absent: true,
        });
    let witness = scan_with_control(&vfs, &clean_plan).expect("clean mk3 produces witness");
    assert_eq!(witness.class(), MeasureClass::Mk3);

    // 2. Mutant: Top-Slot nicht genullt (Rest-Slot verblieben)
    let dirty_slot_plan =
        ScanPlan::new(layer, MeasureClass::Mk3, 3, root, canary).with_mk3_status(Mk3StatusInput {
            top_slot_zeroed: false,
            dek_derivation_fails: true,
            file_slots_absent: true,
        });
    let fail_res = scan_with_control(&vfs, &dirty_slot_plan);
    match fail_res {
        Err(ErasureError::Residue(m)) => assert_eq!(m, 1),
        _ => panic!("Nicht genullter Top-Slot muss als Residue abgewiesen werden"),
    }

    // 3. Mutant: Vergessener F-Slot
    let dirty_f_slot_plan = ScanPlan::new(layer, MeasureClass::Mk3, 3, root, canary)
        .with_mk3_status(Mk3StatusInput {
            top_slot_zeroed: true,
            dek_derivation_fails: true,
            file_slots_absent: false,
        });
    let fail_f_slot = scan_with_control(&vfs, &dirty_f_slot_plan);
    match fail_f_slot {
        Err(ErasureError::Residue(m)) => assert_eq!(m, 1),
        _ => panic!("Vergessener F-Slot muss als Residue abgewiesen werden"),
    }
}

/// SPEC: T-RAW-1, T-RAW-5
/// T-ID: T-RAW-1
#[test]
fn t_raw_1_and_5_structure_decoder_injection() {
    let vfs = TestMemoryVfs::default();
    let root = Path::new("db_root");
    vfs.create_dir_all(root).unwrap();

    let layer = LayerId::new([0x05; 16]);
    let canary = b"structure-canary";
    let target = b"TARGET_INDEX_KEY_65B";

    vfs.write_atomic(
        &root.join("table_01.sst"),
        b"sst_index_block_TARGET_INDEX_KEY_65B_end",
    )
    .unwrap();

    let plan = ScanPlan::new(layer, MeasureClass::Mk1, 1, root, canary)
        .with_target_pattern(target)
        .with_decoder(Arc::new(MockSstDecoder));

    let res = scan_with_control(&vfs, &plan);
    match res {
        Err(ErasureError::Residue(matches)) => {
            assert!(matches > 0, "Decoder muss Residuum im Index finden");
        }
        _ => panic!("Residuum in SSTable-Index wurde nicht erkannt"),
    }
}
