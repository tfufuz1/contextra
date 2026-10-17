// FILE-CONTEXT
// STAND: 2026-10-15
// ZWECK: Bytegenauer Rohsweep ueber ein Store-Verzeichnis fuer Loeschgarantien (Rohsweep).
// INVARIANTEN: #![forbid(unsafe_code)], Fail-closed I/O, streaming window in 1-MiB Blöcken.

use std::fs::{self, File};
use std::io::{self, Read};
use std::path::{Path, PathBuf};

const CHUNK_SIZE: usize = 1_048_576; // 1 MiB

/// Individual hit record found during raw sweep.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SweepHit {
    /// Relative file path within scanned store directory.
    pub relative_path: PathBuf,
    /// Index of the needle that matched.
    pub needle_index: usize,
    /// Absolute byte offset within the file where the needle was found.
    pub offset: u64,
    /// Number of hard links on the file.
    pub nlink: u64,
}

/// Consolidated report returned by raw sweep.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SweepReport {
    /// Total number of files scanned.
    pub files_scanned: u64,
    /// Total bytes scanned across all files.
    pub bytes_scanned: u64,
    /// List of hits found.
    pub hits: Vec<SweepHit>,
}

impl SweepReport {
    /// Returns `true` if no needles were found in any scanned file.
    pub fn is_clean(&self) -> bool {
        self.hits.is_empty()
    }

    /// Returns total number of hits found.
    pub fn hit_count(&self) -> usize {
        self.hits.len()
    }
}

struct SweepContext<'a> {
    root: &'a Path,
    needles: &'a [Vec<u8>],
    overlap: usize,
    files_scanned: u64,
    bytes_scanned: u64,
    hits: Vec<SweepHit>,
}

/// Performs a byte-exact raw sweep over all files in `root` for the given `needles`.
pub fn raw_sweep(root: &Path, needles: &[Vec<u8>]) -> io::Result<SweepReport> {
    if !root.exists() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("Root path does not exist: {}", root.display()),
        ));
    }

    let max_len = needles.iter().map(|n| n.len()).max().unwrap_or(0);
    let overlap = max_len;

    let mut ctx = SweepContext {
        root,
        needles,
        overlap,
        files_scanned: 0,
        bytes_scanned: 0,
        hits: Vec::new(),
    };

    sweep_dir_recursive(&mut ctx, root)?;

    Ok(SweepReport {
        files_scanned: ctx.files_scanned,
        bytes_scanned: ctx.bytes_scanned,
        hits: ctx.hits,
    })
}

fn sweep_dir_recursive(ctx: &mut SweepContext<'_>, dir: &Path) -> io::Result<()> {
    let entries = fs::read_dir(dir)?;

    for entry in entries {
        let entry = entry?;
        let path = entry.path();
        let symlink_meta = fs::symlink_metadata(&path)?;
        let file_type = symlink_meta.file_type();

        if file_type.is_symlink() {
            continue;
        }

        if file_type.is_dir() {
            sweep_dir_recursive(ctx, &path)?;
        } else if file_type.is_file() {
            sweep_single_file(ctx, &path, &symlink_meta)?;
        }
    }

    Ok(())
}

fn sweep_single_file(
    ctx: &mut SweepContext<'_>,
    path: &Path,
    metadata: &fs::Metadata,
) -> io::Result<()> {
    let mut file = File::open(path)?;
    ctx.files_scanned += 1;

    #[cfg(unix)]
    let nlink = {
        use std::os::unix::fs::MetadataExt;
        metadata.nlink()
    };
    #[cfg(not(unix))]
    let nlink = 1u64;

    let rel_path = path.strip_prefix(ctx.root).unwrap_or(path).to_path_buf();

    if ctx.needles.is_empty() {
        ctx.bytes_scanned += metadata.len();
        return Ok(());
    }

    let mut buffer = vec![0u8; CHUNK_SIZE + ctx.overlap];
    let mut prefix_len = 0usize;
    let mut file_bytes_scanned = 0u64;

    loop {
        let read_slice = &mut buffer[prefix_len..prefix_len + CHUNK_SIZE];
        let bytes_read = file.read(read_slice)?;

        if bytes_read == 0 {
            break;
        }

        file_bytes_scanned += bytes_read as u64;
        ctx.bytes_scanned += bytes_read as u64;

        let valid_len = prefix_len + bytes_read;
        let valid_buffer = &buffer[..valid_len];

        for (needle_idx, needle) in ctx.needles.iter().enumerate() {
            if needle.is_empty() || valid_len < needle.len() {
                continue;
            }

            let last_possible_offset = valid_len - needle.len();
            for window_offset in 0..=last_possible_offset {
                if prefix_len > 0 && window_offset + needle.len() <= prefix_len {
                    continue;
                }

                if &valid_buffer[window_offset..window_offset + needle.len()] == needle.as_slice() {
                    let absolute_offset =
                        (file_bytes_scanned - valid_len as u64) + window_offset as u64;
                    ctx.hits.push(SweepHit {
                        relative_path: rel_path.clone(),
                        needle_index: needle_idx,
                        offset: absolute_offset,
                        nlink,
                    });
                }
            }
        }

        if valid_len >= ctx.overlap {
            buffer.copy_within(valid_len - ctx.overlap..valid_len, 0);
            prefix_len = ctx.overlap;
        } else {
            prefix_len = valid_len;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::tempdir;

    #[test]
    #[allow(clippy::unwrap_used)]
    fn test_raw_sweep_finds_known_needles() {
        let dir = tempdir().unwrap();
        let sub_dir = dir.path().join("sst");
        fs::create_dir(&sub_dir).unwrap();

        let file1 = sub_dir.join("000001.sst");
        let mut f1 = File::create(&file1).unwrap();
        f1.write_all(b"HEADER_DATA_SECRET_KEY_12345_FOOTER")
            .unwrap();

        let needles = vec![b"SECRET_KEY_12345".to_vec(), b"MISSING_KEY".to_vec()];
        let report = raw_sweep(dir.path(), &needles).unwrap();

        assert_eq!(report.files_scanned, 1);
        assert_eq!(report.hit_count(), 1);
        assert_eq!(report.hits[0].needle_index, 0);
        assert_eq!(report.hits[0].offset, 12);
        assert_eq!(
            report.hits[0].relative_path,
            PathBuf::from("sst/000001.sst")
        );
    }

    #[test]
    #[allow(clippy::unwrap_used)]
    fn test_defective_erasure_detected() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("data.sst");
        let mut f = File::create(&file_path).unwrap();
        f.write_all(b"DOCUMENT_DATA_KEY_TO_BE_PURGED_RESIDUAL")
            .unwrap();

        let needles = vec![b"KEY_TO_BE_PURGED".to_vec()];
        let report = raw_sweep(dir.path(), &needles).unwrap();

        assert!(
            !report.is_clean(),
            "Raw sweep must detect defective erasure / residual data"
        );
        assert_eq!(report.hit_count(), 1);
    }
}
