//! Spezifikationstests für Abschlussreihenfolge und Ledger (§11.8, D-37, D-74, D-126).

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use contextra_erasure::ledger::{finalize_erasure, FinalizeParams};
use contextra_ports::ledger::LedgerSink;
use contextra_ports::vfs::Vfs;
use contextra_types::ids::ErasureId;
use ed25519_dalek::SigningKey;
use std::path::Path;
use std::sync::Arc;

// Lokale In-Memory Vfs für Tests
#[derive(Default)]
struct LocalMemVfs {
    files: std::sync::Mutex<std::collections::HashMap<std::path::PathBuf, Vec<u8>>>,
}

impl contextra_ports::vfs::Vfs for LocalMemVfs {
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
        Ok(files.get(path).map(|c| c.len() as u64).unwrap_or(0))
    }
    fn list_dir(&self, _path: &Path) -> contextra_ports::Result<Vec<String>> {
        Ok(vec![])
    }
    fn sync_dir(&self, _path: &Path) -> contextra_ports::Result<()> {
        Ok(())
    }
}

#[test]
fn test_t_restore_1_finalize_sequence_and_ledger_failure() {
    let vfs = Arc::new(LocalMemVfs::default());
    let erasure_id = ErasureId::new([0x77; 16]);
    let signing_key = SigningKey::from_bytes(&[0x33; 32]);
    let ledger = Arc::new(contextra_erasure::ledger::InMemoryLedger::new());

    let params = FinalizeParams {
        erasure_id,
        gated_seq: 100,
        completed_seq: 105,
        gated_at: 1000,
        completed_at: 2000,
        commitment: [0x88; 32],
        range_tags: vec![1, 2, 3],
        witnesses: vec![],
        signing_key,
        report_dir: Path::new("erasure/report").to_path_buf(),
    };

    // Erfolgreicher Durchlauf
    let outcome = finalize_erasure(&*vfs, Some(&*ledger), &params);
    assert!(
        outcome.is_ok(),
        "Abschluss mit vorhandenem Ledger muss gelingen"
    );

    // Prüfen ob Report-Datei geschrieben wurde
    let expected_report_path = Path::new("erasure/report/77777777777777777777777777777777.json");
    assert!(vfs.exists(expected_report_path).unwrap());

    // Prüfen ob Ledger-Eintrag existiert
    let entries = ledger.read_all().unwrap();
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].commitment, [0x88; 32]);

    // Wenn Ledger-Ziel fehlt -> MUSS zu Io-Fehler führen (Completed ohne Ledger ist verboten!)
    let outcome_no_ledger = finalize_erasure(&*vfs, None, &params);
    assert!(outcome_no_ledger.is_err());
    match outcome_no_ledger.unwrap_err() {
        contextra_erasure::error::ErasureError::Io(err) => {
            assert_eq!(err.kind(), std::io::ErrorKind::NotFound);
        }
        other => panic!("Erwartet Io(NotFound), erhalten: {:?}", other),
    }
}
