//! # Contextra Framework Adapters
//!
//! Ring 4 adapters providing integration between Contextra vector database/storage engine
//! and high-level LLM framework memory/storage protocols (LangChain, LangGraph, LlamaIndex).

#![forbid(unsafe_code)]

use pyo3::prelude::*;

pub mod langchain;
pub mod langgraph;
pub mod llamaindex;

pub use langchain::LangChainChatMessageHistory;
pub use langgraph::LangGraphStoreAdapter;
pub use llamaindex::LlamaIndexStoreAdapter;

/// PyO3 sub-module initializer exporting framework adapter classes.
#[pymodule]
pub fn _contextra_adapters(_py: Python<'_>, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<LangChainChatMessageHistory>()?;
    m.add_class::<LangGraphStoreAdapter>()?;
    m.add_class::<LlamaIndexStoreAdapter>()?;
    Ok(())
}
