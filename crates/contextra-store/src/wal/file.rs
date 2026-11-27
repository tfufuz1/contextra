//! # `WalFile`: async Dateihandle des WAL über den `Vfs`-Port (Ring 1)
//!
//! Ersetzt `tokio::fs::File` im WAL. Alle Operationen laufen über `VfsFile`
//! (`StoreIo`), damit `CrashVfs`/`FaultVfs` jeden Write, Sync und Truncate des WAL
//! abfangen können (INV-IO1).
//!
//! ## Semantik
//!
//! - **Schreiben** hängt immer am Dateiende an (entspricht dem bisherigen `append(true)`).
//!   Es gibt keinen Schreib-Userspace-Puffer; `flush` ist daher ein No-op und die
//!   Dauerhaftigkeit entsteht erst durch `sync_all`/`sync_data`.
//! - **Lesen** nutzt einen Cursor (`seek`) und einen kleinen Lese-Puffer. Jede
//!   Mutation (`write_all`, `set_len`, `seek`) verwirft den Puffer.
//! - **Blockierende Aufrufe** laufen auf dem Blocking-Pool (`spawn_blocking`).
//! - **Fehler** werden unverändert als `std::io::Error` weitergereicht; es gibt kein
//!   stilles Wiederholen (insbesondere kein Retry nach fsync-Fehler).

use contextra_ports::VfsFile;
use std::io::{Error, ErrorKind, Result, SeekFrom};
use std::sync::Arc;

/// Größe des Lese-Puffers in Bytes.
const READ_CHUNK: usize = 64 * 1024;

/// Async-Handle auf eine WAL-Datei hinter dem `Vfs`-Port.
pub(crate) struct WalFile {
    inner: Arc<dyn VfsFile>,
    /// Logische Leseposition.
    pos: u64,
    /// Lese-Puffer; `buf[0]` liegt bei Dateioffset `buf_start`.
    buf: Vec<u8>,
    buf_start: u64,
}

impl std::fmt::Debug for WalFile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WalFile").field("pos", &self.pos).finish()
    }
}

/// Führt `op` auf dem Blocking-Pool aus und bildet Join-Fehler auf `io::Error` ab.
#[cfg(not(loom))]
async fn blocking<T, F>(inner: &Arc<dyn VfsFile>, op: F) -> Result<T>
where
    T: Send + 'static,
    F: FnOnce(&dyn VfsFile) -> Result<T> + Send + 'static,
{
    let inner = Arc::clone(inner);
    tokio::task::spawn_blocking(move || op(inner.as_ref()))
        .await
        .map_err(|e| Error::other(format!("WAL file task failed: {e}")))?
}

/// Loom steuert den Scheduler selbst; `spawn_blocking` wäre nicht deterministisch.
#[cfg(loom)]
#[allow(clippy::unused_async)] // allow: Signatur-Parität zur Nicht-Loom-Variante
async fn blocking<T, F>(inner: &Arc<dyn VfsFile>, op: F) -> Result<T>
where
    T: Send + 'static,
    F: FnOnce(&dyn VfsFile) -> Result<T> + Send + 'static,
{
    op(inner.as_ref())
}

/// In-Memory-`VfsFile` für Loom-Modelle (ersetzt den früheren `LoomFile`-Mock).
#[cfg(loom)]
struct LoomMemFile {
    data: std::sync::Mutex<Vec<u8>>,
}

#[cfg(loom)]
impl VfsFile for LoomMemFile {
    fn read_at(&self, buf: &mut [u8], offset: u64) -> Result<usize> {
        let data = self
            .data
            .lock()
            .map_err(|_| Error::other("loom file poisoned"))?;
        let start = usize::try_from(offset)
            .unwrap_or(usize::MAX)
            .min(data.len());
        let n = buf.len().min(data.len() - start);
        buf[..n].copy_from_slice(&data[start..start + n]);
        Ok(n)
    }

    fn write_at(&self, buf: &[u8], offset: u64) -> Result<usize> {
        let mut data = self
            .data
            .lock()
            .map_err(|_| Error::other("loom file poisoned"))?;
        let start = usize::try_from(offset).map_err(|_| Error::other("offset overflow"))?;
        if data.len() < start + buf.len() {
            data.resize(start + buf.len(), 0);
        }
        data[start..start + buf.len()].copy_from_slice(buf);
        Ok(buf.len())
    }

    fn set_len(&self, len: u64) -> Result<()> {
        let len = usize::try_from(len).map_err(|_| Error::other("length overflow"))?;
        self.data
            .lock()
            .map_err(|_| Error::other("loom file poisoned"))?
            .resize(len, 0);
        Ok(())
    }

    fn len(&self) -> Result<u64> {
        Ok(self
            .data
            .lock()
            .map_err(|_| Error::other("loom file poisoned"))?
            .len() as u64)
    }

    fn sync_data(&self) -> Result<()> {
        Ok(())
    }

    fn sync_all(&self) -> Result<()> {
        Ok(())
    }
}

