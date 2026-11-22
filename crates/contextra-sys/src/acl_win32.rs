// FILE-CONTEXT
// STAND: 2026-10-28T00:00:00Z
// ZWECK: Win32 & POSIX ACL restriction and file ownership verification wrappers (Ring 0 / SPEC 7.7)
// INVARIANTEN: 100% safe Rust facade; zero panic; cross-platform file permissions enforcement
// SIEHE AUCH: AGENTS.md, docs/SPEC.md §7.7

//! ACL restriction and file ownership verification low-level system call wrappers.

use std::path::Path;

#[cfg(unix)]
/// Restricts permissions on `path` to owner-only read/write (`0600` / `-rw-------`).
///
/// # Errors
/// Returns an [`std::io::Error`] if changing permissions fails.
pub fn set_restrictive_file_acl(path: &Path) -> std::io::Result<()> {
    use std::os::unix::fs::PermissionsExt;
    let perms = std::fs::Permissions::from_mode(0o600);
    std::fs::set_permissions(path, perms)
}

#[cfg(unix)]
/// Verifies that `path` has owner-only permissions (`0600`) and belongs to the current process owner UID.
///
/// # Errors
/// Returns an [`std::io::Error`] if metadata cannot be read or if group/other bits are set or owner UID mismatches.
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
    // SAFETY: `libc::geteuid` is a safe side-effect-free POSIX syscall returning effective UID of current process.
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
struct TokenGuard(windows_sys::Win32::Foundation::HANDLE);

#[cfg(windows)]
impl Drop for TokenGuard {
    fn drop(&mut self) {
        if !self.0.is_null() {
            // SAFETY: `CloseHandle` safely releases process token handle.
            unsafe {
                windows_sys::Win32::Foundation::CloseHandle(self.0);
            }
        }
    }
}

#[cfg(windows)]
fn get_process_owner_sid_buffer() -> std::io::Result<(TokenGuard, Vec<u8>)> {
    use std::ptr::null_mut;
    use windows_sys::Win32::Foundation::{GetLastError, HANDLE};
    use windows_sys::Win32::Security::{GetTokenInformation, TokenUser, TOKEN_QUERY};
    use windows_sys::Win32::System::Threading::{GetCurrentProcess, OpenProcessToken};

    let mut token_handle: HANDLE = null_mut();
    // SAFETY: Queries process token and owner information from current process.
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
/// Restricts permissions on `path` to process owner only via Win32 DACL.
///
/// # Errors
/// Returns an [`std::io::Error`] if setting security descriptor on file object fails.
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

    // SAFETY: Initializes ACL and attaches access-allowed ACE for owner SID before calling SetNamedSecurityInfoW.
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
/// Verifies that `path` has DACL restricted strictly to current process owner.
///
/// # Errors
/// Returns an [`std::io::Error`] if DACL is unprotected, missing, or SID mismatches.
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

    // SAFETY: Retrieves DACL and security descriptor for path_wide object.
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

    struct SecDescGuard(PSECURITY_DESCRIPTOR);
    impl Drop for SecDescGuard {
        fn drop(&mut self) {
            if !self.0.is_null() {
                // SAFETY: LocalFree frees Windows security descriptor memory.
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

    // SAFETY: Reads control flags, ACE count, and compares owner SID with ACE SID.
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
/// Fallback no-op for non-Unix/non-Windows target architectures.
pub fn set_restrictive_file_acl(path: &Path) -> std::io::Result<()> {
    let _ = path;
    Ok(())
}

#[cfg(not(any(unix, windows)))]
/// Fallback no-op for non-Unix/non-Windows target architectures.
pub fn verify_file_acl_owner_only(path: &Path) -> std::io::Result<()> {
    let _ = path;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

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

    #[test]
    fn test_verify_file_acl_owner_only_nonexistent_file() {
        let nonexistent = Path::new("/nonexistent_acl_test_file_path_1234");
        let res = verify_file_acl_owner_only(nonexistent);
        assert!(res.is_err());
    }

    #[test]
    fn test_set_restrictive_file_acl_nonexistent_file() {
        let nonexistent = Path::new("/nonexistent_acl_test_file_path_5678");
        let res = set_restrictive_file_acl(nonexistent);
        assert!(res.is_err());
    }

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
