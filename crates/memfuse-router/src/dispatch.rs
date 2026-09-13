// FILE-CONTEXT
// STAND: 2026-09-10T19:16:25Z (SESSION: 3f3e4637)
// ZWECK: Client-seitiger MCP-Dispatch-Mechanismus über Stdio JSON-RPC 2.0 (ADR-010).
// INVARIANTEN: Sendet ausschließlich ContextWindow (keine ungetrimmten Rohergebnisse).
// SIEHE AUCH: docs/decisions/ADR-010-mcp-transport.md, rules/tag_taxonomy.md

//! Client-side MCP dispatch mechanism for sending routed context to SLM endpoints.

use crate::router::RoutingDecision;
use memfuse_core::ipc::{JsonRpcRequest, JsonRpcResponse};
use memfuse_core::{MemFuseError, Result};
use serde_json::json;
use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::Command;

/// Teilt einen Endpoint-String sicher in Programm + Argumente auf.
/// Unterstützt einfache Anführungszeichen: 'arg with spaces'.
/// Gibt Err bei leerem String oder nicht-geschlossenen Anführungszeichen zurück.
fn split_endpoint(s: &str) -> Result<(String, Vec<String>)> {
    let mut parts: Vec<String> = Vec::new();
    let mut current = String::new();
    let mut in_single_quote = false;
    let mut chars = s.chars().peekable();

    while let Some(c) = chars.next() {
        match c {
            '\'' if !in_single_quote => in_single_quote = true,
            '\'' if in_single_quote => in_single_quote = false,
            ' ' | '\t' if !in_single_quote => {
                if !current.is_empty() {
                    parts.push(std::mem::take(&mut current));
                }
            }
            other => current.push(other),
        }
    }

    if in_single_quote {
        return Err(MemFuseError::InvalidInput(
            "Unclosed single quote in MCP endpoint string".into(),
        ));
    }
    if !current.is_empty() {
        parts.push(current);
    }

    let mut iter = parts.into_iter();
    let program = iter
        .next()
        .ok_or_else(|| MemFuseError::InvalidInput("Empty MCP endpoint".into()))?;
    let args: Vec<String> = iter.collect();
    Ok((program, args))
}