/// Erzeugt die Loom-Datei für `Wal::open_loom`.
#[cfg(loom)]
pub(crate) fn loom_mem_file() -> Box<dyn VfsFile> {
    Box::new(LoomMemFile {
        data: std::sync::Mutex::new(Vec::new()),
    })
}

impl WalFile {
    /// Umschließt ein geöffnetes `VfsFile`.
    pub(crate) fn new(file: Box<dyn VfsFile>) -> Self {
        Self {
            inner: Arc::from(file),
            pos: 0,
            buf: Vec::new(),
            buf_start: 0,
        }
    }

    fn invalidate(&mut self) {
        self.buf.clear();
    }

    /// Liefert die aktuelle physische Dateilänge.
    pub(crate) async fn len(&self) -> Result<u64> {
        blocking(&self.inner, |f| f.len()).await
    }

    /// Hängt alle Bytes ans Dateiende an. Teilschreibungen werden fortgesetzt,
    /// ein Null-Schreibvorgang ergibt `WriteZero`.
    pub(crate) async fn write_all(&mut self, data: &[u8]) -> Result<()> {
        self.invalidate();
        let data = data.to_vec();
        blocking(&self.inner, move |f| {
            let mut rest = data.as_slice();
            while !rest.is_empty() {
                let n = f.append(rest)?;
                if n == 0 {
                    return Err(Error::new(
                        ErrorKind::WriteZero,
                        "failed to write whole WAL buffer",
                    ));
                }
                rest = rest.get(n..).ok_or_else(|| {
                    Error::new(
                        ErrorKind::InvalidData,
                        "append reported more bytes than given",
                    )
                })?;
            }
            Ok(())
        })
        .await
    }

    /// No-op: `WalFile` besitzt keinen Schreibpuffer.
    #[allow(clippy::unused_async)] // allow: API-Parität zu tokio::fs::File, siehe Moduldoku
    pub(crate) async fn flush(&mut self) -> Result<()> {
        Ok(())
    }

    /// `fsync` auf Daten und Metadaten.
    pub(crate) async fn sync_all(&self) -> Result<()> {
        blocking(&self.inner, |f| f.sync_all()).await
    }

    /// Kürzt oder verlängert die Datei auf `len` Bytes.
    pub(crate) async fn set_len(&mut self, len: u64) -> Result<()> {
        self.invalidate();
        blocking(&self.inner, move |f| f.set_len(len)).await
    }

    /// Setzt die Leseposition und liefert die neue absolute Position.
    pub(crate) async fn seek(&mut self, to: SeekFrom) -> Result<u64> {
        let new_pos = match to {
            SeekFrom::Start(n) => n,
            SeekFrom::Current(d) => self.pos.checked_add_signed(d).ok_or_else(|| {
                Error::new(ErrorKind::InvalidInput, "seek before start of WAL file")
            })?,
            SeekFrom::End(d) => {
                let len = self.len().await?;
                len.checked_add_signed(d).ok_or_else(|| {
                    Error::new(ErrorKind::InvalidInput, "seek before start of WAL file")
                })?
            }
        };
        self.pos = new_pos;
        Ok(new_pos)
    }

    /// Liest bis zu `out.len()` Bytes ab der Leseposition. `Ok(0)` bedeutet Dateiende.
    pub(crate) async fn read(&mut self, out: &mut [u8]) -> Result<usize> {
        if out.is_empty() {
            return Ok(0);
        }
        if !self.buffer_covers_pos() {
            let pos = self.pos;
            let chunk = blocking(&self.inner, move |f| {
                let mut chunk = vec![0u8; READ_CHUNK];
                let n = f.read_at(&mut chunk, pos)?;
                chunk.truncate(n);
                Ok(chunk)
            })
            .await?;
            self.buf = chunk;
            self.buf_start = pos;
        }
        let skip = usize::try_from(self.pos - self.buf_start)
            .map_err(|_| Error::new(ErrorKind::InvalidData, "WAL read offset overflow"))?;
        let available = self.buf.get(skip..).unwrap_or_default();
        let n = available.len().min(out.len());
        if let (Some(dst), Some(src)) = (out.get_mut(..n), available.get(..n)) {
            dst.copy_from_slice(src);
        }
        self.pos += n as u64;
        Ok(n)
    }

    /// Liest exakt `out.len()` Bytes oder liefert `UnexpectedEof`.
    pub(crate) async fn read_exact(&mut self, out: &mut [u8]) -> Result<()> {
        let mut filled = 0usize;
        while filled < out.len() {
            let dst = out.get_mut(filled..).ok_or_else(|| {
                Error::new(ErrorKind::InvalidData, "WAL read_exact offset out of range")
            })?;
            let n = self.read(dst).await?;
            if n == 0 {
                return Err(Error::new(
                    ErrorKind::UnexpectedEof,
                    "failed to fill whole WAL buffer",
                ));
            }
            filled += n;
        }
        Ok(())
    }

    /// Prüft, ob der Lese-Puffer die aktuelle Leseposition enthält.
    fn buffer_covers_pos(&self) -> bool {
        let end = self.buf_start + self.buf.len() as u64;
        !self.buf.is_empty() && self.pos >= self.buf_start && self.pos < end
    }
}
