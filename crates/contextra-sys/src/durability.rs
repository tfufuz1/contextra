// STAND: 2026-11-08T00:00:00Z
// ZWECK: Platform-aware low-level durability primitives for file and directory flushes (Ring 0 / SPEC §10.1 / INV-SYS-FSYNC-1)
// INVARIANTEN: Zero panic; typed error propagation; fail-closed on unsupported targets; EINTR retry only; Handle-Poisoning on sync failure
// SIEHE AUCH: AGENTS.md, docs/SPEC.md §7.7, §10.1 (INV-SYS-FSYNC-1)

//! # Modul: Dauerhaftigkeit und physische Synchronisation (`durability`)
//!
//! Dieses Modul stellt plattformspezifische Grundfunktionen (Primitiven) zur Verfügung,
//! um sicherzustellen, dass geschriebene Daten tatsächlich auf dem physischen Speichermedium
//! (SSD/HDD) dauerhaft persistiert werden (Durability gemäß ACID-Kriterien).
//!
//! ## Ring-Zugehörigkeit und Zweck
//! - **Ring 0 (Kern):** Kapselt direkte Betriebssystem-Aufrufe (`libc::fsync`, `libc::fdatasync`,
//!   `libc::fcntl(F_FULLFSYNC)`, `FlushFileBuffers`).
//! - **Verwendung im System:** Wird von `contextra-store` (Write-Ahead-Log WAL und LSM-SSTables)
//!   sowie `contextra-durable-fs` genutzt.
//! - **Persistenz & Fehlergarantie (INV-SYS-FSYNC-1):**
//!   Schlägt ein `fsync`/`fdatasync` fehl, wird sofort ein [`crate::Error::SyncFailed`] bzw.
//!   [`std::io::Error`] an den Aufrufer propagiert. Es gibt keinen stillen Retry auf demselben
//!   Handle: Ein fehlerhaftes Handle wird permanent vergiftet ([`DurableFile`]) und alle
//!   nachfolgenden Flush- oder Schreibversuche scheitern sofort fail-closed.
//!
//! ## Plattformunterschiede
//! - **Linux:** Nutzt `fdatasync` (wenn keine Metadaten wie Dateigröße geändert wurden) oder `fsync`.
//! - **macOS:** Standardmäßiges `fsync` flasht auf Apple-Dateisystemen (APFS) oft nur den Betriebssystem-Cache,
//!   nicht jedoch den Festplattencache. Daher wird zwingend `fcntl(fd, F_FULLFSYNC, 0)` verwendet.
//! - **Windows:** Verwendet `FlushFileBuffers`.
//! - **Signalbehandlung:** Aufrufe werden bei Unterbrechung durch Unix-Signale (`EINTR`) automatisch wiederholt,
//!   da `EINTR` anzeigt, dass der Syscall vor der Ausführung unterbrochen wurde.

use std::fs::File;
use std::io::{Error as IoError, ErrorKind, Result as IoResult, Write};
use std::path::Path;

use crate::error::{Error, Result};

#[cfg(test)]
pub(crate) mod fault_injection {
    use std::sync::atomic::{AtomicBool, AtomicI32, Ordering};

    static FAIL_FSYNC: AtomicBool = AtomicBool::new(false);
    static INJECTED_ERRNO: AtomicI32 = AtomicI32::new(libc::EIO);

    pub fn set_fail_fsync(fail: bool, errno: i32) {
        FAIL_FSYNC.store(fail, Ordering::SeqCst);
        INJECTED_ERRNO.store(errno, Ordering::SeqCst);
    }

    pub fn should_fail_fsync() -> Option<i32> {
        if FAIL_FSYNC.load(Ordering::SeqCst) {
            Some(INJECTED_ERRNO.load(Ordering::SeqCst))
        } else {
            None
        }
    }
}

