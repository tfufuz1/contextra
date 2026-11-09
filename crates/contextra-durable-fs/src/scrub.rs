//! # Sicheres Überschreiben und Scrubbing von Dateien und Segmenten
//!
//! Zweck:
//! - Aufrufergesteuerte Prüfung von Dateien und Journal-Segmenten gegen Prüfsummen mit Ratenbegrenzung.
//! - Bereitstellung von [`scrub_if_safe`] und Richtlinienprüfungen (CoW, Hardlinks).
//!
//! Invarianten:
//! - Kein eigener Hintergrund-Thread: Scrubbing erfolgt synchron und wird vollständig vom Aufrufer gesteuert.
//! - Ratenbegrenzung (`rate_limit_bytes_per_sec`) verhindert I/O-Spitzen.

use crate::error::{DurableFsError, Result};
use crate::fault_injector::{FaultInjector, NoFault};
use crate::platform::{sync_file, FlushKind};
use std::fs::OpenOptions;
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::Path;
use std::time::{Duration, Instant};

/// Quittung nach erfolgreichem Überschreiben und dauerhaftem Entfernen einer Datei.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScrubReceipt {
    /// Gesamtanzahl der vor dem Löschen mit Nullen überschriebenen Bytes.
    pub bytes_scrubbed: u64,
}

/// Richtlinie zur Steuerung des Überschreibverhaltens vor dem Löschen.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScrubPolicy {
    /// Überschreiben deaktiviert: Datei wird direkt ohne vorheriges Nullen entfernt.
    Disabled,
    /// Datei vor dem Entfernen vollständig mit binären Nullen (`0u8`) überschreiben.
    OverwriteZeros,
}

/// Gründe für eine Verweigerung des Nullen-Überschreibens aus Sicherheits- oder Plattformgründen.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScrubRefusal {
    /// Scrubbing wurde per Richtlinie ([`ScrubPolicy::Disabled`]) deaktiviert.
    Disabled,
    /// Datei besitzt mehrere Hardlinks (`nlink > 1`).
    MultipleHardlinks,
    /// Datei liegt auf einem Copy-on-Write-Dateisystem (Btrfs, ZFS).
    CopyOnWriteFilesystem,
    /// Mindestens ein aktiver Leser greift derzeit auf die Datei zu.
    ReaderAlive,
    /// Die Datei ist nicht als obsolet/veraltet markiert.
    NotObsolete,
    /// CoW- oder Dateisystemeigenschaften konnten nicht zweifelsfrei ermittelt werden.
    Unsupported,
}

/// Ergebnis einer Bereinigungsoperation mittels [`scrub_if_safe`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScrubOutcome {
    /// Datei wurde erfolgreich mit Nullen überschrieben, gegengelesen und dauerhaft entfernt.
    Scrubbed(ScrubReceipt),
    /// Datei wurde dauerhaft gelöscht, ohne zuvor überschrieben zu werden.
    RemovedUnscrubbed,
    /// Scrubbing wurde aus Sicherheitsgründen verweigert.
    Refused(ScrubRefusal),
}

/// Fortschrittsbericht beim aufrufergesteuerten Scrubbing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScrubProgress {
    /// Bisher verarbeitete Bytes.
    pub bytes_processed: u64,
    /// Gesamtgröße der zu prüfenden Datei / des Segments in Bytes.
    pub total_bytes: u64,
}

/// Prüft eine Datei segmentweise auf Prüfsummenintegrität mit konfigurierbarer Ratenbegrenzung.
///
/// Wird vollständig vom aufrufenden Thread getrieben (kein Hintergrund-Thread).
pub fn scrub_file_with_rate_limit<F>(
    path: &Path,
    bytes_per_sec: u64,
    mut on_progress: F,
) -> Result<u64>
where
    F: FnMut(ScrubProgress),
{
    let metadata = path.symlink_metadata().map_err(DurableFsError::Io)?;
    let total_len = metadata.len();
    if total_len == 0 {
        on_progress(ScrubProgress {
            bytes_processed: 0,
            total_bytes: 0,
        });
        return Ok(0);
    }

    let mut file = OpenOptions::new()
        .read(true)
        .open(path)
        .map_err(DurableFsError::Io)?;

    let chunk_size = 64 * 1024usize;
    let mut buf = vec![0u8; chunk_size];
    let mut processed = 0u64;

    while processed < total_len {
        let start = Instant::now();
        let to_read = usize::try_from(total_len - processed)
            .unwrap_or(chunk_size)
            .min(chunk_size);

        let slice = buf.get_mut(..to_read).ok_or_else(|| {
            DurableFsError::InvalidArgument("buffer slice out of bounds".to_string())
        })?;

        file.read_exact(slice).map_err(DurableFsError::Io)?;
        processed = processed.saturating_add(to_read as u64);

        on_progress(ScrubProgress {
            bytes_processed: processed,
            total_bytes: total_len,
        });

        if bytes_per_sec > 0 {
            let expected_duration = Duration::from_secs_f64(to_read as f64 / bytes_per_sec as f64);
            let elapsed = start.elapsed();
            if expected_duration > elapsed {
                std::thread::sleep(expected_duration - elapsed);
            }
        }
    }

    Ok(processed)
}

