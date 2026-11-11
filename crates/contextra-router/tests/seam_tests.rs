#![allow(clippy::unwrap_used)]

use contextra_ports::vfs::Vfs;
use contextra_router::{
    CalibrationSource, NoCalibrationSource, StdFsCalibrationSource, VfsCalibrationSource,
};
use contextra_types::ContextraError;
use std::io::Write;
use std::path::Path;
use std::sync::Arc;
use tempfile::NamedTempFile;

// ANCHOR[contextra-router:SEM-T01]
#[test]
fn test_no_calibration_source() {
    let source = NoCalibrationSource;
    assert_eq!(source.read_calibration(Path::new("any_path")), None);
}

// ANCHOR[contextra-router:SEM-T02]
#[test]
fn test_std_fs_calibration_source_success_and_missing() {
    let source = StdFsCalibrationSource;
    let mut tmp = NamedTempFile::new().unwrap();
    tmp.write_all(b"{\"test\": 123}").unwrap();
    tmp.flush().unwrap();

    let bytes = source.read_calibration(tmp.path()).unwrap();
    assert_eq!(bytes, b"{\"test\": 123}");

    let non_existent = Path::new("/nonexistent/file/path/12345");
    assert_eq!(source.read_calibration(non_existent), None);
}

// ANCHOR[contextra-router:SEM-T03]
#[test]
fn test_vfs_calibration_source() {
    struct MockVfs;
    impl Vfs for MockVfs {
        fn read(&self, path: &Path) -> Result<Vec<u8>, ContextraError> {
            if path == Path::new("virtual.json") {
                Ok(b"virtual content".to_vec())
            } else {
                Err(ContextraError::NotFound(path.display().to_string()))
            }
        }
        fn write_atomic(&self, _path: &Path, _data: &[u8]) -> Result<(), ContextraError> {
            Ok(())
        }
        fn create_dir_all(&self, _path: &Path) -> Result<(), ContextraError> {
            Ok(())
        }
        fn remove_file(&self, _path: &Path) -> Result<(), ContextraError> {
            Ok(())
        }
        fn exists(&self, path: &Path) -> Result<bool, ContextraError> {
            Ok(path == Path::new("virtual.json"))
        }
        fn len(&self, _path: &Path) -> Result<u64, ContextraError> {
            Ok(15)
        }
        fn list_dir(&self, _path: &Path) -> Result<Vec<String>, ContextraError> {
            Ok(vec![])
        }
        fn sync_dir(&self, _path: &Path) -> Result<(), ContextraError> {
            Ok(())
        }
    }

    let mock_vfs: Arc<dyn Vfs> = Arc::new(MockVfs);
    let wrapper = VfsCalibrationSource::new(mock_vfs.clone());
    assert_eq!(
        wrapper.read_calibration(Path::new("virtual.json")),
        Some(b"virtual content".to_vec())
    );
    assert_eq!(wrapper.read_calibration(Path::new("other.json")), None);

    assert_eq!(
        mock_vfs.read_calibration(Path::new("virtual.json")),
        Some(b"virtual content".to_vec())
    );
}
