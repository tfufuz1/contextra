//! Compatibility bridge for  (SPEC-v2 §4.3, §6.2).

#[cfg(feature = "workspace")]
pub mod error;
#[cfg(feature = "workspace")]
pub mod provider;
#[cfg(feature = "workspace")]
pub mod purger;
#[cfg(feature = "workspace")]
pub mod graph_index;
#[cfg(feature = "workspace")]
pub mod collection_mutation;
#[cfg(feature = "workspace")]
pub mod storage_bridge;
#[cfg(feature = "workspace")]
pub mod graph_index_ext;
#[cfg(feature = "workspace")]
pub mod replay_hook;
#[cfg(feature = "workspace")]
pub mod algo_error_bridge;
#[cfg(feature = "workspace")]
pub mod llm_bridge;
