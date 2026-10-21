// FILE-CONTEXT
// STAND: 2026-10-22
// ZWECK: Inspection of /proc/self/maps and /proc/self/mem without unsafe for canary verification.
// INVARIANTEN: Zero-Panic, pure std (no unsafe code), fail-closed, explicit Unsupported on non-Linux.

use std::fmt;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Read, Seek, SeekFrom};

/// Error types for process image probing.
#[derive(Debug)]
pub enum ProbeError {
    /// Probing /proc/self/mem is unsupported on non-Linux platforms or due to security policy.
    Unsupported,
    /// I/O error occurred during probing.
    Io(io::Error),
    /// Invalid input parameters.
    InvalidInput(String),
}

impl fmt::Display for ProbeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProbeError::Unsupported => write!(
                f,
                "ProcessImageProbe is unsupported on non-Linux operating systems"
            ),
            ProbeError::Io(err) => write!(f, "I/O error during process image scan: {}", err),
            ProbeError::InvalidInput(msg) => write!(f, "Invalid probe parameter: {}", msg),
        }
    }
}

impl std::error::Error for ProbeError {}

impl From<io::Error> for ProbeError {
    fn from(err: io::Error) -> Self {
        ProbeError::Io(err)
    }
}

/// A hit recorded during process image scanning.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProcessMemoryHit {
    /// Start address of the memory region.
    pub region_start: u64,
    /// End address of the memory region.
    pub region_end: u64,
    /// Exact virtual memory address of the match.
    pub match_address: u64,
}

/// Report summarizing process image scan results.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProbeReport {
    /// Total readable memory regions scanned.
    pub regions_scanned: u64,
    /// Total bytes read from process virtual memory.
    pub bytes_scanned: u64,
    /// Total region read errors encountered (and non-fatally recorded).
    pub region_errors: u64,
    /// Number of matches found.
    pub hits_count: u64,
    /// List of hits.
    pub hits: Vec<ProcessMemoryHit>,
}

/// Inspector for process virtual memory via `/proc/self/maps` and `/proc/self/mem`.
pub struct ProcessImageProbe;

impl ProcessImageProbe {
    /// Scans the virtual memory of the current process for occurrences of `marker`.
    ///
    /// On Linux:
    /// - Parses `/proc/self/maps` to identify readable virtual memory regions (`r--p` or `rw-p`).
    /// - Skips non-readable regions, device mappings, `[vvar]`, `[vdso]`, `[vsyscall]`.
    /// - Opens `/proc/self/mem` and seeks to region start addresses using standard `File` and `Seek` (without `unsafe`).
    /// - Reads memory in chunks, scanning for `marker`.
    /// - Counts errors per region without discarding already collected data.
    ///
    /// On non-Linux:
    /// - Returns `Err(ProbeError::Unsupported)`.
    pub fn scan_self(marker: &[u8]) -> Result<ProbeReport, ProbeError> {
        if marker.is_empty() {
            return Err(ProbeError::InvalidInput(
                "Marker cannot be empty".to_string(),
            ));
        }

        #[cfg(not(target_os = "linux"))]
        {
            let _ = marker;
            Err(ProbeError::Unsupported)
        }

        #[cfg(target_os = "linux")]
        {
            Self::scan_self_linux(marker)
        }
    }

