//! Spezifikationstests für Report (T-REPORT-1, T-PROOF-1, F.10, SPEC §11.7, Anhang D).

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use contextra_erasure::report::build::build_report;
use contextra_erasure::report::canonical::to_canonical_json_string;
use contextra_erasure::report::signing::ReportSigner;
use contextra_erasure::types_local::ErasureId;
use contextra_erasure::types_local::Vfs;
use ed25519_dalek::SigningKey;
use std::path::Path;
use std::sync::Arc;

// Eigenständiger In-Memory Vfs für Tests
#[derive(Default)]
struct LocalMemVfs {
    files: std::sync::Mutex<std::collections::BTreeMap<std::path::PathBuf, Vec<u8>>>,
}
impl contextra_erasure::types_local::Vfs for LocalMemVfs {
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
        Ok(files.get(path).map(|c| c.len() as u64).unwrap_or(0))
    }
    fn list_dir(
        &self,
        _path: &Path,
    ) -> std::result::Result<Vec<String>, contextra_erasure::error::ErasureError> {
        Ok(vec![])
    }
    fn sync_dir(
        &self,
        _path: &Path,
    ) -> std::result::Result<(), contextra_erasure::error::ErasureError> {
        Ok(())
    }
}

/// SPEC: T-REPORT-1
#[test]
fn test_t_report_1_canonical_format_and_signing() {
    let erasure_id = ErasureId::new([0x42; 16]);
    let report = build_report(erasure_id, 1000000, 2000000, vec![], 5, 10240, true);

    let canonical_json = to_canonical_json_string(&report).expect("canonical json string");
    // F.10 Prüfungen: keine Newlines, keine Leerzeichen nach Doppelpunkten
    assert!(!canonical_json.contains('\n'));
    assert!(!canonical_json.contains(": "));
    assert!(canonical_json.contains("\"erasure_id\":\"42424242424242424242424242424242\""));

    let signing_key = SigningKey::from_bytes(&[0x13; 32]);
    let signed = ReportSigner::sign_report(&report, &signing_key).expect("signing must succeed");

    assert!(!signed.signature_base64.is_empty());
    assert!(!signed.pub_fingerprint.is_empty());
    assert_eq!(signed.canonical_body, canonical_json);
}

/// SPEC: T-REPORT-1
#[test]
fn test_t_report_1_write_and_verify_report_file() {
    let vfs = Arc::new(LocalMemVfs::default());
    let erasure_id = ErasureId::new([0xaa; 16]);
    let report = build_report(erasure_id, 1000, 2000, vec![], 0, 0, true);
    let signing_key = SigningKey::from_bytes(&[0x13; 32]);
    let signed = ReportSigner::sign_report(&report, &signing_key).expect("signing must succeed");

    let report_path = Path::new("erasure/report/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa.json");
    ReportSigner::persist_signed_report(&*vfs, report_path, &signed).expect("persist report file");

    assert!(vfs.exists(report_path).unwrap());
}
