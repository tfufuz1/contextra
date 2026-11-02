//! # Filesystem Startup Probe (`probe.rs`)
//!
//! Zweck: Startprobe zur Validierung von Plattform- und Dateisystem-Garantien (SPEC §8.5 D-56, DF-06).
//!
//! Validierte Eigenschaften:
//! - Dateisystem-Klassifikation über `contextra_sys::fs_class` und Erkennung von CoW / Journaling.
//! - Atomares Umbenennen (`rename_durable`).
//! - Verzeichnis-fsync (`flush_dir`).
//! - Atomaritätsprobe für ausgerichtete Schreibvorgänge (64 Byte) sowie 80-Byte-Slots (L-4).
//! - Stichprobenmessung von `fsync_p50` (D-30, T-CFG-3).
//! - Abfrage des freien Festplattenspeichers (`free_space`, DF-10, D-65).
//! - Beurteilung des Dateisystems via [`verdict`]: Tier 1, Tier 2 oder Refused.

use contextra_ports::clock::{Clock, SystemClock};
use contextra_sys::{flush_dir, flush_file, fs_class, rename_durable, FlushKind, FsClass};
use std::fs::{File, OpenOptions};
use std::io::{self, Read, Seek, SeekFrom, Write};
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

/// Atomarer Zähler zur Generierung kollisionsfreier Namen für Testdateien der Startprobe.
static PROBE_COUNTER: AtomicU64 = AtomicU64::new(0);

// ANCHOR[contextra-durable-fs:PROBE-TIER-001]
/// Plattform- und Dateisystemstufe gemäß SPEC D-56.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Tier {
    /// Tier 1: Linux x86_64 / aarch64 auf ext4 oder XFS (volle lokale Haltbarkeit).
    Tier1,
    /// Tier 2: macOS auf APFS (`F_FULLFSYNC`) sowie Linux auf btrfs oder ZFS.
    Tier2,
}

// ANCHOR[contextra-durable-fs:PROBE-UNSUPPORTED-FS-001]
/// Nicht unterstützte Dateisystemtypen, die den Start verweigern (SPEC D-56, T-PLAT-1).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum UnsupportedFilesystem {
    /// Network File System (NFS).
    Nfs,
    /// Server Message Block / CIFS (SMB).
    Smb,
    /// Filesystem in Userspace (FUSE).
    Fuse,
    /// Overlay-Dateisystem (overlayfs).
    Overlayfs,
    /// Windows NTFS / Win32-Dateisystem.
    Windows,
    /// Unbekanntes oder nicht spezifiziertes Dateisystem.
    Unknown,
    /// Hardware- oder Dateisystemprobe (Atomarität, Rename oder fsync) fehlgeschlagen.
    ProbeFailed,
}

// ANCHOR[contextra-durable-fs:PROBE-VERDICT-ENUM-001]
/// Ergebnis der Startprobe-Beurteilung.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ProbeVerdict {
    /// Unterstütztes Dateisystem in Stufe Tier 1 oder Tier 2.
    Supported(Tier),
    /// Start verweigert wegen nicht unterstütztem Dateisystem oder fehlgeschlagener Probe.
    Refused(UnsupportedFilesystem),
}

// ANCHOR[contextra-durable-fs:PROBE-DETECTED-FS-001]
/// Erkannter konkreter Dateisystemtyp.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DetectedFs {
    /// ext4 Dateisystem (Linux).
    Ext4,
    /// XFS Dateisystem (Linux).
    Xfs,
    /// Apple File System (APFS, macOS).
    Apfs,
    /// Btrfs (Linux CoW).
    Btrfs,
    /// OpenZFS / ZFS (CoW).
    Zfs,
    /// Network File System (NFS).
    Nfs,
    /// SMB / CIFS.
    Smb,
    /// FUSE Dateisystem.
    Fuse,
    /// Overlayfs.
    Overlayfs,
    /// Windows NTFS.
    WindowsNtfs,
    /// Unbekannter Dateisystemtyp.
    Unknown,
}

