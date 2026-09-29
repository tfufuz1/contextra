use contextra_testkit::ReferenceModel;
use std::fs;
use std::path::Path;

/// Returns current process open file descriptor count.
#[allow(dead_code)]
pub fn get_open_fd_count() -> usize {
    if let Ok(entries) = fs::read_dir("/proc/self/fd") {
        entries.filter_map(|e| e.ok()).count()
    } else {
        0
    }
}

/// Returns current process Resident Set Size (RSS) in bytes.
#[allow(dead_code)]
pub fn get_rss_bytes() -> usize {
    if let Ok(statm) = fs::read_to_string("/proc/self/statm") {
        let parts: Vec<&str> = statm.split_whitespace().collect();
        if parts.len() >= 2 {
            if let Ok(pages) = parts[1].parse::<usize>() {
                return pages * 4096; // 4KB page size
            }
        }
    }
    0
}

/// Recursively copies directory contents from `src` to `dst`.
#[allow(dead_code)]
pub fn copy_dir_recursive(src: &Path, dst: &Path) -> std::io::Result<()> {
    fs::create_dir_all(dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        let dest_path = dst.join(entry.file_name());
        if file_type.is_dir() {
            copy_dir_recursive(&entry.path(), &dest_path)?;
        } else {
            fs::copy(entry.path(), dest_path)?;
        }
    }
    Ok(())
}

/// Truncates the specified WAL file at `offset` bytes.
#[allow(dead_code)]
pub fn truncate_wal_file(wal_path: &Path, offset: usize) -> std::io::Result<()> {
    if !wal_path.exists() {
        return Ok(());
    }
    let data = fs::read(wal_path)?;
    let cutoff = offset.min(data.len());
    fs::write(wal_path, &data[..cutoff])?;
    Ok(())
}

/// Verifies that all entries in `reference_model` up to `confirmed_seq` are present and equal in `storage`.
#[allow(dead_code)]
pub async fn verify_storage_against_reference_model(
    storage: &contextra_store::lsm::LsmStorage,
    model: &ReferenceModel,
    confirmed_seq: u64,
) -> Result<(), String> {
    use contextra_core::StorageEngine;

    let snapshot_map = model.snapshot_map_at(confirmed_seq);
    for (key, expected_val) in snapshot_map {
        let actual_val = storage
            .get(&key)
            .await
            .map_err(|e| format!("Storage get error for key {:?}: {e}", String::from_utf8_lossy(&key)))?;

        if actual_val.as_deref() != Some(expected_val.as_slice()) {
            return Err(format!(
                "Mismatch at seq {confirmed_seq} for key {:?}: expected {:?}, got {:?}",
                String::from_utf8_lossy(&key),
                String::from_utf8_lossy(&expected_val),
                actual_val.as_ref().map(|b| String::from_utf8_lossy(b))
            ));
        }
    }
    Ok(())
}
