//! POSIX system call abstractions for test and low-level I/O operations.

use std::path::Path;

/// Opens `path` with `flags` (e.g. O_RDONLY or O_RDWR|O_APPEND), then replaces `target_fd` with the new file descriptor via `dup2`.
///
/// Returns `Ok(())` on success, or an `io::Error` on failure.
#[cfg(target_os = "linux")]
pub fn reopen_and_dup2(path: &Path, target_fd: i32, flags: i32) -> std::io::Result<()> {
    use std::ffi::CString;
    use std::os::unix::ffi::OsStrExt;

    let path_c = CString::new(path.as_os_str().as_bytes())
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidInput, e.to_string()))?;

    // SAFETY:
    // 1. `path_c` is a valid NUL-terminated C string.
    // 2. `open`, `dup2`, and `close` are standard POSIX syscall wrappers.
    // 3. `target_fd` is a valid file descriptor in the current process.
    unsafe {
        let fd = libc::open(path_c.as_ptr(), flags);
        if fd < 0 {
            return Err(std::io::Error::last_os_error());
        }
        let dup_res = libc::dup2(fd, target_fd);
        let close_res = libc::close(fd);
        if dup_res < 0 {
            return Err(std::io::Error::last_os_error());
        }
        if close_res < 0 {
            return Err(std::io::Error::last_os_error());
        }
    }
    Ok(())
}

#[cfg(not(target_os = "linux"))]
pub fn reopen_and_dup2(_path: &Path, _target_fd: i32, _flags: i32) -> std::io::Result<()> {
    Err(std::io::Error::new(
        std::io::ErrorKind::Unsupported,
        "reopen_and_dup2 is only supported on Linux target OS",
    ))
}