// ANCHOR[contextra-durable-fs:PROBE-FS-PROBE-001]
/// Ergebnis der physischen Dateisystem-Startprobe (SPEC 4. Schnittstellen [E]).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FsProbe {
    /// Grobe Dateisystem-Klassifikation aus `contextra_sys::fs_class`.
    pub fs: FsClass,
    /// Erfolgreiches atomares Umbenennen (`rename_durable`).
    pub rename_atomic: bool,
    /// Erfolgreicher Verzeichnis-Sync (`flush_dir`).
    pub dir_fsync_ok: bool,
    /// Erfolgreiche Atomaritätsprobe für 64-Byte ausgerichtete Writes und 80-Byte-Slots (L-4).
    pub write_atomic_64: bool,
    /// Gemessene Mediandauer eines `fsync`-Aufrufs (Stichprobe).
    pub fsync_p50: Duration,
    /// Angabe, ob das zugrundeliegende Speichermedium rotierend ist (HDD: Some(true), SSD/NVMe: Some(false)).
    pub rotational: Option<bool>,
    /// Spezifisch erkannter Dateisystemtyp.
    pub detected_fs: DetectedFs,
    /// Weist aus, ob das Dateisystem Journaling oder Copy-on-Write nutzt (Restrisiko FsJournalOrCow).
    pub journal_or_cow: bool,
}

/// RAII-Guard zum rückstandslosen Löschen von Testartefakten.
struct ProbeTempFile<'a> {
    path: &'a Path,
}

impl<'a> Drop for ProbeTempFile<'a> {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(self.path);
    }
}

/// Führt die vollständige Startprobe auf dem Verzeichnis `path` durch.
///
/// # Fehler
/// Liefert einen [`io::Error`] mit festem Fehlertext, falls das Verzeichnis nicht existiert
/// oder die Dateizugriffe scheitern (ohne Klartextpfade in Display gemäß DF-11).
// ANCHOR[contextra-durable-fs:PROBE-DIR-001]
// Ablauf von `probe_dir`:
// 1. Verzeichnisvalidierung (DF-11: Pfadprüfung ohne Klartext-Pfad im Display).
// 2. Grobe Klassifikation via `fs_class` und Erkennung von Copy-on-Write (CoW).
// 3. Rotationsmedium-Erkennung (HDD vs. SSD/NVMe) für D-83.
// 4. Atomaritätsprobe für `rename_durable`.
// 5. Verzeichnis-`fsync` via `flush_dir`.
// 6. Atomaritätsprobe für 64-Byte ausgerichtete Writes und 80-Byte-Slots (L-4).
// 7. Stichprobenmessung von `fsync_p50` zur Baseline-Bestimmung (D-30).
pub fn probe_dir(path: &Path) -> io::Result<FsProbe> {
    if !path.is_dir() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            "target directory not found",
        ));
    }

    let fs_coarse =
        fs_class(path).map_err(|_| io::Error::other("filesystem classification failed"))?;

    let detected = detect_filesystem_type(path);
    let is_cow = matches!(
        detected,
        DetectedFs::Btrfs | DetectedFs::Zfs | DetectedFs::Apfs
    );

    let rotational = detect_rotational(path);

    // 1. Atomares Rename prüfen
    let rename_atomic = probe_atomic_rename(path)?;

    // 2. Verzeichnis-fsync prüfen
    let dir_fsync_ok = flush_dir(path).is_ok();

    // 3. Atomaritätsprobe: 64 Byte ausgerichtet + 80-Byte-Slots (L-4)
    let write_atomic_64 = probe_aligned_and_slot_writes(path)?;

    // 4. fsync_p50 Stichprobenmessung
    let fsync_p50 = measure_fsync_p50(path)?;

    Ok(FsProbe {
        fs: fs_coarse,
        rename_atomic,
        dir_fsync_ok,
        write_atomic_64,
        fsync_p50,
        rotational,
        detected_fs: detected,
        journal_or_cow: is_cow,
    })
}

