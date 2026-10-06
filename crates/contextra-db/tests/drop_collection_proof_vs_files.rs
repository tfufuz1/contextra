use contextra_db::{Contextra, ContextraConfig};
use contextra_engine::DeletionLayer;
use contextra_types::{DistanceMetric, TenantId};
use serde_json::json;
use std::fs;
use std::path::Path;
use tempfile::TempDir;

fn search_dir_for_marker(dir: &Path, marker: &[u8]) -> std::io::Result<bool> {
    if !dir.exists() {
        return Ok(false);
    }
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            if search_dir_for_marker(&path, marker)? {
                return Ok(true);
            }
        } else if path.is_file() {
            if let Ok(contents) = fs::read(&path) {
                if contents.windows(marker.len()).any(|window| window == marker) {
                    return Ok(true);
                }
            }
        }
    }
    Ok(false)
}

#[tokio::test]
async fn test_drop_collection_proof_vs_raw_files() -> Result<(), Box<dyn std::error::Error>> {
    let tmp = TempDir::new()?;
    let config = ContextraConfig {
        dimension: 3,
        max_elements: 1000,
        distance_metric: DistanceMetric::Cosine,
        ..Default::default()
    };
    let db = Contextra::open_with_config(tmp.path(), config).await?;
    let tenant_id = TenantId::try_new(42)?;
    let proof_key = b"01234567890123456789012345678901"; // 32 bytes

    let col_name = "raw_file_scan_col";
    let col = db.collection(col_name).await?;

    let marker_str = "CTX-RAWSCAN-9f8e7d6c5b4a3f2e1d0c9b8a7f6e5d4c3b2a1f0e9d8c"; // 52 bytes
    let marker = marker_str.as_bytes();

    for i in 0..20 {
        let doc_id = format!("doc_{}", i);
        let meta = json!({
            "index": i,
            "marker": marker_str,
            "padding": format!("padding_text_{}_for_data_length", i)
        });
        col.insert(&doc_id, &[1.0, 0.0, 0.0], Some(meta)).await?;
    }

    db.flush().await?;

    let marker_found_pre = search_dir_for_marker(tmp.path(), marker)?;
    if !marker_found_pre {
        return Err(
            format!("Precondition failed: Marker '{}' was not found in any file under DB directory after flush!", marker_str).into()
        );
    }

    let proof = db
        .drop_collection(col_name, tenant_id, proof_key)
        .await?;

    db.flush().await?;

    let residue = search_dir_for_marker(tmp.path(), marker)?;
    eprintln!("Raw file residue scan for drop_collection: residue = {}", residue);

    if residue {
        for layer in &proof.covered_layers {
            if !matches!(layer, DeletionLayer::LsmMemtable) {
                return Err(format!(
                    "Core invariant violated: residue is true, but covered_layers contains unverified layer '{:?}'",
                    layer
                ).into());
            }
        }
    }

    if !proof.covered_layers.contains(&DeletionLayer::LsmMemtable) {
        return Err("covered_layers must contain DeletionLayer::LsmMemtable".into());
    }

    Ok(())
}
