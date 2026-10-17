//! Environment observation and runtime process hardening for Ring 0 system interfaces (§C8, B16).
//!
//! # Observation vs Guarantee (INV-D10)
//!
//! **IMPORTANT**: Environment observation provides empirical system state inspection
//! for evidence generation. Residual risks (CoW filesystems, TRIM/discard, block encryption,
//! active swap, core dump generation) are reported as **observed**, NEVER as **guaranteed**.
//! Software cannot guarantee physical SSD remanence erasure or underlying firmware/hardware
//! behavior.
//!
//! Process hardening (`harden_runtime`, e.g. setting `RLIMIT_CORE=0` or `PR_SET_DUMPABLE=0`)
//! is available via [`harden_runtime`] and returns an explicit per-measure result report
//! ([`HardenRuntimeReport`]).
//!
//! # Copy-On-Write (CoW) Evaluation Rules
//!
//! | Filesystem Kind | CoW Evaluation | Notes |
//! |---|---|---|
//! | `Btrfs` | `Yes` | Native Copy-on-Write filesystem |
//! | `Zfs` | `Yes` | Native Copy-on-Write filesystem |
//! | `Apfs` | `Yes` | Native Copy-on-Write filesystem |
//! | `Ext4` | `No` | In-place overwrite filesystem |
//! | `Tmpfs` | `No` | RAM-backed non-CoW filesystem |
//! | `Ntfs` | `No` | In-place overwrite filesystem |
//! | `Xfs` | `Yes` / `No` / `Unknown` | Inspects mount options (`reflink` => `Yes`, `noreflink` => `No`, else `Unknown`) |
//! | `Other` / `Unknown` | `Unknown` | Fallback when CoW behavior is indeterminate |
//!
//! # Byte Encoding Table for Engine (B05) TLV Serialization
//!
//! | Type | Variant | u8 Code |
//! |---|---|---|
//! | `EnvTri` | `Unknown` | `0` |
//! | `EnvTri` | `Yes` | `1` |
//! | `EnvTri` | `No` | `2` |
//! | `EnvOsKind` | `Unknown` | `0` |
//! | `EnvOsKind` | `Linux` | `1` |
//! | `EnvOsKind` | `MacOs` | `2` |
//! | `EnvOsKind` | `Windows` | `3` |
//! | `EnvOsKind` | `Other` | `4` |
//! | `EnvFsKind` | `Unknown` | `0` |
//! | `EnvFsKind` | `Ext4` | `1` |
//! | `EnvFsKind` | `Xfs` | `2` |
//! | `EnvFsKind` | `Btrfs` | `3` |
//! | `EnvFsKind` | `Zfs` | `4` |
//! | `EnvFsKind` | `Tmpfs` | `5` |
//! | `EnvFsKind` | `Apfs` | `6` |
//! | `EnvFsKind` | `Ntfs` | `7` |
//! | `EnvFsKind` | `Other` | `8` |

use std::path::Path;

/// Status outcome of an individual runtime hardening measure.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HardeningStatus {
    /// Measure successfully applied.
    Applied,
    /// Measure failed with standard OS errno value.
    Failed {
        /// System error number (errno).
        errno: i32,
    },
    /// Measure is not supported on the target OS or platform architecture.
    NotSupported,
}

/// Comprehensive report for runtime hardening operations (`harden_runtime`).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct HardenRuntimeReport {
    /// Result of restricting process core dump size to 0 (`RLIMIT_CORE`).
    pub rlimit_core: HardeningStatus,
    /// Result of setting process dumpable flag to 0 (`PR_SET_DUMPABLE`).
    pub pr_set_dumpable: HardeningStatus,
}

/// Tri-state observation result.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EnvTri {
    /// State could not be conclusively determined.
    Unknown,
    /// State is confirmed active or present.
    Yes,
    /// State is confirmed inactive or absent.
    No,
}

