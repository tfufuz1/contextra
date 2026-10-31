//! # Sicheres Überschreiben und Entfernen von Dateien (Scrubbing)
//!
//! Zweck: Implementierung von Schutzmechanismen und Überschreibverfahren (Scrubbing) vor dem
//! physischen Löschen von Dateien von Speichermedien.
//!
//! Ring-Zugehörigkeit: Ring 0 (Dauerhafte Dateisystem-Hilfsmittel).
//! Wird genutzt von: Bereinigungsroutinen, Lösch-Orchestrierern und DSGVO-Löschpfaden.
//! Wichtigste Typen:
//! - [`ScrubPolicy`]: Konfiguriert, ob Nullen-Überschreiben aktiv ist ([`ScrubPolicy::OverwriteZeros`]) oder nicht ([`ScrubPolicy::Disabled`]).
//! - [`ScrubRefusal`]: Gründe für eine Verweigerung des Überschreibens (z. B. Copy-on-Write-Dateisysteme, Hardlinks, aktive Leser).
//! - [`ScrubOutcome`]: Ergebnis des Bereinigungsversuchs ([`ScrubOutcome::Scrubbed`], [`ScrubOutcome::RemovedUnscrubbed`], [`ScrubOutcome::Refused`]).
//! - [`scrub_if_safe`]: Hauptfunktion zur sicheren Löschung mit vorgeschalteter Risikoanalyse.
//!
//! Invarianten:
//! - `INV-CLAIMS-1`: Überschreiben garantiert auf modernen Flash-/SSD-Medien keine Zerstörung auf Zellebene (Wear-Leveling).
//!   Es werden keine Garantien für physische Nicht-Wiederherstellbarkeit abgegeben (`physical_guarantee: None`).
//! - Fail-Closed: Bei aktivem Leser, Hardlinks (`nlink > 1`) oder erkanntem CoW-Dateisystem (Btrfs, ZFS) wird
//!   das Überschreiben verweigert, um Datenkorruption an anderen Links oder Scheinsicherheit zu verhindern.

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

use crate::{durable_remove_with_injector, FaultInjector, NoFault, ScrubReceipt};
use contextra_sys::{flush_file, FlushKind};
use std::fs::OpenOptions;
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::Path;

// ANCHOR[contextra-durable-fs:SCRUB-POLICY-001]
/// Richtlinie zur Steuerung des Überschreibverhaltens vor dem Löschen.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScrubPolicy {
    /// Überschreiben deaktiviert: Datei wird direkt ohne vorheriges Nullen entfernt.
    Disabled,
    /// Datei vor dem Entfernen vollständig mit binären Nullen (`0u8`) überschreiben.
    OverwriteZeros,
}

// ANCHOR[contextra-durable-fs:SCRUB-REFUSAL-001]
/// Gründe für eine Verweigerung des Nullen-Überschreibens aus Sicherheits- oder Plattformgründen.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScrubRefusal {
    /// Scrubbing wurde per Richtlinie ([`ScrubPolicy::Disabled`]) deaktiviert.
    Disabled,
    /// Datei besitzt mehrere Hardlinks (`nlink > 1`); Überschreiben würde andere Dateiverweise zerstören.
    MultipleHardlinks,
    /// Datei liegt auf einem Copy-on-Write-Dateisystem (z. B. Btrfs, ZFS, Bcachefs), wo Überschreiben neue Blöcke anlegt.
    CopyOnWriteFilesystem,
    /// Mindestens ein aktiver Leser greift derzeit auf die Datei zu.
    ReaderAlive,
    /// Die Datei ist nicht als obsolet/veraltet markiert.
    NotObsolete,
    /// CoW- oder Dateisystemeigenschaften konnten nicht zweifelsfrei ermittelt werden.
    Unsupported,
}

// ANCHOR[contextra-durable-fs:SCRUB-OUTCOME-001]
/// Ergebnis einer Bereinigungsoperation mittels [`scrub_if_safe`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScrubOutcome {
    /// Datei wurde erfolgreich mit Nullen überschrieben, gegengelesen und dauerhaft entfernt.
    Scrubbed(ScrubReceipt),
    /// Datei wurde dauerhaft gelöscht, ohne zuvor überschrieben zu werden (z. B. bei [`ScrubPolicy::Disabled`]).
    RemovedUnscrubbed,
    /// Scrubbing wurde aus Sicherheitsgründen verweigert. Die Datei bleibt auf der Festplatte unverändert erhalten.
    Refused(ScrubRefusal),
}

