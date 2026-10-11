//! Durability primitives, platform-specific flushes, and filesystem classification.

use std::fs::File;
use std::io;
use std::path::Path;

#[cfg(unix)]
use std::os::unix::io::AsRawFd;

#[cfg(windows)]
use std::os::windows::io::AsRawHandle;

/// Specification of the flush scope for file handles.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FlushKind {
    /// Flush file data to stable storage without requiring metadata update synchronization.
    DataOnly,
    /// Flush both file data and metadata changes to stable storage.
    DataAndMetadata,
}

/// Filesystem classification for durability and locking guarantees (§23.4).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FsClass {
    /// Local filesystem with supported POSIX/Win32 durability guarantees (ext4, xfs, btrfs, zfs, APFS, NTFS).
    LocalSupported,
    /// Network filesystem (NFS, SMB, CIFS).
    Network,
    /// User-space filesystem (FUSE).
    Fuse,
    /// Cloud-synchronized directory (OneDrive, Dropbox, iCloud Drive, etc.).
    CloudSync,
    /// Unknown or unclassifiable filesystem.
    Unknown,
}

impl FsClass {
    /// Returns true if the filesystem supports full local persistence and locking guarantees.
    #[must_use]
    pub const fn is_supported_local(&self) -> bool {
        matches!(self, Self::LocalSupported)
    }
}

/// Abstraction trait for platform syscalls to enable deterministic unit testing and Miri simulation.
pub trait SyscallBackend: Send + Sync {
    /// Flushes an open file handle according to `kind`.
    fn flush_file(&self, file: &File, kind: FlushKind) -> io::Result<()>;
    /// Flushes directory entries for `dir`.
    fn flush_dir(&self, dir: &Path) -> io::Result<()>;
    /// Classifies the filesystem containing `path`.
    fn fs_class(&self, path: &Path) -> io::Result<FsClass>;
    /// Atomically renames `from` to `to` with parent directory flush.
    fn rename_durable(&self, from: &Path, to: &Path) -> io::Result<()>;
}

/// System OS implementation of `SyscallBackend`.
#[derive(Debug, Default, Clone, Copy)]
pub struct OsSyscallBackend;

static DEFAULT_BACKEND: OsSyscallBackend = OsSyscallBackend;

/// Flushes an open file to disk using platform durability primitives (INV-DURABILITY-PRIM-1).
pub fn flush_file(file: &File, kind: FlushKind) -> io::Result<()> {
    DEFAULT_BACKEND.flush_file(file, kind)
}

/// Flushes directory metadata entries to stable storage.
pub fn flush_dir(dir: &Path) -> io::Result<()> {
    DEFAULT_BACKEND.flush_dir(dir)
}

/// Classifies the filesystem host of `path` (§23.4).
pub fn fs_class(path: &Path) -> io::Result<FsClass> {
    DEFAULT_BACKEND.fs_class(path)
}

/// Durably renames `from` to `to` and flushes the parent directory.
pub fn rename_durable(from: &Path, to: &Path) -> io::Result<()> {
    DEFAULT_BACKEND.rename_durable(from, to)
}

#[cfg(unix)]
mod posix_ffi {
    use std::os::raw::{c_char, c_int};

    extern "C" {
        pub fn fdatasync(fd: c_int) -> c_int;
        pub fn fsync(fd: c_int) -> c_int;
        #[allow(dead_code)]
        pub fn fcntl(fd: c_int, cmd: c_int, arg: c_int) -> c_int;
        pub fn statfs(path: *const c_char, buf: *mut libc::statfs) -> c_int;
    }
}

#[cfg(windows)]
mod win32_ffi {
    use std::ffi::c_void;

    pub type HANDLE = *mut c_void;
    pub type BOOL = i32;
    pub type DWORD = u32;
    pub type LPCWSTR = *const u16;

    pub const MOVEFILE_REPLACE_EXISTING: DWORD = 0x0000_0001;
    pub const MOVEFILE_WRITE_THROUGH: DWORD = 0x0000_0008;

    extern "system" {
        pub fn FlushFileBuffers(hFile: HANDLE) -> BOOL;
        pub fn GetDriveTypeW(lpRootPathName: LPCWSTR) -> u32;
        pub fn MoveFileExW(
            lpExistingFileName: LPCWSTR,
            lpNewFileName: LPCWSTR,
            dwFlags: DWORD,
        ) -> BOOL;
    }
}

