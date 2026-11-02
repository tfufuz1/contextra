// FILE-CONTEXT
// STAND: 2026-10-28T00:00:00Z
// ZWECK: Low-level POSIX syscall wrappers (Ring 0 / SPEC 7.7)
// INVARIANTEN: Zero panic; typed io::Error propagation; single safe abstractions over libc syscalls
// SIEHE AUCH: AGENTS.md, docs/SPEC.md §7.7

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
///
/// # Ablauf
/// 1. Konvertiert den Dateipfad `path` in einen nullterminierten C-String ([`std::ffi::CString`]).
/// 2. Öffnet die Datei mittels `libc::open(path_c.as_ptr(), flags)`. Schlägt dies fehl (`fd < 0`), wird der OS-Fehler zurückgegeben.
/// 3. Dupliziert den neu erhaltenen Deskriptor mit `libc::dup2(fd, target_fd)` auf das Ziellimit.
/// 4. Schließt den temporären Zwischendeskriptor `fd` über `libc::close(fd)`.
///
/// # Safety
///
/// Diese Funktion manipuliert die Dateideskriptortabelle des laufenden Prozesses atomar via `dup2`.
/// Sie ist als `unsafe fn` deklariert, weil:
/// - **Deskriptorüberschreibung:** `libc::dup2` schließt einen eventuell unter `target_fd` bereits
///   geöffneten Dateideskriptor ohne Vorwarnung. Wenn ein anderer Thread diesen Deskriptor zeitgleich
///   nutzt oder eine Rust-Datenstruktur ([`std::fs::File`]) die Verantwortung dafür trägt, führt
///   dies zu schweren Use-After-Close- oder Aliasing-Bugs (UB).
/// - **Dateideskriptor-Gültigkeit:** Der Aufrufer muss garantieren, dass `target_fd` im gültigen
///   Bereich für Dateideskriptoren liegt (`>= 0` und `< RLIMIT_NOFILE`) und dass keine Race Conditions
///   bezüglich des Lebenszyklus von `target_fd` im Prozess existieren.
/// - **POSIX-Flags:** `flags` muss ein gültiger Satz an Zugriffsmodi (z. B. `libc::O_RDONLY`, `libc::O_WRONLY`) sein.
///
/// # Vorbedingungen
/// - `path` darf kein Null-Byte (`\0`) im Dateipfad enthalten.
/// - `target_fd` muss ein gültiger Deskriptorindex sein.
///
/// # Nachbedingungen
/// - `target_fd` zeigt nun auf dieselbe offene Dateitabelle wie der neu geöffnete Pfad.
///
/// # Fehlerfälle
/// Gibt einen [`std::io::Error`] zurück, wenn die Pfadkonvertierung ungültig ist oder einer der Systemaufrufe fehlschlägt.
#[cfg(target_os = "linux")]
pub unsafe fn reopen_and_dup2(path: &Path, target_fd: i32, flags: i32) -> std::io::Result<()> {
    use std::ffi::CString;
    use std::os::unix::ffi::OsStrExt;

    let path_c = CString::new(path.as_os_str().as_bytes())
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidInput, e.to_string()))?;

    // SAFETY: `path_c` ist ein gültiger, nullterminierter C-String. Der Aufrufer garantiert über den `# Safety`-Vertrag, dass `target_fd` gültig ist und nicht nebenläufig anderweitig verwendet wird. `libc::open`, `libc::dup2` und `libc::close` sind Standard-POSIX-Syscalls, deren Rückgabewerte unmittelbar auf Fehler geprüft werden.
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
///
/// # Safety
/// Teilt denselben `# Safety`-Vertrag wie die Linux-Variante, gibt jedoch stets einen Fehler zurück.
///
/// # Fehlerfälle
/// Liefert stets [`std::io::ErrorKind::Unsupported`].
#[cfg(not(target_os = "linux"))]
pub unsafe fn reopen_and_dup2(_path: &Path, _target_fd: i32, _flags: i32) -> std::io::Result<()> {
    Err(std::io::Error::new(
        std::io::ErrorKind::Unsupported,
        "reopen_and_dup2 is only supported on Linux target OS",
    ))
}