    #[cfg(target_os = "linux")]
    fn scan_self_linux(marker: &[u8]) -> Result<ProbeReport, ProbeError> {
        let maps_file = match File::open("/proc/self/maps") {
            Ok(f) => f,
            Err(e) if e.kind() == io::ErrorKind::NotFound => return Err(ProbeError::Unsupported),
            Err(e) => return Err(ProbeError::Io(e)),
        };

        let mem_file = match File::open("/proc/self/mem") {
            Ok(f) => f,
            Err(e)
                if e.kind() == io::ErrorKind::NotFound
                    || e.kind() == io::ErrorKind::PermissionDenied =>
            {
                return Err(ProbeError::Unsupported)
            }
            Err(e) => return Err(ProbeError::Io(e)),
        };

        let reader = BufReader::new(maps_file);
        let mut report = ProbeReport {
            regions_scanned: 0,
            bytes_scanned: 0,
            region_errors: 0,
            hits_count: 0,
            hits: Vec::new(),
        };

        let mut mem_handle = mem_file;

        for line_res in reader.lines() {
            let line = match line_res {
                Ok(l) => l,
                Err(_) => {
                    report.region_errors += 1;
                    continue;
                }
            };

            let (start_addr, end_addr, perms, pathname) = match parse_maps_line(&line) {
                Some(parsed) => parsed,
                None => continue,
            };

            // Only scan readable memory regions ('r')
            if !perms.starts_with('r') {
                continue;
            }

            // Skip special kernel device/variable mappings that cannot be safely read via /proc/self/mem
            if pathname == "[vvar]"
                || pathname == "[vdso]"
                || pathname == "[vsyscall]"
                || pathname.starts_with("/dev/")
            {
                continue;
            }

            report.regions_scanned += 1;
            let region_len = end_addr.saturating_sub(start_addr);
            if region_len == 0 {
                continue;
            }

            scan_region_linux(&mut mem_handle, start_addr, end_addr, marker, &mut report);
        }

        Ok(report)
    }
}

#[cfg(target_os = "linux")]
fn parse_maps_line(line: &str) -> Option<(u64, u64, &str, &str)> {
    // Example: 7f1234560000-7f1234570000 r--p 00000000 08:01 12345 /lib/...
    let mut parts = line.split_whitespace();
    let addrs = parts.next()?;
    let perms = parts.next()?;
    let _offset = parts.next()?;
    let _dev = parts.next()?;
    let _inode = parts.next()?;
    let pathname = parts.next().unwrap_or("");

    let dash_idx = addrs.find('-')?;
    let start_str = &addrs[..dash_idx];
    let end_str = &addrs[dash_idx + 1..];

    let start_addr = u64::from_str_radix(start_str, 16).ok()?;
    let end_addr = u64::from_str_radix(end_str, 16).ok()?;

    Some((start_addr, end_addr, perms, pathname))
}

#[cfg(target_os = "linux")]
fn scan_region_linux(
    mem_file: &mut File,
    start_addr: u64,
    end_addr: u64,
    marker: &[u8],
    report: &mut ProbeReport,
) {
    if mem_file.seek(SeekFrom::Start(start_addr)).is_err() {
        report.region_errors += 1;
        return;
    }

    let region_len = (end_addr - start_addr) as usize;
    let chunk_size = 65_536.min(region_len); // 64 KiB buffer
    let overlap = marker.len().saturating_sub(1);
    let mut buffer = vec![0u8; chunk_size + overlap];

    let mut remaining = region_len;
    let mut current_addr = start_addr;
    let mut prefix_len = 0;

    while remaining > 0 {
        let to_read = chunk_size.min(remaining);
        let read_slice = &mut buffer[prefix_len..prefix_len + to_read];

        match mem_file.read(read_slice) {
            Ok(0) => break,
            Ok(bytes_read) => {
                report.bytes_scanned += bytes_read as u64;
                remaining -= bytes_read;

                let valid_len = prefix_len + bytes_read;
                let valid_buffer = &buffer[..valid_len];

                if valid_len >= marker.len() {
                    let last_offset = valid_len - marker.len();
                    for window_offset in 0..=last_offset {
                        if prefix_len > 0 && window_offset + marker.len() <= prefix_len {
                            continue;
                        }

                        if &valid_buffer[window_offset..window_offset + marker.len()] == marker {
                            let match_addr =
                                current_addr + window_offset as u64 - prefix_len as u64;
                            report.hits_count += 1;
                            report.hits.push(ProcessMemoryHit {
                                region_start: start_addr,
                                region_end: end_addr,
                                match_address: match_addr,
                            });
                        }
                    }
                }

                current_addr += bytes_read as u64;
                if valid_len >= overlap {
                    buffer.copy_within(valid_len - overlap..valid_len, 0);
                    prefix_len = overlap;
                } else {
                    prefix_len = valid_len;
                }
            }
            Err(_) => {
                report.region_errors += 1;
                break;
            }
        }
    }
}