// ANCHOR[contextra-sys:DUR-FLUSH-FILE-RAW-001]
/// Führt den rohen Betriebssystem-Sync-Syscall auf einem Dateideskriptor aus.
///
/// # Invariante (INV-SYS-FSYNC-1)
/// Nur `EINTR` führt zu einer Wiederholung. Jeder andere Fehlercode führt sofort
/// zum Abbruch und Rückgabe von `Error::SyncFailed(errno)`.
#[cfg(unix)]
fn raw_os_sync_file(file: &File, metadata_changed: bool) -> Result<()> {
    #[cfg(test)]
    if let Some(injected_errno) = fault_injection::should_fail_fsync() {
        return Err(Error::SyncFailed(injected_errno));
    }

    use std::os::unix::io::AsRawFd;
    let fd = file.as_raw_fd();

    #[cfg(target_os = "linux")]
    {
        loop {
            // SAFETY: `fd` ist ein gültiger, offener Dateideskriptor aus `std::fs::File::as_raw_fd()`.
            let res = unsafe {
                if metadata_changed {
                    libc::fsync(fd)
                } else {
                    libc::fdatasync(fd)
                }
            };
            if res == 0 {
                return Ok(());
            }
            let err = IoError::last_os_error();
            if err.kind() == ErrorKind::Interrupted {
                continue;
            }
            let errno = err.raw_os_error().unwrap_or(libc::EIO);
            return Err(Error::SyncFailed(errno));
        }
    }

    #[cfg(target_os = "macos")]
    {
        let _ = metadata_changed;
        loop {
            // SAFETY: `fd` ist ein gültiger Dateideskriptor aus `std::fs::File`. F_FULLFSYNC leert physische APFS-Caches.
            let res = unsafe { libc::fcntl(fd, libc::F_FULLFSYNC, 0) };
            if res != -1 {
                return Ok(());
            }
            let err = IoError::last_os_error();
            if err.kind() == ErrorKind::Interrupted {
                continue;
            }
            let errno = err.raw_os_error().unwrap_or(libc::EIO);
            return Err(Error::SyncFailed(errno));
        }
    }

    #[cfg(all(unix, not(target_os = "linux"), not(target_os = "macos")))]
    {
        let _ = metadata_changed;
        loop {
            // SAFETY: `fd` ist ein gültiger offener Dateideskriptor aus `std::fs::File`.
            let res = unsafe { libc::fsync(fd) };
            if res == 0 {
                return Ok(());
            }
            let err = IoError::last_os_error();
            if err.kind() == ErrorKind::Interrupted {
                continue;
            }
            let errno = err.raw_os_error().unwrap_or(libc::EIO);
            return Err(Error::SyncFailed(errno));
        }
    }
}

#[cfg(windows)]
fn raw_os_sync_file(file: &File, _metadata_changed: bool) -> Result<()> {
    #[cfg(test)]
    if let Some(injected_errno) = fault_injection::should_fail_fsync() {
        return Err(Error::SyncFailed(injected_errno));
    }

    use std::os::windows::io::AsRawHandle;
    let handle = file.as_raw_handle() as windows_sys::Win32::Foundation::HANDLE;
    // SAFETY: `handle` ist ein gültiges OS-Datei-Handle aus `std::fs::File::as_raw_handle()`.
    let res = unsafe { windows_sys::Win32::Storage::FileSystem::FlushFileBuffers(handle) };
    if res != 0 {
        Ok(())
    } else {
        let err = IoError::last_os_error();
        let code = err.raw_os_error().unwrap_or(-1);
        Err(Error::SyncFailed(code))
    }
}

#[cfg(not(any(unix, windows)))]
fn raw_os_sync_file(_file: &File, _metadata_changed: bool) -> Result<()> {
    Err(Error::Unsupported(
        "durable_flush_file is not supported on this target architecture (fail-closed)",
    ))
}

// ANCHOR[contextra-sys:DUR-FLUSH-FILE-001]
/// Flasht Dateiinhalt und optional Metadaten auf den physischen Datenträger gemäß den Garantien der Zielplattform.
pub fn durable_flush_file(file: &File, metadata_changed: bool) -> IoResult<()> {
    raw_os_sync_file(file, metadata_changed).map_err(Into::into)
}

// ANCHOR[contextra-sys:DUR-FLUSH-DIR-001]
/// Flasht den Verzeichniseintrag (übergeordnetes Verzeichnis) einer neu erstellten oder umbenannten Datei.
pub fn durable_flush_dir(path: &Path) -> IoResult<()> {
    #[cfg(unix)]
    {
        let dir = File::open(path)?;
        durable_flush_file(&dir, true)
    }

    #[cfg(windows)]
    {
        let _ = path;
        Ok(())
    }

    #[cfg(not(any(unix, windows)))]
    {
        let _ = path;
        Err(IoError::new(
            ErrorKind::Unsupported,
            "durable_flush_dir is not supported on this target architecture (fail-closed)",
        ))
    }
}