fn is_cloud_sync_path(path: &Path) -> bool {
    let path_str = path.to_string_lossy().to_lowercase();
    path_str.contains("onedrive")
        || path_str.contains("dropbox")
        || path_str.contains("icloud")
        || path_str.contains("mobile documents")
        || path_str.contains("google drive")
        || path_str.contains("googledrive")
        || path_str.contains("box sync")
        || path_str.contains("owncloud")
        || path_str.contains("nextcloud")
}

#[cfg(target_os = "linux")]
fn os_fs_class(path: &Path) -> io::Result<FsClass> {
    use std::ffi::CString;
    use std::os::unix::ffi::OsStrExt;

    let c_path = CString::new(path.as_os_str().as_bytes())
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e.to_string()))?;

    // SAFETY: `stat` is allocated on stack and zero-initialized.
    let mut stat: libc::statfs = unsafe { std::mem::zeroed() };
    // SAFETY: `c_path` is a valid NUL-terminated CString and `stat` points to memory.
    let res = unsafe { posix_ffi::statfs(c_path.as_ptr(), &mut stat) };
    if res != 0 {
        return Err(io::Error::last_os_error());
    }

    // Cast signed statfs f_type (e.g., __SWORD_TYPE / i64) bitwise to u64 for magic comparison.
    #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
    let f_type = (stat.f_type as u64) & 0xFFFF_FFFF;

    match f_type {
        0xEF53 | 0x5846_5342 | 0x9123_683E | 0x2FC1_2FC1 | 0x0102_1994 | 0x794C_7604 => {
            Ok(FsClass::LocalSupported)
        }
        0x6969 | 0xFF53_4D42 | 0x517B => Ok(FsClass::Network),
        0x6573_5546 => Ok(FsClass::Fuse),
        _ => Ok(FsClass::Unknown),
    }
}

#[cfg(target_os = "macos")]
fn os_fs_class(path: &Path) -> io::Result<FsClass> {
    use std::ffi::CString;
    use std::os::unix::ffi::OsStrExt;

    let c_path = CString::new(path.as_os_str().as_bytes())
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e.to_string()))?;

    // SAFETY: `stat` is allocated on stack and zero-initialized.
    let mut stat: libc::statfs = unsafe { std::mem::zeroed() };
    // SAFETY: `c_path` is a valid NUL-terminated CString and `stat` points to memory.
    let res = unsafe { posix_ffi::statfs(c_path.as_ptr(), &mut stat) };
    if res != 0 {
        return Err(io::Error::last_os_error());
    }

    // SAFETY: `stat.f_fstypename` is a valid NUL-terminated C string buffer provided by statfs.
    let fstype_bytes = unsafe { std::ffi::CStr::from_ptr(stat.f_fstypename.as_ptr()) }.to_bytes();

    let fstype = String::from_utf8_lossy(fstype_bytes).to_lowercase();
    match fstype.as_str() {
        "apfs" | "hfs" | "zfs" | "ufs" => Ok(FsClass::LocalSupported),
        "nfs" | "smbfs" | "afpfs" => Ok(FsClass::Network),
        "osxfuse" | "macfuse" | "fuse" | "sshfs" => Ok(FsClass::Fuse),
        _ => Ok(FsClass::Unknown),
    }
}

#[cfg(windows)]
fn os_fs_class(path: &Path) -> io::Result<FsClass> {
    use std::os::windows::ffi::OsStrExt;

    let abs_path = match std::fs::canonicalize(path) {
        Ok(p) => p,
        Err(_) => path.to_path_buf(),
    };
    let path_str = abs_path.to_string_lossy();
    let clean_str = if let Some(stripped) = path_str.strip_prefix(r"\\?\UNC\") {
        format!(r"\\{}", stripped)
    } else if let Some(stripped) = path_str.strip_prefix(r"\\?\") {
        stripped.to_string()
    } else {
        path_str.to_string()
    };

    let clean_path = Path::new(&clean_str);
    let root: Vec<u16> = clean_path
        .ancestors()
        .last()
        .map(|p| {
            p.as_os_str()
                .encode_wide()
                .chain(std::iter::once(0))
                .collect()
        })
        .unwrap_or_default();

    if root.is_empty() {
        return Ok(FsClass::Unknown);
    }

    // SAFETY: `root` is a valid null-terminated UTF-16 string pointer.
    let drive_type = unsafe { win32_ffi::GetDriveTypeW(root.as_ptr()) };

    match drive_type {
        2 | 3 => Ok(FsClass::LocalSupported),
        4 => Ok(FsClass::Network),
        _ => Ok(FsClass::Unknown),
    }
}