// ANCHOR[contextra-durable-fs:SCRUB-IF-SAFE-001]
/// Bereinigt und löscht eine Datei sicher, sofern alle Sicherheitsvorbedingungen erfüllt sind.
///
/// Verwendet den standardmäßigen Null-Kosten-Injektor [`NoFault`].
///
/// Vorbedingungen:
/// - `path` muss ein regulärer Dateipfad sein (kein Symlink, kein Ordner).
/// - `reader_alive` muss `false` sein (keine nebenläufigen Leser).
/// - `not_obsolete` muss `false` sein (Datei muss freigegeben sein).
///
/// Rückgabewert:
/// - `Ok(ScrubOutcome::Scrubbed(receipt))` bei erfolgreichem Überschreiben und Löschen.
/// - `Ok(ScrubOutcome::Refused(reason))` bei Sicherheitsverweigerung.
pub fn scrub_if_safe(
    path: &Path,
    policy: ScrubPolicy,
    reader_alive: bool,
    not_obsolete: bool,
) -> std::io::Result<ScrubOutcome> {
    scrub_if_safe_with_injector(path, policy, reader_alive, not_obsolete, &NoFault)
}

// ANCHOR[contextra-durable-fs:SCRUB-IF-SAFE-WITH-INJECTOR]
/// Bereinigt und löscht eine Datei sicher unter Verwendung eines benutzerdefinierten [`FaultInjector`].
///
/// Ablauf:
/// 1. Richtlinienprüfung: Falls [`ScrubPolicy::Disabled`], Datei direkt via [`durable_remove_with_injector`] löschen.
/// 2. Vorbedingungsprüfungen:
///    - Falls `reader_alive == true` -> Abbruch mit [`ScrubRefusal::ReaderAlive`].
///    - Falls `not_obsolete == true` -> Abbruch mit [`ScrubRefusal::NotObsolete`].
/// 3. Metadatenprüfung: Symlinks und Sonderdateien mit `InvalidInput`-Fehler abweisen.
/// 4. Hardlink-Prüfung: Falls `nlink > 1`, Abbruch mit [`ScrubRefusal::MultipleHardlinks`].
/// 5. CoW-Prüfung: Falls Btrfs/ZFS erkannt, Abbruch mit [`ScrubRefusal::CopyOnWriteFilesystem`].
/// 6. Überschreiben: Datei in 64-KiB-Blöcken mit `0u8` füllen und via `injector.on_write` überwachen.
/// 7. Synchronisieren: `injector.on_fsync` und Hardware-Flush (`flush_file`).
/// 8. Readback-Verifikation: Datei von Byte 0 bis Ende rücklesen; schlägt fehl bei Bytes != 0.
/// 9. Dauerhaftes Entfernen: Datei aus Verzeichnis aushängen und Elternordner synchronisieren.
///
/// Fehlerfälle:
/// - Schlägt das Schreiben, Flushen, Gegenlesen oder Löschen fehl, wird der entsprechende [`std::io::Error`] zurückgegeben.
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

        // NOTE: Chunkweises Überschreiben in 64-KiB-Schritten verhindert hohen Speicherverbrauch.
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

        // NOTE: Readback-Zero-Verifikation: Stellt sicher, dass das Betriebssystem die Nullen tatsächlich übernommen hat.
        verify_zero_readback(&mut file, path, len)?;
    }

    durable_remove_with_injector(path, injector)?;

    Ok(ScrubOutcome::Scrubbed(ScrubReceipt {
        bytes_scrubbed: len,
    }))
}

// ANCHOR[contextra-durable-fs:SCRUB-VERIFY-ZERO]
/// Hilfsfunktion zur Verifikation, dass alle Bytes der Datei nach dem Überschreiben binäre Nullen sind.
///
/// Ablauf:
/// 1. Dateizeiger auf Dateianfang (Offset 0) zurücksetzen.
/// 2. Datei in 64-KiB-Blöcken einlesen.
/// 3. Prüfen, ob irgendein Byte != 0 ist.
/// 4. Liefert Fehler bei Nicht-Null-Bytes oder I/O-Leseproblemen.
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

// ANCHOR[contextra-durable-fs:SCRUB-CHECK-COW]
/// Prüft unter Linux anhand von `/proc/self/mountinfo`, ob der Pfad auf einem Copy-on-Write-Dateisystem liegt.
///
/// Gibt `Some(ScrubRefusal::CopyOnWriteFilesystem)` zurück, wenn Btrfs, ZFS oder Bcachefs erkannt wurden.
/// Liefert `Some(ScrubRefusal::Unsupported)` auf Nicht-Linux-Systemen oder wenn mountinfo unlesbar ist.
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