/// Dispatches the prepared context from a [`RoutingDecision`] to the target SLM's MCP endpoint
/// over stdio JSON-RPC 2.0 (ADR-010 compliant).
///
/// Sends ONLY the tailored [`memfuse_core::ContextWindow`] (not raw full search results)
/// to the executable or script specified in `decision.profile.mcp_endpoint`.
pub async fn dispatch_to_slm(decision: &RoutingDecision) -> Result<String> {
    let endpoint = decision.profile.mcp_endpoint.trim();
    if endpoint.is_empty() {
        return Err(MemFuseError::InvalidInput("Empty MCP endpoint".into()));
    }

    let request_payload = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: Some(json!(1)),
        method: "slm_process_context".to_string(),
        params: json!({
            "context": decision.context,
            "profile_name": decision.profile.name,
        }),
    };

    let mut payload_bytes = serde_json::to_vec(&request_payload)?;
    payload_bytes.push(b'\n');

    let (program, args) = split_endpoint(endpoint)?;
    let mut child = Command::new(&program)
        .args(&args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| {
            MemFuseError::Internal(format!(
                "Fehler bei MCP-Dispatch an {}: {e}",
                decision.profile.mcp_endpoint
            ))
        })?;

    if let Some(mut stdin) = child.stdin.take() {
        if let Err(e) = stdin.write_all(&payload_bytes).await {
            return Err(MemFuseError::Internal(format!(
                "Fehler bei MCP-Dispatch an {}: Stdin write failed: {e}",
                decision.profile.mcp_endpoint
            )));
        }
        if let Err(e) = stdin.flush().await {
            return Err(MemFuseError::Internal(format!(
                "Fehler bei MCP-Dispatch an {}: Stdin flush failed: {e}",
                decision.profile.mcp_endpoint
            )));
        }
    } else {
        return Err(MemFuseError::Internal(format!(
            "Fehler bei MCP-Dispatch an {}: Failed to open child stdin",
            decision.profile.mcp_endpoint
        )));
    }

    let stdout = child.stdout.take().ok_or_else(|| {
        MemFuseError::Internal(format!(
            "Fehler bei MCP-Dispatch an {}: Failed to open child stdout",
            decision.profile.mcp_endpoint
        ))
    })?;

    let timeout_duration = std::time::Duration::from_secs(30);
    let mut reader = BufReader::new(stdout);
    let mut line = String::new();

    let read_result = tokio::time::timeout(timeout_duration, reader.read_line(&mut line)).await;
    match read_result {
        Ok(Ok(0)) | Ok(Err(_)) => {
            return Err(MemFuseError::Internal(format!(
                "Fehler bei MCP-Dispatch an {}: Process closed stdout without sending JSON-RPC response",
                decision.profile.mcp_endpoint
            )));
        }
        Err(_) => {
            let _ = child.kill().await;
            return Err(MemFuseError::Internal(format!(
                "Fehler bei MCP-Dispatch an {}: Timeout",
                decision.profile.mcp_endpoint
            )));
        }
        Ok(Ok(_)) => {}
    }

    let _ = child.wait().await;

    let rpc_response: JsonRpcResponse = serde_json::from_str(&line)
        .map_err(|e| MemFuseError::Internal(format!("Ungültige MCP JSON-RPC Antwort: {e}")))?;

    if let Some(error) = rpc_response.error {
        return Err(MemFuseError::Internal(format!(
            "MCP RPC Fehler [{}]: {}",
            error.code, error.message
        )));
    }

    if let Some(result) = rpc_response.result {
        if let Some(ans) = result.get("answer").and_then(|v| v.as_str()) {
            Ok(ans.to_string())
        } else {
            Ok(result.to_string())
        }
    } else {
        Err(MemFuseError::Internal(
            "MCP-Antwort enthielt weder result noch error".to_string(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::profile::SlmProfile;
    use memfuse_core::{ContextChunk, ContextWindow, DocId, TokenBudget};

    #[test]
    fn test_split_endpoint_simple() {
        let (prog, args) = split_endpoint("/usr/bin/my-slm").unwrap();
        assert_eq!(prog, "/usr/bin/my-slm");
        assert!(args.is_empty());
    }

    #[test]
    fn test_split_endpoint_with_args() {
        let (prog, args) = split_endpoint("/usr/bin/node /opt/slm/server.js --port 9000").unwrap();
        assert_eq!(prog, "/usr/bin/node");
        assert_eq!(args, vec!["/opt/slm/server.js", "--port", "9000"]);
    }

    #[test]
    fn test_split_endpoint_quoted_spaces() {
        let (prog, args) = split_endpoint("'/path with spaces/slm' --mode fast").unwrap();
        assert_eq!(prog, "/path with spaces/slm");
        assert_eq!(args, vec!["--mode", "fast"]);
    }

    #[test]
    fn test_split_endpoint_empty_fails() {
        assert!(split_endpoint("").is_err());
        assert!(split_endpoint("   ").is_err());
    }

    #[test]
    fn test_split_endpoint_unclosed_quote_fails() {
        assert!(split_endpoint("'/unclosed arg").is_err());
    }

    #[test]
    fn test_split_endpoint_no_shell_injection() {
        // Diese Strings dürfen NICHT zur Shell-Ausführung führen — split_endpoint
        // trennt nur Tokens, der zurückgegebene `program`-String wird direkt an
        // Command::new() übergeben, kein Shell-Intermediär.
        let (prog, args) = split_endpoint("myslm; rm -rf /").unwrap();
        assert_eq!(prog, "myslm;");
        // 'rm' wird als Argument übergeben, nicht als zweites Kommando
        assert!(args.iter().any(|a| a == "rm"));
    }

    #[tokio::test]
    async fn test_dispatch_to_slm_invalid_endpoint_fails_gracefully() {
        let profile = SlmProfile::new(
            "test-slm",
            "/nonexistent/binary/path/12345",
            vec![],
            TokenBudget::default(),
            0.5,
        );
        let decision = RoutingDecision {
            profile,
            context: ContextWindow {
                chunks: vec![ContextChunk {
                    doc_id: DocId::new(1),
                    content: "hello world".into(),
                    relevance: 0.9,
                    token_count: 2,
                    metadata: None,
                    contextual_prefix: None,
                    links: vec![],
                }],
                total_tokens: 2,
                truncated: false,
            },
            confidence: None,
            decision_id: crate::DecisionId::new(),
            drift_status: None,
        };

        let result = dispatch_to_slm(&decision).await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Fehler bei MCP-Dispatch"));
    }
}
