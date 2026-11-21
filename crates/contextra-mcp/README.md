# contextra-mcp

**Zweck:** MCP-Server über stdio (ADR-38, SPEC §6.8/§3.2 E3). Macht Contextra für KI-Assistenten (z. B. Claude Desktop) als Werkzeug nutzbar.

**Ebene:** E3

## Öffentliche API (`src/lib.rs`, Stand dieser README)

Ist-Stand. Module oder Namen, die laut PLAN.md entfallen, können hier noch stehen.

```rust
pub mod bulk_exfiltration_detector
pub mod config
pub mod egress_gateway
pub mod egress_guard
pub mod explain
pub mod io
pub mod prompt_injection
pub mod protocol
pub mod routing
pub mod sandbox
pub mod server
pub mod server_dispatch
pub mod server_tools
pub mod tools_crud
pub mod validation
pub use bulk_exfiltration_detector::
pub use config::*
pub use egress_gateway::
pub use explain::{ExplainRequest, ExplainResponse}
pub use io::{read_line_bounded, MAX_RPC_BYTES, MAX_SEARCH_QUERY_BYTES}
pub use prompt_injection::
pub use routing::{setup_routing, RoutingHandle}
pub use server::McpServer
pub use validation::{is_write_allowed_by_env, validate_collection_name}
```

## Features (`Cargo.toml`)

`default`, `test-utils`

## Dokumentation

Implementierungsspezifikation: [`PLAN.md`](PLAN.md). Maßgeblich ist `docs/SPEC.md` v4.1.

