use contextra_testkit::ReferenceModel;
use std::fs;
use std::io;
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

/// Kopiert ein Verzeichnis rekursiv von `src` nach `dst`.
#[allow(dead_code)]
pub fn copy_dir_recursive(src: impl AsRef<Path>, dst: impl AsRef<Path>) -> io::Result<()> {
    let src = src.as_ref();
    let dst = dst.as_ref();
    fs::create_dir_all(dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());
        if ty.is_dir() {
            copy_dir_recursive(&src_path, &dst_path)?;
        } else {
            fs::copy(&src_path, &dst_path)?;
        }
    }
    Ok(())
}

/// Schneidet die neueste (letzte modifizierte) WAL-Datei im Verzeichnis `dir` an `offset` Bytes ab.
#[allow(dead_code)]
pub fn truncate_wal_file(dir: impl AsRef<Path>, offset: u64) -> io::Result<bool> {
    let dir = dir.as_ref();
    let mut wal_files: Vec<_> = fs::read_dir(dir)?
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| {
            p.file_name()
                .and_then(|n| n.to_str())
                .is_some_and(|s| s.contains("wal") || s.ends_with(".log") || s.ends_with(".wal"))
        })
        .collect();

    wal_files.sort();

    if let Some(latest_wal) = wal_files.last() {
        let file = fs::OpenOptions::new().write(true).open(latest_wal)?;
        file.set_len(offset)?;
        Ok(true)
    } else {
        Ok(false)
    }
}

/// Verifiziert, dass die aus `storage` ausgelesenen Daten fuer alle aktiven Keys
/// exakt mit den Erwartungen des `reference_model` zum Zeitpunkt `snapshot_seq` uebereinstimmen.
#[allow(dead_code)]
pub async fn verify_storage_against_reference_model<S>(
    storage: &S,
    model: &ReferenceModel,
    snapshot_seq: u64,
) -> Result<(), String>
where
    S: contextra_core::StorageEngine,
{
    let keys = model.keys_at(snapshot_seq);
    for key in keys {
        let expected = model.get_at(&key, snapshot_seq);
        let actual = storage
            .get(&key)
            .await
            .map_err(|e| format!("Storage read error: {:?}", e))?
            .map(|b| b.to_vec());
        if expected != actual {
            return Err(format!(
                "Mismatch for key {:?}: expected {:?}, got {:?}",
                key, expected, actual
            ));
        }
    }
    Ok(())
}