// ANCHOR[contextra-sys:DUR-WRITE-ALL-001]
/// Schreibt die gesamten Bytes aus `buf` zuverlässig in `file`, unter robuster Behandlung partieller Schreibvorgänge und Signal-Unterbrechungen.
pub fn write_all_durable(file: &mut File, mut buf: &[u8]) -> IoResult<()> {
    while !buf.is_empty() {
        match file.write(buf) {
            Ok(0) => {
                return Err(IoError::new(
                    ErrorKind::WriteZero,
                    "failed to write whole buffer: write returned 0 bytes",
                ));
            }
            Ok(n) => {
                buf = buf.get(n..).unwrap_or(&[]);
            }
            Err(ref e) if e.kind() == ErrorKind::Interrupted => {
                continue;
            }
            Err(e) => return Err(e),
        }
    }
    Ok(())
}

// ANCHOR[contextra-sys:DUR-DURABLE-FILE-001]
/// Ein sicherer RAII-Wrapper um [`std::fs::File`], der die Invariante **INV-SYS-FSYNC-1** erzwingt:
/// Sobald ein `fsync`/`fdatasync` oder Flush-Aufruf fehlschlägt, wird das Handle permanent vergiftet (`is_poisoned() == true`).
/// Alle nachfolgenden Schreib- und Synchronisationsaufrufe auf dieser Instanz schlagen sofort mit [`Error::HandlePoisoned`] fehl.
#[derive(Debug)]
pub struct DurableFile {
    file: File,
    poisoned: bool,
}

impl DurableFile {
    /// Kapselt eine geöffnete [`File`] in einem gesunden, unvergifteten Zustand.
    pub fn new(file: File) -> Self {
        Self {
            file,
            poisoned: false,
        }
    }

    /// Gibt `true` zurück, wenn das Datei-Handle infolge eines früheren Flush-Fehlers vergiftet ist.
    pub fn is_poisoned(&self) -> bool {
        self.poisoned
    }

    /// Gibt eine Referenz auf die innere [`File`] zurück, sofern das Handle nicht vergiftet ist.
    pub fn get_ref(&self) -> Result<&File> {
        if self.poisoned {
            Err(Error::HandlePoisoned)
        } else {
            Ok(&self.file)
        }
    }

    /// Schreibt Daten dauerhaft in die Datei. Schlägt sofort fehl, wenn das Handle vergiftet ist.
    pub fn write_all(&mut self, buf: &[u8]) -> Result<()> {
        if self.poisoned {
            return Err(Error::HandlePoisoned);
        }
        write_all_durable(&mut self.file, buf).map_err(Error::Io)
    }

    /// Flasht Dateiinhalt und Metadaten auf das physische Speichermedium.
    pub fn sync_all(&mut self) -> Result<()> {
        if self.poisoned {
            return Err(Error::HandlePoisoned);
        }
        match raw_os_sync_file(&self.file, true) {
            Ok(()) => Ok(()),
            Err(err) => {
                self.poisoned = true;
                Err(err)
            }
        }
    }

    /// Flasht den Dateiinhalt (ohne Metadaten, z. B. via `fdatasync` unter Linux).
    pub fn sync_data(&mut self) -> Result<()> {
        if self.poisoned {
            return Err(Error::HandlePoisoned);
        }
        match raw_os_sync_file(&self.file, false) {
            Ok(()) => Ok(()),
            Err(err) => {
                self.poisoned = true;
                Err(err)
            }
        }
    }
}

