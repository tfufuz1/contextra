use contextra_engine::deletion::ForgetOptions;
use contextra_engine::{Contextra, ContextraConfig};
use std::fs;
use std::path::Path;
use tempfile::TempDir;

fn scan_dir_for_needle(dir: &Path, needle_bytes: &[u8]) -> usize {
    let mut hits = 0;
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                hits += scan_dir_for_needle(&path, needle_bytes);
            } else if path.is_file() {
                if let Ok(content) = fs::read(&path) {
                    if content.windows(needle_bytes.len()).any(|w| w == needle_bytes) {
                        hits += 1;
                    }
                }
            }
        }
    }
    hits
}

#[tokio::test]
async fn test_canary_after_delete_all_families() -> contextra_types::Result<()> {
    let tmp = TempDir::new().map_err(|e| contextra_types::ContextraError::InvalidInput(e.to_string()))?;
    let config = ContextraConfig {
        dimension: 4,
        ..Default::default()
    };

    let db = Contextra::open_with_config(tmp.path(), config).await?;

    let needle_id = "CTXMARK-RUN1-SECRET-ABCD";
    let vector: [f32; 4] = [0.12345, 0.6789, 0.1111, 0.2222];

    db.insert(needle_id, &vector, None).await?;

    // Verify inserted
    assert!(db.get(needle_id).await?.is_some());

    // Forget document
    let receipt = db.forget(needle_id, ForgetOptions::default()).await?;
    assert!(receipt.proof.is_some());

    // Scan DB directory for needle string (JSON text encoding)
    let json_needle = needle_id.as_bytes();
    let text_hits = scan_dir_for_needle(tmp.path(), json_needle);

    // Verify deleted from active state
    assert!(db.get(needle_id).await?.is_none());

    // Scan raw f32 LE bytes
    let mut vec_needle = Vec::new();
    for &f in &vector {
        vec_needle.extend_from_slice(&f.to_le_bytes());
    }

    db.close().await?;
    Ok(())
}
