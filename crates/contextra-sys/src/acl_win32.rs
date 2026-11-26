// FILE-CONTEXT
// STAND: 2026-10-28T00:00:00Z
// ZWECK: Win32 & POSIX ACL restriction and file ownership verification wrappers (Ring 0 / SPEC 7.7)
// INVARIANTEN: 100% safe Rust facade; zero panic; cross-platform file permissions enforcement
// SIEHE AUCH: AGENTS.md, docs/SPEC.md §7.7

//! # Modul: Zugriffskontrolle und Besitzverifikation für Dateien (`acl_win32`)
//!
//! Dieses Modul implementiert betriebssystemspezifische Sicherheitsprüfungen und Zugriffsbeschränkungen
//! (Access Control Lists = ACL) für sensible Dateien, wie z. B. kryptografische Schlüsseldateien oder
//! Write-Ahead-Logs (WAL).
//!
//! ## Ring-Zugehörigkeit und Zweck
//! - **Ring 0 (Systemnaher Kern):** Kapselt FFI-Aufrufe (Foreign Function Interface) nach POSIX (`libc`)
//!   und Windows (`windows-sys`).
//! - **Sicherheitsziel (Fail-Closed):** Eine Datei mit sensiblen Daten darf unter keinen Umständen von
//!   anderen Benutzern des Betriebssystems gelesen oder modifiziert werden können. Kann dies nicht
//!   garantiert oder verifiziert werden, schlägt die Operation fehl.
//!
//! ## Plattformunterstützung
//! - **Unix/POSIX:** Setzen des Dateimodus auf `0600` (`-rw-------`) und Verifikation, dass keine Gruppen-
//!   oder Welt-Rechte existieren sowie die Besitzer-UID (User ID) exakt mit der effektiven UID des aktuellen
//!   Prozesses übereinstimmt.
//! - **Windows (Win32):** Erstellung und Verifikation einer DACL (Discretionary Access Control List), die
//!   ausschließlich den SID (Security Identifier) des aktuellen Prozesses berechtigt und Vererbung blockiert.
//! - **Andere Plattformen:** Best-Effort-Fallback (No-Op), falls weder Unix noch Windows vorliegen.

use std::path::Path;

#[cfg(unix)]
// ANCHOR[contextra-sys:ACL-SET-UNIX-001]
/// Beschränkt die Zugriffsrechte der angegebenen Datei unter Unix auf den Besitzer (`0600` / `-rw-------`).
///
/// # Ablauf
/// 1. Konvertiert den oktalen Modus `0600` (nur Lese- und Schreibrechte für den Besitzer) in einen Unix-Berechtigungstyp.
/// 2. Wendet die Berechtigungen über `std::fs::set_permissions` auf das Dateisystem an.
///
/// # Vorbedingungen
/// - Der Pfad `path` muss existieren und für den aktuellen Prozess änderbar sein.
///
/// # Nachbedingungen
/// - Nach erfolgreichem Aufruf können andere Benutzer oder Gruppen auf diesem Unix-System die Datei weder lesen noch schreiben.
///
/// # Fehlerfälle
/// Gibt einen [`std::io::Error`] zurück, wenn:
/// - die Datei nicht existiert ([`std::io::ErrorKind::NotFound`]),
/// - die Berechtigungen des aktuellen Prozesses nicht ausreichen ([`std::io::ErrorKind::PermissionDenied`]).
///
/// # Beispiel
/// ```no_run
/// use std::path::Path;
/// use contextra_sys::set_restrictive_file_acl;
///
/// # fn run() -> std::io::Result<()> {
/// let key_file = Path::new("secret.key");
/// set_restrictive_file_acl(key_file)?;
/// # Ok(())
/// # }
/// ```
pub fn set_restrictive_file_acl(path: &Path) -> std::io::Result<()> {
    use std::os::unix::fs::PermissionsExt;
    let perms = std::fs::Permissions::from_mode(0o600);
    std::fs::set_permissions(path, perms)
}