impl EnvTri {
    /// Returns stable u8 byte code for TLV encoding in higher-tier crates.
    #[must_use]
    pub const fn to_u8(&self) -> u8 {
        match self {
            Self::Unknown => 0,
            Self::Yes => 1,
            Self::No => 2,
        }
    }
}

/// Host operating system classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EnvOsKind {
    /// Unknown or unclassified OS.
    Unknown,
    /// Linux kernel host.
    Linux,
    /// Apple macOS host.
    MacOs,
    /// Microsoft Windows host.
    Windows,
    /// Other Unix or POSIX host.
    Other,
}

impl EnvOsKind {
    /// Returns stable u8 byte code for TLV encoding in higher-tier crates.
    #[must_use]
    pub const fn to_u8(&self) -> u8 {
        match self {
            Self::Unknown => 0,
            Self::Linux => 1,
            Self::MacOs => 2,
            Self::Windows => 3,
            Self::Other => 4,
        }
    }
}

/// Filesystem type classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EnvFsKind {
    /// Unknown or unclassified filesystem.
    Unknown,
    /// Fourth extended filesystem (ext4).
    Ext4,
    /// XFS filesystem.
    Xfs,
    /// Btrfs filesystem.
    Btrfs,
    /// ZFS filesystem.
    Zfs,
    /// Temporary RAM filesystem (tmpfs).
    Tmpfs,
    /// Apple File System (APFS).
    Apfs,
    /// New Technology File System (NTFS).
    Ntfs,
    /// Other recognized filesystem.
    Other,
}

impl EnvFsKind {
    /// Returns stable u8 byte code for TLV encoding in higher-tier crates.
    #[must_use]
    pub const fn to_u8(&self) -> u8 {
        match self {
            Self::Unknown => 0,
            Self::Ext4 => 1,
            Self::Xfs => 2,
            Self::Btrfs => 3,
            Self::Zfs => 4,
            Self::Tmpfs => 5,
            Self::Apfs => 6,
            Self::Ntfs => 7,
            Self::Other => 8,
        }
    }
}

/// Complete empirical environment observation snapshot.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnvObservation {
    /// Host OS classification.
    pub os: EnvOsKind,
    /// Filesystem type classification.
    pub fs_kind: EnvFsKind,
    /// Copy-on-Write behavior evaluation.
    pub cow_fs: EnvTri,
    /// Discard/TRIM mount option enabled.
    pub discard_enabled: EnvTri,
    /// Device-mapper block level encryption active.
    pub block_encrypted: EnvTri,
    /// Storage medium is rotational (HDD vs SSD/NVMe).
    pub rotational: EnvTri,
    /// System swap memory is active.
    pub swap_active: EnvTri,
    /// Core dumps are possible (non-zero size limit).
    pub core_dumps_possible: EnvTri,
}

/// Explicitly applies process-level runtime hardening measures (RLIMIT_CORE=0, PR_SET_DUMPABLE=0).
///
/// Returns a [`HardenRuntimeReport`] containing the outcome of each individual measure.
///
/// # Operating System Behavior
/// - Linux: Sets `RLIMIT_CORE` limit to zero and executes `prctl(PR_SET_DUMPABLE, 0)`.
/// - Non-Linux: Returns [`HardeningStatus::NotSupported`] for platform-unsupported measures.
#[must_use]
pub fn harden_runtime() -> HardenRuntimeReport {
    #[cfg(target_os = "linux")]
    {
        harden_runtime_linux()
    }

    #[cfg(not(target_os = "linux"))]
    {
        HardenRuntimeReport {
            rlimit_core: HardeningStatus::NotSupported,
            pr_set_dumpable: HardeningStatus::NotSupported,
        }
    }
}

