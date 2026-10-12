// FILE-CONTEXT
// STAND: 2026-10-12
// ZWECK: Byteweiser Rohsweep ueber das DB-Verzeichnis fuer das Canary-Orakel (Spezifikation T1).
// INVARIANTEN: No unsafe code allowed (#![forbid(unsafe_code)]).
// DOKTRIN: Fail-closed (I/O-Fehler abbrechen). Zero-Panic. Memory-bounded 1-MiB Fenster.

use std::fs::{self, File};
use std::io::{self, Read};
use std::path::Path;
use tracing::debug;

use super::report::{OrcHit, OrcSweepReport};

const CHUNK_SIZE: usize = 1_048_576; // 1 MiB

struct SweepContext<'a> {
    root: &'a Path,
    needles: &'a [Vec<u8>],
    overlap: usize,
    files_scanned: u64,
    bytes_scanned: u64,
    hits: Vec<OrcHit>,
}

/// Sweeps the directory tree at `root` recursively for all `needles`.
///
/// Features:
/// - Streams files in 1-MiB windows with `(maxlen-1)` overlap (no RAM ballooning).
/// - Recursively scans all files including hidden files, `.tmp`, orphan, and alt-manifest files.
/// - Hardlinks are scanned and counted individually, capturing `nlink`.
/// - Symlinks are not followed to avoid cycles, but logged/reported.
/// - Fail-closed: any I/O error immediately halts sweep and returns `Result::Err`.
pub fn raw_sweep(root: &Path, needles: &[Vec<u8>]) -> io::Result<OrcSweepReport> {
    if !root.exists() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("Root path does not exist: {}", root.display()),
        ));
    }

    let maxlen = needles.iter().map(|n| n.len()).max().unwrap_or(0);
    let overlap = if maxlen > 0 { maxlen - 1 } else { 0 };

    let mut ctx = SweepContext {
        root,
        needles,
        overlap,
        files_scanned: 0,
        bytes_scanned: 0,
        hits: Vec::new(),
    };

    sweep_dir_recursive(&mut ctx, root)?;

    Ok(OrcSweepReport {
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
            debug!("Skipping symlink traversal: {}", path.display());
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
        let file_len = metadata.len();
        ctx.bytes_scanned += file_len;
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
                if &valid_buffer[window_offset..window_offset + needle.len()] == needle.as_slice()
                {
                    let absolute_offset =
                        (file_bytes_scanned - valid_len as u64) + window_offset as u64;
                    ctx.hits.push(OrcHit {
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
