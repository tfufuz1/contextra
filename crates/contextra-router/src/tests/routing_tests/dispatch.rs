use crate::{dispatch_to_slm, DecisionId, RoutingDecision, SlmProfile};
use contextra_types::{ContextChunk, ContextWindow, ContextraError, DocId, TokenBudget};

#[tokio::test]
async fn test_dispatch_to_slm_mock_server_receives_trimmed_context_only() {
    let temp_dir = tempfile::tempdir().unwrap();
    let captured_req_path = temp_dir.path().join("request.json");
    let script_path = temp_dir.path().join("mock_slm.sh");
    std::fs::write(
            &script_path,
            format!(
                "#!/bin/sh\nread line\necho \"$line\" > {}\necho '{{\"jsonrpc\":\"2.0\",\"id\":1,\"result\":{{\"answer\":\"SLM successfully processed context\"}}}}'\n",
                captured_req_path.display()
            ),
        )
        .unwrap();

    let endpoint = format!("sh {}", script_path.display());
    let profile = SlmProfile::new("mock-slm", endpoint, vec![1], TokenBudget::new(50, 0), 0.1);

    let chunk = ContextChunk {
        doc_id: DocId::new(1),
        content: "Minimal context content for SLM".to_string(),
        relevance: 0.95,
        token_count: 5,
        metadata: None,
        contextual_prefix: None,
        links: Vec::new(),
    };

    let context_window = ContextWindow {
        chunks: vec![chunk],
        total_tokens: 5,
        truncated: false,
    };

    let decision = RoutingDecision {
        profile,
        context: context_window,
        confidence: None,
        decision_id: DecisionId::from_raw(0),
        drift_status: None,
    };

    let answer = dispatch_to_slm(&decision).await.expect("dispatch ok"); // expect
    assert_eq!(answer, "SLM successfully processed context");

    let captured_str = std::fs::read_to_string(&captured_req_path).expect("read captured request");
    let received_json: serde_json::Value =
        serde_json::from_str(&captured_str).expect("parse captured request");
    assert_eq!(received_json["method"], "slm_process_context");
    let params = &received_json["params"];
    assert_eq!(params["profile_name"], "mock-slm");
    assert!(params.get("context").is_some());
    // Verify that raw full search results are NOT present, only context window
    assert!(params.get("search_results").is_none());
    assert_eq!(
        params["context"]["chunks"][0]["content"],
        "Minimal context content for SLM"
    );
}

#[tokio::test]
async fn test_dispatch_error_paths() {
    // 1. Process spawn error / bad command
    let bad_profile = SlmProfile::new(
        "bad-slm",
        "/nonexistent/binary/path/12345",
        vec![1],
        TokenBudget::new(50, 0),
        0.1,
    );
    let chunk = ContextChunk {
        doc_id: DocId::new(1),
        content: "test content".to_string(),
        relevance: 0.9,
        token_count: 5,
        metadata: None,
        contextual_prefix: None,
        links: Vec::new(),
    };
    let decision = RoutingDecision {
        profile: bad_profile,
        context: ContextWindow {
            chunks: vec![chunk.clone()],
            total_tokens: 5,
            truncated: false,
        },
        confidence: None,
        decision_id: DecisionId::from_raw(0),
        drift_status: None,
    };
    let res_err = dispatch_to_slm(&decision).await;
    assert!(
        matches!(res_err, Err(ContextraError::Internal(msg)) if msg.contains("Fehler bei MCP-Dispatch"))
    );

    // 2. Closed stdout without JSON-RPC response
    let profile_closed =
        SlmProfile::new("slm-closed", "true", vec![1], TokenBudget::new(50, 0), 0.1);
    let decision_closed = RoutingDecision {
        profile: profile_closed,
        context: decision.context.clone(),
        confidence: None,
        decision_id: DecisionId::from_raw(0),
        drift_status: None,
    };
    let res_closed = dispatch_to_slm(&decision_closed).await;
    assert!(
        matches!(res_closed, Err(ContextraError::Internal(msg)) if msg.contains("Fehler bei MCP-Dispatch"))
    );

    // 3. RPC Error response
    let profile_rpc_err = SlmProfile::new(
            "slm-rpc-err",
            "sh -c 'cat > /dev/null; echo \"{\\\"jsonrpc\\\":\\\"2.0\\\",\\\"id\\\":1,\\\"error\\\":{\\\"code\\\":-32601,\\\"message\\\":\\\"Method not found\\\"}}\"'",
            vec![1],
            TokenBudget::new(50, 0),
            0.1,
        );
    let decision_rpc_err = RoutingDecision {
        profile: profile_rpc_err,
        context: decision.context.clone(),
        confidence: None,
        decision_id: DecisionId::from_raw(0),
        drift_status: None,
    };
    let res_rpc_err = dispatch_to_slm(&decision_rpc_err).await;
    assert!(
        matches!(res_rpc_err, Err(ContextraError::Internal(ref msg)) if msg.contains("MCP RPC Fehler [-32601]: Method not found")),
        "res_rpc_err was: {:?}",
        res_rpc_err
    );

    // 4. Custom JSON object result (no "answer" key)
    let profile_obj = SlmProfile::new(
            "slm-obj",
            "sh -c 'cat > /dev/null; echo \"{\\\"jsonrpc\\\":\\\"2.0\\\",\\\"id\\\":1,\\\"result\\\":{\\\"custom_data\\\":42}}\"'",
            vec![1],
            TokenBudget::new(50, 0),
            0.1,
        );
    let decision_obj = RoutingDecision {
        profile: profile_obj,
        context: decision.context.clone(),
        confidence: None,
        decision_id: DecisionId::from_raw(0),
        drift_status: None,
    };
    let res_obj = dispatch_to_slm(&decision_obj).await.unwrap(); // unwrap
    assert_eq!(res_obj, "{\"custom_data\":42}");

    // 5. Neither result nor error present
    let profile_empty = SlmProfile::new(
        "slm-empty",
        "sh -c 'cat > /dev/null; echo \"{\\\"jsonrpc\\\":\\\"2.0\\\",\\\"id\\\":1}\"'",
        vec![1],
        TokenBudget::new(50, 0),
        0.1,
    );
    let decision_empty = RoutingDecision {
        profile: profile_empty,
        context: decision.context.clone(),
        confidence: None,
        decision_id: DecisionId::from_raw(0),
        drift_status: None,
    };
    let res_empty = dispatch_to_slm(&decision_empty).await;
    assert!(
        matches!(res_empty, Err(ContextraError::Internal(msg)) if msg.contains("weder result noch error"))
    );
}