#[cfg(target_os = "linux")]
fn harden_runtime_linux() -> HardenRuntimeReport {
    let rlim = libc::rlimit {
        rlim_cur: 0,
        rlim_max: 0,
    };

    // SAFETY:
    // 1. `rlim` is valid and resides on the stack.
    // 2. `libc::RLIMIT_CORE` is a standard Linux resource limit constant.
    let res_rlim = unsafe { libc::setrlimit(libc::RLIMIT_CORE, &rlim) };

    let rlimit_core = if res_rlim == 0 {
        HardeningStatus::Applied
    } else {
        let errno = std::io::Error::last_os_error().raw_os_error().unwrap_or(0);
        HardeningStatus::Failed { errno }
    };

    // SAFETY:
    // 1. `libc::PR_SET_DUMPABLE` is a standard Linux prctl operation.
    // 2. Passing `0` disables core dumps and ptrace attachments.
    let res_prctl = unsafe { libc::prctl(libc::PR_SET_DUMPABLE, 0, 0, 0, 0) };

    let pr_set_dumpable = if res_prctl == 0 {
        HardeningStatus::Applied
    } else {
        let errno = std::io::Error::last_os_error().raw_os_error().unwrap_or(0);
        HardeningStatus::Failed { errno }
    };

    HardenRuntimeReport {
        rlimit_core,
        pr_set_dumpable,
    }
}

/// Feature-detection helper for kernel `memfd_secret` support (Linux >= 5.14).
///
/// **IMPORTANT NOTE ON SECURITY LIMITATIONS**:
/// `memfd_secret` creates secret memory areas hidden from host process memory maps.
/// However, per Linux man-pages (`memfd_secret(2)`), `memfd_secret` does NOT provide 100% protection
/// against direct kernel-space inspection, hardware DMA access, or privileged kernel exploits.
/// It is checked purely as feature detection without mandatory usage enforcement.
#[must_use]
pub fn check_memfd_secret_support() -> EnvTri {
    #[cfg(target_os = "linux")]
    {
        // Check availability via syscall. SYS_memfd_secret syscall ID on Linux.
        #[cfg(target_arch = "x86_64")]
        const SYS_MEMFD_SECRET: libc::c_long = 447;
        #[cfg(target_arch = "aarch64")]
        const SYS_MEMFD_SECRET: libc::c_long = 447;
        #[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
        const SYS_MEMFD_SECRET: libc::c_long = -1;

        if SYS_MEMFD_SECRET < 0 {
            return EnvTri::No;
        }

        // SAFETY:
        // Invokes syscall with invalid flag (0) or check errno.
        // `syscall` with SYS_MEMFD_SECRET creates an anonymous file descriptor.
        // Calling with invalid flags (e.g., 0) returns ENOSYS if missing or EINVAL if supported.
        let res = unsafe { libc::syscall(SYS_MEMFD_SECRET, 0u32) };

        if res >= 0 {
            // Secret fd opened successfully (must be closed)
            // SAFETY: `res` is a valid file descriptor returned by syscall.
            unsafe {
                #[allow(clippy::cast_possible_truncation)]
                libc::close(res as libc::c_int);
            }
            EnvTri::Yes
        } else {
            let errno = std::io::Error::last_os_error().raw_os_error().unwrap_or(0);
            if errno == libc::ENOSYS {
                EnvTri::No
            } else if errno == libc::EINVAL || errno == libc::EPERM {
                // Syscall exists but rejected flags/permissions => kernel supports memfd_secret
                EnvTri::Yes
            } else {
                EnvTri::Unknown
            }
        }
    }

    #[cfg(not(target_os = "linux"))]
    {
        EnvTri::No
    }
}

