pub mod cancellation;
pub mod channel_kind;
pub mod confirm_handler;
pub mod error_codes;
pub mod forget_guard;
pub mod recall_truncator;
pub mod tool_hints;
pub mod trust_frame;

use crate::protocol::{response_from_error, JsonRpcRequest, JsonRpcResponse, McpError};
use crate::sandbox::TOOL_REGISTRY;
use crate::server::McpServer;
use channel_kind::ChannelKind;
use confirm_handler::handle_confirm;
use error_codes::{McpErrorCode, StructuredMcpError};
use serde_json::{json, Value};
use tool_hints::annotate_tool_schema;

impl McpServer {
    fn error_to_mcp(code: McpErrorCode, msg: impl Into<String>) -> McpError {
        let text = msg.into();
        match code {
            McpErrorCode::SerializationFailure | McpErrorCode::Busy | McpErrorCode::StorageFull => {
                McpError::internal_error(text)
            }
            McpErrorCode::Denied | McpErrorCode::Corrupt | McpErrorCode::ConfirmationRequired => {
                McpError::invalid_params(text)
            }
        }
    }

    /// Public entry point specifying channel kind explicitly.
    pub async fn handle_value_with_channel(
        &self,
        val: Value,
        channel: ChannelKind,
    ) -> Option<Value> {
        self.handle_value_internal(val, channel).await
    }

    /// Verarbeitet einen Parsed JSON-RPC 2.0 Request (Einzelanfrage, Notification oder Batch Array).
    /// Gibt `Some(Value)` zurück, falls eine Antwort nach stdout gesendet werden muss,
    /// oder `None`, falls der Client eine Notification war (keine Antwort erforderlich).
    pub async fn handle_value(&self, val: Value) -> Option<Value> {
        self.handle_value_internal(val, ChannelKind::HostStdio)
            .await
    }

    async fn handle_value_internal(&self, val: Value, channel: ChannelKind) -> Option<Value> {
        if let Some(arr) = val.as_array() {
            if arr.is_empty() {
                return Some(
                    serde_json::to_value(response_from_error(
                        None,
                        McpError::invalid_request("Invalid Request: empty batch array"),
                    ))
                    .unwrap_or_default(),
                );
            }

            let mut responses = Vec::new();
            for item in arr {
                let req_id = item.get("id").cloned();
                match serde_json::from_value::<JsonRpcRequest>(item.clone()) {
                    Ok(req) => {
                        if req.id.is_none() || req.id == Some(Value::Null) {
                            let method = req.method.clone();
                            let resp = self.handle_with_channel(req, channel).await;
                            if let Some(err) = resp.error {
                                tracing::warn!(
                                    method = %method,
                                    code = err.code,
                                    error = %err.message,
                                    "MCP notification handling returned error"
                                );
                            }
                        } else {
                            let resp = self.handle_with_channel(req, channel).await;
                            if let Ok(v) = serde_json::to_value(resp) {
                                responses.push(v);
                            }
                        }
                    }
                    Err(e) => {
                        let resp = response_from_error(
                            req_id,
                            McpError::invalid_request(format!("Invalid Request: {e}")),
                        );
                        if let Ok(v) = serde_json::to_value(resp) {
                            responses.push(v);
                        }
                    }
                }
            }

            if responses.is_empty() {
                None
            } else {
                Some(Value::Array(responses))
            }
        } else if val.is_object() {
            let req_id = val.get("id").cloned();
            match serde_json::from_value::<JsonRpcRequest>(val) {
                Ok(req) => {
                    if req.id.is_none() || req.id == Some(Value::Null) {
                        let method = req.method.clone();
                        let resp = self.handle_with_channel(req, channel).await;
                        if let Some(err) = resp.error {
                            tracing::warn!(
                                method = %method,
                                code = err.code,
                                error = %err.message,
                                "MCP notification handling returned error"
                            );
                        }
                        None
                    } else {
                        let resp = self.handle_with_channel(req, channel).await;
                        serde_json::to_value(resp).ok()
                    }
                }
                Err(e) => {
                    let resp = response_from_error(
                        req_id,
                        McpError::invalid_request(format!("Invalid Request: {e}")),
                    );
                    serde_json::to_value(resp).ok()
                }
            }
        } else {
            let resp = response_from_error(None, McpError::invalid_request("Invalid Request"));
            serde_json::to_value(resp).ok()
        }
    }

    pub async fn handle_with_channel(
        &self,
        req: JsonRpcRequest,
        channel: ChannelKind,
    ) -> JsonRpcResponse {
        self.handle_internal(req, channel).await
    }

    pub async fn handle(&self, req: JsonRpcRequest) -> JsonRpcResponse {
        self.handle_internal(req, ChannelKind::HostStdio).await
    }