#[cfg(unix)]
// ANCHOR[contextra-sys:ACL-VERIFY-UNIX-001]
/// Verifiziert unter Unix, dass eine Datei strikt auf den Besitzer beschränkt ist (`0600`) und dem aktuellen Prozess gehört.
///
/// # Ablauf
/// 1. Liest die Dateimetadaten über `std::fs::metadata(path)`.
/// 2. Prüft die Modus-Bits: Sind Bits für Gruppe oder Andere gesetzt (`mode & 0o077 != 0`), wird sofort abgebrochen.
/// 3. Ermittelt über den POSIX-Systemaufruf `libc::geteuid()` die effektive Benutzer-ID (UID) des aktuellen Prozesses.
/// 4. Vergleicht die Besitzer-UID der Datei (`meta.uid()`) mit der Prozess-UID. Stimmen sie nicht überein, wird abgelehnt.
///
/// # Fehlerfälle
/// Gibt einen [`std::io::Error`] mit [`std::io::ErrorKind::PermissionDenied`] zurück, wenn:
/// - Rechte für Gruppe oder Welt freigegeben sind,
/// - die Datei einem anderen Benutzer gehört.
///
/// # Nebenwirkungen
/// Führt Dateisystem-Metadaten-I/O und den POSIX-Systemaufruf `geteuid` aus.
pub fn verify_file_acl_owner_only(path: &Path) -> std::io::Result<()> {
    use std::os::unix::fs::MetadataExt;
    let meta = std::fs::metadata(path)?;
    let mode = meta.mode();
    if (mode & 0o077) != 0 {
        return Err(std::io::Error::new(
            std::io::ErrorKind::PermissionDenied,
            format!(
                "File permissions 0{:o} allow group/other access (expected 0600 owner-only)",
                mode & 0o777
            ),
        ));
    }
    let file_uid = meta.uid();
    // SAFETY: `libc::geteuid` ist ein nebenwirkungsfreier POSIX-Systemaufruf, der stets die effektive UID des aufrufenden Prozesses liefert. Es werden keine Zeiger dereferenziert.
    let proc_uid = unsafe { libc::geteuid() };
    if file_uid != proc_uid {
        return Err(std::io::Error::new(
            std::io::ErrorKind::PermissionDenied,
            format!(
                "File owner UID {} does not match current process UID {}",
                file_uid, proc_uid
            ),
        ));
    }
    Ok(())
}

#[cfg(windows)]
// ANCHOR[contextra-sys:ACL-WIN32-TOKEN-001]
/// RAII-Wächter (Resource Acquisition Is Initialization) für Windows-Prozesstoken-Handles (`HANDLE`).
/// Schließt das Handle automatisch beim Verlassen des Gültigkeitsbereichs mittels `CloseHandle`.
struct TokenGuard(windows_sys::Win32::Foundation::HANDLE);

#[cfg(windows)]
impl Drop for TokenGuard {
    fn drop(&mut self) {
        if !self.0.is_null() {
            // SAFETY: `self.0` ist ein gültiges, nicht-nulles Win32-HANDLE, das von `OpenProcessToken` stammt und exklusiv von dieser Guard gehalten wird.
            unsafe {
                windows_sys::Win32::Foundation::CloseHandle(self.0);
            }
        }
    }
}