#[tokio::test]
async fn test_dispatch_invalid_json_response() {
    let profile = SlmProfile::new(
        "bad-json-slm",
        "sh -c 'cat > /dev/null; echo {invalid json'",
        vec![1],
        TokenBudget::new(50, 0),
        0.1,
    );

    let chunk = ContextChunk {
        doc_id: DocId::new(1),
        content: "test content".to_string(),
        relevance: 0.9,
        token_count: 5,
        metadata: None,
        contextual_prefix: None,
        links: Vec::new(),
    };

    let decision = RoutingDecision {
        profile,
        context: ContextWindow {
            chunks: vec![chunk],
            total_tokens: 5,
            truncated: false,
        },
        confidence: None,
        decision_id: DecisionId::from_raw(0),
        drift_status: None,
    };

    let res = dispatch_to_slm(&decision).await;
    assert!(
        matches!(res, Err(ContextraError::Internal(msg)) if msg.contains("Ungültige MCP JSON-RPC Antwort"))
    );
}

#[tokio::test]
async fn test_dispatch_empty_endpoint_err() {
    let profile = SlmProfile::new("empty-ep", "  ", vec![1], TokenBudget::default(), 0.1);
    let decision = RoutingDecision {
        profile,
        context: ContextWindow {
            chunks: vec![],
            total_tokens: 0,
            truncated: false,
        },
        confidence: None,
        decision_id: DecisionId::from_raw(0),
        drift_status: None,
    };

    let res = dispatch_to_slm(&decision).await;
    assert!(
        matches!(res, Err(ContextraError::InvalidInput(msg)) if msg.contains("Empty MCP endpoint"))
    );
}

#[tokio::test]
async fn test_dispatch_additional_error_and_format_paths() {
    use crate::dispatch_to_slm;
    use contextra_types::{ContextWindow, TokenBudget};

    let profile_exit = SlmProfile::new(
        "test-exit",
        "true", // exits immediately without writing
        vec![],
        TokenBudget::default(),
        0.5,
    );
    let decision_exit = RoutingDecision {
        profile: profile_exit,
        context: ContextWindow {
            chunks: vec![],
            total_tokens: 0,
            truncated: false,
        },
        confidence: None,
        decision_id: DecisionId::from_raw(0),
        drift_status: None,
    };

    let res_exit = dispatch_to_slm(&decision_exit).await;
    assert!(res_exit.is_err());
    let err_msg = res_exit.unwrap_err().to_string();
    assert!(
        err_msg.contains("Fehler bei MCP-Dispatch") || err_msg.contains("Process closed stdout"),
        "Unexpected error message: {err_msg}"
    );

    // Test response returning result object without "answer" key
    let profile_json_obj = SlmProfile::new(
            "test-json-obj",
            "sh -c 'cat > /dev/null; echo \"{\\\"jsonrpc\\\":\\\"2.0\\\",\\\"id\\\":1,\\\"result\\\":{\\\"custom_key\\\":\\\"val\\\"}}\"'",
            vec![],
            TokenBudget::default(),
            0.5,
        );
    let decision_json_obj = RoutingDecision {
        profile: profile_json_obj,
        context: ContextWindow {
            chunks: vec![],
            total_tokens: 0,
            truncated: false,
        },
        confidence: None,
        decision_id: DecisionId::from_raw(0),
        drift_status: None,
    };

    let res_obj = dispatch_to_slm(&decision_json_obj).await;
    assert!(res_obj.is_ok());
    assert!(res_obj.unwrap_or_default().contains("custom_key"));
}