/// Evaluates Copy-on-Write (`cow_fs`) behavior for a given filesystem type and mount options string.
#[must_use]
pub fn cow_fs_from_fs_kind_and_options(fs_kind: EnvFsKind, options: &str) -> EnvTri {
    match fs_kind {
        EnvFsKind::Btrfs | EnvFsKind::Zfs | EnvFsKind::Apfs => EnvTri::Yes,
        EnvFsKind::Ext4 | EnvFsKind::Tmpfs | EnvFsKind::Ntfs => EnvTri::No,
        EnvFsKind::Xfs => {
            let opts: Vec<&str> = options.split(',').map(str::trim).collect();
            if opts.contains(&"reflink") {
                EnvTri::Yes
            } else if opts.contains(&"noreflink") {
                EnvTri::No
            } else {
                EnvTri::Unknown
            }
        }
        EnvFsKind::Other | EnvFsKind::Unknown => EnvTri::Unknown,
    }
}

/// Pure parser for Linux `/proc/self/mountinfo` format text.
///
/// Returns `(EnvFsKind, discard: EnvTri)` and extracts mount options string for CoW checks.
#[must_use]
pub fn parse_mountinfo_details(
    text: &str,
    mount_point_or_dev: &Path,
) -> (EnvFsKind, EnvTri, String) {
    let target = mount_point_or_dev.to_string_lossy();
    let mut best_match: Option<(&str, &str, &str)> = None;
    let mut max_prefix_len = 0;

    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        // Split into main parts separated by ` - `
        let mut parts = line.split(" - ");
        let left = match parts.next() {
            Some(l) => l,
            None => continue,
        };
        let right = match parts.next() {
            Some(r) => r,
            None => continue,
        };

        let left_fields: Vec<&str> = left.split_whitespace().collect();
        let right_fields: Vec<&str> = right.split_whitespace().collect();

        if left_fields.len() < 6 || right_fields.len() < 2 {
            continue;
        }

        let mount_point = left_fields[4];
        let mount_opts = left_fields[5];
        let fs_type = right_fields[0];
        let mount_source = right_fields[1];
        let super_opts = if right_fields.len() >= 3 {
            right_fields[2]
        } else {
            ""
        };

        // Match by mount source exact or mount point prefix
        if mount_source == target || target.starts_with(mount_source) {
            best_match = Some((fs_type, mount_opts, super_opts));
            break;
        }

        if target.starts_with(mount_point) && mount_point.len() >= max_prefix_len {
            max_prefix_len = mount_point.len();
            best_match = Some((fs_type, mount_opts, super_opts));
        }
    }

    let (fs_type_str, mount_opts, super_opts) = match best_match {
        Some(m) => m,
        None => return (EnvFsKind::Unknown, EnvTri::Unknown, String::new()),
    };

    let fs_kind = match fs_type_str.to_lowercase().as_str() {
        "ext4" => EnvFsKind::Ext4,
        "xfs" => EnvFsKind::Xfs,
        "btrfs" => EnvFsKind::Btrfs,
        "zfs" | "fuse.zfs" => EnvFsKind::Zfs,
        "tmpfs" => EnvFsKind::Tmpfs,
        "apfs" => EnvFsKind::Apfs,
        "ntfs" | "fuseblk" | "ntfs-3g" => EnvFsKind::Ntfs,
        _ => EnvFsKind::Other,
    };

    let combined_opts = format!("{},{}", mount_opts, super_opts);
    let opts_vec: Vec<&str> = combined_opts.split(',').map(str::trim).collect();

    let discard = if opts_vec
        .iter()
        .any(|o| *o == "discard" || o.starts_with("discard="))
    {
        EnvTri::Yes
    } else {
        EnvTri::No
    };

    (fs_kind, discard, combined_opts)
}

/// Pure parser for Linux `/proc/self/mountinfo` format text.
///
/// Returns `(EnvFsKind, discard: EnvTri)`.
#[must_use]
pub fn parse_mountinfo(text: &str, mount_point_or_dev: &Path) -> (EnvFsKind, EnvTri) {
    let (fs_kind, discard, _) = parse_mountinfo_details(text, mount_point_or_dev);
    (fs_kind, discard)
}