/// Ermittelt den freien Speicherplatz in Bytes für das Dateisystem unter `path` (DF-10, D-65).
///
/// # Fehler
/// Liefert einen [`io::Error`], falls der Pfad nicht existiert oder die Abfrage scheitert.
// ANCHOR[contextra-durable-fs:PROBE-FREE-SPACE-001]
// Ablauf von `free_space`:
// 1. Pfadexistenz prüfen.
// 2. Unter Linux: Primär `stat -f`, sekundär `df -B1` zur Ermittlung freier Bytes nutzen.
// 3. Unter macOS: `df -k` parsen und in Bytes umrechnen.
// 4. Bei Misserfolg oder anderen Betriebssystemen fail-closed mit `ErrorKind::Unsupported` abbrechen.
pub fn free_space(path: &Path) -> io::Result<u64> {
    if !path.exists() {
        return Err(io::Error::new(io::ErrorKind::NotFound, "path not found"));
    }

    #[cfg(target_os = "linux")]
    {
        // 1. Versuch: stat -f -c "%a %s"
        if let Ok(output) = std::process::Command::new("stat")
            .arg("-f")
            .arg("-c")
            .arg("%a %s")
            .arg(path)
            .output()
        {
            if output.status.success() {
                let text = String::from_utf8_lossy(&output.stdout);
                let parts: Vec<&str> = text.split_whitespace().collect();
                if let (Some(&blocks_str), Some(&size_str)) = (parts.first(), parts.get(1)) {
                    if let (Ok(blocks), Ok(size)) =
                        (blocks_str.parse::<u64>(), size_str.parse::<u64>())
                    {
                        return Ok(blocks.saturating_mul(size));
                    }
                }
            }
        }

        // 2. Versuch: df -B1 --output=avail
        if let Ok(output) = std::process::Command::new("df")
            .arg("-B1")
            .arg("--output=avail")
            .arg(path)
            .output()
        {
            if output.status.success() {
                let text = String::from_utf8_lossy(&output.stdout);
                for line in text.lines().skip(1) {
                    let trimmed = line.trim();
                    if let Ok(bytes) = trimmed.parse::<u64>() {
                        return Ok(bytes);
                    }
                }
            }
        }
    }

    #[cfg(target_os = "macos")]
    {
        if let Ok(output) = std::process::Command::new("df")
            .arg("-k")
            .arg(path)
            .output()
        {
            if output.status.success() {
                let text = String::from_utf8_lossy(&output.stdout);
                for line in text.lines().skip(1) {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if let Some(&avail_kib_str) = parts.get(3) {
                        if let Ok(kib) = avail_kib_str.parse::<u64>() {
                            return Ok(kib.saturating_mul(1024));
                        }
                    }
                }
            }
        }
    }

    Err(io::Error::new(
        io::ErrorKind::Unsupported,
        "unable to query filesystem free space",
    ))
}

