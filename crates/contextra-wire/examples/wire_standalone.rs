//! # Eigenständiges Einbettungsbeispiel für `contextra-wire`
//!
//! Demonstriert die autarke Verwendung von `contextra-wire` ohne Abhängigkeiten
//! zu anderen Contextra-Crates:
//! 1. Zero-Copy-Pufferverwaltung mit `WireBuffer`.
//! 2. Parsing und Validierung von JSON-RPC 2.0 Payloads (`JsonRpcRequest`, `JsonRpcResponse`).
//! 3. Härtung und strukturierte Verifikation von FlatBuffers-Nachrichten (`verify_flatbuffer`).

use bytes::Bytes;
use contextra_wire::{
    parse_jsonrpc_payload, verify_and_parse_search_response, FlatBufferConfig, JsonRpcPayload,
    JsonRpcResponse, SearchResponse, WireBuffer,
};
use flatbuffers::FlatBufferBuilder;

fn main() {
    println!("=== contextra-wire: Standalone-Einbettungsbeispiel ===");

    // 1. WireBuffer: Allokationsfreier Zero-Copy-Byte-Puffer
    let raw_data = b"CONTEXTRA_WIRE_PAYLOAD_V1";
    let bytes_obj = Bytes::from_static(raw_data);
    let wire_buf = WireBuffer::new(bytes_obj);

    println!(
        "[1] WireBuffer initialisiert: len = {}, data = {:?}",
        wire_buf.as_slice().len(),
        std::str::from_utf8(wire_buf.as_slice()).unwrap_or("")
    );

    // 2. JSON-RPC 2.0: Parsing, Validierung und Antwort-Generierung
    let rpc_json =
        r#"{"jsonrpc": "2.0", "id": 1, "method": "agent.query", "params": {"query": "vector"}}"#;
    match parse_jsonrpc_payload(rpc_json.as_bytes()) {
        Ok(JsonRpcPayload::Single(req)) => {
            println!(
                "[2] JSON-RPC Request erfolgreich geparst: method = '{}', id = {:?}",
                req.method, req.id
            );
            let resp = JsonRpcResponse::ok(req.id, serde_json::json!({"status": "acknowledged"}));
            let resp_json = serde_json::to_string(&resp).unwrap_or_default();
            println!("[2] JSON-RPC Response generiert: {}", resp_json);
        }
        Ok(JsonRpcPayload::Batch(_)) => println!("[2] Unerwarteter Batch"),
        Err(e) => eprintln!("[2] JSON-RPC Parse-Fehler: {e}"),
    }

    // 3. FlatBuffers: Erstellung und gehärtete Verifikation
    let mut builder = FlatBufferBuilder::new();
    let resp_offset = SearchResponse::create(
        &mut builder,
        &contextra_wire::SearchResponseArgs {
            results: None,
            total_hits: 42,
            processing_time_ms: 12.5,
        },
    );
    builder.finish(resp_offset, None);
    let finished_buf = builder.finished_data();

    let config = FlatBufferConfig::default();
    match verify_and_parse_search_response(finished_buf, &config) {
        Ok(verified_resp) => {
            println!(
                "[3] FlatBuffers-Nachricht verifiziert: total_hits = {}, processing_time_ms = {}",
                verified_resp.total_hits(),
                verified_resp.processing_time_ms()
            );
        }
        Err(e) => eprintln!("[3] FlatBuffers-Verifikationsfehler: {e}"),
    }

    println!("=== Ausführung erfolgreich abgeschlossen ===");
}
