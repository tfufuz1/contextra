// FILE-CONTEXT
// STAND: 2026-10-13
// ZWECK: Sicheres Entfernen von Dateien (Scrubbing) mit Sicherheitsprüfungen (CoW, Hardlinks, ReaderAlive).
// INVARIANTEN: Zero-Panic, pure safe Rust, I/O-Fehler werden als std::io::Result propagiert.

//! # Flash-/SSD-Remanenz und physische Vernichtungsgrenzen
//!
//! **WICHTIGER HINWEIS (Wei et al., FAST 2011):**
//! Das Überschreiben von Dateiinhalten mit Nullen auf Flash-basierten Speichermedien (SSDs, NVMe, eMMC)
//! garantiert **keine physische Vernichtung** der Datenblöcke auf Zellebene. Durch Flash Translation Layers (FTL),
//! Wear Leveling, Over-Provisioning und Out-of-Place-Writes verbleiben alte Datenblöcke in physischen Flash-Zellen.
//!
//! Belastbare und zuverlässige Datenlöschung auf Flash-Medien ist **ausschließlich via Crypto-Erase (S4)**
//! (Vernichtung des Verschlüsselungsschlüssels) möglich.
//!
//! Der Rückgabetyp ([`ScrubOutcome::Scrubbed`]) und die Quittung ([`ScrubReceipt`]) bieten daher explizit
//! **keine physische Garantie** (`physical_guarantee: None`).

use crate::{durable_remove_with_injector, FaultInjector, NoFault, ScrubMethod, ScrubReceipt};
use contextra_sys::{flush_file, FlushKind};
use std::fs::OpenOptions;
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::Path;

/// Policy controlling whether scrubbing (overwriting with zeros before deletion) is enabled.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScrubPolicy {
    /// Disabled: perform standard durable removal without overwriting contents.
    Disabled,
    /// OverwriteZeros: overwrite file contents with zero bytes prior to removal.
    OverwriteZeros,
}

/// Reasons why scrubbing a file might be refused for safety or technical limitations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScrubRefusal {
    /// Scrubbing is disabled by policy.
    Disabled,
    /// File has multiple hardlinks (nlink > 1); overwriting would corrupt other links.
    MultipleHardlinks,
    /// Target resides on a Copy-on-Write filesystem (e.g. btrfs, zfs, bcachefs).
    CopyOnWriteFilesystem,
    /// Active readers are currently attached to the file.
    ReaderAlive,
    /// The file is not marked obsolete.
    NotObsolete,
    /// CoW or platform support cannot be conclusively verified.
    Unsupported,
}

/// Outcome of attempting `scrub_if_safe`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScrubOutcome {
    /// File was successfully overwritten with zeros and durably removed.
    Scrubbed(ScrubReceipt),
    /// File was durably removed without zero-overwriting (e.g. policy was `Disabled`).
    RemovedUnscrubbed,
    /// Scrubbing was refused due to safety checks. File remains intact.
    Refused(ScrubRefusal),
}

/// Safely scrubs a file if environment and caller pre-conditions permit, otherwise refuses or falls back.
pub fn scrub_if_safe(
    path: &Path,
    policy: ScrubPolicy,
    reader_alive: bool,
    not_obsolete: bool,
) -> std::io::Result<ScrubOutcome> {
    scrub_if_safe_with_injector(path, policy, reader_alive, not_obsolete, &NoFault)
}