#[cfg(not(any(target_os = "linux", target_os = "macos", windows)))]
fn os_fs_class(_path: &Path) -> io::Result<FsClass> {
    Ok(FsClass::Unknown)
}

impl SyscallBackend for OsSyscallBackend {
    fn flush_file(&self, file: &File, kind: FlushKind) -> io::Result<()> {
        #[cfg(target_os = "linux")]
        {
            let fd = file.as_raw_fd();
            let res = match kind {
                FlushKind::DataOnly => {
                    // SAFETY: `fd` is a valid open file descriptor from `file`.
                    unsafe { posix_ffi::fdatasync(fd) }
                }
                FlushKind::DataAndMetadata => {
                    // SAFETY: `fd` is a valid open file descriptor from `file`.
                    unsafe { posix_ffi::fsync(fd) }
                }
            };
            if res == 0 {
                Ok(())
            } else {
                Err(io::Error::last_os_error())
            }
        }

        #[cfg(target_os = "macos")]
        {
            let _ = kind;
            let fd = file.as_raw_fd();
            const F_FULLFSYNC: std::os::raw::c_int = 48;
            // SAFETY: `fd` is a valid open file descriptor from `file`.
            let res = unsafe { posix_ffi::fcntl(fd, F_FULLFSYNC, 0) };
            if res == 0 {
                Ok(())
            } else {
                // INV-DURABILITY-PRIM-1: Error MUST NOT fall back silently to fsync.
                Err(io::Error::last_os_error())
            }
        }

        #[cfg(windows)]
        {
            let _ = kind;
            let handle = file.as_raw_handle();
            // SAFETY: `handle` is a valid open Win32 file handle from `file`.
            let res = unsafe { win32_ffi::FlushFileBuffers(handle as win32_ffi::HANDLE) };
            if res != 0 {
                Ok(())
            } else {
                Err(io::Error::last_os_error())
            }
        }

        #[cfg(not(any(target_os = "linux", target_os = "macos", windows)))]
        {
            let _ = kind;
            file.sync_all()
        }
    }

    fn flush_dir(&self, dir: &Path) -> io::Result<()> {
        #[cfg(unix)]
        {
            let file = File::open(dir)?;
            let fd = file.as_raw_fd();
            // SAFETY: `fd` is a valid open directory descriptor for `dir`.
            let res = unsafe { posix_ffi::fsync(fd) };
            if res == 0 {
                Ok(())
            } else {
                Err(io::Error::last_os_error())
            }
        }

        #[cfg(windows)]
        {
            if !dir.exists() {
                return Err(io::Error::new(
                    io::ErrorKind::NotFound,
                    "directory not found",
                ));
            }
            Ok(())
        }

        #[cfg(not(any(unix, windows)))]
        {
            let _ = dir;
            Ok(())
        }
    }

    fn fs_class(&self, path: &Path) -> io::Result<FsClass> {
        if is_cloud_sync_path(path) {
            Ok(FsClass::CloudSync)
        } else {
            os_fs_class(path)
        }
    }

