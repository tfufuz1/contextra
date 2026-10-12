use contextra_durable_fs::{RealFs, Vfs, VfsFile, VfsOpenOptions};
use std::collections::HashMap;
use std::io::Result;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use tempfile::TempDir;

#[derive(Default, Clone)]
pub struct MemoryVfs {
    files: Arc<Mutex<HashMap<PathBuf, Vec<u8>>>>,
}

impl Vfs for MemoryVfs {
    fn open(&self, path: &Path, opts: &VfsOpenOptions) -> Result<Box<dyn VfsFile>> {
        let mut map = self.files.lock().unwrap();
        let path_buf = path.to_path_buf();

        if opts.create_new && map.contains_key(&path_buf) {
            return Err(std::io::Error::new(
                std::io::ErrorKind::AlreadyExists,
                "file already exists",
            ));
        }

        let entry = map.entry(path_buf.clone()).or_insert_with(Vec::new);
        if opts.truncate {
            entry.clear();
        }

        Ok(Box::new(MemoryVfsFile {
            files: Arc::clone(&self.files),
            path: path_buf,
        }))
    }

    fn create_dir_all(&self, _path: &Path) -> Result<()> {
        Ok(())
    }

    fn remove_file(&self, path: &Path) -> Result<()> {
        let mut map = self.files.lock().unwrap();
        map.remove(path);
        Ok(())
    }

    fn rename(&self, from: &Path, to: &Path) -> Result<()> {
        let mut map = self.files.lock().unwrap();
        if let Some(contents) = map.remove(from) {
            map.insert(to.to_path_buf(), contents);
            Ok(())
        } else {
            Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "file not found",
            ))
        }
    }

    fn read_dir(&self, _path: &Path) -> Result<Vec<PathBuf>> {
        let map = self.files.lock().unwrap();
        Ok(map.keys().cloned().collect())
    }

    fn exists(&self, path: &Path) -> Result<bool> {
        let map = self.files.lock().unwrap();
        Ok(map.contains_key(path))
    }

    fn sync_dir(&self, _path: &Path) -> Result<()> {
        Ok(())
    }
}

pub struct MemoryVfsFile {
    files: Arc<Mutex<HashMap<PathBuf, Vec<u8>>>>,
    path: PathBuf,
}

impl VfsFile for MemoryVfsFile {
    fn read_at(&self, buf: &mut [u8], offset: u64) -> Result<usize> {
        let map = self.files.lock().unwrap();
        if let Some(data) = map.get(&self.path) {
            let offset = offset as usize;
            if offset >= data.len() {
                return Ok(0);
            }
            let slice = &data[offset..];
            let read_len = slice.len().min(buf.len());
            buf[..read_len].copy_from_slice(&slice[..read_len]);
            Ok(read_len)
        } else {
            Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "file not found",
            ))
        }
    }

    fn write_at(&self, buf: &[u8], offset: u64) -> Result<usize> {
        let mut map = self.files.lock().unwrap();
        let data = map.entry(self.path.clone()).or_insert_with(Vec::new);
        let offset = offset as usize;
        if offset + buf.len() > data.len() {
            data.resize(offset + buf.len(), 0);
        }
        data[offset..offset + buf.len()].copy_from_slice(buf);
        Ok(buf.len())
    }

    fn set_len(&self, len: u64) -> Result<()> {
        let mut map = self.files.lock().unwrap();
        if let Some(data) = map.get_mut(&self.path) {
            data.resize(len as usize, 0);
        }
        Ok(())
    }

    fn len(&self) -> Result<u64> {
        let map = self.files.lock().unwrap();
        Ok(map.get(&self.path).map_or(0, |v| v.len() as u64))
    }

    fn sync_data(&self) -> Result<()> {
        Ok(())
    }

    fn sync_all(&self) -> Result<()> {
        Ok(())
    }
}

#[test]
fn test_p03_vfs_realfs_vs_memory_equivalence() {
    let tmp = TempDir::new().unwrap();
    let real_fs = RealFs;
    let mem_fs = MemoryVfs::default();

    let real_path = tmp.path().join("test_file.bin");
    let mem_path = PathBuf::from("/virtual/test_file.bin");

    let opts = VfsOpenOptions {
        read: true,
        write: true,
        create: true,
        create_new: true,
        truncate: false,
        append: false,
    };

    let real_file = real_fs.open(&real_path, &opts).unwrap();
    let mem_file = mem_fs.open(&mem_path, &opts).unwrap();

    let payload = b"Contextra VFS equivalence test payload";

    real_file.write_at(payload, 0).unwrap();
    mem_file.write_at(payload, 0).unwrap();

    real_file.sync_all().unwrap();
    mem_file.sync_all().unwrap();

    assert_eq!(real_file.len().unwrap(), payload.len() as u64);
    assert_eq!(mem_file.len().unwrap(), payload.len() as u64);

    let mut real_buf = vec![0u8; payload.len()];
    let mut mem_buf = vec![0u8; payload.len()];

    real_file.read_at(&mut real_buf, 0).unwrap();
    mem_file.read_at(&mut mem_buf, 0).unwrap();

    assert_eq!(real_buf, payload);
    assert_eq!(mem_buf, payload);
    assert_eq!(real_buf, mem_buf);
}
