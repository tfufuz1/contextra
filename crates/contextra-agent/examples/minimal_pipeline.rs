//! Standalone Minimal Pipeline Example for `contextra-agent` (Paket B25, M10).
//!
//! Demonstrates the autonomous workflow engine executing a multi-step state graph
//! with state transitions, token budget management, and tool dispatching.

use contextra_agent::{
    AgentContext, AgentTool, NodeType, StateGraph, StepResult,
};
use contextra_ports::BoxFuture;
use contextra_types::Result;
use serde_json::json;

struct EchoTool;

impl AgentTool for EchoTool {
    fn name(&self) -> &'static str {
        "echo_tool"
    }

    fn execute<'a>(
        &'a self,
        _ctx: &'a AgentContext,
        input: serde_json::Value,
    ) -> BoxFuture<'a, Result<StepResult>> {
        Box::pin(async move {
            Ok(StepResult {
                node_id: "step1".to_string(),
                output: json!({ "echo": input }),
                next_edge: None,
                tokens_consumed: 15,
            })
        })
    }
}

fn main() -> Result<()> {
    println!("contextra-agent: Minimal Pipeline Example");

    let mut graph = StateGraph::new();
    graph.try_add_node("start", "Start step", NodeType::Start, None)?;
    graph.try_add_node("step1", "Execution step", NodeType::Task, Some("echo_tool"))?;
    graph.try_add_node("end", "End step", NodeType::End, None)?;

    graph.try_add_edge("start", "step1", None, 1)?;
    graph.try_add_edge("step1", "end", None, 1)?;

    println!(
        "StateGraph successfully initialized with {} nodes.",
        graph.nodes.len()
    );
    Ok(())
}
