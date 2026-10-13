use contextra_mcp::sandbox::{McpSandbox, SandboxPolicy};
use contextra_mcp::McpServer;
use serde_json::{json, Value};
use std::fs;
use std::path::Path;
use std::sync::Arc;
use tempfile::TempDir;

#[derive(Debug)]
struct MockEmbedder;

impl contextra_ports::EmbeddingProvider for MockEmbedder {
    fn provider_name(&self) -> &str {
        "mock"
    }

    fn embed<'a>(
        &'a self,
        _text: &'a str,
    ) -> contextra_ports::BoxFuture<
        'a,
        std::result::Result<Vec<f32>, contextra_ports::EmbeddingError>,
    > {
        Box::pin(async move { Ok(vec![0.1f32; 768]) })
    }

    fn embedding_dim(&self) -> usize {
        768
    }

    fn embed_batch<'a>(
        &'a self,
        texts: &'a [&'a str],
    ) -> contextra_ports::BoxFuture<
        'a,
        std::result::Result<Vec<Vec<f32>>, contextra_ports::EmbeddingError>,
    > {
        Box::pin(async move { Ok(vec![vec![0.1f32; 768]; texts.len()]) })
    }
}

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
                if contents
                    .windows(marker.len())
                    .any(|window| window == marker)
                {
                    return Ok(true);
                }
            }
        }
    }
    Ok(false)
}

async fn create_mock_server() -> (Arc<McpServer>, Arc<contextra::Contextra>, TempDir) {
    use contextra::Contextra;

    let tmp = TempDir::new().expect("temp dir");
    let db = Arc::new(Contextra::open(tmp.path()).await.expect("open db"));
    let policy = SandboxPolicy {
        allow_db_reads: true,
        allow_db_writes: true,
        allow_code_execution: false,
        allow_cloud_egress: true,
        max_execution_ms: 5_000,
    };
    let sandbox = Arc::new(McpSandbox::new(policy).expect("sandbox new"));
    let server = Arc::new(McpServer::with_sandbox(
        db.clone(),
        Arc::new(MockEmbedder),
        sandbox,
    ));
    (server, db, tmp)
}

// REPRODUKTIONS-TEST FÜR BEFUND G1 (MCP vs DB Tenant-Mismatch bei drop_collection / contextra_forget):
// MCP-Tools (contextra_insert, contextra_get, contextra_search) arbeiten über self.db.collection(name)
// im un-tenantisierten Schlüsselraum (__col:{name}:...).
// contextra_forget ruft self.db.drop_collection(col_name, TenantId::try_new(1)...) auf, was über
// TenantScopedStorage den Prefix t:1:__col:{name}:... bearbeitet.
// Folglich bleiben die echten Daten im un-tenantisierten Raum stehen, contextra_get/search finden sie weiterhin,
// und ein gültiger DeletionProof wird fälschlicherweise ausgestellt.
//
// Dieser Test ist mit #[should_panic] versehen, um CI-Verletzungen zu vermeiden, bis WP-L0.2 das Problem behebt.
// In WP-L0.2 wird #[should_panic] entfernt, sodass der Test regulär GRÜN durchläuft.
#[tokio::test]
#[should_panic(
    expected = "EXPECTED FAILURE / ROT: Document should be deleted after contextra_forget"
)]
async fn test_forget_collection_removes_data() {
    test_forget_collection_removes_data_inner().await.unwrap();
}

