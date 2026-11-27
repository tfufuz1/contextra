use contextra_ports::VfsFile;
use std::io;

/// Durable positional append helper shared by manifest, WAL, and journals.
pub(crate) struct AppendFile {
    file: Box<dyn VfsFile>,
    committed_len: u64,
    poisoned: bool,
}

#[derive(Debug, thiserror::Error)]
pub(crate) enum IoFault {
    #[error("partial append failed after {written} bytes: {source}")]
    PartialWrite { written: usize, source: io::Error },
    #[error("append ran out of space after {written} bytes")]
    NoSpace { written: usize },
    #[error("sync failed after append: {0}")]
    SyncFailed(io::Error),
    #[error("append file handle is poisoned")]
    Poisoned,
    #[error("blocking append task failed: {0}")]
    TaskFailed(String),
    #[error("append returned zero bytes after {written} bytes")]
    WriteZero { written: usize },
}

impl AppendFile {
    pub(crate) fn new(file: Box<dyn VfsFile>, committed_len: u64) -> Self {
        Self {
            file,
            committed_len,
            poisoned: false,
        }
    }

    pub(crate) fn committed_len(&self) -> u64 {
        self.committed_len
    }

    pub(crate) fn append_durable(&mut self, bytes: &[u8]) -> Result<(), IoFault> {
        if self.poisoned {
            return Err(IoFault::Poisoned);
        }

        let byte_len = u64::try_from(bytes.len()).map_err(|_| IoFault::Poisoned)?;
        let mut written = 0usize;
        while written < bytes.len() {
            let offset = self
                .committed_len
                .checked_add(u64::try_from(written).map_err(|_| IoFault::Poisoned)?)
                .ok_or(IoFault::Poisoned)?;
            let chunk = bytes.get(written..).ok_or(IoFault::Poisoned)?;
            match self.file.write_at(chunk, offset) {
                Ok(0) => return self.rollback_error(IoFault::WriteZero { written }, written),
                Ok(count) => written += count,
                Err(error) if error.kind() == io::ErrorKind::WriteZero => {
                    return self.rollback_error(IoFault::NoSpace { written }, written)
                }
                Err(error) if error.kind() == io::ErrorKind::StorageFull => {
                    return self.rollback_error(IoFault::NoSpace { written }, written)
                }
                Err(source) => {
                    return self.rollback_error(IoFault::PartialWrite { written, source }, written)
                }
            }
        }

        if let Err(error) = self.file.sync_data() {
            self.poisoned = true;
            return Err(IoFault::SyncFailed(error));
        }
        self.committed_len = self
            .committed_len
            .checked_add(byte_len)
            .ok_or(IoFault::Poisoned)?;
        Ok(())
    }

    pub(crate) fn read_all(&self) -> io::Result<Vec<u8>> {
        let len = self.file.len()?;
        let cap = usize::try_from(len).map_err(|_| {
            io::Error::new(io::ErrorKind::InvalidData, "file too large for address space")
        })?;
        let mut data = vec![0; cap];
        let mut read = 0usize;
        while read < data.len() {
            let offset = u64::try_from(read)
                .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "offset overflow"))?;
            let tail = data.get_mut(read..).ok_or_else(|| {
                io::Error::new(io::ErrorKind::InvalidInput, "read buffer range invalid")
            })?;
            let count = self.file.read_at(tail, offset)?;
            if count == 0 {
                break;
            }
            read += count;
        }
        data.truncate(read);
        Ok(data)
    }

    pub(crate) fn truncate_durable(&mut self, len: u64) -> io::Result<()> {
        self.file.set_len(len)?;
        if let Err(error) = self.file.sync_data() {
            self.poisoned = true;
            return Err(error);
        }
        self.committed_len = len;
        Ok(())
    }

    fn rollback_error(&mut self, error: IoFault, _written: usize) -> Result<(), IoFault> {
        if self.file.set_len(self.committed_len).is_err() || self.file.sync_data().is_err() {
            self.poisoned = true;
            return Err(IoFault::Poisoned);
        }
        Err(error)
    }

    pub(crate) fn replace_file(&mut self, file: Box<dyn VfsFile>, committed_len: u64) {
        self.file = file;
        self.committed_len = committed_len;
        self.poisoned = false;
    }

    pub(crate) fn sync_all(&mut self) -> Result<(), IoFault> {
        if self.poisoned {
            return Err(IoFault::Poisoned);
        }
        self.file.sync_all().map_err(|error| {
            self.poisoned = true;
            IoFault::SyncFailed(error)
        })
    }
}
