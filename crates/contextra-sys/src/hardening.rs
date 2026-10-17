//! Process hardening and dump-prevention abstractions for contextra-sys (Ring 0).
//!
//! Enforces process protection against core dumps and ptrace attachments
//! to prevent sensitive key material leakage to disk or unauthorized debuggers.

use std::io;

/// Status summary report for process hardening operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProcessHardeningStatus {
    /// Indicates whether RLIMIT_CORE was successfully set to zero.
    pub core_dump_disabled: bool,
    /// Indicates whether PR_SET_DUMPABLE was successfully set to zero.
    pub dumpable_disabled: bool,
    /// Indicates whether process hardening is natively supported on the target platform.
    pub platform_supported: bool,
}

/// Hardens the current process by disabling core dumps and setting PR_SET_DUMPABLE = 0.
///
/// On Linux platforms (`cfg(target_os = "linux")`), this sets `RLIMIT_CORE` to `0` via
/// `setrlimit` and disables process dumpability via `prctl(PR_SET_DUMPABLE, 0)`.
/// On non-Linux platforms, this is a safe no-op that reports `platform_supported = false`.
///
/// # Errors
/// Returns an [`std::io::Error`] if any system call fails during hardening on supported platforms.
#[cfg(target_os = "linux")]
pub fn harden_process() -> io::Result<ProcessHardeningStatus> {
    let rlim = libc::rlimit {
        rlim_cur: 0,
        rlim_max: 0,
    };

    // SAFETY: setrlimit passing RLIMIT_CORE with a valid pointer to libc::rlimit struct
    // on Linux is safe and does not modify invalid memory locations.
    let res_rlimit = unsafe { libc::setrlimit(libc::RLIMIT_CORE, &rlim) };
    if res_rlimit != 0 {
        return Err(io::Error::last_os_error());
    }

    // SAFETY: prctl with PR_SET_DUMPABLE and arg2 = 0 is a standard Linux syscall
    // that operates strictly on the calling thread/process flags.
    let res_prctl = unsafe { libc::prctl(libc::PR_SET_DUMPABLE, 0, 0, 0, 0) };
    if res_prctl != 0 {
        return Err(io::Error::last_os_error());
    }

    Ok(ProcessHardeningStatus {
        core_dump_disabled: true,
        dumpable_disabled: true,
        platform_supported: true,
    })
}

/// Hardens the current process on non-Linux platforms (fallback no-op stub).
#[cfg(not(target_os = "linux"))]
pub fn harden_process() -> io::Result<ProcessHardeningStatus> {
    Ok(ProcessHardeningStatus {
        core_dump_disabled: false,
        dumpable_disabled: false,
        platform_supported: false,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_harden_process_execution() {
        let res = harden_process();
        assert!(res.is_ok(), "harden_process failed: {res:?}");
        let status = res.expect("status ok");
        if cfg!(target_os = "linux") {
            assert!(status.platform_supported);
            assert!(status.core_dump_disabled);
            assert!(status.dumpable_disabled);
        } else {
            assert!(!status.platform_supported);
        }
    }
}