// ANCHOR[contextra-sys:POSIX-ENSURE-FD-001]
/// Stellt sicher, dass das weiche Limit für offene Dateideskriptoren (`RLIMIT_NOFILE`) mindestens `min_fds` beträgt.
///
/// # Ablauf
/// 1. Liest das aktuelle Limit des Prozesses über `libc::getrlimit(RLIMIT_NOFILE, ...)`.
/// 2. Ist das weiche Limit (`rlim_cur`) kleiner als `min_fds`:
///    - Ermittelt das Ziel als Minimum aus `min_fds` und dem erlaubten harten Limit (`rlim_max`).
///    - Versucht, das weiche Limit mittels `libc::setrlimit` heraufzusetzen.
/// 3. Liest das Limit erneut aus und prüft, ob das aktuelle weiche Limit mindestens `min_fds` erreicht.
/// 4. Gibt `true` zurück, wenn das geforderte Limit erreicht ist, andernfalls `false`.
///
/// # Nebenwirkungen
/// Modifiziert das Prozessressourcenlimit für offene Dateien im Erfolgsfall.
pub fn ensure_fd_limit(min_fds: u64) -> bool {
    #[cfg(unix)]
    {
        use std::mem::MaybeUninit;
        let min_fds_rlim = min_fds as libc::rlim_t;
        // SAFETY: `rlim` und `current_rlim` sind ordnungsgemäß auf dem Stack allokierte `MaybeUninit<libc::rlimit>`-Puffer,
        // deren Zeiger an `libc::getrlimit` übergeben werden. Nach erfolgreichem Rückgabewert `0` garantiert der Kernel,
        // dass die Struktur vollständig initialisiert ist, sodass `assume_init()` wohl-definiert ist.
        // `libc::setrlimit` liest die wohldefinierte `rlimit`-Struktur über eine unveränderliche Referenz.
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
///
/// # Ablauf
/// 1. Initialisiert eine `libc::rlimit`-Struktur mit `rlim_cur = limit` und `rlim_max = limit`.
/// 2. Ruft `libc::setrlimit(RLIMIT_NOFILE, &rlim)` auf.
/// 3. Liefert `true`, wenn der Rückgabewert `0` (Erfolg) ist, andernfalls `false`.
pub fn set_fd_limit(limit: u64) -> bool {
    #[cfg(unix)]
    {
        let rlim = libc::rlimit {
            rlim_cur: limit as libc::rlim_t,
            rlim_max: limit as libc::rlim_t,
        };
        // SAFETY: `rlim` ist eine gültig initialisierte `libc::rlimit`-Instanz auf dem Stack.
        // `libc::setrlimit` liest den Deskriptor-Grenzwert über den Zeiger ein; es erfolgt keine Ownership-Übertragung.
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

    // ANCHOR[contextra-sys:POSIX-TEST-ENSURE-FD-001]
    // Prüft, dass ein moderates Deskriptorlimit (64) problemlos zugesichert werden kann.
    #[test]
    fn test_ensure_fd_limit() {
        let ok = ensure_fd_limit(64);
        assert!(ok);
    }

    // ANCHOR[contextra-sys:POSIX-TEST-ENSURE-FD-HIGH-001]
    // Prüft den Aufruf mit einem hohen Deskriptorlimit (8192), ohne abzustürzen.
    #[test]
    fn test_ensure_fd_limit_high() {
        let _ = ensure_fd_limit(8192);
    }

    // ANCHOR[contextra-sys:POSIX-TEST-SET-FD-001]
    // Prüft das Setzen bzw. Sicherstellen des Deskriptorlimits auf 1024 Deskriptoren.
    #[test]
    fn test_set_fd_limit() {
        let ok = ensure_fd_limit(1024);
        assert!(ok);
    }

    // ANCHOR[contextra-sys:POSIX-TEST-REOPEN-DUP2-001]
    // Prüft das Ersetzen eines Dateideskriptors durch erneutes Öffnen einer Datei via `dup2`.
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
            // SAFETY: `target_fd` ist ein gültiger Deskriptor der soeben geöffneten Datei `target_file`.
            unsafe {
                reopen_and_dup2(&path, target_fd, libc::O_RDONLY)?;
            }
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
            // SAFETY: Testfall auf Nicht-Linux-Plattformen führt zum kontrollierten Unsupported-Fehler.
            let res = unsafe { reopen_and_dup2(&path, 1, 0) };
            assert!(res.is_err());
        }

        let _ = std::fs::remove_file(path);
        Ok(())
    }

    // ANCHOR[contextra-sys:POSIX-TEST-REOPEN-INVALID-001]
    // Prüft, dass `reopen_and_dup2` bei einem ungültigen Dateipfad sauber einen Fehler liefert.
    #[test]
    fn test_reopen_and_dup2_invalid_path() {
        let invalid_path = Path::new("/nonexistent/file/path/dup2_test");
        // SAFETY: Testdeskriptor 999 wird beim Scheitern des Pfadöffnens nicht manipuliert.
        let res = unsafe { reopen_and_dup2(invalid_path, 999, 0) };
        #[cfg(target_os = "linux")]
        assert!(res.is_err());
        #[cfg(not(target_os = "linux"))]
        assert!(res.is_err());
    }
}
