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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::{Read, Write};

    #[test]
    fn test_reopen_and_dup2() {
        let dir = std::env::temp_dir();
        let path = dir.join(format!(
            "contextra_sys_posix_test_{}.bin",
            std::process::id()
        ));
        {
            let mut file = File::create(&path).expect("create file");
            file.write_all(b"posix dup2 test").expect("write");
        }

        #[cfg(target_os = "linux")]
        {
            let dummy_path = dir.join(format!(
                "contextra_sys_posix_dummy_{}.bin",
                std::process::id()
            ));
            let target_file = File::create(&dummy_path).expect("create dummy");
            use std::os::unix::io::AsRawFd;
            let target_fd = target_file.as_raw_fd();
            reopen_and_dup2(&path, target_fd, libc::O_RDONLY).expect("reopen_and_dup2 failed");
            let mut buf = Vec::new();
            // SAFETY: target_fd is valid and open.
            let mut duped_file = unsafe {
                <File as std::os::unix::io::FromRawFd>::from_raw_fd(libc::dup(target_fd))
            };
            duped_file
                .read_to_end(&mut buf)
                .expect("read from duped fd");
            assert_eq!(&buf[..], b"posix dup2 test");
            let _ = std::fs::remove_file(dummy_path);
        }

        #[cfg(not(target_os = "linux"))]
        {
            let res = reopen_and_dup2(&path, 1, 0);
            assert!(res.is_err());
        }

        let _ = std::fs::remove_file(path);
    }
}
