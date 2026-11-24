//! # Salt- und Verzeichnisinfrastruktur fuer Recovery (Ring 1)
//!
//! ZWECK: Generierung und atomare Persistierung von kryptographischem Salt sowie Verzeichnis-Pristine-Pruefung.
//! INVARIANTEN: Atomares Schreiben ueber temporaere Datei und fsync des Parent-Directories.

use crate::error::{StoreError as ContextraError, StoreResult as Result};
use std::sync::atomic::{AtomicU64, Ordering};

/// Process-wide counter for deterministic temporary file naming.
/// `Ordering::Relaxed` is sufficient because process ID + atomic sequence guarantees
/// intra-process collision freedom, while process ID guarantees inter-process collision freedom.
static TEMP_FILE_COUNTER: AtomicU64 = AtomicU64::new(1);

/// Liefert den naechsten Zaehler fuer temporaere Dateien im Prozess.
pub(crate) fn next_temp_file_counter() -> u64 {
    TEMP_FILE_COUNTER.fetch_add(1, Ordering::Relaxed)
}

/// Erzeugt ein 32-Byte kryptographisches Salt mittels uebergebener [`KeyMaterialSource`].
pub(crate) fn generate_crypto_salt_with_source(
    key_source: &dyn crate::key_source::KeyMaterialSource,
) -> Result<[u8; 32]> {
    let mut buf = [0u8; 32];
    key_source.fill_key_bytes(&mut buf)?;
    Ok(buf)
}

/// Erzeugt ein 32-Byte kryptographisches Salt mit der Standard-OS-Quelle.
pub(crate) fn generate_crypto_salt() -> Result<[u8; 32]> {
    let source = crate::key_source::OsKeyMaterialSource;
    generate_crypto_salt_with_source(&source)
}

/// Schreibt das Salt atomar ueber eine temporaere Datei mit fsync.
pub(crate) async fn write_salt_atomically(
    salt_path: &std::path::Path,
    buf: &[u8; 32],
) -> Result<()> {
    let parent = salt_path
        .parent()
        .ok_or_else(|| ContextraError::Storage("Invalid salt path parent".into()))?;

    let pid = std::process::id();

    const MAX_ATTEMPTS: usize = 100;
    let mut attempt = 0;
    let mut tmp_path;
    let file = loop {
        attempt += 1;
        let counter = next_temp_file_counter();
        tmp_path = parent.join(format!("SALT.tmp.{}.{}", pid, counter));

        match tokio::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&tmp_path)
            .await
        {
            Ok(f) => break f,
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {
                if attempt >= MAX_ATTEMPTS {
                    return Err(ContextraError::Storage(format!(
                        "Exhausted {} attempts trying to create temporary SALT file {:?}",
                        MAX_ATTEMPTS, tmp_path
                    )));
                }
                continue;
            }
            Err(e) => {
                return Err(ContextraError::Storage(format!(
                    "Failed to create temp SALT file: {}",
                    e
                )));
            }
        }
    };

    let buf_copy = *buf;
    let write_res: Result<()> = async {
        let mut std_file = file.into_std().await;

        tokio::task::spawn_blocking(move || -> Result<()> {
            use std::io::Write;
            std_file.write_all(&buf_copy).map_err(|e| {
                ContextraError::Storage(format!("Failed to write SALT bytes: {}", e))
            })?;
            std_file.sync_all().map_err(|e| {
                ContextraError::Storage(format!("Failed to sync temp SALT file: {}", e))
            })?;
            Ok(())
        })
        .await
        .map_err(|e| ContextraError::Storage(format!("Join error during SALT write: {}", e)))??;

        tokio::fs::rename(&tmp_path, salt_path).await.map_err(|e| {
            ContextraError::Storage(format!("Failed to rename temp SALT file: {}", e))
        })?;

        crate::util::fsync_parent_dir(salt_path).await?;
        Ok(())
    }
    .await;

    if write_res.is_err() {
        if let Err(e) = tokio::fs::remove_file(&tmp_path).await {
            if e.kind() != std::io::ErrorKind::NotFound {
                tracing::warn!("Failed to remove temporary SALT file {:?}: {}", tmp_path, e);
            }
        }
    }

    write_res
}

/// Prueft, ob das angegebene Verzeichnis frei von WALs, SSTables und Manifest ist.
pub(crate) async fn directory_is_pristine(path: &std::path::Path) -> Result<bool> {
    let mut entries = tokio::fs::read_dir(path).await.map_err(|e| {
        ContextraError::Storage(format!("Failed to read data dir for pristine check: {e}"))
    })?;

    while let Some(entry) = entries.next_entry().await.map_err(|e| {
        ContextraError::Storage(format!(
            "Failed to read directory entry for pristine check: {e}"
        ))
    })? {
        let name = entry.file_name();
        let name_str = name.to_string_lossy();
        if name_str.starts_with("wal-")
            || name_str == "wal.log"
            || name_str.ends_with(".sst")
            || name_str == "MANIFEST"
        {
            return Ok(false);
        }
    }

    Ok(true)
}