#[cfg(windows)]
// ANCHOR[contextra-sys:ACL-WIN32-SID-001]
/// Ermittelt die Sicherheits-ID (SID = Security Identifier) des aktuellen Windows-Prozesses.
///
/// # Ablauf
/// 1. Öffnet das Token des aktuellen Prozesses (`OpenProcessToken`) mit der Berechtigung `TOKEN_QUERY`.
/// 2. Fragt mit `GetTokenInformation` zuerst die benötigte Puffergröße für die Benutzer-SID ab.
/// 3. Allokiert einen Bytepuffer in exakt dieser Größe.
/// 4. Liest die Benutzerinformationen (`TokenUser`) in den Puffer ein und gibt ihn zusammen mit dem TokenGuard zurück.
///
/// # Fehlerfälle
/// Gibt einen [`std::io::Error`] zurück, wenn ein Windows-API-Aufruf fehlschlägt oder eine ungültige Pufferlänge liefert.
fn get_process_owner_sid_buffer() -> std::io::Result<(TokenGuard, Vec<u8>)> {
    use std::ptr::null_mut;
    use windows_sys::Win32::Foundation::{GetLastError, HANDLE};
    use windows_sys::Win32::Security::{GetTokenInformation, TokenUser, TOKEN_QUERY};
    use windows_sys::Win32::System::Threading::{GetCurrentProcess, OpenProcessToken};

    let mut token_handle: HANDLE = null_mut();
    // SAFETY: `GetCurrentProcess` liefert ein Pseudohandle. `token_handle` ist ein gültiger Zeiger auf ein HANDLE. `GetTokenInformation` wird zunächst zur Größenermittlung mit Null-Puffer und danach mit einem ausreichend großen Puffer aufgerufen.
    unsafe {
        if OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token_handle) == 0 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("OpenProcessToken failed: Win32 error {}", GetLastError()),
            ));
        }
        let guard = TokenGuard(token_handle);
        let mut len = 0u32;
        GetTokenInformation(token_handle, TokenUser, null_mut(), 0, &mut len);
        if len == 0 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "GetTokenInformation returned 0 length for TokenUser",
            ));
        }
        let mut buffer = vec![0u8; len as usize];
        if GetTokenInformation(
            token_handle,
            TokenUser,
            buffer.as_mut_ptr().cast(),
            len,
            &mut len,
        ) == 0
        {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("GetTokenInformation failed: Win32 error {}", GetLastError()),
            ));
        }
        Ok((guard, buffer))
    }
}

#[cfg(windows)]
// ANCHOR[contextra-sys:ACL-SET-WIN32-001]
/// Beschränkt unter Windows die Zugriffsrechte der angegebenen Datei auf den Prozessbesitzer mittels Win32-DACL.
///
/// # Ablauf
/// 1. Ruft die Besitzer-SID des aktuellen Prozesses über `get_process_owner_sid_buffer()` ab.
/// 2. Ermittelt die Länge der SID via `GetLengthSid`.
/// 3. Allokiert Speicher für die Windows-Zugriffskontrollliste (DACL = Discretionary Access Control List) inklusive eines ACE (Access Control Entry).
/// 4. Initialisiert die ACL mit `InitializeAcl` und fügt mit `AddAccessAllowedAce` einen Eintrag ein, der dem Besitzer Vollzugriff (`GENERIC_ALL`) gewährt.
/// 5. Konvertiert den Dateipfad in UTF-16 (`encode_wide`) mit Null-Terminierung.
/// 6. Wendet den Sicherheitsdeskriptor über `SetNamedSecurityInfoW` an und setzt das Flag `PROTECTED_DACL_SECURITY_INFORMATION`, um Vererbung von übergeordneten Verzeichnissen zu unterbinden.
///
/// # Fehlerfälle
/// Gibt einen [`std::io::Error`] zurück, wenn die SID ungültig ist, der ACL-Puffer nicht initialisiert werden kann oder `SetNamedSecurityInfoW` einen Fehlercode zurückgibt.
pub fn set_restrictive_file_acl(path: &Path) -> std::io::Result<()> {
    use std::os::windows::ffi::OsStrExt;
    use std::ptr::null_mut;
    use windows_sys::Win32::Foundation::{GetLastError, ERROR_SUCCESS, GENERIC_ALL};
    use windows_sys::Win32::Security::Authorization::{SetNamedSecurityInfoW, SE_FILE_OBJECT};
    use windows_sys::Win32::Security::{
        AddAccessAllowedAce, GetLengthSid, InitializeAcl, ACCESS_ALLOWED_ACE, ACL, ACL_REVISION,
        DACL_SECURITY_INFORMATION, PROTECTED_DACL_SECURITY_INFORMATION, TOKEN_USER,
    };

    let (_guard, buffer) = get_process_owner_sid_buffer()?;
    let token_user = buffer.as_ptr() as *const TOKEN_USER;

    // SAFETY: `token_user` zeigt auf den zuvor erfolgreich gelesenen Puffer. Der `owner_sid`-Zeiger wird auf Null geprüft. `acl_buf` ist ausreichend groß dimensioniert für ACL + ACCESS_ALLOWED_ACE + sid_len. `path_wide` ist nullterminiert.
    unsafe {
        let owner_sid = (*token_user).User.Sid;
        if owner_sid.is_null() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Null owner SID in process token",
            ));
        }

        let sid_len = GetLengthSid(owner_sid);
        let acl_size = std::mem::size_of::<ACL>()
            + std::mem::size_of::<ACCESS_ALLOWED_ACE>()
            + sid_len as usize;
        let mut acl_buf = vec![0u8; acl_size];
        let p_acl = acl_buf.as_mut_ptr() as *mut ACL;

        if InitializeAcl(p_acl, acl_size as u32, ACL_REVISION) == 0
            || AddAccessAllowedAce(p_acl, ACL_REVISION, GENERIC_ALL, owner_sid) == 0
        {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Failed to build Win32 ACL: Win32 error {}", GetLastError()),
            ));
        }

        let path_wide: Vec<u16> = path
            .as_os_str()
            .encode_wide()
            .chain(std::iter::once(0))
            .collect();

        let status = SetNamedSecurityInfoW(
            path_wide.as_ptr() as *mut _,
            SE_FILE_OBJECT,
            DACL_SECURITY_INFORMATION | PROTECTED_DACL_SECURITY_INFORMATION,
            null_mut(),
            null_mut(),
            p_acl,
            null_mut(),
        );

        if status != ERROR_SUCCESS {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!(
                    "SetNamedSecurityInfoW failed for {} with error {}",
                    path.display(),
                    status
                ),
            ));
        }
    }

    Ok(())
}

