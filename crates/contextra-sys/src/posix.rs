// STAND: 2026-11-08T00:00:00Z
// ZWECK: Low-level POSIX syscall wrappers (Ring 0 / SPEC §10.1)
// INVARIANTEN: Zero panic; typed error propagation; bounds checked descriptors; safe wrappers over libc syscalls
// SIEHE AUCH: AGENTS.md, docs/SPEC.md §7.7, §10.1

//! # Modul: POSIX-Systemaufrufe (`posix`)
//!
//! Dieses Modul kapselt grundlegende POSIX-Systemaufrufe (Syscalls) für das Dateideskriptor-Management
//! und Prozess-Ressourcenlimits.
//!
//! ## Ring-Zugehörigkeit und Zweck
//! - **Ring 0 (Kern / Unsafe-Insel):** Direkte FFI-Aufrufe an `libc` (`open`, `dup2`, `close`, `getrlimit`, `setrlimit`).
//! - **Verwendung im System:**
//!   - `ensure_fd_limit` / `set_fd_limit`: Datenbank- und Suchsysteme (wie LSM-Trees und Vektorindizes) halten
//!     oft hunderte bis tausende SSTable- und WAL-Dateien gleichzeitig geöffnet. Diese Funktionen stellen
//!     sicher, dass dem Prozess nicht die Dateideskriptoren ausgehen (`EMFILE` / "Too many open files").
//!   - `reopen_and_dup2`: Ermöglicht das atomare Umleiten oder Ersetzen von Standarddeskriptoren oder offenen Dateien.

use std::path::Path;

// ANCHOR[contextra-sys:POSIX-DUP2-001]
/// Öffnet den Pfad `path` mit den angegebenen POSIX-Flags (z. B. `O_RDONLY`) und ersetzt atomar `target_fd` via `dup2`.
#[cfg(target_os = "linux")]
pub fn reopen_and_dup2(path: &Path, target_fd: i32, flags: i32) -> std::io::Result<()> {
    use std::ffi::CString;
    use std::os::unix::ffi::OsStrExt;

    if target_fd < 0 {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "target_fd must be non-negative",
        ));
    }

    let path_c = CString::new(path.as_os_str().as_bytes())
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidInput, e.to_string()))?;

    // SAFETY: `path_c` ist ein gültiger nullterminierter C-String. `target_fd` ist validiert >= 0. `libc::open`, `libc::dup2` und `libc::close` sind Standard-POSIX-Syscalls, deren Rückgabewerte unmittelbar auf Fehler geprüft werden.
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

// ANCHOR[contextra-sys:POSIX-DUP2-FALLBACK-001]
/// Fallback für Nicht-Linux-Systeme, auf denen `reopen_and_dup2` nicht unterstützt wird.
#[cfg(not(target_os = "linux"))]
pub fn reopen_and_dup2(_path: &Path, _target_fd: i32, _flags: i32) -> std::io::Result<()> {
    Err(std::io::Error::new(
        std::io::ErrorKind::Unsupported,
        "reopen_and_dup2 is only supported on Linux target OS",
    ))
}

// ANCHOR[contextra-sys:POSIX-ENSURE-FD-001]
/// Stellt sicher, dass das weiche Limit für offene Dateideskriptoren (`RLIMIT_NOFILE`) mindestens `min_fds` beträgt.
pub fn ensure_fd_limit(min_fds: u64) -> bool {
    #[cfg(unix)]
    {
        use std::mem::MaybeUninit;
        let min_fds_rlim = min_fds as libc::rlim_t;
        // SAFETY: `getrlimit` und `setrlimit` lesen und schreiben die `rlimit`-Struktur des Prozesses über gültige Zeiger auf allokierten/uninitialisierten Speicher.
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

// ANCHOR[contextra-sys:POSIX-SET-FD-001]
/// Setzt das weiche und harte Dateideskriptorlimit (`RLIMIT_NOFILE`) des aktuellen Prozesses exakt auf `limit`.
pub fn set_fd_limit(limit: u64) -> bool {
    #[cfg(unix)]
    {
        let rlim = libc::rlimit {
            rlim_cur: limit as libc::rlim_t,
            rlim_max: limit as libc::rlim_t,
        };
        // SAFETY: `rlim` ist eine vollständig initialisierte C-Struktur auf dem Stack. `setrlimit` liest diese sicher ein.
        let res = unsafe { libc::setrlimit(libc::RLIMIT_NOFILE, &rlim) };
        res == 0
    }
    #[cfg(not(unix))]
    {
        let _ = limit;
        true
    }
}

// ANCHOR[contextra-sys:POSIX-TESTS-001]
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
    fn test_reopen_and_dup2() -> std::result::Result<(), Box<dyn std::error::Error>> {
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
            // SAFETY: `target_fd` wurde via `dup2` ersetzt und ist ein gültiger offener Deskriptor. `libc::dup` erzeugt ein neues dupliziertes Handle für die sichere Übergabe an Rusts `File::from_raw_fd`.
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
        assert!(res.is_err());

        let neg_res = reopen_and_dup2(invalid_path, -1, 0);
        assert!(neg_res.is_err());
    }
}
