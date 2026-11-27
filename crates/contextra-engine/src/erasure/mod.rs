//! Engine orchestration building blocks for durable erasure.

pub mod config;
pub mod registry;
pub mod resolver;

pub use config::{EraseOptions, ErasureConfig};
pub use registry::{LayerOrder, LayerRegistry};
pub use resolver::ScopeResolverImpl;
