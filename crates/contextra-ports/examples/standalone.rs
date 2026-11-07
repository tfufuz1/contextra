//! Standalone example demonstrating direct usage of `contextra-ports` contracts (Contract M10).

use contextra_ports::env::prelude::*;
use contextra_ports::fs::prelude::*;
use contextra_ports::policy::prelude::*;
use contextra_ports::{ContextraError, RequestContext, TenantId, TxId, VfsOpenOptions};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

/// In-memory VFS implementation for example purposes.
#[derive(Default, Clone)]
struct InMemoryVfs {
    files: Arc<Mutex<HashMap<PathBuf, Vec<u8>>>>,
}

struct InMemoryFile {
    path: PathBuf,
    files: Arc<Mutex<HashMap<PathBuf, Vec<u8>>>>,
}

impl VfsFile for InMemoryFile {
    fn read_at(&self, buf: &mut [u8], offset: u64) -> std::io::Result<usize> {
        let files = self.files.lock().map_err(|_| std::io::Error::other("lock"))?;
        let data = files.get(&self.path).cloned().unwrap_or_default();
        let off = usize::try_from(offset).map_err(|e| std::io::Error::other(e.to_string()))?;
        if off >= data.len() {
            return Ok(0);
        }
        let available = &data[off..];
        let n = available.len().min(buf.len());
        buf[..n].copy_from_slice(&available[..n]);
        Ok(n)
    }

    fn write_at(&self, buf: &[u8], offset: u64) -> std::io::Result<usize> {
        let mut files = self.files.lock().map_err(|_| std::io::Error::other("lock"))?;
        let entry = files.entry(self.path.clone()).or_default();
        let off = usize::try_from(offset).map_err(|e| std::io::Error::other(e.to_string()))?;
        if off + buf.len() > entry.len() {
            entry.resize(off + buf.len(), 0);
        }
        entry[off..off + buf.len()].copy_from_slice(buf);
        Ok(buf.len())
    }

    fn set_len(&self, len: u64) -> std::io::Result<()> {
        let mut files = self.files.lock().map_err(|_| std::io::Error::other("lock"))?;
        let entry = files.entry(self.path.clone()).or_default();
        let target_len = usize::try_from(len).map_err(|e| std::io::Error::other(e.to_string()))?;
        entry.resize(target_len, 0);
        Ok(())
    }

    fn len(&self) -> std::io::Result<u64> {
        let files = self.files.lock().map_err(|_| std::io::Error::other("lock"))?;
        let len = files.get(&self.path).map(|v| v.len()).unwrap_or(0);
        Ok(u64::try_from(len).map_err(|e| std::io::Error::other(e.to_string()))?)
    }

    fn sync_data(&self) -> std::io::Result<()> {
        Ok(())
    }

    fn sync_all(&self) -> std::io::Result<()> {
        Ok(())
    }
}

impl Vfs for InMemoryVfs {
    fn read(&self, path: &Path) -> Result<Vec<u8>, ContextraError> {
        let files = self.files.lock().map_err(|_| ContextraError::Internal("lock failed".into()))?;
        files.get(path).cloned().ok_or_else(|| ContextraError::Io(std::io::Error::new(std::io::ErrorKind::NotFound, "file not found")))
    }

    fn write_atomic(&self, path: &Path, content: &[u8]) -> Result<(), ContextraError> {
        let mut files = self.files.lock().map_err(|_| ContextraError::Internal("lock failed".into()))?;
        files.insert(path.to_path_buf(), content.to_vec());
        Ok(())
    }

    fn create_dir_all(&self, _path: &Path) -> Result<(), ContextraError> {
        Ok(())
    }

    fn remove_file(&self, path: &Path) -> Result<(), ContextraError> {
        let mut files = self.files.lock().map_err(|_| ContextraError::Internal("lock failed".into()))?;
        files.remove(path);
        Ok(())
    }

    fn exists(&self, path: &Path) -> Result<bool, ContextraError> {
        let files = self.files.lock().map_err(|_| ContextraError::Internal("lock failed".into()))?;
        Ok(files.contains_key(path))
    }

    fn len(&self, path: &Path) -> Result<u64, ContextraError> {
        let files = self.files.lock().map_err(|_| ContextraError::Internal("lock failed".into()))?;
        let len = files.get(path).map(|v| v.len()).unwrap_or(0);
        u64::try_from(len).map_err(|e| ContextraError::Internal(e.to_string()))
    }

    fn list_dir(&self, path: &Path) -> Result<Vec<String>, ContextraError> {
        let files = self.files.lock().map_err(|_| ContextraError::Internal("lock failed".into()))?;
        let prefix = path.to_string_lossy();
        let entries = files.keys()
            .filter(|p| p.to_string_lossy().starts_with(prefix.as_ref()))
            .map(|p| p.to_string_lossy().to_string())
            .collect();
        Ok(entries)
    }

    fn sync_dir(&self, _path: &Path) -> Result<(), ContextraError> {
        Ok(())
    }

    fn open(&self, path: &Path, _opts: &VfsOpenOptions) -> Result<Box<dyn VfsFile>, ContextraError> {
        Ok(Box::new(InMemoryFile {
            path: path.to_path_buf(),
            files: self.files.clone(),
        }))
    }
}

struct SimplePolicy;
impl EgressPolicy for SimplePolicy {
    fn evaluate(&self, _ctx: &RequestContext, payload: &str) -> EgressDecision {
        if payload.contains("forbidden") {
            EgressDecision::Deny("policy violation".to_string())
        } else {
            EgressDecision::Allow
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Injected environment (M4)
    let clock = contextra_ports::env::SystemClock::new();
    let id_gen = contextra_ports::env::SequentialIdGen::new(100);
    let metrics = contextra_ports::env::InMemoryMetricsSink::new();

    let now = clock.now_unix_nanos();
    let next_id = id_gen.next_id();
    metrics.record_counter("example_runs", 1, &[("module", "ports")]);

    println!("Clock: {now}, Next ID: {next_id}");

    // 2. Filesystem abstraction (VFS)
    let vfs = InMemoryVfs::default();
    let path = Path::new("/test_ports.txt");
    let file = vfs.open(path, &VfsOpenOptions::write_create())?;
    let written = file.write_at(b"Hello Contextra Ports", 0)?;
    println!("VFS written {written} bytes to {path:?}");

    // 3. Policy evaluation
    let policy = SimplePolicy;
    let ctx = RequestContext::new(TenantId(1), TxId(1));
    let decision = policy.evaluate(&ctx, "clean data");
    println!("Policy evaluation result: {decision:?}");

    Ok(())
}
