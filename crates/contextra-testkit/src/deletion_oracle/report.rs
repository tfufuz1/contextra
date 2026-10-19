// FILE-CONTEXT
// STAND: 2026-10-13
// ZWECK: Report-Typen und Verifikation fuer den Canary-Orakel Rohsweep.
// INVARIANTEN: No unsafe code allowed (#![forbid(unsafe_code)]).
// DOKTRIN: Keine Nutzdaten/Nadel-Inhalte in Debug/Display ausgeben. Zero-Panic.

use std::fmt;
use std::io;
use std::path::PathBuf;

/// Single needle match location found during a raw byte sweep.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OrcHit {
    /// Relative path of the file containing the marker.
    pub relative_path: PathBuf,
    /// Index of the needle in the searched needle list.
    pub needle_index: usize,
    /// Absolute byte offset within the file where the needle starts.
    pub offset: u64,
    /// Number of hard links for the file.
    pub nlink: u64,
}

impl fmt::Display for OrcHit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "path={}, needle_index={}, offset={}, nlink={}",
            self.relative_path.display(),
            self.needle_index,
            self.offset,
            self.nlink
        )
    }
}

/// Comprehensive report produced by `raw_sweep`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OrcSweepReport {
    /// Scope of the sweep (always "files_only", as process memory is excluded).
    pub scope: &'static str,
    /// Total count of files scanned during the sweep.
    pub files_scanned: u64,
    /// Total bytes scanned across all files.
    pub bytes_scanned: u64,
    /// List of all detected marker hits.
    pub hits: Vec<OrcHit>,
}

impl fmt::Display for OrcSweepReport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "OrcSweepReport {{ scope: {}, files_scanned: {}, bytes_scanned: {}, hits: {} }}",
            self.scope,
            self.files_scanned,
            self.bytes_scanned,
            self.hits.len()
        )?;
        for hit in &self.hits {
            write!(f, "\n  Hit: {}", hit)?;
        }
        Ok(())
    }
}

impl OrcSweepReport {
    /// Asserts that no residual markers were found in the swept directory.
    ///
    /// Returns `Ok(())` if `hits` is empty, or an `io::Error` containing details
    /// of all detected hits.
    pub fn assert_clean(&self) -> io::Result<()> {
        if self.hits.is_empty() {
            Ok(())
        } else {
            Err(io::Error::new(
                io::ErrorKind::Other,
                format!(
                    "Deletion oracle raw sweep found {} residual marker hit(s):\n{}",
                    self.hits.len(),
                    self
                ),
            ))
        }
    }
}
