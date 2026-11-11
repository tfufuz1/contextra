//! Dateisystem-Abstraktion (Shim) für das Write-Ahead-Log.
//!
//! Stellt einheitliche async Dateischnittstellen unter `cfg(not(loom))` und `cfg(loom)` bereit.

#[cfg(not(loom))]
pub use tokio::fs::*;

#[cfg(loom)]
#[allow(dead_code)]
pub mod loom_fs {
    use std::path::Path;

    pub async fn create_dir_all<P: AsRef<Path>>(path: P) -> std::io::Result<()> {
        let _ = path;
        Ok(())
    }
    pub async fn read<P: AsRef<Path>>(path: P) -> std::io::Result<Vec<u8>> {
        std::fs::read(path)
    }
    pub async fn write<P: AsRef<Path>, C: AsRef<[u8]>>(
        path: P,
        contents: C,
    ) -> std::io::Result<()> {
        std::fs::write(path, contents)
    }
    pub async fn remove_file<P: AsRef<Path>>(path: P) -> std::io::Result<()> {
        let _ = path;
        Ok(())
    }
    pub async fn rename<P: AsRef<Path>, Q: AsRef<Path>>(from: P, to: Q) -> std::io::Result<()> {
        let _ = (from, to);
        Ok(())
    }
    pub async fn copy<P: AsRef<Path>, Q: AsRef<Path>>(from: P, to: Q) -> std::io::Result<u64> {
        let _ = (from, to);
        Ok(0)
    }
    pub async fn hard_link<P: AsRef<Path>, Q: AsRef<Path>>(from: P, to: Q) -> std::io::Result<()> {
        let _ = (from, to);
        Ok(())
    }
    pub async fn try_exists<P: AsRef<Path>>(path: P) -> std::io::Result<bool> {
        let _ = path;
        Ok(false)
    }
    pub async fn set_permissions<P: AsRef<Path>>(
        path: P,
        perm: std::fs::Permissions,
    ) -> std::io::Result<()> {
        let _ = (path, perm);
        Ok(())
    }
    pub async fn metadata<P: AsRef<Path>>(path: P) -> std::io::Result<LoomMetadata> {
        let _ = path;
        Ok(LoomMetadata)
    }

