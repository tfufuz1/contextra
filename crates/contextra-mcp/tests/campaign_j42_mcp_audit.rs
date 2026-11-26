#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use contextra::collection_profile::DeploymentTier;
use contextra_crypto::AuditChain;
use contextra_mcp::sandbox::{McpSandbox, ToolCategory, TOOL_REGISTRY};
use contextra_types::AutoExtractionMode;

/// TEST 1: MCP Tool Registry Integrity & Dispatch Completeness
/// Oracle Source: `TOOL_REGISTRY` in `contextra-mcp/src/sandbox.rs` and `server_tools.rs`
/// Proves: All 10 MCP tools are actively registered in TOOL_REGISTRY and none are phantom/unclassified.
#[test]
fn test_mcp_tool_registry_active_count_and_classification() {
    // Independent Oracle: Hardcoded set of expected 10 tools defined in MCP specification
    let expected_tools = [
        "contextra_search",
        "contextra_insert",
        "contextra_get",
        "contextra_forget",
        "contextra_collections",
        "contextra_explain",
        "contextra_upsert",
        "contextra_delete",
        "contextra_create_collection",
        "contextra_drop_collection",
    ];

    assert_eq!(
        TOOL_REGISTRY.len(),
        10,
        "TOOL_REGISTRY must contain exactly 10 active tools"
    );

    for tool_name in expected_tools {
        let category = McpSandbox::try_classify_method(tool_name).unwrap();
        assert_ne!(
            category,
            ToolCategory::CodeExecution,
            "Tool {tool_name} must have a specific category, not default CodeExecution fallback"
        );
    }
}

/// TEST 2: DeploymentTier Preset Memory Budget Verification
/// Oracle Source: Specification B.1.8 & ENG-024 (EdgeMinimal=64MB, PowerUserLocal=512MB, EnterpriseShared=4096MB, EnterpriseRegulated=4096MB)
/// Proves: DeploymentTier presets match exact specified RAM limits and auto-extraction defaults.
#[test]
fn test_deployment_tier_presets_ram_limits() {
    let edge = DeploymentTier::EdgeMinimal.resolve();
    assert_eq!(edge.lsm_tuning.max_ram_mb, 64);

    let power = DeploymentTier::PowerUserLocal.resolve();
    assert_eq!(power.lsm_tuning.max_ram_mb, 512);

    let shared = DeploymentTier::EnterpriseShared.resolve();
    assert_eq!(shared.lsm_tuning.max_ram_mb, 4096);

    let regulated = DeploymentTier::EnterpriseRegulated.resolve();
    assert_eq!(regulated.lsm_tuning.max_ram_mb, 4096);
    assert_eq!(
        regulated.auto_extraction,
        AutoExtractionMode::Disabled,
        "EnterpriseRegulated must disable auto-extraction by default"
    );
}