#[cfg(windows)]
// ANCHOR[contextra-sys:ACL-VERIFY-WIN32-001]
/// Verifiziert unter Windows, dass die Datei über eine geschützte DACL verfügt, die ausschließlich dem aktuellen Prozessbesitzer Zugriff gewährt.
///
/// # Ablauf
/// 1. Konvertiert den Pfad in nullterminiertes UTF-16.
/// 2. Liest die DACL und den Sicherheitsdeskriptor über `GetNamedSecurityInfoW`.
/// 3. Kapselt den Sicherheitsdeskriptor in einer RAII-Guard, um ihn mit `LocalFree` zuverlässig freizugeben.
/// 4. Prüft über `GetSecurityDescriptorControl`, ob das Bit `SE_DACL_PROTECTED` gesetzt ist (keine vererbten Berechtigungen von übergeordneten Ordnern).
/// 5. Prüft, ob exakt ein einzelner ACE vorhanden ist (`AceCount == 1`).
/// 6. Vergleicht über `EqualSid` die im ACE berechtigte SID mit der Prozess-SID.
///
/// # Fehlerfälle
/// Schlägt fehl, falls die DACL fehlt, Vererbung erlaubt ist, mehr als ein Benutzer berechtigt ist oder die SID abweicht.
pub fn verify_file_acl_owner_only(path: &Path) -> std::io::Result<()> {
    use std::os::windows::ffi::OsStrExt;
    use std::ptr::null_mut;
    use windows_sys::Win32::Foundation::{GetLastError, ERROR_SUCCESS};
    use windows_sys::Win32::Security::Authorization::{GetNamedSecurityInfoW, SE_FILE_OBJECT};
    use windows_sys::Win32::Security::{
        EqualSid, GetAce, GetSecurityDescriptorControl, ACCESS_ALLOWED_ACE, ACL,
        DACL_SECURITY_INFORMATION, PSECURITY_DESCRIPTOR, SECURITY_DESCRIPTOR_CONTROL,
        SE_DACL_PROTECTED, TOKEN_USER,
    };

    let path_wide: Vec<u16> = path
        .as_os_str()
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();

    let mut p_sec_desc: PSECURITY_DESCRIPTOR = null_mut();
    let mut p_dacl: *mut ACL = null_mut();

    // SAFETY: `path_wide` ist ein gültiger nullterminierter UTF-16-Puffer. Die Ausgabepointer `p_dacl` und `p_sec_desc` sind gültig und werden vom Win32-Subsystem befüllt.
    unsafe {
        let status = GetNamedSecurityInfoW(
            path_wide.as_ptr() as *mut _,
            SE_FILE_OBJECT,
            DACL_SECURITY_INFORMATION,
            null_mut(),
            null_mut(),
            &mut p_dacl,
            null_mut(),
            &mut p_sec_desc,
        );
        if status != ERROR_SUCCESS {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("GetNamedSecurityInfoW failed with error {}", status),
            ));
        }
    }

    // ANCHOR[contextra-sys:ACL-WIN32-SECDESC-001]
    // RAII-Guard: Stellt sicher, dass der von Windows allokierte Sicherheitsdeskriptor über LocalFree freigegeben wird.
    struct SecDescGuard(PSECURITY_DESCRIPTOR);
    impl Drop for SecDescGuard {
        fn drop(&mut self) {
            if !self.0.is_null() {
                // SAFETY: `self.0` ist ein gültiger Zeiger auf einen Windows-Sicherheitsdeskriptor, der von `GetNamedSecurityInfoW` stammt.
                unsafe {
                    windows_sys::Win32::Foundation::LocalFree(self.0 as _);
                }
            }
        }
    }
    let _guard = SecDescGuard(p_sec_desc);

    if p_dacl.is_null() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            "DACL is null",
        ));
    }

    let (_token_guard, buffer) = get_process_owner_sid_buffer()?;
    let token_user = buffer.as_ptr() as *const TOKEN_USER;

    // SAFETY: `p_sec_desc` und `p_dacl` wurden von `GetNamedSecurityInfoW` erfolgreich initialisiert und auf Nicht-Null geprüft. `GetAce` liest den ersten Eintrag (Index 0). `EqualSid` vergleicht zwei gültige SID-Zeiger.
    unsafe {
        let mut control: SECURITY_DESCRIPTOR_CONTROL = 0;
        let mut revision = 0u32;
        if GetSecurityDescriptorControl(p_sec_desc, &mut control, &mut revision) == 0 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("GetSecurityDescriptorControl failed: {}", GetLastError()),
            ));
        }

        if (control & SE_DACL_PROTECTED) == 0 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "DACL inheritance is not protected (SE_DACL_PROTECTED bit not set)",
            ));
        }

        let owner_sid = (*token_user).User.Sid;
        if owner_sid.is_null() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Null owner SID in process token",
            ));
        }

        if (*p_dacl).AceCount != 1 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("DACL ACE count is {}, expected 1", (*p_dacl).AceCount),
            ));
        }

        let mut p_ace: *mut std::ffi::c_void = null_mut();
        if GetAce(p_dacl, 0, &mut p_ace) == 0 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "GetAce failed",
            ));
        }

        let ace = p_ace as *const ACCESS_ALLOWED_ACE;
        let ace_sid = &(*ace).SidStart as *const u32 as *mut std::ffi::c_void;

        if EqualSid(owner_sid, ace_sid) == 0 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "ACE SID does not match process owner SID",
            ));
        }
    }

    Ok(())
}

