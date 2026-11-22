// FILE-CONTEXT Header (Format v3)
// ZWECK: Registry for validated agent tools and fail-closed capability checking before execution.
// INVARIANTEN: Fail-closed capability authorization before tool execution; manifest validation.
// STAND: TS:2026-10-29T00:00:00Z

//! Registry managing validated tool manifests and execution dispatch for agent steps.

use crate::context::AgentContext;
use crate::step::AgentTool;
use crate::tools::manifest::ToolManifest;
use contextra_types::{ContextraError, Result};
use std::collections::BTreeMap;

/// Registry managing validated tool manifests and execution dispatch for agent steps.
#[derive(Default)]
pub struct ToolRegistry {
    entries: BTreeMap<String, (ToolManifest, Box<dyn AgentTool>)>,
}

impl ToolRegistry {
    /// Creates a new empty `ToolRegistry`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Registers an `AgentTool` with its validated `ToolManifest`.
    pub fn register(&mut self, manifest: ToolManifest, tool: Box<dyn AgentTool>) -> Result<()> {
        manifest.validate()?;
        if manifest.name != tool.name() {
            return Err(ContextraError::InvalidInput(format!(
                "Manifest name '{}' does not match tool name '{}'",
                manifest.name,
                tool.name()
            )));
        }
        self.entries.insert(manifest.name.clone(), (manifest, tool));
        Ok(())
    }

    /// Convenience helper to register a tool using default manifest settings derived from `AgentTool`.
    pub fn register_tool(&mut self, tool: Box<dyn AgentTool>) -> Result<()> {
        let name = tool.name();
        let manifest = ToolManifest::new(name, format!("Tool {name}"))
            .with_timeout_ms(tool.timeout_ms())
            .with_retry(tool.is_retriable(), tool.max_retries());
        self.register(manifest, tool)
    }

    /// Retrieves a reference to a registered tool by name.
    pub fn get_tool(&self, name: &str) -> Option<&dyn AgentTool> {
        self.entries.get(name).map(|(_, tool)| tool.as_ref())
    }

    /// Retrieves a reference to a registered tool's manifest by name.
    pub fn get_manifest(&self, name: &str) -> Option<&ToolManifest> {
        self.entries.get(name).map(|(manifest, _)| manifest)
    }

    /// Checks pre-execution permissions for a tool (fail-closed).
    ///
    /// Returns `Ok(&ToolManifest)` if the context is authorized, or `Err(ContextraError::PermissionDenied)` otherwise.
    pub fn authorize(&self, ctx: &AgentContext, tool_name: &str) -> Result<&ToolManifest> {
        let (manifest, _) = self.entries.get(tool_name).ok_or_else(|| {
            ContextraError::InvalidInput(format!("Tool '{tool_name}' not registered"))
        })?;

        if let Some(ref required_cap) = manifest.required_capability {
            let is_granted = ctx
                .memory
                .get("capabilities")
                .and_then(|v| v.as_array())
                .is_some_and(|caps| {
                    caps.iter()
                        .any(|c| c.as_str() == Some(required_cap.as_str()))
                })
                || ctx
                    .memory
                    .get(required_cap)
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);

            if !is_granted {
                return Err(ContextraError::PolicyViolation(format!(
                    "Missing required capability '{required_cap}' for tenant {} on execute_tool:{tool_name}",
                    ctx.state_collection.name()
                )));
            }
        }

        Ok(manifest)
    }

    /// Returns whether a tool with the given name is registered.
    pub fn contains(&self, name: &str) -> bool {
        self.entries.contains_key(name)
    }

    /// Returns the number of registered tools.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Returns `true` if no tools are registered.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}
