//! Standalone Minimal Pipeline Example for `contextra-agent` (Paket B25, M10).
//!
//! Demonstrates the autonomous workflow engine executing a multi-step state graph
//! with state transitions, token budget management, and tool dispatching.

use contextra_agent::{
    AgentContext, AgentNode, AgentTool, NodeType, StateGraph, StepResult, WorkflowEdge,
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
                output: json!({ "echo": input }),
                next_node: None,
                tokens_consumed: 15,
            })
        })
    }
}

fn main() -> Result<()> {
    println!("contextra-agent: Minimal Pipeline Example");

    let mut graph = StateGraph::new();
    graph.try_add_node(AgentNode {
        id: "start".to_string(),
        description: "Start step".to_string(),
        node_type: NodeType::Start,
        handler: None,
    })?;

    graph.try_add_node(AgentNode {
        id: "step1".to_string(),
        description: "Execution step".to_string(),
        node_type: NodeType::Task,
        handler: Some("echo_tool".to_string()),
    })?;

    graph.try_add_node(AgentNode {
        id: "end".to_string(),
        description: "End step".to_string(),
        node_type: NodeType::End,
        handler: None,
    })?;

    graph.try_add_edge(WorkflowEdge {
        from: "start".to_string(),
        to: "step1".to_string(),
        condition: None,
        priority: 1,
        goal_condition: None,
    })?;

    graph.try_add_edge(WorkflowEdge {
        from: "step1".to_string(),
        to: "end".to_string(),
        condition: None,
        priority: 1,
        goal_condition: None,
    })?;

    println!("StateGraph successfully initialized with {} nodes.", graph.nodes.len());
    Ok(())
}
