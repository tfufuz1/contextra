// FILE-CONTEXT Header (Format v3)
// ZWECK: Tool subsystem module re-exporting ToolRegistry, ToolManifest, and productive RetrieveTool.
// INVARIANTEN: Re-exports tool abstractions for orchestration; fail-closed capability checking.
// STAND: TS:2026-10-29T00:00:00Z

//! Tool management subsystem for persistent agent workflows.

pub mod manifest;
pub mod registry;
pub mod retrieve;

pub use manifest::ToolManifest;
pub use registry::ToolRegistry;
pub use retrieve::RetrieveTool;

#[cfg(test)]
mod tests;