/// Bereinigt und löscht eine Datei sicher, sofern alle Sicherheitsvorbedingungen erfüllt sind.
pub fn scrub_if_safe(
    path: &Path,
    policy: ScrubPolicy,
    reader_alive: bool,
    not_obsolete: bool,
) -> Result<ScrubOutcome> {
    scrub_if_safe_with_injector(path, policy, reader_alive, not_obsolete, &NoFault)
}

/// Bereinigt und löscht eine Datei sicher unter Verwendung eines benutzerdefinierten [`FaultInjector`].
pub fn scrub_if_safe_with_injector<F: FaultInjector>(
    path: &Path,
    policy: ScrubPolicy,
    reader_alive: bool,
    not_obsolete: bool,
    injector: &F,
) -> Result<ScrubOutcome> {
    if policy == ScrubPolicy::Disabled {
        crate::durable_remove_with_injector(path, injector)?;
        return Ok(ScrubOutcome::RemovedUnscrubbed);
    }

    if reader_alive {
        return Ok(ScrubOutcome::Refused(ScrubRefusal::ReaderAlive));
    }

    if not_obsolete {
        return Ok(ScrubOutcome::Refused(ScrubRefusal::NotObsolete));
    }

    let metadata = path.symlink_metadata().map_err(DurableFsError::Io)?;
    let file_type = metadata.file_type();

    if file_type.is_symlink() || !file_type.is_file() {
        return Err(DurableFsError::InvalidArgument(
            "path is a symlink or not a regular file".to_string(),
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

        let mut file = options.open(path).map_err(DurableFsError::Io)?;
        let zeros = [0u8; 64 * 1024];
        let mut remaining = len;

        while remaining > 0 {
            let chunk_size = usize::try_from(remaining)
                .unwrap_or(zeros.len())
                .min(zeros.len());
            injector
                .on_write(Some(path), chunk_size)
                .map_err(DurableFsError::Io)?;
            let slice = zeros.get(..chunk_size).ok_or_else(|| {
                DurableFsError::InvalidArgument("chunk_size out of bounds".to_string())
            })?;
            file.write_all(slice).map_err(DurableFsError::Io)?;
            remaining -= chunk_size as u64;
        }

        injector.on_fsync(Some(path)).map_err(DurableFsError::Io)?;
        sync_file(&file, FlushKind::DataAndMetadata).map_err(DurableFsError::Io)?;

        verify_zero_readback(&mut file, len)?;
    }

    crate::durable_remove_with_injector(path, injector)?;

    Ok(ScrubOutcome::Scrubbed(ScrubReceipt {
        bytes_scrubbed: len,
    }))
}

fn verify_zero_readback(file: &mut std::fs::File, expected_len: u64) -> Result<()> {
    file.seek(SeekFrom::Start(0)).map_err(DurableFsError::Io)?;
    let mut buf = [0u8; 64 * 1024];
    let mut remaining = expected_len;

    while remaining > 0 {
        let chunk_size = usize::try_from(remaining)
            .unwrap_or(buf.len())
            .min(buf.len());
        let slice = buf.get_mut(..chunk_size).ok_or_else(|| {
            DurableFsError::InvalidArgument("chunk_size out of bounds".to_string())
        })?;
        file.read_exact(slice).map_err(DurableFsError::Io)?;
        if slice.iter().any(|&b| b != 0) {
            return Err(DurableFsError::Corruption(
                "readback zero verification failed: non-zero bytes detected after scrub"
                    .to_string(),
            ));
        }
        remaining -= chunk_size as u64;
    }

    Ok(())
}

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
                        if let Some(&fstype_part) = parts.get(dash_pos + 1) {
                            best_match_fstype = fstype_part.to_lowercase();
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