    async fn handle_internal(&self, req: JsonRpcRequest, channel: ChannelKind) -> JsonRpcResponse {
        let principal = "default_principal";
        let _permit = match self.concurrency_governor.acquire_permit(principal) {
            Ok(p) => p,
            Err(e) => return response_from_error(req.id, Self::error_to_mcp(e.code, e.message)),
        };

        let id = req.id.clone();
        if req.jsonrpc != "2.0" {
            return response_from_error(
                id,
                McpError::invalid_request(format!(
                    "Invalid Request: jsonrpc version must be '2.0', got '{}'",
                    req.jsonrpc
                )),
            );
        }
        match req.method.as_str() {
            "contextra/confirm" => {
                let token = req
                    .params
                    .get("token")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                let res = handle_confirm(
                    &self.forget_guard,
                    principal,
                    token,
                    channel,
                    |_scope, collection| async move {
                        let tenant_id = crate::validation::extract_tenant_from_args_or_env(None);
                        let proof_key = crate::proof_key_env::resolve_proof_key_from_env()
                            .key
                            .ok_or_else(|| {
                                StructuredMcpError::new(
                                    McpErrorCode::Denied,
                                    "deletion proof key not configured",
                                )
                            })?;
                        let db = self.db.clone();
                        let col_name = collection;
                        let proof_res = db
                            .drop_collection(&col_name, tenant_id, proof_key.as_bytes())
                            .await;
                        match proof_res {
                            Ok(_) => Ok(1),
                            Err(e) => {
                                Err(StructuredMcpError::new(McpErrorCode::Denied, e.to_string()))
                            }
                        }
                    },
                )
                .await;
                match res {
                    Ok(val) => JsonRpcResponse::ok(id, val),
                    Err(e) => response_from_error(id, Self::error_to_mcp(e.code, e.message)),
                }
            }

            // ── Lifecycle ──────────────────────────────────────────────────────
            "initialize" => JsonRpcResponse::ok(
                id,
                json!({
                    "protocolVersion": "2024-11-05",
                    "capabilities": { "tools": {} },
                    "serverInfo": { "name": "contextra", "version": "0.1.0" }
                }),
            ),
            "initialized" => JsonRpcResponse::ok(id, json!({})),

            // ── Tool-Discovery ─────────────────────────────────────────────────
            "tools/list" => {
                let annotated_tools: Vec<Value> = TOOL_REGISTRY
                    .iter()
                    .map(|tool| {
                        let schema = (tool.input_schema)();
                        let annotated_schema = annotate_tool_schema(schema, tool.category.clone());
                        json!({
                            "name": tool.name,
                            "description": tool.description,
                            "inputSchema": annotated_schema
                        })
                    })
                    .collect();

                JsonRpcResponse::ok(id, json!({ "tools": annotated_tools }))
            }

            // ── Tool-Dispatch ──────────────────────────────────────────────────
            "tools/call" => {
                let tool_name = match req.params.get("name").and_then(|v| v.as_str()) {
                    Some(name) if !name.is_empty() => name,
                    _ => {
                        return response_from_error(
                            id,
                            McpError::invalid_params(
                                "Invalid params: missing or empty tool 'name'",
                            ),
                        );
                    }
                };
                let args = req.params.get("arguments").cloned().unwrap_or_default();

                let dispatch_res = self
                    .sandbox
                    .execute_with_timeout(tool_name, self.call_tool(tool_name, &args))
                    .await;

                match dispatch_res {
                    Ok(content) => {
                        let text_str = content.to_string();
                        let (truncated_text, _marker) = self.recall_truncator.truncate(&text_str);
                        JsonRpcResponse::ok(
                            id,
                            json!({
                                "content": [{ "type": "text", "text": truncated_text }]
                            }),
                        )
                    }
                    Err(e) => JsonRpcResponse::ok(
                        id,
                        json!({
                            "isError": true,
                            "content": [{ "type": "text", "text": e.to_string() }]
                        }),
                    ),
                }
            }

            "contextra_search"
            | "contextra_insert"
            | "contextra_get"
            | "contextra_forget"
            | "contextra_collections"
            | "contextra_upsert"
            | "contextra_delete"
            | "contextra_create_collection"
            | "contextra_drop_collection"
            | "contextra_plugin_status" => {
                let tool_name = req.method.as_str();
                match self
                    .sandbox
                    .execute_with_timeout(tool_name, self.call_tool(tool_name, &req.params))
                    .await
                {
                    Ok(res) => JsonRpcResponse::ok(id, res),
                    Err(e) => response_from_error(id, e),
                }
            }

            // ── Ping ───────────────────────────────────────────────────────────
            "ping" => JsonRpcResponse::ok(id, json!({})),

            // ── Unbekannte Methode ──────────────────────────────────────────────
            other => response_from_error(
                id,
                McpError::method_not_found(format!("Method not found: {other}")),
            ),
        }
    }
}
