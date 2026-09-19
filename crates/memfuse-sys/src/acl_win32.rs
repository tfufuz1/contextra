//! Win32 ACL restriction low-level system call wrapper.

use std::path::Path;

#[cfg(windows)]
pub fn set_restrictive_file_acl(path: &Path) -> std::io::Result<()> {
    use std::os::windows::ffi::OsStrExt;
    use std::ptr::null_mut;
    use windows_sys::Win32::Foundation::{
        CloseHandle, GetLastError, ERROR_SUCCESS, GENERIC_ALL, HANDLE,
    };
    use windows_sys::Win32::Security::Authorization::{SetNamedSecurityInfoW, SE_FILE_OBJECT};
    use windows_sys::Win32::Security::{
        AddAccessAllowedAce, GetLengthSid, GetTokenInformation, InitializeAcl, TokenUser,
        ACCESS_ALLOWED_ACE, ACL, ACL_REVISION, DACL_SECURITY_INFORMATION,
        PROTECTED_DACL_SECURITY_INFORMATION, TOKEN_QUERY, TOKEN_USER,
    };
    use windows_sys::Win32::System::Threading::{GetCurrentProcess, OpenProcessToken};

    let mut token_handle: HANDLE = std::ptr::null_mut();
    // SAFETY: OpenProcessToken accesses process handle and writes token_handle output pointer safely.
    let res = unsafe { OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token_handle) };
    if res == 0 {
        let err = unsafe { GetLastError() };
        return Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!(
                "Failed to open process token for ACL restriction: Win32 error {}",
                err
            ),
        ));
    }

    struct TokenGuard(HANDLE);
    impl Drop for TokenGuard {
        fn drop(&mut self) {
            if !self.0.is_null() {
                // SAFETY: CloseHandle closes valid token_handle on drop.
                unsafe { CloseHandle(self.0) };
            }
        }
    }
    let _guard = TokenGuard(token_handle);

    let mut len = 0u32;
    // SAFETY: Querying length buffer required for TokenUser.
    unsafe {
        GetTokenInformation(token_handle, TokenUser, null_mut(), 0, &mut len);
    }

    if len == 0 {
        return Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            "GetTokenInformation returned 0 buffer length for TokenUser",
        ));
    }

    let mut buffer = vec![0u8; len as usize];
    // SAFETY: Passing allocated buffer of size len to GetTokenInformation.
    let res = unsafe {
        GetTokenInformation(
            token_handle,
            TokenUser,
            buffer.as_mut_ptr().cast(),
            len,
            &mut len,
        )
    };
    if res == 0 {
        let err = unsafe { GetLastError() };
        return Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("Failed to retrieve process owner SID: Win32 error {}", err),
        ));
    }

    let token_user = buffer.as_ptr() as *const TOKEN_USER;
    // SAFETY: Dereferencing token_user structure provided by Windows Kernel.
    let owner_sid = unsafe { (*token_user).User.Sid };
    if owner_sid.is_null() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            "Retrieved null owner SID from process token",
        ));
    }

    // SAFETY: GetLengthSid called on valid owner_sid pointer.
    let sid_len = unsafe { GetLengthSid(owner_sid) };
    let acl_size =
        std::mem::size_of::<ACL>() + std::mem::size_of::<ACCESS_ALLOWED_ACE>() + sid_len as usize;

    let mut acl_buf = vec![0u8; acl_size];
    let p_acl = acl_buf.as_mut_ptr() as *mut ACL;

    // SAFETY: InitializeAcl initialises the allocated ACL buffer.
    if unsafe { InitializeAcl(p_acl, acl_size as u32, ACL_REVISION) } == 0 {
        let err = unsafe { GetLastError() };
        return Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("Failed to initialize ACL: Win32 error {}", err),
        ));
    }

    // SAFETY: AddAccessAllowedAce populates the initialized ACL for owner_sid.
    if unsafe { AddAccessAllowedAce(p_acl, ACL_REVISION, GENERIC_ALL, owner_sid) } == 0 {
        let err = unsafe { GetLastError() };
        return Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("Failed to add ACE to ACL: Win32 error {}", err),
        ));
    }

    let path_wide: Vec<u16> = path
        .as_os_str()
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();

    // SAFETY: SetNamedSecurityInfoW sets DACL security information on path_wide file object.
    let status = unsafe {
        SetNamedSecurityInfoW(
            path_wide.as_ptr() as *mut _,
            SE_FILE_OBJECT,
            DACL_SECURITY_INFORMATION | PROTECTED_DACL_SECURITY_INFORMATION,
            null_mut(),
            null_mut(),
            p_acl,
            null_mut(),
        )
    };

    if status != ERROR_SUCCESS {
        return Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!(
                "SetNamedSecurityInfoW failed for {} with Win32 error code {}",
                path.display(),
                status
            ),
        ));
    }

    Ok(())
}

