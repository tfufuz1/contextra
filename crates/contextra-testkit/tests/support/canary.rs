use std::fs::{self, File};
use std::io::{self, Read};
use std::path::Path;

/// Independent verification oracle for residual canary marker scanning in raw database images,
/// WAL files, and sparse disk regions.
///
/// **Constraint (Project Evaluation §9.4):**
/// Must NOT import or use any data structures or logic from `contextra-store`.
#[derive(Debug, Default)]
pub struct CanaryOracle;

impl CanaryOracle {
    /// Creates a new instance of `CanaryOracle`.
    pub fn new() -> Self {
        Self
    }

    /// Recursively scans a directory for any occurrence of `marker` bytes across all files
    /// (including WAL logs, SSTables, temporary files, and sparse regions).
    ///
    /// Returns the total number of occurrences found.
    pub fn scan_directory(&self, dir: &Path, marker: &[u8]) -> io::Result<usize> {
        if marker.is_empty() || !dir.exists() {
            return Ok(0);
        }
        let mut total_occurrences = 0;
        self.scan_dir_recursive(dir, marker, &mut total_occurrences)?;
        Ok(total_occurrences)
    }

    fn scan_dir_recursive(&self, dir: &Path, marker: &[u8], total: &mut usize) -> io::Result<()> {
        let entries = fs::read_dir(dir)?;
        for entry in entries {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                self.scan_dir_recursive(&path, marker, total)?;
            } else if path.is_file() {
                let occurrences = self.scan_file(&path, marker)?;
                *total += occurrences;
            }
        }
        Ok(())
    }

    /// Scans a single file byte-by-byte for occurrences of `marker`.
    ///
    /// Handles streaming chunk reading with boundary overlaps to ensure markers straddling
    /// chunk boundaries and sparse file regions are completely detected.
    pub fn scan_file(&self, path: &Path, marker: &[u8]) -> io::Result<usize> {
        if marker.is_empty() {
            return Ok(0);
        }
        let mut file = match File::open(path) {
            Ok(f) => f,
            Err(e) => return Err(e),
        };

        let metadata = file.metadata()?;
        if metadata.len() < marker.len() as u64 {
            return Ok(0);
        }

        let mut occurrences = 0;
        let chunk_size = 64 * 1024; // 64 KiB chunks
        let overlap = marker.len() - 1;
        let mut buffer = vec![0u8; chunk_size + overlap];
        let mut prefix_len = 0;

        loop {
            let read_slice = &mut buffer[prefix_len..prefix_len + chunk_size];
            let bytes_read = match file.read(read_slice) {
                Ok(n) => n,
                Err(e) => return Err(e),
            };

            if bytes_read == 0 {
                break;
            }

            let valid_len = prefix_len + bytes_read;
            let current_window = &buffer[..valid_len];

            if valid_len >= marker.len() {
                for i in 0..=(valid_len - marker.len()) {
                    if &current_window[i..i + marker.len()] == marker {
                        occurrences += 1;
                    }
                }
            }

            if valid_len >= overlap {
                buffer.copy_within(valid_len - overlap..valid_len, 0);
                prefix_len = overlap;
            } else {
                prefix_len = valid_len;
            }
        }

        Ok(occurrences)
    }

    /// Returns `true` if `marker` is found anywhere within `dir`.
    pub fn contains_marker(&self, dir: &Path, marker: &[u8]) -> io::Result<bool> {
        Ok(self.scan_directory(dir, marker)? > 0)
    }
}
