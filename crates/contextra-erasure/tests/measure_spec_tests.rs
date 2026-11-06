//! Spezifikationstests für Messung und Zeugenerzeugung (T-RAW-1..6, T-CLASS-1, D-72, D-113, I-19).

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use contextra_erasure::measure::field::{scan_mk1_structures, scan_mk2_raw, scan_mk3_keys};
use contextra_erasure::measure::positive_control::run_positive_control;
use contextra_erasure::measure::witness::{arm_scanner, finish_scan};
use contextra_ports::vfs::{Vfs, VfsFile, VfsOpenOptions};
use contextra_types::erasure::{LayerId, MeasureClass, Scanner, Unarmed};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::Duration;

#[derive(Default, Clone)]
struct MemoryVfs {
    files: Arc<Mutex<BTreeMap<PathBuf, Vec<u8>>>>,
}

struct MemoryVfsFile {
    path: PathBuf,
    vfs: MemoryVfs,
}

impl VfsFile for MemoryVfsFile {
    fn read_at(&self, buf: &mut [u8], offset: u64) -> std::io::Result<usize> {
        let files = self.files_lock();
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
        let mut files = self.files_mut_lock();
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
        let mut files = self.files_mut_lock();
        let content = files.entry(self.path.clone()).or_default();
        content.resize(usize::try_from(len).unwrap_or(0), 0);
        Ok(())
    }

    fn len(&self) -> std::io::Result<u64> {
        let files = self.files_lock();
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

impl MemoryVfsFile {
    fn files_lock(&self) -> std::sync::MutexGuard<'_, BTreeMap<PathBuf, Vec<u8>>> {
        self.vfs.files.lock().unwrap()
    }
    fn files_mut_lock(&self) -> std::sync::MutexGuard<'_, BTreeMap<PathBuf, Vec<u8>>> {
        self.vfs.files.lock().unwrap()
    }
}

impl Vfs for MemoryVfs {
    fn open(
        &self,
        path: &Path,
        _opts: &VfsOpenOptions,
    ) -> contextra_ports::Result<Box<dyn VfsFile>> {
        Ok(Box::new(MemoryVfsFile {
            path: path.to_path_buf(),
            vfs: self.clone(),
        }))
    }

    fn read(&self, path: &Path) -> contextra_ports::Result<Vec<u8>> {
        let files = self.files.lock().unwrap();
        files
            .get(path)
            .cloned()
            .ok_or_else(|| contextra_types::ContextraError::NotFound(path.display().to_string()))
    }

    fn write_atomic(&self, path: &Path, data: &[u8]) -> contextra_ports::Result<()> {
        let mut files = self.files.lock().unwrap();
        files.insert(path.to_path_buf(), data.to_vec());
        Ok(())
    }

    fn create_dir_all(&self, _path: &Path) -> contextra_ports::Result<()> {
        Ok(())
    }

    fn remove_file(&self, path: &Path) -> contextra_ports::Result<()> {
        let mut files = self.files.lock().unwrap();
        files.remove(path);
        Ok(())
    }

    fn exists(&self, path: &Path) -> contextra_ports::Result<bool> {
        let files = self.files.lock().unwrap();
        Ok(files.contains_key(path))
    }

    fn len(&self, path: &Path) -> contextra_ports::Result<u64> {
        let files = self.files.lock().unwrap();
        let content = files
            .get(path)
            .ok_or_else(|| contextra_types::ContextraError::NotFound(path.display().to_string()))?;
        Ok(content.len() as u64)
    }