    fn rename_durable(&self, from: &Path, to: &Path) -> io::Result<()> {
        let parent = to.parent().ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                "target path has no parent directory",
            )
        })?;

        #[cfg(not(windows))]
        {
            std::fs::rename(from, to)?;
            self.flush_dir(parent)?;
        }

        #[cfg(windows)]
        {
            use std::os::windows::ffi::OsStrExt;

            let from_wide: Vec<u16> = from
                .as_os_str()
                .encode_wide()
                .chain(std::iter::once(0))
                .collect();
            let to_wide: Vec<u16> = to
                .as_os_str()
                .encode_wide()
                .chain(std::iter::once(0))
                .collect();

            // SAFETY: `from_wide` and `to_wide` are null-terminated UTF-16 wide strings.
            let res = unsafe {
                win32_ffi::MoveFileExW(
                    from_wide.as_ptr(),
                    to_wide.as_ptr(),
                    win32_ffi::MOVEFILE_REPLACE_EXISTING | win32_ffi::MOVEFILE_WRITE_THROUGH,
                )
            };

            if res == 0 {
                return Err(io::Error::last_os_error());
            }
            self.flush_dir(parent)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicBool, Ordering};

    #[derive(Default)]
    struct MockSyscallBackend {
        fail_flush: AtomicBool,
        class_to_return: std::sync::Mutex<Option<FsClass>>,
    }

    impl SyscallBackend for MockSyscallBackend {
        fn flush_file(&self, _file: &File, _kind: FlushKind) -> io::Result<()> {
            if self.fail_flush.load(Ordering::Relaxed) {
                Err(io::Error::other("simulated flush failure"))
            } else {
                Ok(())
            }
        }

        fn flush_dir(&self, _dir: &Path) -> io::Result<()> {
            if self.fail_flush.load(Ordering::Relaxed) {
                Err(io::Error::other("simulated dir flush failure"))
            } else {
                Ok(())
            }
        }

        fn fs_class(&self, _path: &Path) -> io::Result<FsClass> {
            let guard = self
                .class_to_return
                .lock()
                .map_err(|_| io::Error::other("mutex poisoned"))?;
            if let Some(cls) = *guard {
                Ok(cls)
            } else {
                Ok(FsClass::LocalSupported)
            }
        }

        fn rename_durable(&self, from: &Path, to: &Path) -> io::Result<()> {
            std::fs::rename(from, to)?;
            Ok(())
        }
    }

    #[test]
    fn test_mock_backend_miri_friendly() -> io::Result<()> {
        let mock = MockSyscallBackend::default();
        let temp_dir = std::env::temp_dir();
        let file_path = temp_dir.join(format!("mock_test_{}.tmp", std::process::id()));
        let file = File::create(&file_path)?;

        mock.flush_file(&file, FlushKind::DataOnly)?;
        assert_eq!(mock.fs_class(&file_path)?, FsClass::LocalSupported);

        mock.fail_flush.store(true, Ordering::Relaxed);
        assert!(mock.flush_file(&file, FlushKind::DataAndMetadata).is_err());

        drop(file);
        match std::fs::remove_file(file_path) {
            Ok(()) => {}
            Err(_err) => {}
        }
        Ok(())
    }

    #[test]
    fn test_cloud_sync_heuristics() -> io::Result<()> {
        let path1 = Path::new("/Users/dev/Library/CloudStorage/OneDrive-Corp/db.bin");
        let path2 = Path::new("C:\\Users\\dev\\Dropbox\\data\\log.wal");
        let path3 = Path::new("/Users/dev/Library/Mobile Documents/com~apple~CloudDocs/doc");

        assert_eq!(fs_class(path1)?, FsClass::CloudSync);
        assert_eq!(fs_class(path2)?, FsClass::CloudSync);
        assert_eq!(fs_class(path3)?, FsClass::CloudSync);
        Ok(())
    }

    #[test]
    fn test_flush_kind_equality() {
        assert_eq!(FlushKind::DataOnly, FlushKind::DataOnly);
        assert_ne!(FlushKind::DataOnly, FlushKind::DataAndMetadata);
    }

    #[test]
    fn test_os_flush_file_and_dir() -> io::Result<()> {
        let temp_dir = std::env::temp_dir();
        let file_path = temp_dir.join(format!(
            "contextra_sys_durability_test_{}.bin",
            std::process::id()
        ));

        let file = File::create(&file_path)?;
        flush_file(&file, FlushKind::DataOnly)?;
        flush_file(&file, FlushKind::DataAndMetadata)?;
        drop(file);

        flush_dir(&temp_dir)?;

        let class = fs_class(&file_path)?;
        assert!(matches!(
            class,
            FsClass::LocalSupported | FsClass::Unknown | FsClass::CloudSync
        ));

        match std::fs::remove_file(file_path) {
            Ok(()) => {}
            Err(_err) => {}
        }
        Ok(())
    }
}
