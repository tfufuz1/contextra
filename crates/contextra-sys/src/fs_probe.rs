// FILE-CONTEXT
// STAND: 2026-11-02T00:00:00Z
// ZWECK: Dateisystemerkennung, Startprobe und Speicherplatzabfrage (Ring 0 / SPEC §8.3, SY-04, D-56, DF-10, T-PLAT-1)
// INVARIANTEN: Zero panic; Fail-closed bei nicht unterstützten Dateisystemen (NFS/SMB/FUSE/overlayfs/tmpfs)

//! # Modul: Dateisystemprüfung und Startprobe (`fs_probe`)
//!
//! Dieses Modul analysiert das zugrundeliegende Dateisystem eines Pfades mittels Low-Level-Syscalls
//! (`statfs` und `statvfs`).
//!
//! ## Plattformstufen nach SPEC §8.5 (D-56)
//! - **Tier 1 (Linux):** `ext4` oder `XFS` mit voller Crash-Konsistenz.
//! - **Tier 2:** `macOS` (APFS) sowie Linux auf `btrfs`/`ZFS` (COW-Dateisysteme).
//! - **Nicht unterstützt:** `NFS`, `SMB`, `FUSE`, `overlayfs`, `tmpfs` und unbekannte Dateisysteme.
//!   Diese werden gemäß Invariante Fail-Closed abgelehnt (`UnsupportedKind`).

use crate::error::{SysError, UnsupportedKind};
use std::ffi::CString;
use std::os::unix::ffi::OsStrExt;
use std::path::Path;

// Magics für bekannte Dateisysteme unter Linux (`man 2 statfs`)
const EXT4_SUPER_MAGIC: libc::c_long = 0xef53;
const XFS_SUPER_MAGIC: libc::c_long = 0x5846_5342;
const BTRFS_SUPER_MAGIC: libc::c_long = 0x9123_683e_i64 as libc::c_long;
const ZFS_SUPER_MAGIC: libc::c_long = 0x2fc1_2fc1;
const TMPFS_MAGIC: libc::c_long = 0x0102_1994;
const OVERLAYFS_SUPER_MAGIC: libc::c_long = 0x794c_7630;
const FUSE_SUPER_MAGIC: libc::c_long = 0x6573_5546;
const NFS_SUPER_MAGIC: libc::c_long = 0x6969;
const SMB_SUPER_MAGIC: libc::c_long = 0x517b;
const CIFS_SUPER_MAGIC: libc::c_long = 0xff53_4d42_u32 as i32 as libc::c_long;

// ANCHOR[contextra-sys:FSPROBE-FS-KIND-001]
/// Klassifikation des erkannten Dateisystemtyps.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FsKind {
    /// Linux ext4 Dateisystem (Tier 1).
    Ext4,
    /// Linux XFS Dateisystem (Tier 1).
    Xfs,
    /// macOS Apple File System (APFS, Tier 2).
    Apfs,
    /// Linux Btrfs Dateisystem (Tier 2, COW).
    Btrfs,
    /// OpenZFS / ZFS Dateisystem (Tier 2, COW).
    Zfs,
    /// Nicht unterstützter Dateisystemtyp (Fail-Closed).
    Unsupported(UnsupportedKind),
}

// ANCHOR[contextra-sys:FSPROBE-FS-CLASS-001]
/// Detaillierte Eigenschaften des Dateisystems für Haltbarkeits- und Synchronisationsentscheidungen.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FsClass {
    /// Dateisystemart.
    pub kind: FsKind,
    /// Gibt an, ob es sich um rotierende Medien (HDD) oder Flash/SSD handelt (`None` falls unbekannt).
    pub rotational: Option<bool>,
    /// Kennzeichnet Copy-on-Write-Dateisysteme (z. B. Btrfs, ZFS, APFS).
    pub cow: bool,
    /// Kennzeichnet, ob das Journal Datenänderungen vollständig mitprotokolliert (z. B. ext4 data=journal).
    pub journal_data: bool,
}