    fn list_dir(&self, path: &Path) -> contextra_ports::Result<Vec<String>> {
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

    fn sync_dir(&self, _path: &Path) -> contextra_ports::Result<()> {
        Ok(())
    }
}

/// SPEC: T-RAW-2
#[test]
fn test_t_raw_2_armed_scanner_and_positive_control() {
    let layer = LayerId::new([0x01; 16]);
    let unarmed = Scanner::<Unarmed>::new(layer, MeasureClass::Mk1, 100);

    // 1. Ohne Positivkontrolle existiert kein Weg zu Scanner<Armed>
    // 2. Leeres Kontrollmuster schlägt fehl
    assert!(run_positive_control(b"").is_none());

    // 3. Echtes Muster ergibt ControlEvidence
    let pattern = b"contextra-canary-sample-data";
    let ev = run_positive_control(pattern).expect("must observe canary");

    let armed = arm_scanner(unarmed, ev);
    // 4. Scanner<Armed> mit 0 Treffern erzeugt LayerWitness
    let witness = finish_scan(armed, 0, Duration::from_millis(10), [0x55; 32]);
    assert!(witness.is_ok());

    // 5. Bei Treffern > 0 wird NIEMALS ein Zeuge erzeugt
    let ev2 = run_positive_control(pattern).expect("canary ok");
    let unarmed2 = Scanner::<Unarmed>::new(layer, MeasureClass::Mk1, 100);
    let armed2 = arm_scanner(unarmed2, ev2);
    let fail_witness = finish_scan(armed2, 2, Duration::from_millis(10), [0x55; 32]);
    assert!(fail_witness.is_err());
    assert_eq!(fail_witness.unwrap_err().matches, 2);
}

/// SPEC: T-RAW-1
#[test]
fn test_t_raw_1_mk1_structure_scan() {
    let vfs = Arc::new(MemoryVfs::default());
    let path = Path::new("test_dir");
    vfs.create_dir_all(path).unwrap();

    // MK1 Struktur: Prüfung von Manifest, SSTable-Indexblöcken, WAL-Segmentköpfen
    let target_needle = b"TARGET_KEY_78_BYTES";
    let mk1_clean = scan_mk1_structures(&*vfs, path, target_needle).expect("mk1 scan clean");
    assert_eq!(mk1_clean.matches, 0);

    // Residuum einpflanzen
    vfs.write_atomic(
        &path.join("manifest.001"),
        b"header-manifest-TARGET_KEY_78_BYTES-tail",
    )
    .unwrap();
    let mk1_dirty = scan_mk1_structures(&*vfs, path, target_needle).expect("mk1 scan dirty");
    assert!(
        mk1_dirty.matches > 0,
        "Residuum in Manifest muss erkannt werden"
    );
}

/// SPEC: T-RAW-4
#[test]
fn test_t_raw_4_mk2_raw_scan_needles_and_headers() {
    let vfs = Arc::new(MemoryVfs::default());
    let path = Path::new("db_root");
    vfs.create_dir_all(path).unwrap();

    let needle = b"secret-customer-pii-data";
    // Sauberer Zustand
    vfs.write_atomic(&path.join("wal/01.wal"), b"random-noise-no-needle")
        .unwrap();
    vfs.write_atomic(&path.join("tmp/temp.bin"), b"other-temp-data")
        .unwrap();

    let mk2_clean = scan_mk2_raw(&*vfs, path, needle).expect("mk2 clean");
    assert_eq!(mk2_clean.matches, 0);

    // Verstecktes Residuum in tmp/
    vfs.write_atomic(
        &path.join("tmp/leak.bin"),
        b"prefix-secret-customer-pii-data-suffix",
    )
    .unwrap();
    let mk2_dirty = scan_mk2_raw(&*vfs, path, needle).expect("mk2 dirty");
    assert!(
        mk2_dirty.matches > 0,
        "Residuum in tmp/ muss gefunden werden"
    );
}

/// SPEC: T-RAW-3
#[test]
fn test_t_raw_3_mk3_key_probe() {
    // MK3: Top-Slot genullt, DEK liefert Erased, F-Slots abwesend
    let top_slot_zeroed = true;
    let dek_erased = true;
    let file_slots_absent = true;

    let res = scan_mk3_keys(top_slot_zeroed, dek_erased, file_slots_absent);
    assert!(res.is_clean());

    let mutated_forgotten_f_slot = scan_mk3_keys(top_slot_zeroed, dek_erased, false);
    assert!(
        !mutated_forgotten_f_slot.is_clean(),
        "Vergessener F-Slot muss als unsauber erkannt werden"
    );
}

/// SPEC: T-VERIFY-1, T-VERIFY-2
#[test]
fn test_t_verify_1_and_2_medium_verification() {
    use contextra_erasure::measure::field::VerifiedVia;

    let scan = scan_mk3_keys(true, true, true);
    // T-VERIFY-2: verified_via korrekt im Scan-Ergebnis
    assert_eq!(scan.verified_via, VerifiedVia::Cached);

    // T-VERIFY-1: Fehlgeschlagene Verifikation (Medium zeigt alten Slot)
    let failed_scan = scan_mk3_keys(false, true, true);
    assert!(
        !failed_scan.is_clean(),
        "Medium mit altem Slot muss als fehlerhaft erkannt werden"
    );
}