#[cfg(windows)]
pub fn verify_file_acl_owner_only(path: &Path) -> std::io::Result<()> {
    use std::os::windows::ffi::OsStrExt;
    use std::ptr::null_mut;
    use windows_sys::Win32::Foundation::{CloseHandle, GetLastError, ERROR_SUCCESS, HANDLE};
    use windows_sys::Win32::Security::Authorization::{GetNamedSecurityInfoW, SE_FILE_OBJECT};
    use windows_sys::Win32::Security::{
        EqualSid, GetAce, GetSecurityDescriptorControl, GetTokenInformation, TokenUser,
        ACCESS_ALLOWED_ACE, ACL, DACL_SECURITY_INFORMATION, PSECURITY_DESCRIPTOR,
        SECURITY_DESCRIPTOR_CONTROL, SE_DACL_PROTECTED, TOKEN_QUERY, TOKEN_USER,
    };
    use windows_sys::Win32::System::Threading::{GetCurrentProcess, OpenProcessToken};

    let path_wide: Vec<u16> = path
        .as_os_str()
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();

    let mut p_sec_desc: PSECURITY_DESCRIPTOR = null_mut();
    let mut p_dacl: *mut ACL = null_mut();
    let mut control: SECURITY_DESCRIPTOR_CONTROL = 0;
    let mut revision = 0u32;

    // SAFETY: Querying DACL and security descriptor for Windows ACL testing.
    let status = unsafe {
        GetNamedSecurityInfoW(
            path_wide.as_ptr() as *mut _,
            SE_FILE_OBJECT,
            DACL_SECURITY_INFORMATION,
            null_mut(),
            null_mut(),
            &mut p_dacl,
            null_mut(),
            &mut p_sec_desc,
        )
    };
    if status != ERROR_SUCCESS {
        return Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("GetNamedSecurityInfoW failed with error {}", status),
        ));
    }

    struct SecDescGuard(PSECURITY_DESCRIPTOR);
    impl Drop for SecDescGuard {
        fn drop(&mut self) {
            if !self.0.is_null() {
                // SAFETY: LocalFree releases security descriptor memory.
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

    // SAFETY: GetSecurityDescriptorControl checks DACL inheritance protection.
    let status = unsafe { GetSecurityDescriptorControl(p_sec_desc, &mut control, &mut revision) };
    if status == 0 {
        let err = unsafe { GetLastError() };
        return Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("GetSecurityDescriptorControl failed: {}", err),
        ));
    }

    if (control & SE_DACL_PROTECTED) == 0 {
        return Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            "DACL inheritance is not protected (SE_DACL_PROTECTED bit not set)",
        ));
    }

    let mut token_handle: HANDLE = null_mut();
    // SAFETY: OpenProcessToken gets token for current process.
    let res = unsafe { OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token_handle) };
    if res == 0 {
        return Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            "OpenProcessToken failed",
        ));
    }

    let mut len = 0u32;
    // SAFETY: GetTokenInformation query size.
    unsafe {
        GetTokenInformation(token_handle, TokenUser, null_mut(), 0, &mut len);
    }
    let mut buffer = vec![0u8; len as usize];
    // SAFETY: GetTokenInformation retrieves TokenUser.
    let res = unsafe {
        GetTokenInformation(
            token_handle,
            TokenUser,
            buffer.as_mut_ptr().cast(),
            len,
            &mut len,
        )
    };
    if res == 0 {
        return Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            "GetTokenInformation failed",
        ));
    }
    // SAFETY: CloseHandle closes process token handle.
    unsafe { CloseHandle(token_handle) };

    let token_user = buffer.as_ptr() as *const TOKEN_USER;
    // SAFETY: Accessing User.Sid.
    let owner_sid = unsafe { (*token_user).User.Sid };
    if owner_sid.is_null() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            "Null owner SID",
        ));
    }

    // SAFETY: Accessing AceCount.
    let ace_count = unsafe { (*p_dacl).AceCount };
    if ace_count != 1 {
        return Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("DACL ACE count is {}, expected 1", ace_count),
        ));
    }

    let mut p_ace: *mut std::ffi::c_void = null_mut();
    // SAFETY: GetAce retrieves ACE at index 0.
    let res = unsafe { GetAce(p_dacl, 0, &mut p_ace) };
    if res == 0 {
        return Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            "GetAce failed",
        ));
    }

    let ace = p_ace as *const ACCESS_ALLOWED_ACE;
    // SAFETY: Casting SidStart to SID pointer.
    let ace_sid = unsafe { &(*ace).SidStart as *const u32 as *mut std::ffi::c_void };

    // SAFETY: EqualSid compares process owner SID with ACE SID.
    let same_sid = unsafe { EqualSid(owner_sid, ace_sid) };
    if same_sid == 0 {
        return Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            "ACE SID does not match process owner SID",
        ));
    }

    Ok(())
}

#[cfg(not(windows))]
pub fn verify_file_acl_owner_only(path: &Path) -> std::io::Result<()> {
    let _ = path;
    Ok(())
}

#[cfg(not(windows))]
pub fn set_restrictive_file_acl(path: &Path) -> std::io::Result<()> {
    let _ = path;
    Ok(())
}