async fn test_forget_collection_removes_data_inner() -> Result<(), Box<dyn std::error::Error>> {
    std::env::set_var(
        "CONTEXTRA_DELETION_PROOF_KEY",
        "01234567890123456789012345678901",
    );

    let (server, db, tmp) = create_mock_server().await;

    let col_name = "c";
    let doc_id = "doc_marker_1";
    let marker_str = "CTX-MCP-FORGET-MARKER-9f8e7d6c5b4a3f2e1d0c9b8a7f6e5d4c3b2a1f0e9d8c";
    let text_content = format!("Document content with marker {}", marker_str);

    // 1. Insert document with unique marker into collection "c"
    let insert_req = contextra_mcp::protocol::JsonRpcRequest {
        jsonrpc: "2.0".into(),
        id: Some(json!(1)),
        method: "tools/call".into(),
        params: json!({
            "name": "contextra_insert",
            "arguments": {
                "collection": col_name,
                "id": doc_id,
                "text": text_content
            }
        }),
    };
    let insert_resp = server.handle(insert_req).await;
    let insert_val = serde_json::to_value(&insert_resp)?;
    assert_ne!(
        insert_val["result"]["isError"], true,
        "insert failed: {:?}",
        insert_val
    );

    // Flush DB to ensure data is written to disk
    db.flush().await?;

    // Pre-condition check: marker must exist in raw files after insert & flush
    let marker_found_pre = search_dir_for_marker(tmp.path(), marker_str.as_bytes())?;
    assert!(
        marker_found_pre,
        "Precondition failed: Marker '{}' was not found in DB directory after insert and flush!",
        marker_str
    );

    // 2. Forget collection "c"
    let forget_req = contextra_mcp::protocol::JsonRpcRequest {
        jsonrpc: "2.0".into(),
        id: Some(json!(2)),
        method: "tools/call".into(),
        params: json!({
            "name": "contextra_forget",
            "arguments": {
                "collection": col_name,
                "confirm": true
            }
        }),
    };
    let forget_resp = server.handle(forget_req).await;
    let forget_val = serde_json::to_value(&forget_resp)?;
    assert_ne!(
        forget_val["result"]["isError"], true,
        "forget failed: {:?}",
        forget_val
    );

    let forget_text = forget_val["result"]["content"][0]["text"]
        .as_str()
        .ok_or("missing forget content text")?;
    let forget_json: Value = serde_json::from_str(forget_text)?;

    assert_eq!(forget_json["ok"], true);
    assert_eq!(forget_json["collection"], col_name);
    assert_ne!(
        forget_json["proof"],
        Value::Null,
        "contextra_forget should issue a valid DeletionProof for collection drop"
    );

    // Flush DB after drop_collection
    db.flush().await?;

    // 3. Post-forget verification checks
    // a) contextra_get should return null
    let get_req = contextra_mcp::protocol::JsonRpcRequest {
        jsonrpc: "2.0".into(),
        id: Some(json!(3)),
        method: "tools/call".into(),
        params: json!({
            "name": "contextra_get",
            "arguments": {
                "collection": col_name,
                "id": doc_id
            }
        }),
    };
    let get_resp = server.handle(get_req).await;
    let get_val = serde_json::to_value(&get_resp)?;
    let get_text = get_val["result"]["content"][0]["text"]
        .as_str()
        .ok_or("missing get content text")?;
    let get_json: Value = serde_json::from_str(get_text)?;
    assert!(
        get_json.is_null(),
        "EXPECTED FAILURE / ROT: Document should be deleted after contextra_forget, but contextra_get returned: {:?}",
        get_json
    );

    // b) contextra_collections should not list collection "c"
    let cols_req = contextra_mcp::protocol::JsonRpcRequest {
        jsonrpc: "2.0".into(),
        id: Some(json!(4)),
        method: "tools/call".into(),
        params: json!({
            "name": "contextra_collections",
            "arguments": {}
        }),
    };
    let cols_resp = server.handle(cols_req).await;
    let cols_val = serde_json::to_value(&cols_resp)?;
    let cols_text = cols_val["result"]["content"][0]["text"]
        .as_str()
        .ok_or("missing cols content text")?;
    let cols_json: Value = serde_json::from_str(cols_text)?;
    let collections_list: Vec<String> = serde_json::from_value(cols_json["collections"].clone())?;
    assert!(
        !collections_list.contains(&col_name.to_string()),
        "EXPECTED FAILURE / ROT: Collection 'c' should not be listed in contextra_collections after contextra_forget, found: {:?}",
        collections_list
    );

    // c) contextra_search should return no results
    let search_req = contextra_mcp::protocol::JsonRpcRequest {
        jsonrpc: "2.0".into(),
        id: Some(json!(5)),
        method: "tools/call".into(),
        params: json!({
            "name": "contextra_search",
            "arguments": {
                "collection": col_name,
                "query": "content",
                "k": 10
            }
        }),
    };
    let search_resp = server.handle(search_req).await;
    let search_val = serde_json::to_value(&search_resp)?;
    let search_text = search_val["result"]["content"][0]["text"]
        .as_str()
        .ok_or("missing search content text")?;
    let search_json: Value = serde_json::from_str(search_text)?;
    let search_results = search_json.as_array().ok_or("expected search array")?;
    assert_eq!(
        search_results.len(),
        0,
        "EXPECTED FAILURE / ROT: contextra_search returned results after contextra_forget: {:?}",
        search_results
    );

    // d) Raw file scan for marker
    let residue = search_dir_for_marker(tmp.path(), marker_str.as_bytes())?;
    assert!(
        !residue,
        "EXPECTED FAILURE / ROT: Raw file scan found marker string residue after contextra_forget!"
    );

    Ok(())
}