/// Pure parser for Linux `/proc/swaps` format text.
///
/// Active swap => `EnvTri::Yes`; no active swap => `EnvTri::No`; malformed => `EnvTri::Unknown`.
#[must_use]
pub fn parse_proc_swaps(text: &str) -> EnvTri {
    let lines: Vec<&str> = text
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty())
        .collect();

    if lines.is_empty() {
        return EnvTri::No;
    }

    let mut has_header = false;
    let mut valid_swap_count = 0;
    let mut malformed_lines = 0;

    for line in lines {
        if line.starts_with("Filename") || line.starts_with("Path") {
            has_header = true;
            continue;
        }

        let fields: Vec<&str> = line.split_whitespace().collect();
        if fields.len() >= 4 {
            let size_ok = fields[2].parse::<u64>().is_ok();
            let used_ok = fields[3].parse::<u64>().is_ok();
            if size_ok && used_ok {
                valid_swap_count += 1;
            } else {
                malformed_lines += 1;
            }
        } else {
            malformed_lines += 1;
        }
    }

    if malformed_lines > 0 {
        EnvTri::Unknown
    } else if valid_swap_count > 0 {
        EnvTri::Yes
    } else if has_header {
        EnvTri::No
    } else {
        EnvTri::Unknown
    }
}

/// Pure parser for Linux `/proc/self/limits` format text.
///
/// Checks RLIMIT_CORE ("Max core file size"). Non-zero limit => `EnvTri::Yes`; limit "0" => `EnvTri::No`.
#[must_use]
pub fn parse_limits(text: &str) -> EnvTri {
    for line in text.lines() {
        let line = line.trim();
        if line.starts_with("Max core file size") {
            let fields: Vec<&str> = line.split_whitespace().collect();
            // Format: Max core file size <soft_limit> <hard_limit> <units>
            if fields.len() >= 5 {
                let soft_limit = fields[4];
                if soft_limit == "0" {
                    return EnvTri::No;
                }
                return EnvTri::Yes;
            }
        }
    }
    EnvTri::Unknown
}

/// Pure parser for Linux `/proc/sys/kernel/core_pattern` text.
#[must_use]
pub fn parse_core_pattern(text: &str) -> EnvTri {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        EnvTri::Unknown
    } else {
        EnvTri::Yes
    }
}

/// Pure parser for device-mapper UUID text (`/sys/dev/block/<maj:min>/dm/uuid`).
///
/// Prefix `CRYPT-` => `EnvTri::Yes`; non-CRYPT prefix => `EnvTri::No`; empty/missing => `EnvTri::Unknown`.
#[must_use]
pub fn parse_dm_uuid(text: &str) -> EnvTri {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        EnvTri::Unknown
    } else if trimmed.starts_with("CRYPT-") {
        EnvTri::Yes
    } else if trimmed.starts_with("LVM-") || trimmed.starts_with("part") || trimmed.contains('-') {
        EnvTri::No
    } else {
        EnvTri::Unknown
    }
}

/// Pure parser for block device rotational status (`/sys/dev/block/<maj:min>/queue/rotational`).
///
/// `"1"` => `EnvTri::Yes` (HDD); `"0"` => `EnvTri::No` (SSD/NVMe); other => `EnvTri::Unknown`.
#[must_use]
pub fn parse_rotational(text: &str) -> EnvTri {
    match text.trim() {
        "1" => EnvTri::Yes,
        "0" => EnvTri::No,
        _ => EnvTri::Unknown,
    }
}

/// Samples empirical environment observation for `path`.
///
/// Never panics. Any unreadable or unparseable source yields `EnvTri::Unknown`.
#[must_use]
pub fn observe(path: &Path) -> EnvObservation {
    #[cfg(target_os = "linux")]
    {
        observe_linux(path)
    }

    #[cfg(not(target_os = "linux"))]
    {
        observe_non_linux(path)
    }
}

