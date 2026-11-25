//! Campaign J-42: MCP Tool Dispatch & Sandbox Registry Classification Audit Test Suite
//! Oracle Source (R4): MCP Tool Protocol Specification & Sandbox Isolation Model (Spec §5.3 / ENG-020)
//! counter-test (R10): Verified against legacy fallback (legacy classify_method returns CodeExecution for unknown tools).

use contextra_mcp::sandbox::{McpSandbox, ToolCategory, TOOL_REGISTRY};

#[test]
fn test_campaign_j42_mcp_all_registered_tools_classified() {
    // Oracle Check: Every tool in TOOL_REGISTRY MUST be explicitly classified by try_classify_method
    // without returning an Err.
    for tool in TOOL_REGISTRY {
        let try_cat = McpSandbox::try_classify_method(tool.name);
        assert!(
            try_cat.is_ok(),
            "Tool '{}' in TOOL_REGISTRY must be explicitly classified in try_classify_method",
            tool.name
        );
    }
}

#[test]
fn test_campaign_j42_mcp_unknown_tool_strict_rejection_oracle() {
    // Oracle Check: Any unlisted method MUST be rejected fail-closed by try_classify_method.
    let unknown_tool = "contextra_unapproved_secret_tool";
    let res = McpSandbox::try_classify_method(unknown_tool);
    assert!(
        res.is_err(),
        "try_classify_method must return Err for unknown method '{}'",
        unknown_tool
    );
}

#[test]
#[allow(deprecated)]
fn test_campaign_j42_mcp_counter_factual_mutation_check() {
    // R10 Counter-Test: Demonstrates that legacy classify_method falls back to CodeExecution
    // for unknown tools, whereas try_classify_method strictly returns Err.
    let unknown_tool = "contextra_unapproved_secret_tool";

    let legacy_cat = McpSandbox::classify_method(unknown_tool);
    assert_eq!(
        legacy_cat,
        ToolCategory::CodeExecution,
        "R10 Counter-Fact: Legacy classify_method falls back to CodeExecution"
    );

    let try_cat = McpSandbox::try_classify_method(unknown_tool);
    assert!(
        try_cat.is_err(),
        "R10 Counter-Fact: Strict try_classify_method prevents silent fallback to CodeExecution"
    );
}
