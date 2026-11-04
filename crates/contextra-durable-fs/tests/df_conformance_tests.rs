//! Vfs-Conformance-Suite gegen RealVfs (Zusammenarbeit B15).
//! SPEC: PO-12, DF-01..DF-03, T-FSYNC-1, T-LOCK-1

use contextra_durable_fs::RealVfs;
use contextra_testkit::conformance::vfs::run_vfs_suite;
use tempfile::tempdir;

/// SPEC: T-PLAT-1
#[test]
fn test_real_vfs_conformance_suite() -> Result<(), Box<dyn std::error::Error>> {
    let tmp = tempdir()?;
    let base = tmp.path().to_path_buf();

    // TempScopedRealVfs kapselt RealVfs innerhalb eines isolierten temporären Verzeichnisses.
    #[derive(Clone)]
    struct TempScopedRealVfs {
        base: std::path::PathBuf,
        inner: RealVfs,
    }

    impl contextra_ports::Vfs for TempScopedRealVfs {
        fn open(
            &self,
            path: &std::path::Path,
            opts: &contextra_ports::VfsOpenOptions,
        ) -> contextra_ports::Result<Box<dyn contextra_ports::VfsFile>> {
            self.inner.open(&self.base.join(path), opts)
        }
        fn read_at(
            &self,
            path: &std::path::Path,
            offset: u64,
            buf: &mut [u8],
        ) -> contextra_ports::Result<usize> {
            self.inner.read_at(&self.base.join(path), offset, buf)
        }
        fn read_verified(
            &self,
            path: &std::path::Path,
            offset: u64,
            len: usize,
        ) -> contextra_ports::Result<(Vec<u8>, contextra_ports::vfs::VerifiedVia)> {
            self.inner.read_verified(&self.base.join(path), offset, len)
        }
        fn read(&self, path: &std::path::Path) -> contextra_ports::Result<Vec<u8>> {
            self.inner.read(&self.base.join(path))
        }
        fn write_atomic(&self, path: &std::path::Path, data: &[u8]) -> contextra_ports::Result<()> {
            self.inner.write_atomic(&self.base.join(path), data)
        }
        fn create_dir_all(&self, path: &std::path::Path) -> contextra_ports::Result<()> {
            self.inner.create_dir_all(&self.base.join(path))
        }
        fn remove_file(&self, path: &std::path::Path) -> contextra_ports::Result<()> {
            self.inner.remove_file(&self.base.join(path))
        }
        fn unlink(&self, path: &std::path::Path) -> contextra_ports::Result<()> {
            self.inner.unlink(&self.base.join(path))
        }
        fn rename(
            &self,
            from: &std::path::Path,
            to: &std::path::Path,
        ) -> contextra_ports::Result<()> {
            self.inner
                .rename(&self.base.join(from), &self.base.join(to))
        }
        fn exists(&self, path: &std::path::Path) -> contextra_ports::Result<bool> {
            self.inner.exists(&self.base.join(path))
        }
        fn len(&self, path: &std::path::Path) -> contextra_ports::Result<u64> {
            self.inner.len(&self.base.join(path))
        }
        fn list_dir(&self, path: &std::path::Path) -> contextra_ports::Result<Vec<String>> {
            self.inner.list_dir(&self.base.join(path))
        }
        fn list(&self, path: &std::path::Path) -> contextra_ports::Result<Vec<String>> {
            self.inner.list(&self.base.join(path))
        }
        fn sync_dir(&self, path: &std::path::Path) -> contextra_ports::Result<()> {
            self.inner.sync_dir(&self.base.join(path))
        }
    }

    let res = run_vfs_suite(|| TempScopedRealVfs {
        base: base.clone(),
        inner: RealVfs::new(),
    });

    assert!(
        res.is_ok(),
        "RealVfs must satisfy Vfs conformance suite: {res:?}"
    );
    Ok(())
}