#[cfg(target_os = "linux")]
fn observe_linux(path: &Path) -> EnvObservation {
    let os = EnvOsKind::Linux;

    let mountinfo_text = std::fs::read_to_string("/proc/self/mountinfo").unwrap_or_default();
    let (fs_kind, discard_enabled, opts_str) = parse_mountinfo_details(&mountinfo_text, path);
    let cow_fs = cow_fs_from_fs_kind_and_options(fs_kind, &opts_str);

    let swaps_text = std::fs::read_to_string("/proc/swaps").unwrap_or_default();
    let swap_active = parse_proc_swaps(&swaps_text);

    let limits_text = std::fs::read_to_string("/proc/self/limits").unwrap_or_default();
    let limit_tri = parse_limits(&limits_text);

    let core_pat_text =
        std::fs::read_to_string("/proc/sys/kernel/core_pattern").unwrap_or_default();
    let core_pat_tri = parse_core_pattern(&core_pat_text);

    let core_dumps_possible = match (limit_tri, core_pat_tri) {
        (EnvTri::No, _) => EnvTri::No,
        (EnvTri::Yes, EnvTri::Yes) => EnvTri::Yes,
        _ => EnvTri::Unknown,
    };

    let (rotational, block_encrypted) = observe_sysfs_block(path);

    EnvObservation {
        os,
        fs_kind,
        cow_fs,
        discard_enabled,
        block_encrypted,
        rotational,
        swap_active,
        core_dumps_possible,
    }
}

#[cfg(target_os = "linux")]
fn observe_sysfs_block(path: &Path) -> (EnvTri, EnvTri) {
    use std::os::unix::fs::MetadataExt;

    let meta = match std::fs::metadata(path) {
        Ok(m) => m,
        Err(_) => return (EnvTri::Unknown, EnvTri::Unknown),
    };

    let dev_id = meta.dev();
    #[allow(clippy::cast_possible_truncation)]
    let maj = (dev_id >> 8) & 0xfff;
    #[allow(clippy::cast_possible_truncation)]
    let min = (dev_id & 0xff) | ((dev_id >> 12) & 0xfff00);

    let block_dir = format!("/sys/dev/block/{}:{}", maj, min);
    let block_path = Path::new(&block_dir);

    let rotational = {
        let rot_file = block_path.join("queue/rotational");
        if rot_file.exists() {
            std::fs::read_to_string(&rot_file)
                .map(|s| parse_rotational(&s))
                .unwrap_or(EnvTri::Unknown)
        } else {
            let parent_rot = block_path.join("../queue/rotational");
            if parent_rot.exists() {
                std::fs::read_to_string(&parent_rot)
                    .map(|s| parse_rotational(&s))
                    .unwrap_or(EnvTri::Unknown)
            } else {
                EnvTri::Unknown
            }
        }
    };

    let block_encrypted = {
        let dm_file = block_path.join("dm/uuid");
        if dm_file.exists() {
            std::fs::read_to_string(&dm_file)
                .map(|s| parse_dm_uuid(&s))
                .unwrap_or(EnvTri::Unknown)
        } else {
            EnvTri::Unknown
        }
    };

    (rotational, block_encrypted)
}

#[cfg(not(target_os = "linux"))]
fn observe_non_linux(_path: &Path) -> EnvObservation {
    #[cfg(target_os = "macos")]
    let os = EnvOsKind::MacOs;
    #[cfg(windows)]
    let os = EnvOsKind::Windows;
    #[cfg(not(any(target_os = "macos", windows)))]
    let os = EnvOsKind::Other;

    EnvObservation {
        os,
        fs_kind: EnvFsKind::Unknown,
        cow_fs: EnvTri::Unknown,
        discard_enabled: EnvTri::Unknown,
        block_encrypted: EnvTri::Unknown,
        rotational: EnvTri::Unknown,
        swap_active: EnvTri::Unknown,
        core_dumps_possible: EnvTri::Unknown,
    }
}