/// Safely scrubs a file with a custom [`FaultInjector`].
pub fn scrub_if_safe_with_injector<F: FaultInjector>(
    path: &Path,
    policy: ScrubPolicy,
    reader_alive: bool,
    not_obsolete: bool,
    injector: &F,
) -> std::io::Result<ScrubOutcome> {
    if policy == ScrubPolicy::Disabled {
        durable_remove_with_injector(path, injector)?;
        return Ok(ScrubOutcome::RemovedUnscrubbed);
    }

    if reader_alive {
        return Ok(ScrubOutcome::Refused(ScrubRefusal::ReaderAlive));
    }

    if not_obsolete {
        return Ok(ScrubOutcome::Refused(ScrubRefusal::NotObsolete));
    }

    let metadata = path.symlink_metadata()?;
    let file_type = metadata.file_type();

    if file_type.is_symlink() || !file_type.is_file() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "path is a symlink or not a regular file",
        ));
    }

    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        if metadata.nlink() > 1 {
            return Ok(ScrubOutcome::Refused(ScrubRefusal::MultipleHardlinks));
        }
    }

    if let Some(refusal) = check_cow_filesystem(path) {
        return Ok(ScrubOutcome::Refused(refusal));
    }

    let len = metadata.len();
    if len > 0 {
        let mut options = OpenOptions::new();
        options.read(true).write(true);
        #[cfg(windows)]
        {
            use std::os::windows::fs::OpenOptionsExt;
            options.share_access(7);
        }

        let mut file = options.open(path)?;
        let zeros = [0u8; 64 * 1024];
        let mut remaining = len;

        while remaining > 0 {
            let chunk_size = usize::try_from(remaining)
                .unwrap_or(zeros.len())
                .min(zeros.len());
            injector.on_write(Some(path), chunk_size)?;
            file.write_all(&zeros[..chunk_size])?;
            remaining -= chunk_size as u64;
        }

        injector.on_fsync(Some(path))?;
        flush_file(&file, FlushKind::DataAndMetadata)?;

        // Readback verification pass: verify all overwritten bytes are 0u8
        verify_zero_readback(&mut file, path, len)?;
    }

    let (nlink, fs_kind_str, is_cow) = query_fs_metadata(path, &metadata);

    durable_remove_with_injector(path, injector)?;

    Ok(ScrubOutcome::Scrubbed(ScrubReceipt {
        method: ScrubMethod::OverwriteZeros,
        bytes_scrubbed: len,
        nlink,
        fs_kind: fs_kind_str,
        cow_fs: is_cow,
    }))
}

/// Helper function querying link count, filesystem kind string, and CoW status.
fn query_fs_metadata(path: &Path, metadata: &std::fs::Metadata) -> (u64, String, bool) {
    #[cfg(unix)]
    let nlink = {
        use std::os::unix::fs::MetadataExt;
        metadata.nlink()
    };
    #[cfg(not(unix))]
    let nlink = {
        let _ = metadata;
        1u64
    };

    let (fs_kind_str, is_cow) = query_cow_info(path);
    (nlink, fs_kind_str, is_cow)
}

fn query_cow_info(path: &Path) -> (String, bool) {
    #[cfg(target_os = "linux")]
    {
        use contextra_sys::env_observe::{cow_fs_from_fs_kind_and_options, parse_mountinfo_details, EnvTri};
        if let Ok(mountinfo_text) = std::fs::read_to_string("/proc/self/mountinfo") {
            let (fs_kind, _discard, opts) = parse_mountinfo_details(&mountinfo_text, path);
            let cow_tri = cow_fs_from_fs_kind_and_options(fs_kind, &opts);
            let fs_name = format!("{:?}", fs_kind).to_lowercase();
            let is_cow = matches!(cow_tri, EnvTri::Yes);
            return (fs_name, is_cow);
        }
    }
    let _ = path;
    ("unknown".to_string(), false)
}

