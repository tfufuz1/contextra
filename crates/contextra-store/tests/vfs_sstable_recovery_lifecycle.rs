//! Integration tests for SSTable lifecycle and recovery on VFS / in-memory Vfs.

use contextra_ports::{ContextraError, Result, Vfs, VfsFile, VfsOpenOptions};
use contextra_store::sstable::{BlockCache, SstableBuilder, SstableReader};
use std::collections::BTreeMap;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

#[derive(Clone, Default)]
struct MemFs {
    files: Arc<Mutex<BTreeMap<PathBuf, Vec<u8>>>>,
}

struct SharedMemFile {
    data: Arc<Mutex<Vec<u8>>>,
    files: Arc<Mutex<BTreeMap<PathBuf, Vec<u8>>>>,
    path: PathBuf,
}

impl VfsFile for SharedMemFile {
    fn read_at(&self, buf: &mut [u8], offset: u64) -> std::io::Result<usize> {
        let lock = self.data.lock().unwrap();
        let offset = offset as usize;
        if offset >= lock.len() {
            return Ok(0);
        }
        let available = lock.len() - offset;
        let to_read = buf.len().min(available);
        buf[..to_read].copy_from_slice(&lock[offset..offset + to_read]);
        Ok(to_read)
    }

    fn write_at(&self, buf: &[u8], offset: u64) -> std::io::Result<usize> {
        let mut lock = self.data.lock().unwrap();
        let offset = offset as usize;
        if offset + buf.len() > lock.len() {
            lock.resize(offset + buf.len(), 0);
        }
        lock[offset..offset + buf.len()].copy_from_slice(buf);
        let mut files = self.files.lock().unwrap();
        files.insert(self.path.clone(), lock.clone());
        Ok(buf.len())
    }

    fn sync_all(&self) -> std::io::Result<()> {
        Ok(())
    }

    fn sync_data(&self) -> std::io::Result<()> {
        Ok(())
    }

    fn len(&self) -> std::io::Result<u64> {
        let lock = self.data.lock().unwrap();
        Ok(lock.len() as u64)
    }

    fn set_len(&self, size: u64) -> std::io::Result<()> {
        let mut lock = self.data.lock().unwrap();
        lock.resize(size as usize, 0);
        let mut files = self.files.lock().unwrap();
        files.insert(self.path.clone(), lock.clone());
        Ok(())
    }
}

impl Read for SharedMemFile {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.read_at(buf, 0)
    }
}

impl Write for SharedMemFile {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let len = self.len()?;
        self.write_at(buf, len)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

impl Vfs for MemFs {
    fn open(&self, path: &Path, options: &VfsOpenOptions) -> Result<Box<dyn VfsFile>> {
        let mut files = self.files.lock().unwrap();
        let path_buf = path.to_path_buf();
        if options.create_new {
            if files.contains_key(&path_buf) {
                return Err(ContextraError::Io(std::io::Error::new(
                    std::io::ErrorKind::AlreadyExists,
                    format!("{:?}", path),
                )));
            }
            files.insert(path_buf.clone(), Vec::new());
        } else if options.create {
            files.entry(path_buf.clone()).or_insert_with(Vec::new);
        } else if !files.contains_key(&path_buf) {
            return Err(ContextraError::NotFound(format!("{:?}", path)));
        }

        let data = files.get(&path_buf).unwrap().clone();
        let data_arc = Arc::new(Mutex::new(data));

        Ok(Box::new(SharedMemFile {
            data: data_arc,
            files: Arc::clone(&self.files),
            path: path_buf,
        }))
    }

    fn read(&self, path: &Path) -> Result<Vec<u8>> {
        let files = self.files.lock().unwrap();
        files
            .get(path)
            .cloned()
            .ok_or_else(|| ContextraError::NotFound(format!("{:?}", path)))
    }

    fn write_atomic(&self, path: &Path, data: &[u8]) -> Result<()> {
        let mut files = self.files.lock().unwrap();
        files.insert(path.to_path_buf(), data.to_vec());
        Ok(())
    }

    fn remove_file(&self, path: &Path) -> Result<()> {
        let mut files = self.files.lock().unwrap();
        files
            .remove(path)
            .map(|_| ())
            .ok_or_else(|| ContextraError::NotFound(format!("{:?}", path)))
    }

    fn rename(&self, from: &Path, to: &Path) -> Result<()> {
        let mut files = self.files.lock().unwrap();
        let data = files
            .remove(from)
            .ok_or_else(|| ContextraError::NotFound(format!("{:?}", from)))?;
        files.insert(to.to_path_buf(), data);
        Ok(())
    }

    fn exists(&self, path: &Path) -> Result<bool> {
        let files = self.files.lock().unwrap();
        Ok(files.contains_key(path))
    }

    fn len(&self, path: &Path) -> Result<u64> {
        let files = self.files.lock().unwrap();
        files
            .get(path)
            .map(|d| d.len() as u64)
            .ok_or_else(|| ContextraError::NotFound(format!("{:?}", path)))
    }