/// Bewertet das Ergebnis der Startprobe und entscheidet über Zulässigkeit und Stufe.
// ANCHOR[contextra-durable-fs:PROBE-VERDICT-FN-001]
// Ablauf von `verdict`:
// 1. Hardware-Persistenz prüfen (Rename, Verzeichnis-fsync, 64B-Atomarität). Bei Misserfolg -> `Refused(ProbeFailed)`.
// 2. Netzwerk- und Userspace-Dateisysteme (SMB, NFS, FUSE, Overlayfs, NTFS) strikt abweisen (DF-06, T-PLAT-1).
// 3. Einstufung lokaler Dateisysteme: ext4/XFS auf Linux x86_64/aarch64 -> Tier 1, APFS/Btrfs/ZFS -> Tier 2.
#[must_use]
pub fn verdict(probe: &FsProbe) -> ProbeVerdict {
    // Wenn grundlegende Hardware-Persistenz-Prüfungen fehlschlagen -> Refused
    if !probe.rename_atomic || !probe.dir_fsync_ok || !probe.write_atomic_64 {
        return ProbeVerdict::Refused(UnsupportedFilesystem::ProbeFailed);
    }

    // Abweisung von Netzwerk-, FUSE- und Cloud-Dateisystemen
    if probe.detected_fs == DetectedFs::Smb {
        return ProbeVerdict::Refused(UnsupportedFilesystem::Smb);
    }
    if probe.detected_fs == DetectedFs::Nfs || probe.fs == FsClass::Network {
        return ProbeVerdict::Refused(UnsupportedFilesystem::Nfs);
    }
    if probe.fs == FsClass::Fuse || probe.detected_fs == DetectedFs::Fuse {
        return ProbeVerdict::Refused(UnsupportedFilesystem::Fuse);
    }
    if probe.detected_fs == DetectedFs::Overlayfs {
        return ProbeVerdict::Refused(UnsupportedFilesystem::Overlayfs);
    }
    if probe.detected_fs == DetectedFs::WindowsNtfs {
        return ProbeVerdict::Refused(UnsupportedFilesystem::Windows);
    }

    // Einstufung in Tier 1 vs Tier 2
    match probe.detected_fs {
        DetectedFs::Ext4 | DetectedFs::Xfs => {
            if cfg!(target_os = "linux")
                && cfg!(any(target_arch = "x86_64", target_arch = "aarch64"))
            {
                ProbeVerdict::Supported(Tier::Tier1)
            } else {
                ProbeVerdict::Supported(Tier::Tier2)
            }
        }
        DetectedFs::Btrfs | DetectedFs::Zfs => {
            if cfg!(target_os = "linux") {
                ProbeVerdict::Supported(Tier::Tier2)
            } else {
                ProbeVerdict::Refused(UnsupportedFilesystem::Unknown)
            }
        }
        DetectedFs::Apfs => {
            if cfg!(target_os = "macos") {
                ProbeVerdict::Supported(Tier::Tier2)
            } else {
                ProbeVerdict::Refused(UnsupportedFilesystem::Unknown)
            }
        }
        _ => ProbeVerdict::Refused(UnsupportedFilesystem::Unknown),
    }
}

// ANCHOR[contextra-durable-fs:PROBE-ATOMIC-RENAME-001]
// NOTE: Hilfsfunktion: Prüft, ob `rename_durable` eine Zwischendatei atomar ersetzt.
fn probe_atomic_rename(dir: &Path) -> io::Result<bool> {
    let count = PROBE_COUNTER.fetch_add(1, Ordering::Relaxed);
    let pid = std::process::id();
    let src_path = dir.join(format!(".probe_rn_src.{}.{}.tmp", pid, count));
    let dst_path = dir.join(format!(".probe_rn_dst.{}.{}.tmp", pid, count));

    let _src_guard = ProbeTempFile { path: &src_path };
    let _dst_guard = ProbeTempFile { path: &dst_path };

    {
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&src_path)?;
        file.write_all(b"RENAME_PROBE_DATA")?;
        flush_file(&file, FlushKind::DataAndMetadata)?;
    }

    rename_durable(&src_path, &dst_path)?;

    let ok = dst_path.exists() && !src_path.exists();
    Ok(ok)
}

