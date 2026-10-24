#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use contextra_mcp::sandbox::{McpSandbox, ToolCategory, TOOL_REGISTRY};
use contextra_mcp::McpServer;
use serde_json::json;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tempfile::TempDir;

#[derive(Debug)]
struct MockEmbedder {
    dimension: usize,
}

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
        Box::pin(async move { Ok(vec![0.1f32; self.dimension]) })
    }

    fn embedding_dim(&self) -> usize {
        self.dimension
    }

    fn embed_batch<'a>(
        &'a self,
        texts: &'a [&'a str],
    ) -> contextra_ports::BoxFuture<
        'a,
        std::result::Result<Vec<Vec<f32>>, contextra_ports::EmbeddingError>,
    > {
        Box::pin(async move { Ok(vec![vec![0.1f32; self.dimension]; texts.len()]) })
    }
}

async fn create_mock_server() -> (Arc<McpServer>, TempDir) {
    use contextra::Contextra;
    use contextra_mcp::sandbox::SandboxPolicy;

    let tmp = TempDir::new().expect("temp dir");
    let db = Contextra::open(tmp.path()).await.expect("open db");
    let collection = db.collection("default").await.expect("collection");
    let dim = collection.dimension();
    let embedder = Arc::new(MockEmbedder { dimension: dim });
    let policy = SandboxPolicy {
        allow_db_reads: true,
        allow_db_writes: true,
        allow_code_execution: false,
        allow_cloud_egress: true,
        max_execution_ms: 5_000,
    };
    let sandbox = Arc::new(McpSandbox::new(policy).expect("sandbox new"));
    let server = Arc::new(McpServer::with_sandbox(Arc::new(db), embedder, sandbox));
    (server, tmp)
}

#[tokio::test]
async fn test_tool_registry_alignment_with_tools_list_and_dispatch() {
    let (server, _tmp) = create_mock_server().await;

    // 1. Abfrage von tools/list über Server
    let req = contextra_mcp::protocol::JsonRpcRequest {
        jsonrpc: "2.0".into(),
        id: Some(json!(1)),
        method: "tools/list".into(),
        params: json!({}),
    };
    let response = server.handle(req).await;
    assert_eq!(response.jsonrpc, "2.0");
    let tools_list_val = response.result.expect("result expected")["tools"]
        .as_array()
        .expect("tools array expected")
        .clone();

    let tools_list_names: HashSet<String> = tools_list_val
        .iter()
        .filter_map(|t| t["name"].as_str().map(|s| s.to_string()))
        .collect();

    let registry_names: HashSet<String> =
        TOOL_REGISTRY.iter().map(|t| t.name.to_string()).collect();

    // Verify 1:1 match between TOOL_REGISTRY and tools/list
    assert_eq!(
        registry_names, tools_list_names,
        "Mismatch between TOOL_REGISTRY and tools/list response"
    );

    // Parse server_dispatch.rs to ensure match arms in handle() match TOOL_REGISTRY bidirectionally
    let dispatch_src = std::fs::read_to_string("src/server_dispatch.rs")
        .or_else(|_| std::fs::read_to_string("crates/contextra-mcp/src/server_dispatch.rs"))
        .expect("read server_dispatch.rs");

    for tool_name in &registry_names {
        assert!(
            dispatch_src.contains(&format!("\"{tool_name}\"")),
            "Handler match arm for '{tool_name}' missing in server_dispatch.rs"
        );
    }

    // Bidirectional check: Extract all "contextra_*" tool methods dispatched in server_dispatch.rs
    // and verify that every dispatched tool method is present in TOOL_REGISTRY.
    let mut dispatched_tools: HashSet<String> = HashSet::new();
    for line in dispatch_src.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("\"contextra_") {
            if let Some(end) = trimmed[1..].find('"') {
                let name = &trimmed[1..=end];
                dispatched_tools.insert(name.to_string());
            }
        }
    }

    for dispatched in &dispatched_tools {
        assert!(
            registry_names.contains(dispatched),
            "Dispatched tool '{dispatched}' in server_dispatch.rs is missing from TOOL_REGISTRY"
        );
    }

    // 2. Erwartete Kategorien festschreiben
    // Rationale for contextra_consolidate:
    // contextra_consolidate triggert synchrone/asynchrone Memory Consolidation Passes (Tombstone-Erstellung,
    // Synthese-Chunks, Graph-Anpassungen im Disk-/Speicher-State).
    // Aufgrund dieser realen Schreib- und Mutationswirkung ist contextra_consolidate als DatabaseWrite klassifiziert.
    let expected_categories: HashMap<&'static str, ToolCategory> = HashMap::from([
        ("contextra_search", ToolCategory::DatabaseRead),
        ("contextra_get", ToolCategory::DatabaseRead),
        ("contextra_collections", ToolCategory::DatabaseRead),
        ("contextra_plugin_status", ToolCategory::DatabaseRead),
        ("contextra_explain", ToolCategory::DatabaseRead),
        ("contextra_insert", ToolCategory::DatabaseWrite),
        ("contextra_upsert", ToolCategory::DatabaseWrite),
        ("contextra_delete", ToolCategory::DatabaseWrite),
        ("contextra_forget", ToolCategory::DatabaseWrite),
        ("contextra_relate", ToolCategory::DatabaseWrite),
        ("contextra_relate_n_ary", ToolCategory::DatabaseWrite),
        ("contextra_create_collection", ToolCategory::DatabaseWrite),
        ("contextra_drop_collection", ToolCategory::DatabaseWrite),
        ("contextra_consolidate", ToolCategory::DatabaseWrite),
        ("contextra_cloud_query", ToolCategory::CloudEgress),
    ]);

    assert_eq!(
        registry_names.len(),
        expected_categories.len(),
        "Registry size ({}) does not match expected_categories size ({})",
        registry_names.len(),
        expected_categories.len()
    );

    for tool in TOOL_REGISTRY {
        let name = tool.name;
        let expected_cat = expected_categories
            .get(name)
            .unwrap_or_else(|| panic!("Unmapped tool in test: {name}"));

        // (2.1) Classification via try_classify_method matches expected category
        let actual_cat = McpSandbox::try_classify_method(name)
            .unwrap_or_else(|_| panic!("Tool '{name}' failed try_classify_method"));

        assert_eq!(
            &actual_cat, expected_cat,
            "Tool '{name}' category mismatch: expected {:?}, got {:?}",
            expected_cat, actual_cat
        );

        // (2.2) Tool definition category in TOOL_REGISTRY matches expected category
        assert_eq!(
            &tool.category, expected_cat,
            "TOOL_REGISTRY category for '{name}' mismatch: expected {:?}, got {:?}",
            expected_cat, tool.category
        );

        // (3) Keines der gelisteten Werkzeuge ergibt CodeExecution
        assert_ne!(
            actual_cat,
            ToolCategory::CodeExecution,
            "Tool '{name}' must not classify as CodeExecution"
        );
    }
}