    fn sync_dir(&self, _path: &Path) -> Result<()> {
        Ok(())
    }

    fn list_dir(&self, path: &Path) -> Result<Vec<String>> {
        let files = self.files.lock().unwrap();
        let mut entries = Vec::new();
        for key in files.keys() {
            if let Some(parent) = key.parent() {
                if parent == path {
                    if let Some(file_name) = key.file_name().and_then(|n| n.to_str()) {
                        entries.push(file_name.to_string());
                    }
                }
            }
        }
        Ok(entries)
    }

    fn create_dir_all(&self, _path: &Path) -> Result<()> {
        Ok(())
    }
}

#[tokio::test]
async fn test_sstable_lifecycle_on_vfs() {
    let vfs = Arc::new(MemFs::default());
    let path = PathBuf::from("/data/test.sst");
    let block_cache = Arc::new(BlockCache::new(1024 * 1024));

    let mut builder =
        SstableBuilder::create_with_vfs(&path, None, Arc::clone(&vfs) as Arc<dyn Vfs>)
            .await
            .unwrap();

    builder.add(b"key1", b"val1", 1, 1).await.unwrap();
    builder.add(b"key2", b"val2", 2, 2).await.unwrap();
    builder.add(b"key3", b"val3", 3, 3).await.unwrap();

    let meta = builder.finish().await.unwrap();
    assert!(meta.file_size > 0);

    // Verify file exists on VFS and tmp is gone
    assert!(vfs.exists(&path).unwrap());
    let listed = vfs.list_dir(Path::new("/data")).unwrap();
    assert_eq!(listed.len(), 1);
    assert_eq!(listed[0], "test.sst");

    // Open reader on VFS
    let reader =
        SstableReader::open_with_vfs(&path, block_cache, None, Arc::clone(&vfs) as Arc<dyn Vfs>)
            .await
            .unwrap();

    let val1 = reader.get(b"key1").await.unwrap();
    assert_eq!(
        val1.map(|(v, _, _)| v),
        Some(bytes::Bytes::from_static(b"val1"))
    );

    let val2 = reader.get(b"key2").await.unwrap();
    assert_eq!(
        val2.map(|(v, _, _)| v),
        Some(bytes::Bytes::from_static(b"val2"))
    );

    let missing = reader.get(b"nonexistent").await.unwrap();
    assert_eq!(missing, None);
}

#[derive(Clone, Default)]
struct FaultFs {
    inner: MemFs,
    fail_rename: Arc<Mutex<bool>>,
}

impl Vfs for FaultFs {
    fn open(&self, path: &Path, options: &VfsOpenOptions) -> Result<Box<dyn VfsFile>> {
        self.inner.open(path, options)
    }

    fn read(&self, path: &Path) -> Result<Vec<u8>> {
        self.inner.read(path)
    }

    fn write_atomic(&self, path: &Path, data: &[u8]) -> Result<()> {
        self.inner.write_atomic(path, data)
    }

    fn remove_file(&self, path: &Path) -> Result<()> {
        self.inner.remove_file(path)
    }

    fn rename(&self, from: &Path, to: &Path) -> Result<()> {
        if *self.fail_rename.lock().unwrap() {
            return Err(ContextraError::Io(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Fault injection: rename failed",
            )));
        }
        self.inner.rename(from, to)
    }

    fn exists(&self, path: &Path) -> Result<bool> {
        self.inner.exists(path)
    }

    fn len(&self, path: &Path) -> Result<u64> {
        self.inner.len(path)
    }

    fn sync_dir(&self, path: &Path) -> Result<()> {
        self.inner.sync_dir(path)
    }

    fn list_dir(&self, path: &Path) -> Result<Vec<String>> {
        self.inner.list_dir(path)
    }

    fn create_dir_all(&self, path: &Path) -> Result<()> {
        self.inner.create_dir_all(path)
    }
}

#[tokio::test]
async fn test_fault_vfs_rename_failure_cleans_up_tmp() {
    let fault_fs = Arc::new(FaultFs {
        inner: MemFs::default(),
        fail_rename: Arc::new(Mutex::new(true)),
    });
    let path = PathBuf::from("/data/fault.sst");

    let mut builder =
        SstableBuilder::create_with_vfs(&path, None, Arc::clone(&fault_fs) as Arc<dyn Vfs>)
            .await
            .unwrap();

    builder.add(b"bad_key", b"bad_val", 1, 1).await.unwrap();
    let res = builder.finish().await;
    assert!(res.is_err(), "Builder finish must fail when rename fails");

    // Ensure destination does not exist
    assert!(!fault_fs.exists(&path).unwrap());

    // Ensure all tmp files in /data are cleaned up fail-closed
    let listed = fault_fs.list_dir(Path::new("/data")).unwrap();
    assert!(
        listed.is_empty(),
        "Tmp file must be cleaned up on rename failure: {:?}",
        listed
    );
}