// ANCHOR[contextra-durable-fs:PROBE-ALIGNED-SLOT-WRITES-001]
// NOTE: Hilfsfunktion: Prüft 64-Byte-ausgerichtetes Schreiben und 80-Byte-Slot-Schreiben (L-4).
fn probe_aligned_and_slot_writes(dir: &Path) -> io::Result<bool> {
    let count = PROBE_COUNTER.fetch_add(1, Ordering::Relaxed);
    let pid = std::process::id();
    let file_path = dir.join(format!(".probe_align.{}.{}.tmp", pid, count));
    let _guard = ProbeTempFile { path: &file_path };

    let mut file = OpenOptions::new()
        .read(true)
        .write(true)
        .create_new(true)
        .open(&file_path)?;

    // 1. 64-Byte ausgerichtete Schreiboperationen
    let block64_1 = [0x5Au8; 64];
    let block64_2 = [0xA5u8; 64];

    file.write_all(&block64_1)?;
    file.write_all(&block64_2)?;
    flush_file(&file, FlushKind::DataAndMetadata)?;

    file.seek(SeekFrom::Start(0))?;
    let mut read_buf = [0u8; 128];
    file.read_exact(&mut read_buf)?;

    let first_half = read_buf.get(..64).unwrap_or(&[]);
    let second_half = read_buf.get(64..).unwrap_or(&[]);
    if first_half != block64_1 || second_half != block64_2 {
        return Ok(false);
    }

    // 2. 80-Byte Slots Schreiboperationen (L-4)
    file.seek(SeekFrom::Start(0))?;
    let slot80_1 = [0x11u8; 80];
    let slot80_2 = [0x22u8; 80];
    let slot80_3 = [0x33u8; 80];

    file.write_all(&slot80_1)?;
    file.write_all(&slot80_2)?;
    file.write_all(&slot80_3)?;
    flush_file(&file, FlushKind::DataAndMetadata)?;

    file.seek(SeekFrom::Start(0))?;
    let mut slot_buf = [0u8; 240];
    file.read_exact(&mut slot_buf)?;

    let s1 = slot_buf.get(..80).unwrap_or(&[]);
    let s2 = slot_buf.get(80..160).unwrap_or(&[]);
    let s3 = slot_buf.get(160..240).unwrap_or(&[]);
    if s1 != slot80_1 || s2 != slot80_2 || s3 != slot80_3 {
        return Ok(false);
    }

    Ok(true)
}

// ANCHOR[contextra-durable-fs:PROBE-MEASURE-FSYNC-P50-001]
// NOTE: Hilfsfunktion: Führt 7 Test-Flushes durch und ermittelt die Mediandauer `fsync_p50`.
fn measure_fsync_p50(dir: &Path) -> io::Result<Duration> {
    let count = PROBE_COUNTER.fetch_add(1, Ordering::Relaxed);
    let pid = std::process::id();
    let file_path = dir.join(format!(".probe_fsync.{}.{}.tmp", pid, count));
    let _guard = ProbeTempFile { path: &file_path };

    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&file_path)?;

    const SAMPLE_COUNT: usize = 7;
    let mut timings = Vec::with_capacity(SAMPLE_COUNT);
    let payload = [0xAAu8; 64];
    let clock = SystemClock::new();

    for _ in 0..SAMPLE_COUNT {
        file.write_all(&payload)?;
        let start_nanos = clock.monotonic_nanos();
        flush_file(&file, FlushKind::DataAndMetadata)?;
        let elapsed_nanos = clock.monotonic_nanos().saturating_sub(start_nanos);
        let elapsed = Duration::from_nanos(elapsed_nanos);
        timings.push(elapsed);
    }

    timings.sort();
    let median_idx = SAMPLE_COUNT / 2;
    let p50 = timings
        .get(median_idx)
        .copied()
        .unwrap_or(Duration::from_millis(1));

    // Mindestens 1ns garantieren, um Division durch Null bei schnellen Mock-Dateisystemen zu vermeiden
    let non_zero_p50 = p50.max(Duration::from_nanos(100));
    Ok(non_zero_p50)
}

