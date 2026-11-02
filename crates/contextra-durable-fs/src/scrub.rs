//! # Sicheres Überschreiben und bedingtes Scrubbing (`scrub`)
//!
//! Zweck: Implementierung von Schutzmechanismen und Überschreibverfahren (Scrubbing) vor dem
//! physischen Löschen von Dateien von Speichermedien gemäß SPEC DF-07 und D-83.
//!
//! Ring-Zugehörigkeit: Ring 1 (Dauerhafte Dateisystem-Hilfsmittel).
//! Wichtigste Typen und Funktionen:
//! - [`scrub_before_unlink`]: Überschreibt Dateien nur bei `rotational == true` und `!is_cow`; sonst No-Op (DF-07).
//! - [`ScrubPolicy`]: Konfiguriert das Scrubbing-Verhalten ([`ScrubPolicy::Disabled`], [`ScrubPolicy::Auto`]).
//! - [`ScrubOutcome`]: Ergebnis des Bereinigungsversuchs ([`ScrubOutcome::Scrubbed`], [`ScrubOutcome::NoOp`], [`ScrubOutcome::Refused`]).
//! - [`ScrubReceipt`]: Quittung mit Anzahl überschriebener Bytes.
//!
//! Invarianten:
//! - `INV-SCRUB-1`: Überschreiben erfolgt gemäß D-83 ausschließlich dann, wenn `rotational == true`
//!   UND kein Copy-on-Write-Dateisystem (`!is_cow`) vorliegt (DF-07).
//! - `INV-SCRUB-2`: Auf Flash/SSD (`rotational == false`) oder CoW entfällt das Überschreiben vollständig (No-Op),
//!   da Out-of-Place-Writes und Wear-Leveling ein In-Place-Nullen wirkungslos machen.
//! - `INV-SCRUB-3`: Hardlinks (`nlink > 1`) werden strikt abgewiesen (`ScrubRefusal::MultipleHardlinks`).
//! - Zero-Panic: Alle Fehler werden als deterministisches [`std::io::Result`] propagiert.

use crate::{durable_remove_with_injector, FaultInjector, NoFault};
use contextra_sys::{flush_file, FlushKind};
use std::fs::OpenOptions;
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::Path;

// ANCHOR[contextra-durable-fs:SCRUB-RECEIPT-001]
/// Quittung nach erfolgreichem Überschreiben einer Datei mit Nullen vor dem Löschen.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScrubReceipt {
    /// Gesamtanzahl der vor dem Löschen mit Nullen überschriebenen Bytes.
    pub bytes_scrubbed: u64,
}

// ANCHOR[contextra-durable-fs:SCRUB-POLICY-001]
/// Richtlinie zur Steuerung des Überschreibverhaltens vor dem Löschen (DF-07).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ScrubPolicy {
    /// Überschreiben vollständig deaktiviert: Vor dem Löschen findet kein Scrubbing statt.
    Disabled,
    /// Automatisches Überschreiben: Nur aktiv bei Rotationsmedien ohne Copy-on-Write (D-83).
    #[default]
    Auto,
    /// Abwärtskompatible Variante für explizites Nullen.
    OverwriteZeros,
}

// ANCHOR[contextra-durable-fs:SCRUB-REFUSAL-001]
/// Gründe für eine Verweigerung des Nullen-Überschreibens aus Sicherheits- oder Plattformgründen.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScrubRefusal {
    /// Scrubbing wurde per Richtlinie deaktiviert.
    Disabled,
    /// Datei besitzt mehrere Hardlinks (`nlink > 1`); Überschreiben würde andere Dateiverweise beschädigen.
    MultipleHardlinks,
    /// Datei liegt auf einem Copy-on-Write-Dateisystem (z. B. Btrfs, ZFS).
    CopyOnWriteFilesystem,
    /// Mindestens ein aktiver Leser greift derzeit auf die Datei zu.
    ReaderAlive,
    /// Die Datei ist nicht als obsolet markiert.
    NotObsolete,
    /// Dateisystemeigenschaften konnten nicht zweifelsfrei ermittelt werden.
    Unsupported,
}

// ANCHOR[contextra-durable-fs:SCRUB-OUTCOME-001]
/// Ergebnis einer Bereinigungsoperation vor dem Löschen (DF-07).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScrubOutcome {
    /// Datei wurde erfolgreich mit Nullen überschrieben, gegengelesen und synchronisiert.
    Scrubbed(ScrubReceipt),
    /// Kein Überschreiben ausgeführt (No-Op gemäß D-83: Flash/SSD oder CoW-Dateisystem).
    NoOp,
    /// Abwärtskompatible Variante für nicht-überschriebene gelöschte Dateien.
    RemovedUnscrubbed,
    /// Scrubbing wurde aus Sicherheitsgründen verweigert. Die Datei bleibt unverändert erhalten.
    Refused(ScrubRefusal),
}