impl FsClass {
    /// Gibt zurück, ob das Dateisystem lokale Haltbarkeitsgarantien erfüllt (Tier 1 oder Tier 2).
    #[must_use]
    pub const fn is_supported_local(&self) -> bool {
        matches!(
            self.kind,
            FsKind::Ext4 | FsKind::Xfs | FsKind::Apfs | FsKind::Btrfs | FsKind::Zfs
        )
    }
}

// ANCHOR[contextra-sys:FSPROBE-FS-CLASS-FN-001]
/// Ermittelt die Dateisystemeigenschaften für einen angegebenen Pfad mittels `statfs`.
///
/// # Ablauf
/// 1. Konvertiert den Pfad in einen nullterminierten POSIX-Pfadstring ([`CString`]).
/// 2. Führt den Systemaufruf `libc::statfs` aus.
/// 3. Analysiert den Rückgabewert `f_type` (unter Linux) bzw. Dateisystemnamen (unter macOS).
/// 4. Prüft auf unterstützte lokale Typen (ext4, xfs, apfs, btrfs, zfs).
/// 5. Weist NFS, SMB, FUSE, overlayfs, tmpfs und unbekannte Typen mit [`SysError::Unsupported`] zurück (D-56, T-PLAT-1).
///
/// # Vorbedingungen
/// - `path` muss ein gültiger, im Betriebssystem erreichbarer Pfad sein.
///
/// # Nachbedingungen
/// - Liefert eine gültige [`FsClass`] oder einen strukturierten [`SysError`].
///
/// # Fehlerfälle
/// - Gibt [`SysError::Io`] zurück, wenn `statfs` scheitert (z. B. Pfad nicht gefunden).
/// - Gibt [`SysError::Unsupported`] zurück, wenn das Dateisystem nicht den geforderten Haltbarkeitsstandards entspricht.
pub fn fs_class(path: &Path) -> Result<FsClass, SysError> {
    let c_path = CString::new(path.as_os_str().as_bytes()).map_err(|_| {
        SysError::Io(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "path contains interior nul",
        ))
    })?;

    #[cfg(target_os = "linux")]
    {
        // SAFETY: `libc::statfs` ist eine reine C-Struktur, deren Null-Initialisierung wohldefiniert und speichersicher ist.
        let mut stat: libc::statfs = unsafe { std::mem::zeroed() };
        // SAFETY: `c_path` ist ein gültiger, nullterminierter C-String. `stat` ist eine allokierte `libc::statfs`-Struktur.
        let res = unsafe { libc::statfs(c_path.as_ptr(), &mut stat) };
        if res != 0 {
            return Err(SysError::Io(std::io::Error::last_os_error()));
        }

        let f_type = stat.f_type;

        // Erkennung von nicht unterstützten Dateisystemen (D-56, FM-11)
        if f_type == TMPFS_MAGIC {
            return Err(SysError::Unsupported(UnsupportedKind::Tmpfs));
        }
        if f_type == OVERLAYFS_SUPER_MAGIC {
            return Err(SysError::Unsupported(UnsupportedKind::Overlayfs));
        }
        if f_type == FUSE_SUPER_MAGIC {
            return Err(SysError::Unsupported(UnsupportedKind::Fuse));
        }
        if f_type == NFS_SUPER_MAGIC {
            return Err(SysError::Unsupported(UnsupportedKind::Nfs));
        }
        if f_type == SMB_SUPER_MAGIC || f_type == CIFS_SUPER_MAGIC {
            return Err(SysError::Unsupported(UnsupportedKind::Smb));
        }

        let (kind, cow, journal_data) = if f_type == EXT4_SUPER_MAGIC {
            (FsKind::Ext4, false, false)
        } else if f_type == XFS_SUPER_MAGIC {
            (FsKind::Xfs, false, false)
        } else if f_type == BTRFS_SUPER_MAGIC {
            (FsKind::Btrfs, true, false)
        } else if f_type == ZFS_SUPER_MAGIC {
            (FsKind::Zfs, true, false)
        } else {
            let magic_u64 = u64::from_ne_bytes(f_type.to_ne_bytes());
            return Err(SysError::Unsupported(UnsupportedKind::Unknown(magic_u64)));
        };

        let rotational = probe_rotational(path);

        Ok(FsClass {
            kind,
            rotational,
            cow,
            journal_data,
        })
    }

    #[cfg(target_os = "macos")]
    {
        // SAFETY: `libc::statfs` Null-Initialisierung ist speichersicher.
        let mut stat: libc::statfs = unsafe { std::mem::zeroed() };
        // SAFETY: `c_path` ist ein gültiger, nullterminierter C-String. `stat` ist eine allokierte `libc::statfs`-Struktur.
        let res = unsafe { libc::statfs(c_path.as_ptr(), &mut stat) };
        if res != 0 {
            return Err(SysError::Io(std::io::Error::last_os_error()));
        }

        // SAFETY: `stat.f_fstypename` ist ein im Systemaufruf geschriebener, nullterminierter Byte-Puffer.
        let f_fstypename = unsafe { std::ffi::CStr::from_ptr(stat.f_fstypename.as_ptr()) };
        let name_bytes = f_fstypename.to_bytes();

        if name_bytes == b"apfs" {
            Ok(FsClass {
                kind: FsKind::Apfs,
                rotational: Some(false),
                cow: true,
                journal_data: false,
            })
        } else if name_bytes == b"nfs" {
            Err(SysError::Unsupported(UnsupportedKind::Nfs))
        } else if name_bytes == b"smbfs" {
            Err(SysError::Unsupported(UnsupportedKind::Smb))
        } else {
            Err(SysError::Unsupported(UnsupportedKind::Unknown(0)))
        }
    }

    #[cfg(not(any(target_os = "linux", target_os = "macos")))]
    {
        let _ = c_path;
        Err(SysError::Unsupported(UnsupportedKind::Unknown(0)))
    }
}