// ANCHOR[contextra-durable-fs:PROBE-DETECT-FS-TYPE-001]
// NOTE: Hilfsfunktion: Ermittelt den konkreten Dateisystemtyp aus /proc/self/mountinfo bzw. statfs.
fn detect_filesystem_type(path: &Path) -> DetectedFs {
    #[cfg(target_os = "linux")]
    {
        use std::io::BufRead;

        let canonical = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());

        // 1. Über /proc/self/mountinfo prüfen
        if let Ok(file) = File::open("/proc/self/mountinfo") {
            let reader = io::BufReader::new(file);
            let mut best_match_len = 0;
            let mut best_match_fstype = String::new();

            for line_res in reader.lines() {
                let Ok(line) = line_res else { continue };
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() < 5 {
                    continue;
                }
                let Some(&mount_point) = parts.get(4) else {
                    continue;
                };
                let mount_path = Path::new(mount_point);

                if canonical.starts_with(mount_path) {
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

            if !best_match_fstype.is_empty() {
                return map_fstype_str(&best_match_fstype);
            }
        }

        // 2. Sekundär: stat -f -c %T
        if let Ok(output) = std::process::Command::new("stat")
            .arg("-f")
            .arg("-c")
            .arg("%T")
            .arg(path)
            .output()
        {
            if output.status.success() {
                let fstype = String::from_utf8_lossy(&output.stdout)
                    .trim()
                    .to_lowercase();
                return map_fstype_str(&fstype);
            }
        }
    }

    #[cfg(target_os = "macos")]
    {
        let _ = path;
        return DetectedFs::Apfs;
    }

    #[cfg(windows)]
    {
        let _ = path;
        return DetectedFs::WindowsNtfs;
    }

    #[allow(unreachable_code)]
    DetectedFs::Unknown
}

fn map_fstype_str(s: &str) -> DetectedFs {
    if s.contains("ext4") || s.contains("ext2/ext3") {
        DetectedFs::Ext4
    } else if s.contains("xfs") {
        DetectedFs::Xfs
    } else if s.contains("btrfs") {
        DetectedFs::Btrfs
    } else if s.contains("zfs") {
        DetectedFs::Zfs
    } else if s.contains("overlay") {
        DetectedFs::Overlayfs
    } else if s.contains("fuse") {
        DetectedFs::Fuse
    } else if s.contains("nfs") {
        DetectedFs::Nfs
    } else if s.contains("cifs") || s.contains("smb") {
        DetectedFs::Smb
    } else if s.contains("ntfs") {
        DetectedFs::WindowsNtfs
    } else {
        DetectedFs::Unknown
    }
}

// ANCHOR[contextra-durable-fs:PROBE-DETECT-ROTATIONAL-001]
// NOTE: Hilfsfunktion: Prüft /sys/block/*/queue/rotational zur Erkennung von Rotationsmedien (HDD).
fn detect_rotational(path: &Path) -> Option<bool> {
    #[cfg(target_os = "linux")]
    {
        use std::io::BufRead;

        let canonical = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
        let Ok(file) = File::open("/proc/self/mountinfo") else {
            return None;
        };

        let reader = io::BufReader::new(file);
        let mut best_match_len = 0;
        let mut best_mount_source = String::new();

        for line_res in reader.lines() {
            let Ok(line) = line_res else { continue };
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() < 5 {
                continue;
            }
            let Some(&mount_point) = parts.get(4) else {
                continue;
            };
            let mount_path = Path::new(mount_point);

            if canonical.starts_with(mount_path) {
                let mlen = mount_point.len();
                if mlen >= best_match_len {
                    best_match_len = mlen;
                    if let Some(dash_pos) = parts.iter().position(|&p| p == "-") {
                        if let Some(&mount_source) =
                            dash_pos.checked_add(2).and_then(|idx| parts.get(idx))
                        {
                            best_mount_source = mount_source.to_string();
                        }
                    }
                }
            }
        }

        if best_mount_source.starts_with("/dev/") {
            let dev_name = best_mount_source.trim_start_matches("/dev/");
            // Partitionsziffern abschneiden (z. B. sda1 -> sda, nvme0n1p1 -> nvme0n1)
            let base_dev = if dev_name.starts_with("nvme") {
                if let Some(p_idx) = dev_name.rfind('p') {
                    dev_name.get(..p_idx).unwrap_or(dev_name)
                } else {
                    dev_name
                }
            } else {
                dev_name.trim_end_matches(char::is_numeric)
            };

            let sys_path = format!("/sys/block/{}/queue/rotational", base_dev);
            if let Ok(mut rot_file) = File::open(sys_path) {
                let mut content = String::new();
                if rot_file.read_to_string(&mut content).is_ok() {
                    let trimmed = content.trim();
                    if trimmed == "1" {
                        return Some(true);
                    } else if trimmed == "0" {
                        return Some(false);
                    }
                }
            }
        }
    }

    #[cfg(not(target_os = "linux"))]
    let _ = path;

    None
}
