# AGENTS.md — `contextra-wire`

> Werkzeug/Crate-Kontext für Agenten. Der obere Teil ist generiert; manuelle Notizen unterhalb ergänzen.

<!-- BEGIN GENERATED: crate_inventory.py -->

## Inventar (generiert)

- **Pfad:** `crates/contextra-wire` · **Ring:** 0 · **Version:** 0.1.0
- **Beschreibung:** Ring 0 Unsafe Island: Auto-generated FlatBuffers IPC code and zero-copy adapters for Contextra
- **src:** 6 Dateien, 2131 LOC · **unsafe:** 45 Vorkommen, Policy `deny`
- **Interne Abhängigkeiten (normal, 0):** –
- **Interne Dev/Build-Abhängigkeiten (0):** –
- **Externe Abhängigkeiten (normal, 6):** `bytes`, `flatbuffers`, `serde`, `serde_json`, `thiserror`, `tracing`
- **Reverse-Abhängigkeiten (3):** `contextra-mcp`, `contextra-py`, `contextra-router`
- **Kopplung an Hub-Crates:** ausgehend –; eingehend –
- **Features (5):** `default`, `inspect`, `serde`, `std`, `tracing`

### Öffentliche API (grob, aus src/)
- struct (32): `Embedding`, `EmbeddingArgs`, `EmbeddingBuilder`, `FlatBufferConfig`, `HyperEdge`, `HyperEdgeArgs`, `HyperEdgeBuilder`, `HyperEdgeId`, `HyperEdgeIdArgs`, `HyperEdgeIdBuilder`, `JsonRpcConfig`, `JsonRpcError`, `JsonRpcRequest`, `JsonRpcResponse`, `RoleBinding` … (+17)
- enum (11): `EmbeddingOffset`, `Error`, `FlatBufferError`, `HyperEdgeIdOffset`, `HyperEdgeOffset`, `JsonRpcPayload`, `RoleBindingOffset`, `RoleIdOffset`, `ScoredDocumentOffset`, `SearchResponseOffset`, `VectorIndexUpdateOffset`
- trait (1): `VerifiableRoot`
- fn (80): `add_business_valid_from`, `add_business_valid_to`, `add_child_edge_ids`, `add_data`, `add_embedding`, `add_entity`, `add_id`, `add_metadata`, `add_metric`, `add_participants`, `add_predicate`, `add_processing_time_ms`, `add_quantized_u8`, `add_results`, `add_role` … (+65)
- type (1): `Result`
- const (21): `VT_BUSINESS_VALID_FROM`, `VT_BUSINESS_VALID_TO`, `VT_CHILD_EDGE_IDS`, `VT_DATA`, `VT_EMBEDDING`, `VT_ENTITY`, `VT_ID`, `VT_METADATA`, `VT_METRIC`, `VT_PARTICIPANTS`, `VT_PREDICATE`, `VT_PROCESSING_TIME_MS`, `VT_QUANTIZED_U8`, `VT_RESULTS`, `VT_ROLE` … (+6)
- mod (7): `adapter`, `contextra`, `contextra_generated`, `generated`, `ipc`, `jsonrpc`, `verification`
- pub use: `adapter::*`, `generated::contextra_generated`, `generated::contextra_generated::contextra::ipc::*`, `jsonrpc::*`, `verification::*`

Regeneriert mit `python3 tools/crate_inventory.py agents`. Manuelle Notizen stehen außerhalb dieses Blocks.

<!-- END GENERATED: crate_inventory.py -->`.

<!-- END GENERATED: crate_inventory.py -->
