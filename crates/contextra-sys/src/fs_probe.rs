//! # Modul: Dateisystem-Sonden (`fs_probe`)
//!
//! Klassifiziert das Dateisystem eines Pfades fuer die Startprobe gemaess SPEC 8.3 / D-56.

use std::path::Path;

use crate::error::{SysError, UnsupportedKind};

/// Bekannte und unterstuetzte Dateisystemtypen.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FsKind {
    /// Linux Ext4
    Ext4,
    /// Linux XFS
    Xfs,
    /// Apple APFS
    Apfs,
    /// Btrfs
    Btrfs,
    /// OpenZFS / ZFS
    Zfs,
    /// Nicht unterstuetztes oder nicht auditierbares Dateisystem.
    Unsupported(UnsupportedKind),
}

/// Dateisystemklassifikation fuer Durability- und Lock-Garantien (SPEC 8.3 / D-56).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FsClass {
    /// Local filesystem with supported POSIX durability guarantees.
    LocalSupported,
    /// Network filesystem.
    Network,
    /// User-space filesystem.
    Fuse,
    /// Cloud-synchronized directory.
    CloudSync,
    /// Unknown filesystem.
    Unknown,
    /// Detaillierte Klassifikation gemaess Spezifikation 03-contextra-sys.md.
    Detailed {
        /// Dateisystemtyp
        kind: FsKind,
        /// Ob das Medium rotierend ist.
        rotational: Option<bool>,
        /// Copy-on-Write
        cow: bool,
        /// Data Journaling
        journal_data: bool,
    },
}

impl FsClass {
    /// Liefert `true`, wenn das Dateisystem ein vollstaendig unterstuetztes lokales Dateisystem ist.
    #[must_use]
    pub const fn is_supported_local(&self) -> bool {
        match self {
            Self::LocalSupported => true,
            Self::Detailed { kind, .. } => matches!(
                kind,
                FsKind::Ext4 | FsKind::Xfs | FsKind::Apfs | FsKind::Btrfs | FsKind::Zfs
            ),
            _ => false,
        }
    }
}

// Magic constants for Linux statfs (f_type)
#[cfg(target_os = "linux")]
const EXT4_SUPER_MAGIC: libc::__fsword_t = 0xef53;
#[cfg(target_os = "linux")]
const XFS_SUPER_MAGIC: libc::__fsword_t = 0x5846_5342;
#[cfg(target_os = "linux")]
const BTRFS_SUPER_MAGIC: libc::__fsword_t = 0x9123_683e;
#[cfg(target_os = "linux")]
const ZFS_SUPER_MAGIC: libc::__fsword_t = 0x2fc1_2fc1;
#[cfg(target_os = "linux")]
const OVERLAYFS_SUPER_MAGIC: libc::__fsword_t = 0x794c_7630;
#[cfg(target_os = "linux")]
const FUSE_SUPER_MAGIC: libc::__fsword_t = 0x6573_5546;
#[cfg(target_os = "linux")]
const NFS_SUPER_MAGIC: libc::__fsword_t = 0x6969;
#[cfg(target_os = "linux")]
const TMPFS_MAGIC: libc::__fsword_t = 0x0102_1994;
#[cfg(target_os = "linux")]
const RAMFS_MAGIC: libc::__fsword_t = 0x8584_58f6;

/// Klassifiziert das Dateisystem eines gegebenen Pfades.
pub fn fs_class(path: &Path) -> std::io::Result<FsClass> {
    let path_str = path.to_string_lossy().to_lowercase();
    if path_str.contains("onedrive")
        || path_str.contains("dropbox")
        || path_str.contains("icloud")
        || path_str.contains("google drive")
    {
        return Ok(FsClass::CloudSync);
    }

    match fs_class_spec(path) {
        Ok(class) => Ok(class),
        Err(SysError::NotFound) => Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "Path not found",
        )),
        Err(_) => Ok(FsClass::LocalSupported),
    }
}

