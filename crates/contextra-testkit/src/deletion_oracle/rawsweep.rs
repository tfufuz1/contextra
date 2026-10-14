// FILE-CONTEXT
// STAND: 2026-10-13
// ZWECK: Byteweiser Rohsweep ueber das DB-Verzeichnis fuer das Canary-Orakel (Spezifikation T1).
// INVARIANTEN: No unsafe code allowed (##![forbid(unsafe_code)]).
// DOKTRIN: Fail-closed (I/O-Fehler abbrechen). Zero-Panic. Memory-bounded 1-MiB Fenster. Scope: files_only.

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

/// Sweeps the directory tree at `root` recursively for all `needles` and their multi-encoding variants.
///
/// Features:
/// - Automatically expands needles into raw, hex, base64, and UTF-16LE variants.
/// - Streams files in 1-MiB windows with `overlap >= needle.len()` (no RAM ballooning).
/// - Deduplicates boundary matches across streaming chunks.
/// - Recursively scans all files including hidden files, `.tmp`, orphan, and alt-manifest files.
/// - Hardlinks are scanned and counted individually, capturing `nlink`.
/// - Symlinks are not followed to avoid cycles.
/// - Report scope is explicitly set to `files_only` (process RAM excluded).
/// - Fail-closed: any I/O error immediately halts sweep and returns `Result::Err`.
pub fn raw_sweep(root: &Path, needles: &[Vec<u8>]) -> io::Result<OrcSweepReport> {
    if !root.exists() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("Root path does not exist: {}", root.display()),
        ));
    }

    let expanded_needles = expand_needle_encodings(needles);
    let search_needles = if expanded_needles.is_empty() {
        needles
    } else {
        &expanded_needles
    };

    let maxlen = search_needles.iter().map(|n| n.len()).max().unwrap_or(0);
    let overlap = maxlen;

    let mut ctx = SweepContext {
        root,
        needles: search_needles,
        overlap,
        files_scanned: 0,
        bytes_scanned: 0,
        hits: Vec::new(),
    };

    sweep_dir_recursive(&mut ctx, root)?;

    Ok(OrcSweepReport {
        scope: "files_only",
        files_scanned: ctx.files_scanned,
        bytes_scanned: ctx.bytes_scanned,
        hits: ctx.hits,
    })
}

/// Encodes raw needles into four representations: raw, hex, base64, and UTF-16LE.
pub fn expand_needle_encodings(needles: &[Vec<u8>]) -> Vec<Vec<u8>> {
    let mut expanded = Vec::with_capacity(needles.len() * 4);
    for needle in needles {
        if needle.is_empty() {
            continue;
        }

        // 1. Raw
        if !expanded.contains(needle) {
            expanded.push(needle.clone());
        }

        // 2. Hex
        let hex = hex_encode(needle);
        if !hex.is_empty() && !expanded.contains(&hex) {
            expanded.push(hex);
        }

        // 3. Base64
        let b64 = base64_encode(needle);
        if !b64.is_empty() && !expanded.contains(&b64) {
            expanded.push(b64);
        }

        // 4. UTF-16LE
        let u16le = utf16le_encode(needle);
        if !u16le.is_empty() && !expanded.contains(&u16le) {
            expanded.push(u16le);
        }
    }
    expanded
}

fn hex_encode(input: &[u8]) -> Vec<u8> {
    const HEX_CHARS: &[u8; 16] = b"0123456789abcdef";
    let mut out = Vec::with_capacity(input.len() * 2);
    for &b in input {
        out.push(HEX_CHARS[(b >> 4) as usize]);
        out.push(HEX_CHARS[(b & 0x0f) as usize]);
    }
    out
}

fn base64_encode(input: &[u8]) -> Vec<u8> {
    const CHARS: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = Vec::with_capacity((input.len() + 2) / 3 * 4);
    let mut i = 0;
    while i < input.len() {
        let b0 = input[i];
        let b1 = if i + 1 < input.len() { input[i + 1] } else { 0 };
        let b2 = if i + 2 < input.len() { input[i + 2] } else { 0 };

        let triple = ((b0 as u32) << 16) | ((b1 as u32) << 8) | (b2 as u32);

        out.push(CHARS[((triple >> 18) & 0x3f) as usize]);
        out.push(CHARS[((triple >> 12) & 0x3f) as usize]);

        if i + 1 < input.len() {
            out.push(CHARS[((triple >> 6) & 0x3f) as usize]);
        } else {
            out.push(b'=');
        }

        if i + 2 < input.len() {
            out.push(CHARS[(triple & 0x3f) as usize]);
        } else {
            out.push(b'=');
        }

        i += 3;
    }
    out
}

fn utf16le_encode(input: &[u8]) -> Vec<u8> {
    if let Ok(s) = std::str::from_utf8(input) {
        let mut out = Vec::with_capacity(input.len() * 2);
        for u in s.encode_utf16() {
            out.extend_from_slice(&u.to_le_bytes());
        }
        out
    } else {
        let mut out = Vec::with_capacity(input.len() * 2);
        for &b in input {
            out.push(b);
            out.push(0);
        }
        out
    }
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
                if prefix_len > 0 && window_offset + needle.len() <= prefix_len {
                    continue;
                }

                if &valid_buffer[window_offset..window_offset + needle.len()] == needle.as_slice() {
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