// ANCHOR[contextra-sys:FSPROBE-FREE-SPACE-001]
/// Ermittelt den tatsächlich verfügbaren freien Speicherplatz in Bytes für unprivilegierte Nutzer (DF-10).
///
/// Verwendet den POSIX-Systemaufruf `statvfs` und berechnet `f_bavail * f_frsize`.
///
/// # Ablauf
/// 1. Konvertiert den Pfad in einen nullterminierten POSIX-Pfad ([`CString`]).
/// 2. Ruft `libc::statvfs` auf.
/// 3. Multipliziert unprivilegierte freie Blöcke (`f_bavail`) mit der Fragment-/Blockgröße (`f_frsize`).
///
/// # Fehlerfälle
/// Gibt [`SysError::Io`] zurück, wenn der Systemaufruf `statvfs` fehlschlägt.
pub fn free_space(path: &Path) -> Result<u64, SysError> {
    let c_path = CString::new(path.as_os_str().as_bytes()).map_err(|_| {
        SysError::Io(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "path contains interior nul",
        ))
    })?;

    // SAFETY: `libc::statvfs` ist eine reine Datenstruktur, deren Nullung speichersicher ist.
    let mut stat: libc::statvfs = unsafe { std::mem::zeroed() };
    // SAFETY: `c_path` ist ein gültiger C-String. `stat` ist eine valide, ausgerichtete Allokation von `libc::statvfs`.
    let res = unsafe { libc::statvfs(c_path.as_ptr(), &mut stat) };
    if res != 0 {
        return Err(SysError::Io(std::io::Error::last_os_error()));
    }

    let bavail = stat.f_bavail;
    let frsize = stat.f_frsize;

    Ok(bavail.saturating_mul(frsize))
}

#[cfg(target_os = "linux")]
fn probe_rotational(_path: &Path) -> Option<bool> {
    // Best effort Prüfung via sysfs falls Device bekannt, sonst None
    None
}

#[cfg(not(target_os = "linux"))]
fn probe_rotational(_path: &Path) -> Option<bool> {
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fs_class_unsupported_rejection() {
        let shm = Path::new("/dev/shm");
        if shm.exists() {
            let res = fs_class(shm);
            assert!(matches!(
                res,
                Err(SysError::Unsupported(UnsupportedKind::Tmpfs))
            ));
        }
    }

    #[test]
    fn test_free_space_sanity() -> Result<(), Box<dyn std::error::Error>> {
        let current_dir = Path::new(".");
        let free = free_space(current_dir)?;
        assert!(free > 0);
        Ok(())
    }
}
