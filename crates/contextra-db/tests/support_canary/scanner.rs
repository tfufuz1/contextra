use super::canary::{Hit, SearchNeedle};
use std::fs::{self, File};
use std::io::{Read, Result as IoResult};
use std::path::Path;

const CHUNK_SIZE: usize = 64 * 1024; // 64 KiB buffer size

/// Recursively scans a directory for occurrences of any `SearchNeedle`.
/// Uses pure `std::fs` and stream buffer scanning. Does NOT rely on any Contextra decoder or witness logic.
pub fn scan_dir(root: &Path, needles: &[SearchNeedle]) -> Vec<Hit> {
    let mut hits = Vec::new();
    if needles.is_empty() || !root.exists() {
        return hits;
    }

    let max_needle_len = needles.iter().map(|n| n.bytes.len()).max().unwrap_or(0);
    if max_needle_len == 0 {
        return hits;
    }

    let mut stack = vec![root.to_path_buf()];

    while let Some(current_path) = stack.pop() {
        if current_path.is_dir() {
            if let Ok(entries) = fs::read_dir(&current_path) {
                for entry in entries.flatten() {
                    stack.push(entry.path());
                }
            }
        } else if current_path.is_file() {
            if let Ok(file_hits) = scan_file(&current_path, needles, max_needle_len) {
                hits.extend(file_hits);
            }
        }
    }

    hits
}

/// Scans a single file for occurrences of any `SearchNeedle`.
pub fn scan_file(
    file_path: &Path,
    needles: &[SearchNeedle],
    max_needle_len: usize,
) -> IoResult<Vec<Hit>> {
    let mut hits = Vec::new();
    let mut file = File::open(file_path)?;

    let mut buffer = vec![0u8; CHUNK_SIZE];
    let mut carryover = Vec::new();
    let mut file_offset: u64 = 0;

    loop {
        let bytes_read = file.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }

        // Active chunk is carryover from previous chunk + newly read bytes
        let carryover_len = carryover.len();
        let mut active_chunk = Vec::with_capacity(carryover_len + bytes_read);
        active_chunk.extend_from_slice(&carryover);
        active_chunk.extend_from_slice(&buffer[..bytes_read]);

        let chunk_base_offset = file_offset.saturating_sub(carryover_len as u64);

        for needle in needles {
            if needle.bytes.is_empty() {
                continue;
            }
            let n_len = needle.bytes.len();
            if active_chunk.len() < n_len {
                continue;
            }

            for idx in 0..=(active_chunk.len() - n_len) {
                // Deduplication guard: Skip matches that lie completely within carryover from previous chunk
                if carryover_len > 0 && idx + n_len <= carryover_len {
                    continue;
                }

                if &active_chunk[idx..idx + n_len] == needle.bytes.as_slice() {
                    let hit_offset = chunk_base_offset + idx as u64;
                    hits.push(Hit {
                        file_path: file_path.to_path_buf(),
                        offset: hit_offset,
                        encoding: needle.encoding.clone(),
                        canary_id: needle.canary_id.clone(),
                        owner_person: needle.owner_person.clone(),
                    });
                }
            }
        }

        file_offset += bytes_read as u64;

        // Keep carryover slice of length max_needle_len - 1 for boundary straddling
        let overlap_len = max_needle_len.saturating_sub(1);
        if active_chunk.len() >= overlap_len {
            carryover = active_chunk[active_chunk.len() - overlap_len..].to_vec();
        } else {
            carryover = active_chunk;
        }
    }

    Ok(hits)
}