/// Spezifikationsgemaeße Klassifikation gemaess 03-contextra-sys.md.
pub fn fs_class_spec(path: &Path) -> Result<FsClass, SysError> {
    use std::ffi::CString;
    use std::os::unix::ffi::OsStrExt;

    let path_str = path.to_string_lossy().to_lowercase();
    if path_str.contains("onedrive")
        || path_str.contains("dropbox")
        || path_str.contains("icloud")
        || path_str.contains("google drive")
    {
        return Ok(FsClass::CloudSync);
    }

    let c_path = CString::new(path.as_os_str().as_bytes()).map_err(|_| SysError::InvalidInput)?;

    #[cfg(target_os = "linux")]
    {
        use std::mem::MaybeUninit;
        let mut stat = MaybeUninit::<libc::statfs>::uninit();

        // SAFETY: `c_path` ist ein gueltiger nullterminierter Pfad.
        // `stat.as_mut_ptr()` zeigt auf allokierten Speicher fuer libc::statfs auf dem Stack.
        let ret = unsafe { libc::statfs(c_path.as_ptr(), stat.as_mut_ptr()) };
        if ret != 0 {
            return Err(SysError::from(std::io::Error::last_os_error()));
        }

        // SAFETY: `libc::statfs` lieferte 0, daher hat der Kernel `stat` vollstaendig initialisiert.
        let stat = unsafe { stat.assume_init() };
        let f_type = stat.f_type;

        let kind = match f_type {
            EXT4_SUPER_MAGIC => FsKind::Ext4,
            XFS_SUPER_MAGIC => FsKind::Xfs,
            BTRFS_SUPER_MAGIC => FsKind::Btrfs,
            ZFS_SUPER_MAGIC => FsKind::Zfs,
            OVERLAYFS_SUPER_MAGIC => return Ok(FsClass::LocalSupported),
            FUSE_SUPER_MAGIC => return Ok(FsClass::Fuse),
            NFS_SUPER_MAGIC => return Ok(FsClass::Network),
            TMPFS_MAGIC | RAMFS_MAGIC => FsKind::Unsupported(UnsupportedKind::RamFs),
            _ => return Ok(FsClass::Unknown),
        };

        let cow = matches!(kind, FsKind::Btrfs | FsKind::Zfs);
        let journal_data = matches!(kind, FsKind::Ext4 | FsKind::Xfs);

        if matches!(
            kind,
            FsKind::Ext4 | FsKind::Xfs | FsKind::Btrfs | FsKind::Zfs
        ) {
            Ok(FsClass::LocalSupported)
        } else {
            Ok(FsClass::Detailed {
                kind,
                rotational: None,
                cow,
                journal_data,
            })
        }
    }

    #[cfg(target_os = "macos")]
    {
        use std::mem::MaybeUninit;
        let mut stat = MaybeUninit::<libc::statfs>::uninit();

        // SAFETY: `c_path` ist ein gueltiger nullterminierter Pfad.
        // `stat.as_mut_ptr()` zeigt auf Stack-Speicher passender Groesse.
        let ret = unsafe { libc::statfs(c_path.as_ptr(), stat.as_mut_ptr()) };
        if ret != 0 {
            return Err(SysError::from(std::io::Error::last_os_error()));
        }

        // SAFETY: Rueckgabewert 0 sichert Initialisierung durch macOS Kernel.
        let stat = unsafe { stat.assume_init() };
        let f_fstypename = &stat.f_fstypename;

        let len = f_fstypename
            .iter()
            .position(|&c| c == 0)
            .unwrap_or(f_fstypename.len());
        let type_name_bytes = f_fstypename.get(..len).unwrap_or(&[]);
        let type_name = std::str::from_utf8(
            // SAFETY: In C-String sind alle Bytes als u8 interpretierbar
            unsafe { &*(type_name_bytes as *const [libc::c_char] as *const [u8]) },
        )
        .unwrap_or("");

        match type_name {
            "apfs" | "zfs" => Ok(FsClass::LocalSupported),
            "nfs" | "smbfs" | "afpfs" => Ok(FsClass::Network),
            _ => Ok(FsClass::Unknown),
        }
    }

    #[cfg(all(unix, not(any(target_os = "linux", target_os = "macos"))))]
    {
        let _ = c_path;
        Ok(FsClass::LocalSupported)
    }

    #[cfg(not(unix))]
    {
        let _ = c_path;
        Err(SysError::Unsupported)
    }
}
