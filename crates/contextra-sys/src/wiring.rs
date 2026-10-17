//! Facade wiring entry point for contextra-sys (Ring 0) per Rule 7.

use crate::acl_win32::{set_restrictive_file_acl, verify_file_acl_owner_only};
use crate::hardening::{harden_process, ProcessHardeningStatus};
use std::io;
use std::path::Path;

/// Wiring container for system level abstractions and FFI platform primitives.
#[derive(Debug, Clone, Default)]
pub struct SysWiring;

impl SysWiring {
    /// Builds the `SysWiring` instance.
    pub fn build(_deps: ()) -> Result<Self, io::Error> {
        Ok(Self)
    }

    /// Sets restrictive platform ACL/permissions on the specified file path.
    #[inline]
    pub fn set_restrictive_file_acl(&self, path: &Path) -> io::Result<()> {
        set_restrictive_file_acl(path)
    }

    /// Verifies that owner-only restrictive platform ACL/permissions are applied on the path.
    #[inline]
    pub fn verify_file_acl_owner_only(&self, path: &Path) -> io::Result<()> {
        verify_file_acl_owner_only(path)
    }

    /// Hardens current process against core dumps and ptrace attachments.
    #[inline]
    pub fn harden_process(&self) -> io::Result<ProcessHardeningStatus> {
        harden_process()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sys_wiring_build_and_acl() {
        let wiring = SysWiring::build(()).expect("SysWiring build failed");

        let test_file = std::env::temp_dir().join(format!(
            "test_sys_wiring_file_acl_{}.bin",
            std::process::id()
        ));
        std::fs::write(&test_file, b"secret data").expect("write failed");

        let set_res = wiring.set_restrictive_file_acl(&test_file);
        assert!(
            set_res.is_ok(),
            "set_restrictive_file_acl failed: {set_res:?}"
        );

        let verify_res = wiring.verify_file_acl_owner_only(&test_file);
        assert!(
            verify_res.is_ok(),
            "verify_file_acl_owner_only failed: {verify_res:?}"
        );

        let harden_res = wiring.harden_process();
        assert!(harden_res.is_ok(), "harden_process failed: {harden_res:?}");

        let _ = std::fs::remove_file(&test_file);
    }
}
