#![deny(dead_code)]
#![forbid(unsafe_code)]
//! Layer-7-Rand-Crate ohne jegliche unsafe-Toleranz — verarbeitet direkt untrusted stdio-Input, siehe ADR-010.
//!
//! ## MCP Tool Category Mapping
//!
//! | Tool Name | ToolCategory | Description |
//! |---|---|---|
//! | `contextra_search` | `DatabaseRead` | Hybrid semantic search over stored documents |
//! | `contextra_get` | `DatabaseRead` | Retrieve a document by ID |
//! | `contextra_collections` | `DatabaseRead` | List all collections |
//! | `contextra_explain` | `DatabaseRead` | Provenance breakdown & retrieval explanation |
//! | `contextra_plugin_status` | `DatabaseRead` | Status of active plugins & rings |
//! | `contextra_insert` | `DatabaseWrite` | Store a document with auto-chunking & auto-embedding |
//! | `contextra_upsert` | `DatabaseWrite` | Insert/update document idempotently by key |
//! | `contextra_delete` | `DatabaseWrite` | Delete document & issue DeletionProof v3 |
//! | `contextra_forget` | `DatabaseWrite` | Delete document/collection with DeletionProof |
//! | `contextra_relate` | `DatabaseWrite` | Create binary directed/bidirectional relationship |
//! | `contextra_relate_n_ary` | `DatabaseWrite` | Create n-ary hyperedge relationship |
//! | `contextra_create_collection` | `DatabaseWrite` | Create collection with specified DeploymentTier |
//! | `contextra_drop_collection` | `DatabaseWrite` | Delete entire collection with DeletionProof |
//! | `contextra_consolidate` | `DatabaseWrite` | Synchronous trigger for memory consolidation pass |
//! | `contextra_cloud_query` | `CloudEgress` | External cloud query under egress classification |

pub mod bulk_exfiltration_detector;
pub mod config;
pub mod egress_gateway;
pub mod egress_guard;
pub mod explain;
pub mod io;
pub mod plugin_status;
pub mod prompt_injection;
pub mod proof_key;
pub mod proof_key_env;
pub mod protocol;
pub mod routing;
pub mod sandbox;
pub mod server;
pub mod server_dispatch;
pub mod server_tools;
pub mod tools_crud;
pub mod validation;

#[cfg(test)]
mod tests;

pub use bulk_exfiltration_detector::{
    BulkExfiltrationDetector, BulkExfiltrationOutcome, SessionId,
};
pub use config::*;

pub use egress_gateway::{
    CloudQueryRequest, CloudQueryResponse, DefaultEgressClassifier, EgressClassification,
    EgressClassifier,
};

pub use explain::{ExplainRequest, ExplainResponse};
pub use plugin_status::{handle_plugin_status, PluginStatusEntry, PluginStatusResponse};
pub use proof_key::deletion_proof_key_from_env;

pub use io::{read_line_bounded, MAX_RPC_BYTES, MAX_SEARCH_QUERY_BYTES};

pub use prompt_injection::{
    PromptInjectionConfig, PromptInjectionGuard, QuarantinePolicy, SecurityAuditLogger,
    SecurityAuditRecord, DEFAULT_REDACTION_PLACEHOLDER,
};

#[cfg(feature = "kv-bridge")]
pub use routing::setup_kv_bridge;
pub use routing::{setup_routing, RoutingHandle};

pub use server::McpServer;
pub use validation::{is_write_allowed_by_env, validate_collection_name};

// FILE-CONTEXT
// STAND:       2026-09-10T19:25:24Z (SESSION: ae8c2fb9)
// ZWECK:       stdio JSON-RPC 2.0 MCP-Server (kein HTTP! ADR-010)
// INVARIANTEN: Transport ist ausschließlich stdin/stdout — niemals TCP/axum, bounded RPC message size
// HOTSPOTS:    run_stdio_loop(), handle_request(), read_line_bounded()
// SIEHE AUCH:  ADR-010, rules/async-io.md
