//! # Fuzz-Target für JSON-RPC 2.0 Parser
//!
//! **Ring & Klasse:** Ring 0 (Fuzzing / REQ-QA-003)
//!
//! ## Zweck
//! Dieses Fuzz-Target speist beliebige, ungetestete Byte-Arrays in den gehärteten
//! JSON-RPC 2.0 Parser von `contextra-wire` ein (`jsonrpc.rs`).
//!
//! ## Invarianten
//! - Kein Panic (P7 Zero-Panic).
//! - Keine unbegrenzte Allokation (DoS-Schutz via `JsonRpcConfig`: `max_message_size`, `max_depth`, `max_batch_size`).
//! - Bei erfolgreichem Parsen (`Ok(...)`) ist die Payload stabil serialisierbar und validierbar.

#![no_main]

use contextra_wire::{
    parse_jsonrpc_payload, parse_jsonrpc_payload_with_config, JsonRpcConfig, JsonRpcPayload,
};
use libfuzzer_sys::fuzz_target;

// ANCHOR[contextra-wire:FUZ-003]
fuzz_target!(|data: &[u8]| {
    // 1. Standard-Parsing
    if let Ok(payload) = parse_jsonrpc_payload(data) {
        match payload {
            JsonRpcPayload::Single(req) => {
                let _ = req.validate();
                let _ = serde_json::to_string(&req);
            }
            JsonRpcPayload::Batch(reqs) => {
                for req in reqs {
                    let _ = req.validate();
                    let _ = serde_json::to_string(&req);
                }
            }
        }
    }

    // 2. Striktes Parsing mit niedrigen Grenzwerten (Depth-Bombs, Batch-Limits)
    let strict_config = JsonRpcConfig {
        max_message_size: 16 * 1024,
        max_depth: 8,
        max_batch_size: 10,
        reject_duplicate_keys: true,
    };
    if let Ok(payload) = parse_jsonrpc_payload_with_config(data, &strict_config) {
        match payload {
            JsonRpcPayload::Single(req) => {
                let _ = req.validate();
            }
            JsonRpcPayload::Batch(reqs) => {
                for req in reqs {
                    let _ = req.validate();
                }
            }
        }
    }

    // 3. Konfiguration ohne Duplicate Key Rejection
    let permissive_config = JsonRpcConfig {
        max_message_size: 32 * 1024,
        max_depth: 16,
        max_batch_size: 20,
        reject_duplicate_keys: false,
    };
    let _ = parse_jsonrpc_payload_with_config(data, &permissive_config);
});
