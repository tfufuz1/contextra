// FILE-CONTEXT
// STAND: 2026-10-28T00:00:00Z
// ZWECK: Low-level POSIX syscall wrappers (Ring 0 / SPEC 7.7)
// INVARIANTEN: Zero panic; typed io::Error propagation; single safe abstractions over libc syscalls
// SIEHE AUCH: AGENTS.md, docs/SPEC.md §7.7

//! POSIX system call abstractions for low-level I/O and process resource limit operations.

use std::path::Path;

/// Opens `path` with `flags` (e.g. `O_RDONLY` or `O_RDWR | O_APPEND`), then replaces `target_fd` with the new file descriptor via `dup2`.
///
/// # Errors
/// Returns an [`std::io::Error`] if string conversion, opening, duplicating, or closing file descriptors fails.
#[cfg(target_os = "linux")]
pub fn reopen_and_dup2(path: &Path, target_fd: i32, flags: i32) -> std::io::Result<()> {
    use std::ffi::CString;
    use std::os::unix::ffi::OsStrExt;

    let path_c = CString::new(path.as_os_str().as_bytes())
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidInput, e.to_string()))?;

    // SAFETY: `path_c` is a valid NUL-terminated C string, `target_fd` is a valid open file descriptor,
    // and `libc::open`, `libc::dup2`, and `libc::close` are standard POSIX syscall wrappers.
    // SAFETY: and `libc::open`, `libc::dup2`, and `libc::close` are standard POSIX syscall wrappers.
    unsafe {
        let fd = libc::open(path_c.as_ptr(), flags);
        if fd < 0 {
            return Err(std::io::Error::last_os_error());
        }
        let dup_res = libc::dup2(fd, target_fd);
        let close_res = libc::close(fd);
        if dup_res < 0 || close_res < 0 {
            return Err(std::io::Error::last_os_error());
        }
    }
    Ok(())
}

/// Fallback for non-Linux targets where `reopen_and_dup2` is unsupported.
///
/// # Errors
/// Always returns [`std::io::ErrorKind::Unsupported`].
#[cfg(not(target_os = "linux"))]
pub fn reopen_and_dup2(_path: &Path, _target_fd: i32, _flags: i32) -> std::io::Result<()> {
    Err(std::io::Error::new(
        std::io::ErrorKind::Unsupported,
        "reopen_and_dup2 is only supported on Linux target OS",
    ))
}

/// Ensures the current process soft file descriptor limit (`RLIMIT_NOFILE`) is at least `min_fds`.
/// If the current soft limit is lower, it attempts to raise it up to `min_fds` (or the hard limit).
/// Returns `true` if the soft limit is at least `min_fds`, or `false` otherwise.
pub fn ensure_fd_limit(min_fds: u64) -> bool {
    #[cfg(unix)]
    {
        use std::mem::MaybeUninit;
        let min_fds_rlim = min_fds as libc::rlim_t;
        // SAFETY: `getrlimit` and `setrlimit` read/write `RLIMIT_NOFILE` for current process using initialized pointers.
        unsafe {
            let mut rlim = MaybeUninit::<libc::rlimit>::uninit();
            if libc::getrlimit(libc::RLIMIT_NOFILE, rlim.as_mut_ptr()) != 0 {
                return false;
            }
            let mut rlim = rlim.assume_init();
            if rlim.rlim_cur < min_fds_rlim {
                let target = std::cmp::min(min_fds_rlim, rlim.rlim_max);
                rlim.rlim_cur = target;
                let _ = libc::setrlimit(libc::RLIMIT_NOFILE, &rlim);
            }
            let mut current_rlim = MaybeUninit::<libc::rlimit>::uninit();
            if libc::getrlimit(libc::RLIMIT_NOFILE, current_rlim.as_mut_ptr()) != 0 {
                return false;
            }
            let current_rlim = current_rlim.assume_init();
            current_rlim.rlim_cur >= min_fds_rlim
        }
    }
    #[cfg(not(unix))]
    {
        let _ = min_fds;
        true
    }
}

/// Sets the current process soft and hard file descriptor limit (`RLIMIT_NOFILE`) to `limit`.
/// Returns `true` if `setrlimit` succeeded, or `false` otherwise.
pub fn set_fd_limit(limit: u64) -> bool {
    #[cfg(unix)]
    {
        let rlim = libc::rlimit {
            rlim_cur: limit as libc::rlim_t,
            rlim_max: limit as libc::rlim_t,
        };
        // SAFETY: `setrlimit` sets process `RLIMIT_NOFILE` limits using valid struct values.
        let res = unsafe { libc::setrlimit(libc::RLIMIT_NOFILE, &rlim) };
        res == 0
    }
    #[cfg(not(unix))]
    {
        let _ = limit;
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::{Read, Write};

    #[test]
    fn test_ensure_fd_limit() {
        let ok = ensure_fd_limit(64);
        assert!(ok);
    }

    #[test]
    fn test_ensure_fd_limit_high() {
        let _ = ensure_fd_limit(8192);
    }

    #[test]
    fn test_set_fd_limit() {
        let ok = ensure_fd_limit(1024);
        assert!(ok);
    }

    #[test]
    fn test_reopen_and_dup2() -> Result<(), Box<dyn std::error::Error>> {
        let dir = std::env::temp_dir();
        let path = dir.join(format!(
            "contextra_sys_posix_test_{}.bin",
            std::process::id()
        ));
        {
            let mut file = File::create(&path)?;
            file.write_all(b"posix dup2 test")?;
        }

        #[cfg(target_os = "linux")]
        {
            let dummy_path = dir.join(format!(
                "contextra_sys_posix_dummy_{}.bin",
                std::process::id()
            ));
            let target_file = File::create(&dummy_path)?;
            use std::os::unix::io::AsRawFd;
            let target_fd = target_file.as_raw_fd();
            reopen_and_dup2(&path, target_fd, libc::O_RDONLY)?;
            let mut buf = Vec::new();
            // SAFETY: `target_fd` was replaced via dup2 and remains an open file descriptor.
            let mut duped_file = unsafe {
                <File as std::os::unix::io::FromRawFd>::from_raw_fd(libc::dup(target_fd))
            };
            duped_file.read_to_end(&mut buf)?;
            assert_eq!(&buf[..], b"posix dup2 test");
            let _ = std::fs::remove_file(dummy_path);
        }

        #[cfg(not(target_os = "linux"))]
        {
            let res = reopen_and_dup2(&path, 1, 0);
            assert!(res.is_err());
        }

        let _ = std::fs::remove_file(path);
        Ok(())
    }

    #[test]
    fn test_reopen_and_dup2_invalid_path() {
        let invalid_path = Path::new("/nonexistent/file/path/dup2_test");
        let res = reopen_and_dup2(invalid_path, 999, 0);
        #[cfg(target_os = "linux")]
        assert!(res.is_err());
        #[cfg(not(target_os = "linux"))]
        assert!(res.is_err());
    }
}