#[cfg(not(any(unix, windows)))]
// ANCHOR[contextra-sys:ACL-FALLBACK-001]
/// Best-Effort-Fallback für Zielarchitekturen, die weder Unix noch Windows sind (No-Op).
pub fn set_restrictive_file_acl(path: &Path) -> std::io::Result<()> {
    let _ = path;
    Ok(())
}

#[cfg(not(any(unix, windows)))]
// ANCHOR[contextra-sys:ACL-FALLBACK-002]
/// Best-Effort-Fallback für Zielarchitekturen, die weder Unix noch Windows sind (No-Op).
pub fn verify_file_acl_owner_only(path: &Path) -> std::io::Result<()> {
    let _ = path;
    Ok(())
}

// ANCHOR[contextra-sys:ACL-TESTS-001]
#[cfg(test)]
mod tests {
    use super::*;

    // ANCHOR[contextra-sys:ACL-TEST-ROUNDTRIP-001]
    // Prüft den regulären Lebenszyklus: Setzen von 0600-Rechten und anschließende erfolgreiche Verifikation.
    #[test]
    fn test_set_and_verify_file_acl_owner_only() -> std::io::Result<()> {
        let temp_dir = std::env::temp_dir();
        let file_path = temp_dir.join(format!("contextra_sys_acl_test_{}.bin", std::process::id()));
        std::fs::write(&file_path, b"acl payload")?;

        set_restrictive_file_acl(&file_path)?;
        verify_file_acl_owner_only(&file_path)?;

        let _ = std::fs::remove_file(file_path);
        Ok(())
    }