// ANCHOR[contextra-sys:DUR-TESTS-001]
#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::OpenOptions;
    use std::io::Read;

    #[test]
    fn test_write_all_durable_and_flush_file() -> IoResult<()> {
        let temp_dir = std::env::temp_dir();
        let file_path = temp_dir.join(format!(
            "contextra_sys_durability_test_{}.bin",
            std::process::id()
        ));

        {
            let mut file = OpenOptions::new()
                .write(true)
                .create(true)
                .truncate(true)
                .open(&file_path)?;

            write_all_durable(&mut file, b"contextra durability payload")?;
            durable_flush_file(&file, false)?;
            durable_flush_file(&file, true)?;
        }

        {
            let mut file = File::open(&file_path)?;
            let mut content = Vec::new();
            file.read_to_end(&mut content)?;
            assert_eq!(&content[..], b"contextra durability payload");
        }

        let _ = std::fs::remove_file(file_path);
        Ok(())
    }

    #[test]
    fn test_durable_flush_file_metadata_variations() -> IoResult<()> {
        let temp_dir = std::env::temp_dir();
        let file_path = temp_dir.join(format!(
            "contextra_sys_durability_meta_{}.bin",
            std::process::id()
        ));

        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(&file_path)?;

        write_all_durable(&mut file, b"metadata variation test")?;
        durable_flush_file(&file, false)?;
        durable_flush_file(&file, true)?;

        let _ = std::fs::remove_file(file_path);
        Ok(())
    }

    #[test]
    fn test_durable_flush_dir() -> IoResult<()> {
        let temp_dir = std::env::temp_dir();
        let test_subdir = temp_dir.join(format!(
            "contextra_sys_durability_dir_{}",
            std::process::id()
        ));
        std::fs::create_dir_all(&test_subdir)?;

        let file_path = test_subdir.join("data.txt");
        {
            let mut file = File::create(&file_path)?;
            write_all_durable(&mut file, b"test payload")?;
            durable_flush_file(&file, true)?;
        }

        durable_flush_dir(&test_subdir)?;

        let _ = std::fs::remove_file(file_path);
        let _ = std::fs::remove_dir(test_subdir);
        Ok(())
    }

    #[test]
    fn test_write_all_durable_empty_buffer() -> IoResult<()> {
        let temp_dir = std::env::temp_dir();
        let file_path = temp_dir.join(format!(
            "contextra_sys_empty_test_{}.bin",
            std::process::id()
        ));

        let mut file = File::create(&file_path)?;
        write_all_durable(&mut file, &[])?;
        durable_flush_file(&file, false)?;

        let _ = std::fs::remove_file(file_path);
        Ok(())
    }

    #[test]
    fn test_write_all_durable_chunked() -> IoResult<()> {
        let temp_dir = std::env::temp_dir();
        let file_path = temp_dir.join(format!(
            "contextra_sys_chunk_test_{}.bin",
            std::process::id()
        ));

        let mut file = File::create(&file_path)?;
        let data = vec![0x37u8; 64 * 1024];
        write_all_durable(&mut file, &data)?;
        durable_flush_file(&file, true)?;

        let _ = std::fs::remove_file(file_path);
        Ok(())
    }

    #[test]
    fn test_durable_flush_dir_nonexistent_path() {
        let nonexistent = std::path::Path::new("/nonexistent_path_contextra_sys_test_12345");
        #[cfg(unix)]
        {
            assert_eq!(
                durable_flush_dir(nonexistent).err().map(|e| e.kind()),
                Some(ErrorKind::NotFound)
            );
        }
        #[cfg(windows)]
        {
            let res = durable_flush_dir(nonexistent);
            assert!(res.is_ok());
        }
    }

    // ANCHOR[contextra-sys:INV-SYS-FSYNC-1-TEST-001]
    /// Prüft INV-SYS-FSYNC-1: Ein fehlgeschlagener Flush vergiftet das Handle unwiderruflich.
    #[test]
    fn test_inv_sys_fsync_1_poisoning() -> Result<()> {
        let temp_dir = std::env::temp_dir();
        let file_path = temp_dir.join(format!(
            "contextra_sys_poison_test_{}.bin",
            std::process::id()
        ));

        let raw_file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(true)
            .open(&file_path)
            .map_err(Error::Io)?;

        let mut durable_file = DurableFile::new(raw_file);
        assert!(!durable_file.is_poisoned());

        durable_file.write_all(b"initial healthy write")?;
        durable_file.sync_data()?;
        assert!(!durable_file.is_poisoned());

        fault_injection::set_fail_fsync(true, libc::EIO);

        let sync_res = durable_file.sync_all();
        assert!(sync_res.is_err());
        assert!(
            matches!(sync_res, Err(Error::SyncFailed(code)) if code == libc::EIO),
            "expected SyncFailed(EIO)"
        );

        assert!(durable_file.is_poisoned());

        fault_injection::set_fail_fsync(false, 0);

        let follow_up_sync = durable_file.sync_all();
        assert!(matches!(follow_up_sync, Err(Error::HandlePoisoned)));

        let follow_up_write = durable_file.write_all(b"must fail");
        assert!(matches!(follow_up_write, Err(Error::HandlePoisoned)));

        let follow_up_data = durable_file.sync_data();
        assert!(matches!(follow_up_data, Err(Error::HandlePoisoned)));

        let get_ref = durable_file.get_ref();
        assert!(matches!(get_ref, Err(Error::HandlePoisoned)));

        let _ = std::fs::remove_file(file_path);
        Ok(())
    }
}