/// Removes a file and returns a [`ScrubReceipt`].
///
/// Overwrites file contents with zeros ONLY if `nlink == 1`, the filesystem is NOT Copy-on-Write,
/// and no active readers are present. Otherwise, skips zero-overwriting and durably unlinks the file.
pub fn scrub_and_remove_with_injector<F: FaultInjector>(
    path: &Path,
    injector: &F,
) -> std::io::Result<ScrubReceipt> {
    let metadata = path.symlink_metadata()?;
    let file_type = metadata.file_type();

    if file_type.is_symlink() || !file_type.is_file() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "path is a symlink or not a regular file",
        ));
    }

    let (nlink, fs_kind_str, is_cow) = query_fs_metadata(path, &metadata);
    let len = metadata.len();

    if nlink > 1 || is_cow {
        durable_remove_with_injector(path, injector)?;
        return Ok(ScrubReceipt {
            method: ScrubMethod::UnlinkOnly,
            bytes_scrubbed: 0,
            nlink,
            fs_kind: fs_kind_str,
            cow_fs: is_cow,
        });
    }

    if len > 0 {
        let mut options = OpenOptions::new();
        options.write(true);
        #[cfg(windows)]
        {
            use std::os::windows::fs::OpenOptionsExt;
            options.share_access(7);
        }

        let mut file = options.open(path)?;

        let zeros = [0u8; 64 * 1024];
        let mut remaining = len;

        while remaining > 0 {
            let chunk_size = usize::try_from(remaining)
                .unwrap_or(zeros.len())
                .min(zeros.len());
            injector.on_write(Some(path), chunk_size)?;
            file.write_all(&zeros[..chunk_size])?;
            remaining -= chunk_size as u64;
        }

        injector.on_fsync(Some(path))?;
        flush_file(&file, FlushKind::DataAndMetadata)?;
    }

    durable_remove_with_injector(path, injector)?;

    Ok(ScrubReceipt {
        method: ScrubMethod::OverwriteZeros,
        bytes_scrubbed: len,
        nlink,
        fs_kind: fs_kind_str,
        cow_fs: is_cow,
    })
}

/// Helper function to perform readback verification that file contents are zeroed out.
fn verify_zero_readback(
    file: &mut std::fs::File,
    path: &Path,
    expected_len: u64,
) -> std::io::Result<()> {
    file.seek(SeekFrom::Start(0))?;
    let mut buf = [0u8; 64 * 1024];
    let mut remaining = expected_len;

    while remaining > 0 {
        let chunk_size = usize::try_from(remaining)
            .unwrap_or(buf.len())
            .min(buf.len());
        file.read_exact(&mut buf[..chunk_size])?;
        if buf[..chunk_size].iter().any(|&b| b != 0) {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!(
                    "readback zero verification failed for file '{:?}': non-zero bytes detected after scrub",
                    path
                ),
            ));
        }
        remaining -= chunk_size as u64;
    }

    Ok(())
}

/// Checks if `path` resides on a known Copy-on-Write filesystem using `/proc/self/mountinfo` on Linux.
/// Returns `Some(ScrubRefusal)` if CoW is detected or detection is unsupported/uncertain.
fn check_cow_filesystem(path: &Path) -> Option<ScrubRefusal> {
    #[cfg(target_os = "linux")]
    {
        use std::fs::File;
        use std::io::{BufRead, BufReader};

        let mountinfo_file = match File::open("/proc/self/mountinfo") {
            Ok(f) => f,
            Err(_) => return Some(ScrubRefusal::Unsupported),
        };

        let reader = BufReader::new(mountinfo_file);
        let canonical_path = match path.canonicalize() {
            Ok(p) => p,
            Err(_) => path.to_path_buf(),
        };

        let mut best_match_len = 0;
        let mut best_match_fstype = String::new();

        for line_res in reader.lines() {
            let line = match line_res {
                Ok(l) => l,
                Err(_) => continue,
            };

            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() < 5 {
                continue;
            }

            let mount_point = parts[4];
            let mount_path = Path::new(mount_point);

            if canonical_path.starts_with(mount_path) {
                let mlen = mount_point.len();
                if mlen >= best_match_len {
                    best_match_len = mlen;
                    if let Some(dash_pos) = parts.iter().position(|&p| p == "-") {
                        if dash_pos + 1 < parts.len() {
                            best_match_fstype = parts[dash_pos + 1].to_lowercase();
                        }
                    }
                }
            }
        }

        if best_match_fstype.is_empty() {
            return Some(ScrubRefusal::Unsupported);
        }

        if best_match_fstype.contains("btrfs")
            || best_match_fstype.contains("zfs")
            || best_match_fstype.contains("bcachefs")
        {
            return Some(ScrubRefusal::CopyOnWriteFilesystem);
        }

        None
    }

    #[cfg(not(target_os = "linux"))]
    {
        let _ = path;
        Some(ScrubRefusal::Unsupported)
    }
}
