use crate::protocol::McpError;
use crate::server::McpServer;
use crate::validation::validate_collection_name;
use contextra_rank::fusion::ProvenanceRecord;
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Request payload for `contextra_explain`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExplainRequest {
    /// Target document identifier.
    pub id: String,
    /// Target collection name (defaults to "default").
    #[serde(default = "default_collection")]
    pub collection: String,
}

fn default_collection() -> String {
    "default".to_string()
}

/// Response payload for `contextra_explain`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ExplainResponse {
    /// Document identifier.
    pub id: String,
    /// Collection name.
    pub collection: String,
    /// Detailed provenance record if available.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provenance: Option<ProvenanceRecord>,
    /// Deterministic human-readable explanation string.
    pub explanation: String,
    /// Normative content provenance tag (§5.3.1).
    pub content_provenance: String,
}

impl McpServer {
    /// Handles the `contextra_explain` tool call.
    pub(crate) async fn handle_explain(&self, args: &Value) -> Result<Value, McpError> {
        self.sandbox
            .validate_tool_call("contextra_explain", args)
            .map_err(McpError::from)?;

        let id = match args.get("id") {
            Some(v) => {
                let s = v.as_str().ok_or_else(|| {
                    McpError::invalid_params("Invalid params: 'id' must be a string")
                })?;
                if s.trim().is_empty() {
                    return Err(McpError::invalid_params("id cannot be empty"));
                }
                if s.len() > 256 {
                    return Err(McpError::invalid_params(
                        "id length exceeds limit: max 256 chars",
                    ));
                }
                s
            }
            None => {
                return Err(McpError::invalid_params(
                    "id fehlt: missing required field 'id'",
                ));
            }
        };

        let col_name = if let Some(col_val) = args.get("collection") {
            let s = col_val.as_str().ok_or_else(|| {
                McpError::invalid_params("Invalid params: 'collection' must be a string")
            })?;
            if s.trim().is_empty() {
                "default"
            } else {
                validate_collection_name(s)?;
                s
            }
        } else {
            "default"
        };

        let col = self.db.collection(col_name).await.map_err(McpError::from)?;
        let doc_opt = col.get(id).await.map_err(McpError::from)?;

        let (provenance_opt, explanation) = match doc_opt {
            Some(doc) => {
                let prov_opt: Option<ProvenanceRecord> = doc
                    .metadata
                    .as_ref()
                    .and_then(|m| m.get("provenance"))
                    .and_then(|p| serde_json::from_value(p.clone()).ok());

                let expl = match &prov_opt {
                    Some(prov) => prov.explain_human_readable(),
                    None => "Keine Provenienzdaten für dieses Dokument vorhanden.".to_string(),
                };
                (prov_opt, expl)
            }
            None => (None, "Dokument nicht gefunden.".to_string()),
        };

        let response = ExplainResponse {
            id: id.to_string(),
            collection: col_name.to_string(),
            provenance: provenance_opt,
            explanation,
            content_provenance: "retrieved_untrusted_data".to_string(),
        };

        serde_json::to_value(&response)
            .map_err(|e| McpError::internal_error(format!("Response serialization error: {e}")))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocol::JsonRpcRequest;
    use crate::sandbox::{McpSandbox, SandboxPolicy};
    use contextra::Contextra;
    use contextra_ports::BoxFuture;
    use serde_json::json;
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
        ) -> BoxFuture<'a, std::result::Result<Vec<f32>, contextra_ports::EmbeddingError>> {
            Box::pin(async move { Ok(vec![0.1f32; self.dimension]) })
        }

        fn embedding_dim(&self) -> usize {
            self.dimension
        }

        fn embed_batch<'a>(
            &'a self,
            texts: &'a [&'a str],
        ) -> BoxFuture<'a, std::result::Result<Vec<Vec<f32>>, contextra_ports::EmbeddingError>> {
            Box::pin(async move { Ok(vec![vec![0.1f32; self.dimension]; texts.len()]) })
        }
    }

    async fn create_explain_test_server() -> (Arc<McpServer>, TempDir) {
        let tmp = TempDir::new().expect("temp dir");
        let db = Contextra::open(tmp.path()).await.expect("open db");
        let collection = db.collection("default").await.expect("collection");
        let dim = collection.dimension();
        let embedder = Arc::new(MockEmbedder { dimension: dim });
        let policy = SandboxPolicy {
            allow_db_reads: true,
            allow_db_writes: true,
            allow_code_execution: true,
            allow_cloud_egress: false,
            max_execution_ms: 5_000,
        };
        let sandbox = Arc::new(McpSandbox::new(policy).expect("sandbox new"));
        let server = Arc::new(McpServer::with_sandbox(Arc::new(db), embedder, sandbox));
        (server, tmp)
    }

    fn make_request(method: &str, params: Value) -> JsonRpcRequest {
        JsonRpcRequest {
            jsonrpc: "2.0".into(),
            id: Some(json!(1)),
            method: method.into(),
            params,
        }
    }

    #[tokio::test]
    async fn test_tools_list_contains_contextra_explain_schema() {
        let (server, _tmp) = create_explain_test_server().await;
        let req = make_request("tools/list", json!({}));
        let response = server.handle(req).await;
        let tools = response.result.expect("result")["tools"]
            .as_array()
            .expect("array")
            .clone();

        let explain_tool = tools
            .iter()
            .find(|t| t["name"] == "contextra_explain")
            .expect("contextra_explain must be listed in tools/list");

        let schema = &explain_tool["inputSchema"];
        assert_eq!(schema["type"], "object");
        let req_fields = schema["required"].as_array().expect("required array");
        assert!(req_fields.iter().any(|v| v == "id"));
        assert!(schema["properties"].get("id").is_some());
        assert!(schema["properties"].get("collection").is_some());
    }

    #[tokio::test]
    async fn test_contextra_explain_missing_id_returns_32602() {
        let (server, _tmp) = create_explain_test_server().await;
        let req = make_request(
            "tools/call",
            json!({
                "name": "contextra_explain",
                "arguments": {
                    "collection": "default"
                }
            }),
        );
        let resp = server.handle(req).await;
        let res_val = serde_json::to_value(&resp).expect("value");
        assert_eq!(res_val["result"]["isError"], true);
        let text = res_val["result"]["content"][0]["text"].as_str().expect("str");
        assert!(text.contains("id"));

        let direct_req = make_request("contextra_explain", json!({ "collection": "default" }));
        let direct_resp = server.handle(direct_req).await;
        let err = direct_resp
            .error
            .expect("error expected for missing id param in direct RPC call");
        assert_eq!(err.code, -32602);
    }

    #[tokio::test]
    async fn test_contextra_explain_returns_content_provenance_untrusted_data() {
        let (server, _tmp) = create_explain_test_server().await;

        let insert_req = make_request(
            "contextra_insert",
            json!({
                "id": "explain_doc_test",
                "text": "Provenance explanation test document",
                "collection": "default"
            }),
        );
        let resp = server.handle(insert_req).await;
        assert!(resp.error.is_none());

        let explain_req = make_request(
            "contextra_explain",
            json!({
                "id": "explain_doc_test",
                "collection": "default"
            }),
        );
        let explain_resp = server.handle(explain_req).await;
        assert!(explain_resp.error.is_none());

        let val = explain_resp.result.expect("result payload expected");
        let exp_res: ExplainResponse =
            serde_json::from_value(val).expect("ExplainResponse deserialization");

        assert_eq!(exp_res.id, "explain_doc_test");
        assert_eq!(exp_res.collection, "default");
        assert_eq!(exp_res.content_provenance, "retrieved_untrusted_data");
        assert!(!exp_res.explanation.is_empty());
    }
}
