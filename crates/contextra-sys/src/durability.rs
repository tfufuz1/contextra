// FILE-CONTEXT
// STAND: 2026-10-27T15:00:00Z
// ZWECK: Platform-aware low-level durability primitives for file and directory flushes (Ring 0 / SPEC 7.7)
// INVARIANTEN: Zero panic; typed io::Error propagation; fail-closed on unsupported targets; EINTR handling
// SIEHE AUCH: AGENTS.md, docs/SPEC.md §7.7 (INV-DURABILITY-PRIM-1)

//! Platform-aware durability primitives for durable storage flushes (SPEC 7.7).

use std::fs::File;
use std::io::{Error, ErrorKind, Result, Write};
use std::path::Path;

/// Flushes file data and/or metadata to durable physical storage according to platform guarantees (SPEC 7.7).
///
/// - **Linux**: Calls `fdatasync` (if `metadata_changed` is false) or `fsync` (if `metadata_changed` is true).
/// - **macOS**: Calls `fcntl(fd, F_FULLFSYNC, 0)` because plain `fsync` on macOS does not guarantee physical disk flush.
/// - **Windows**: Calls `FlushFileBuffers` (Best-Effort durability on Windows).
/// - **Other POSIX**: Calls `fsync`.
/// - **Unsupported OS**: Returns `ErrorKind::Unsupported` (Fail-Closed).
///
/// Automatically retries if interrupted by `EINTR`.
///
/// # Errors
/// Returns a typed [`std::io::Error`] if flushing fails.
pub fn durable_flush_file(file: &File, metadata_changed: bool) -> Result<()> {
    #[cfg(target_os = "linux")]
    {
        use std::os::unix::io::AsRawFd;
        let fd = file.as_raw_fd();
        loop {
            // SAFETY: `fd` is a valid open file descriptor obtained from `std::fs::File::as_raw_fd()`.
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
            let err = Error::last_os_error();
            if err.kind() != ErrorKind::Interrupted {
                return Err(err);
            }
        }
    }

    #[cfg(target_os = "macos")]
    {
        use std::os::unix::io::AsRawFd;
        let fd = file.as_raw_fd();
        let _ = metadata_changed;
        loop {
            // SAFETY: `fd` is a valid open file descriptor from `std::fs::File`, and `F_FULLFSYNC` forces physical storage sync on macOS.
            let res = unsafe { libc::fcntl(fd, libc::F_FULLFSYNC, 0) };
            if res != -1 {
                return Ok(());
            }
            let err = Error::last_os_error();
            if err.kind() != ErrorKind::Interrupted {
                return Err(err);
            }
        }
    }

    #[cfg(all(unix, not(target_os = "linux"), not(target_os = "macos")))]
    {
        use std::os::unix::io::AsRawFd;
        let fd = file.as_raw_fd();
        let _ = metadata_changed;
        loop {
            // SAFETY: `fd` is a valid open file descriptor from `std::fs::File`.
            let res = unsafe { libc::fsync(fd) };
            if res == 0 {
                return Ok(());
            }
            let err = Error::last_os_error();
            if err.kind() != ErrorKind::Interrupted {
                return Err(err);
            }
        }
    }

    #[cfg(windows)]
    {
        use std::os::windows::io::AsRawHandle;
        let handle = file.as_raw_handle() as windows_sys::Win32::Foundation::HANDLE;
        let _ = metadata_changed;
        // SAFETY: `handle` is a valid OS file handle obtained from `std::fs::File::as_raw_handle()`.
        let res = unsafe { windows_sys::Win32::Storage::FileSystem::FlushFileBuffers(handle) };
        if res != 0 {
            Ok(())
        } else {
            Err(Error::last_os_error())
        }
    }

    #[cfg(not(any(unix, windows)))]
    {
        let _ = (file, metadata_changed);
        Err(Error::new(
            ErrorKind::Unsupported,
            "durable_flush_file is not supported on this target architecture (fail-closed)",
        ))
    }
}

/// Flushes the directory entry (parent directory) containing newly created or renamed files (SPEC 7.7).
///
/// - **Linux / POSIX**: Opens directory read-only and issues `fsync` / `fdatasync` on the directory file descriptor.
/// - **macOS**: Opens directory read-only and issues `fcntl(F_FULLFSYNC)` on the directory file descriptor.
/// - **Windows**: Best-Effort directory sync.
/// - **Unsupported OS**: Returns `ErrorKind::Unsupported` (Fail-Closed).
///
/// # Errors
/// Returns a typed [`std::io::Error`] if opening or flushing the directory fails.
pub fn durable_flush_dir(path: &Path) -> Result<()> {
    #[cfg(unix)]
    {
        let dir = File::open(path)?;
        durable_flush_file(&dir, true)
    }

    #[cfg(windows)]
    {
        let _ = path;
        // Windows directory handles do not support FlushFileBuffers; changes are managed via filesystem semantics (Best Effort).
        Ok(())
    }

    #[cfg(not(any(unix, windows)))]
    {
        let _ = path;
        Err(Error::new(
            ErrorKind::Unsupported,
            "durable_flush_dir is not supported on this target architecture (fail-closed)",
        ))
    }
}

/// Writes the entire slice `buf` to `file` while handling `EINTR` and partial writes safely.
///
/// Retries writing on [`ErrorKind::Interrupted`] until all bytes in `buf` are written or a fatal error occurs.
///
/// # Errors
/// Returns a typed [`std::io::Error`] if a non-interrupted write error occurs or writing zero bytes fails.
pub fn write_all_durable(file: &mut File, mut buf: &[u8]) -> Result<()> {
    while !buf.is_empty() {
        match file.write(buf) {
            Ok(0) => {
                return Err(Error::new(
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::OpenOptions;
    use std::io::Read;

    #[test]
    fn test_write_all_durable_and_flush_file() -> std::io::Result<()> {
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
    fn test_durable_flush_dir() -> std::io::Result<()> {
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
    fn test_write_all_durable_empty_buffer() -> std::io::Result<()> {
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
    fn test_durable_flush_dir_nonexistent_path() {
        let nonexistent = std::path::Path::new("/nonexistent_path_contextra_sys_test_12345");
        #[cfg(unix)]
        {
            assert_eq!(
                durable_flush_dir(nonexistent).err().map(|e| e.kind()),
                Some(std::io::ErrorKind::NotFound)
            );
        }
        #[cfg(windows)]
        {
            let res = durable_flush_dir(nonexistent);
            assert!(res.is_ok());
        }
    }
}
