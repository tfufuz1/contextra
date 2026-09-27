use crate::protocol::McpError;
use serde::Serialize;

/// Response payload for the `contextra_plugin_status` MCP tool.
///
/// SECURITY BOUNDARY NOTICE: This response strictly contains high-level feature ring indicators
/// (`FeatureRing`) and plugin status entries. It MUST NEVER include raw license keys, ticket signatures,
/// or confidential licensing payloads.
#[derive(Debug, Serialize, PartialEq, Eq, Clone)]
pub struct PluginStatusResponse {
    pub plugins: Vec<PluginStatusEntry>,
    pub feature_ring_active: contextra_ports::license::FeatureRing,
}

/// Status entry describing an active plugin.
#[derive(Debug, Serialize, PartialEq, Eq, Clone)]
pub struct PluginStatusEntry {
    pub name: String,
    pub version: String,
    pub ring: u8,
    pub feature_ring_required: contextra_ports::license::FeatureRing,
}

impl From<contextra_ports::plugin::PluginCapability> for PluginStatusEntry {
    fn from(cap: contextra_ports::plugin::PluginCapability) -> Self {
        Self {
            name: cap.name.to_string(),
            version: format!("{}.{}.{}", cap.version.0, cap.version.1, cap.version.2),
            ring: cap.ring,
            feature_ring_required: cap.feature_ring_required,
        }
    }
}

/// Handler for the `contextra_plugin_status` MCP tool call.
pub async fn handle_plugin_status(
    registry: &contextra_ports::plugin::PluginRegistry,
) -> Result<PluginStatusResponse, McpError> {
    Ok(PluginStatusResponse {
        plugins: registry.snapshot().into_iter().map(Into::into).collect(),
        feature_ring_active: registry.current_feature_ring(),
    })
}