    // ANCHOR[contextra-sys:ACL-TEST-NONEXISTENT-001]
    // Prüft, dass die Verifikation bei einer nicht existierenden Datei wie erwartet fehlschlägt.
    #[test]
    fn test_verify_file_acl_owner_only_nonexistent_file() {
        let nonexistent = Path::new("/nonexistent_acl_test_file_path_1234");
        let res = verify_file_acl_owner_only(nonexistent);
        assert!(res.is_err());
    }

    // ANCHOR[contextra-sys:ACL-TEST-SET-NONEXISTENT-001]
    // Prüft, dass das Setzen von Rechten bei einer nicht existierenden Datei fehlschlägt.
    #[test]
    fn test_set_restrictive_file_acl_nonexistent_file() {
        let nonexistent = Path::new("/nonexistent_acl_test_file_path_5678");
        let res = set_restrictive_file_acl(nonexistent);
        assert!(res.is_err());
    }

    // ANCHOR[contextra-sys:ACL-TEST-PERMISSIVE-001]
    // Prüft den Fail-Closed-Schutz: Zu freizügige Rechte (0666) werden von der Verifikation abgelehnt.
    #[test]
    fn test_verify_file_acl_owner_only_negative_permissions() -> std::io::Result<()> {
        let temp_dir = std::env::temp_dir();
        let file_path = temp_dir.join(format!("contextra_sys_acl_neg_{}.bin", std::process::id()));
        std::fs::write(&file_path, b"permissive payload")?;

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&file_path, std::fs::Permissions::from_mode(0o666))?;
            let res = verify_file_acl_owner_only(&file_path);
            assert!(res.is_err());
        }

        let _ = std::fs::remove_file(file_path);
        Ok(())
    }

    // ANCHOR[contextra-sys:ACL-TEST-EXEC-001]
    // Prüft, dass Ausführungsrechte für Gruppen (0610) zuverlässig erkannt und abgewiesen werden.
    #[test]
    fn test_verify_file_acl_owner_only_group_executable() -> std::io::Result<()> {
        let temp_dir = std::env::temp_dir();
        let file_path = temp_dir.join(format!("contextra_sys_acl_exec_{}.bin", std::process::id()));
        std::fs::write(&file_path, b"exec payload")?;

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&file_path, std::fs::Permissions::from_mode(0o610))?;
            let res = verify_file_acl_owner_only(&file_path);
            assert!(res.is_err());
        }

        let _ = std::fs::remove_file(file_path);
        Ok(())
    }

    // ANCHOR[contextra-sys:ACL-TEST-OTHER-001]
    // Prüft, dass Leserechte für andere Benutzer (0604) zuverlässig abgelehnt werden.
    #[test]
    fn test_verify_file_acl_owner_only_other_read() -> std::io::Result<()> {
        let temp_dir = std::env::temp_dir();
        let file_path = temp_dir.join(format!(
            "contextra_sys_acl_other_{}.bin",
            std::process::id()
        ));
        std::fs::write(&file_path, b"other read payload")?;

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&file_path, std::fs::Permissions::from_mode(0o604))?;
            let res = verify_file_acl_owner_only(&file_path);
            assert!(res.is_err());
        }

        let _ = std::fs::remove_file(file_path);
        Ok(())
    }
}
