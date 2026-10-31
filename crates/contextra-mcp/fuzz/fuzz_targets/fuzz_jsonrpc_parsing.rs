#![no_main]

use libfuzzer_sys::fuzz_target;
use contextra_mcp::protocol::{JsonRpcRequest, JsonRpcResponse, McpError};
use serde_json::Value;

fuzz_target!(|data: &[u8]| {
    // 1. Direct JSON-RPC request deserialization from raw arbitrary bytes
    if let Ok(req) = serde_json::from_slice::<JsonRpcRequest>(data) {
        let _ = req.jsonrpc;
        let _ = req.id;
        let _ = req.method;
        let _ = req.params;
    }

    // 2. Direct JSON-RPC response deserialization
    if let Ok(resp) = serde_json::from_slice::<JsonRpcResponse>(data) {
        let _ = resp.jsonrpc;
        let _ = resp.id;
        let _ = resp.result;
        let _ = resp.error;
    }

    // 3. General JSON value parsing + request conversion
    if let Ok(val) = serde_json::from_slice::<Value>(data) {
        if let Ok(req) = serde_json::from_value::<JsonRpcRequest>(val.clone()) {
            let _ = req.method;
        }
    }

    // 4. McpError construction and code conversion from UTF-8 string
    if let Ok(s) = std::str::from_utf8(data) {
        let err = McpError::parse_error(s);
        let _ = err.code();
        let _ = err.to_string();
    }
});