// ANCHOR[contextra-durable-fs:SCRUB-BEFORE-UNLINK-001]
/// Führt das Überschreiben vor dem Löschen bedingt nach Vorgabe D-83 / DF-07 aus.
///
/// Ablauf:
/// 1. Bedingungsprüfung: `rotational == true` UND `is_cow == false`.
///    Ist die Bedingung nicht erfüllt, erfolgt kein Überschreiben (sofortiges `Ok(ScrubOutcome::NoOp)`).
/// 2. Metadaten- und Hardlink-Prüfung: Symlinks abweisen; bei `nlink > 1` Verweigerung mit [`ScrubRefusal::MultipleHardlinks`].
/// 3. Überschreiben der Datei in 64-KiB-Blöcken mit binären Nullen (`0u8`).
/// 4. Synchronisieren via [`flush_file`].
/// 5. Readback-Verifikation: Sicherstellen, dass alle gelesenen Bytes `0u8` sind.
pub fn scrub_before_unlink(
    path: &Path,
    rotational: bool,
    is_cow: bool,
) -> std::io::Result<ScrubOutcome> {
    scrub_before_unlink_with_injector(path, rotational, is_cow, &NoFault)
}

// ANCHOR[contextra-durable-fs:SCRUB-BEFORE-UNLINK-WITH-INJECTOR]
/// Führt das bedingte Überschreiben vor dem Löschen mit Injektor-Unterstützung aus.
pub fn scrub_before_unlink_with_injector<F: FaultInjector>(
    path: &Path,
    rotational: bool,
    is_cow: bool,
    injector: &F,
) -> std::io::Result<ScrubOutcome> {
    // D-83 / DF-07: Überschreiben NUR bei rotational == true UND nicht-COW.
    if !rotational || is_cow {
        return Ok(ScrubOutcome::NoOp);
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
            let slice = zeros.get(..chunk_size).unwrap_or(&zeros);
            file.write_all(slice)?;
            remaining = remaining.saturating_sub(chunk_size as u64);
        }

        injector.on_fsync(Some(path))?;
        flush_file(&file, FlushKind::DataAndMetadata)?;

        verify_zero_readback(&mut file, len)?;
    }

    Ok(ScrubOutcome::Scrubbed(ScrubReceipt {
        bytes_scrubbed: len,
    }))
}

// ANCHOR[contextra-durable-fs:SCRUB-IF-SAFE-COMPAT]
/// Kompatibilitätsfunktion für bestehende Aufrufer von `scrub_if_safe`.
pub fn scrub_if_safe(
    path: &Path,
    policy: ScrubPolicy,
    reader_alive: bool,
    not_obsolete: bool,
) -> std::io::Result<ScrubOutcome> {
    scrub_if_safe_with_injector(path, policy, reader_alive, not_obsolete, &NoFault)
}

// ANCHOR[contextra-durable-fs:SCRUB-IF-SAFE-WITH-INJECTOR-COMPAT]
/// Kompatibilitätsfunktion für bestehende Aufrufer von `scrub_if_safe_with_injector`.
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
            let slice = zeros.get(..chunk_size).unwrap_or(&zeros);
            file.write_all(slice)?;
            remaining = remaining.saturating_sub(chunk_size as u64);
        }

        injector.on_fsync(Some(path))?;
        flush_file(&file, FlushKind::DataAndMetadata)?;

        verify_zero_readback(&mut file, len)?;
    }

    durable_remove_with_injector(path, injector)?;

    Ok(ScrubOutcome::Scrubbed(ScrubReceipt {
        bytes_scrubbed: len,
    }))
}

// ANCHOR[contextra-durable-fs:SCRUB-VERIFY-ZERO-001]
/// Verifiziert, dass alle Bytes der Datei nach dem Überschreiben binäre Nullen sind.
fn verify_zero_readback(file: &mut std::fs::File, expected_len: u64) -> std::io::Result<()> {
    file.seek(SeekFrom::Start(0))?;
    let mut buf = [0u8; 64 * 1024];
    let mut remaining = expected_len;

    while remaining > 0 {
        let chunk_size = usize::try_from(remaining)
            .unwrap_or(buf.len())
            .min(buf.len());
        let buf_slice = buf.get_mut(..chunk_size).ok_or_else(|| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "readback buffer slice out of bounds",
            )
        })?;
        file.read_exact(buf_slice)?;
        if buf_slice.iter().any(|&b| b != 0) {
            return Err(std::io::Error::other(
                "readback zero verification failed: non-zero bytes detected after scrub",
            ));
        }
        remaining = remaining.saturating_sub(chunk_size as u64);
    }

    Ok(())
}

// ANCHOR[contextra-durable-fs:SCRUB-CHECK-COW-001]
/// Ermittelt, ob ein Pfad auf einem Copy-on-Write-Dateisystem liegt.
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

            let Some(&mount_point) = parts.get(4) else {
                continue;
            };
            let mount_path = Path::new(mount_point);

            if canonical_path.starts_with(mount_path) {
                let mlen = mount_point.len();
                if mlen >= best_match_len {
                    best_match_len = mlen;
                    if let Some(dash_pos) = parts.iter().position(|&p| p == "-") {
                        if let Some(&fstype) =
                            dash_pos.checked_add(1).and_then(|idx| parts.get(idx))
                        {
                            best_match_fstype = fstype.to_lowercase();
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
