//! Standalone text generator port for LLM-driven graph extraction and summarization (N6).

use crate::error::Result;
use crate::storage::GraphFuture;

pub trait TextGenerator: Send + Sync + 'static {
    fn generate<'a>(&'a self, prompt: &'a str) -> GraphFuture<'a, Result<String>>;
}