    #[derive(Debug, Clone)]
    pub struct LoomMetadata;
    impl LoomMetadata {
        pub fn len(&self) -> u64 {
            0
        }
        pub fn permissions(&self) -> std::fs::Permissions {
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                std::fs::Permissions::from_mode(0o644)
            }
            #[cfg(not(unix))]
            {
                let mut perm_res = std::fs::metadata(".").map(|m| m.permissions());
                if perm_res.is_err() {
                    perm_res = std::fs::metadata(std::env::temp_dir()).map(|m| m.permissions());
                }
                if perm_res.is_err() {
                    perm_res = std::fs::metadata("NUL").map(|m| m.permissions());
                }
                if let Ok(p) = perm_res {
                    p
                } else {
                    let tmp = std::env::temp_dir().join("loom_perm.tmp");
                    let p = std::fs::File::create(&tmp)
                        .and_then(|f| f.metadata())
                        .map(|m| m.permissions());
                    let _ = std::fs::remove_file(tmp);
                    p.unwrap_or_else(|_| {
                        std::fs::metadata(std::env::current_dir().unwrap_or_default())
                            .map(|m| m.permissions())
                            .unwrap_or_else(|_| {
                                std::fs::metadata(".")
                                    .map(|m| m.permissions())
                                    .unwrap_or_else(|_| {
                                        std::fs::metadata(std::env::temp_dir())
                                            .map(|m| m.permissions())
                                            .unwrap_or_else(|_| {
                                                std::fs::metadata(
                                                    std::env::current_exe().unwrap_or_default(),
                                                )
                                                .map(|m| m.permissions())
                                                .unwrap_or_else(|_| {
                                                    let local_tmp = std::path::PathBuf::from(
                                                        "loom_local_perm.tmp",
                                                    );
                                                    let p2 = std::fs::File::create(&local_tmp)
                                                        .and_then(|f| f.metadata())
                                                        .map(|m| m.permissions());
                                                    let _ = std::fs::remove_file(local_tmp);
                                                    p2.unwrap_or_else(|_| loop {
                                                        if let Ok(m) = std::fs::metadata(".") {
                                                            return m.permissions();
                                                        }
                                                    })
                                                })
                                            })
                                    })
                            })
                    })
                }
            }
        }
    }

    pub struct OpenOptions;
    impl OpenOptions {
        pub fn new() -> Self {
            Self
        }
        pub fn read(&mut self, _read: bool) -> &mut Self {
            self
        }
        pub fn write(&mut self, _write: bool) -> &mut Self {
            self
        }
        pub fn create(&mut self, _create: bool) -> &mut Self {
            self
        }
        pub fn create_new(&mut self, _create_new: bool) -> &mut Self {
            self
        }
        pub fn append(&mut self, _append: bool) -> &mut Self {
            self
        }
        pub fn mode(&mut self, _mode: u32) -> &mut Self {
            self
        }
        pub async fn open<P: AsRef<Path>>(&self, _path: P) -> std::io::Result<LoomFile> {
            Ok(LoomFile::new())
        }
    }

    #[derive(Default)]
    pub struct LoomFile {
        pub pos: usize,
        pub buf: Vec<u8>,
    }

    impl LoomFile {
        pub fn new() -> Self {
            Self {
                pos: 0,
                buf: Vec::new(),
            }
        }
        pub async fn open<P: AsRef<Path>>(_path: P) -> std::io::Result<Self> {
            Ok(Self::new())
        }
        pub async fn write_all(&mut self, buf: &[u8]) -> std::io::Result<()> {
            self.buf.extend_from_slice(buf);
            Ok(())
        }
        pub async fn flush(&self) -> std::io::Result<()> {
            Ok(())
        }
        pub async fn sync_all(&self) -> std::io::Result<()> {
            Ok(())
        }
        pub async fn set_len(&mut self, len: u64) -> std::io::Result<()> {
            self.buf.truncate(len as usize);
            Ok(())
        }
        pub async fn seek(&mut self, pos: std::io::SeekFrom) -> std::io::Result<u64> {
            match pos {
                std::io::SeekFrom::Start(offset) => self.pos = offset as usize,
                std::io::SeekFrom::End(offset) => {
                    self.pos = (self.buf.len() as i64 + offset).max(0) as usize;
                }
                std::io::SeekFrom::Current(offset) => {
                    self.pos = (self.pos as i64 + offset).max(0) as usize;
                }
            }
            Ok(self.pos as u64)
        }
        pub async fn metadata(&self) -> std::io::Result<LoomMetadata> {
            Ok(LoomMetadata)
        }
    }

    impl tokio::io::AsyncRead for LoomFile {
        fn poll_read(
            mut self: std::pin::Pin<&mut Self>,
            _cx: &mut std::task::Context<'_>,
            buf: &mut tokio::io::ReadBuf<'_>,
        ) -> std::task::Poll<std::io::Result<()>> {
            let pos = self.pos;
            if pos < self.buf.len() {
                let rem = &self.buf[pos..];
                let amt = rem.len().min(buf.remaining());
                buf.put_slice(&rem[..amt]);
                self.pos += amt;
            }
            std::task::Poll::Ready(Ok(()))
        }
    }

    impl tokio::io::AsyncWrite for LoomFile {
        fn poll_write(
            mut self: std::pin::Pin<&mut Self>,
            _cx: &mut std::task::Context<'_>,
            buf: &[u8],
        ) -> std::task::Poll<std::io::Result<usize>> {
            self.buf.extend_from_slice(buf);
            std::task::Poll::Ready(Ok(buf.len()))
        }
        fn poll_flush(
            self: std::pin::Pin<&mut Self>,
            _cx: &mut std::task::Context<'_>,
        ) -> std::task::Poll<std::io::Result<()>> {
            std::task::Poll::Ready(Ok(()))
        }
        fn poll_shutdown(
            self: std::pin::Pin<&mut Self>,
            _cx: &mut std::task::Context<'_>,
        ) -> std::task::Poll<std::io::Result<()>> {
            std::task::Poll::Ready(Ok(()))
        }
    }

    impl tokio::io::AsyncSeek for LoomFile {
        fn start_seek(
            mut self: std::pin::Pin<&mut Self>,
            pos: std::io::SeekFrom,
        ) -> std::io::Result<()> {
            match pos {
                std::io::SeekFrom::Start(offset) => self.pos = offset as usize,
                std::io::SeekFrom::End(offset) => {
                    self.pos = (self.buf.len() as i64 + offset).max(0) as usize;
                }
                std::io::SeekFrom::Current(offset) => {
                    self.pos = (self.pos as i64 + offset).max(0) as usize;
                }
            }
            Ok(())
        }
        fn poll_complete(
            self: std::pin::Pin<&mut Self>,
            _cx: &mut std::task::Context<'_>,
        ) -> std::task::Poll<std::io::Result<u64>> {
            std::task::Poll::Ready(Ok(self.pos as u64))
        }
    }

    pub type File = LoomFile;
}

#[cfg(loom)]
pub use loom_fs::*;
