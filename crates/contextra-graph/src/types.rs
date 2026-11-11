//! Canonical domain types for  (Ring 1, Klasse A).

#[doc(hidden)]
pub mod local {
    use serde::{Deserialize, Serialize};
    use std::fmt;
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::sync::Arc;

    pub const MAX_SEARCH_K: usize = 1_000;

    fn hash_key_u64(key: &str) -> u64 {
        let hash = blake3::hash(key.as_bytes());
        let mut bytes = [0u8; 8];
        bytes.copy_from_slice(&hash.as_bytes()[..8]);
        u64::from_le_bytes(bytes)
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, Default)]
    #[repr(transparent)]
    pub struct EntityId(pub u64);

    impl EntityId {
        #[inline]
        pub const fn new(id: u64) -> Self { Self(id) }
        #[inline]
        pub const fn inner(self) -> u64 { self.0 }
        pub fn from_key(key: &str) -> Result<Self, std::num::ParseIntError> {
            DocId::from_key(key).map(|d| Self(d.0))
        }
    }

    impl fmt::Display for EntityId {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { write!(f, "EntityId({})", self.0) }
    }

    impl From<u64> for EntityId {
        #[inline]
        fn from(val: u64) -> Self { Self(val) }
    }

    impl From<EntityId> for u64 {
        #[inline]
        fn from(id: EntityId) -> Self { id.0 }
    }

    impl From<&str> for EntityId {
        fn from(s: &str) -> Self {
            if let Ok(val) = s.parse::<u64>() { Self(val) } else { Self(hash_key_u64(s)) }
        }
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, Default)]
    #[repr(transparent)]
    pub struct DocId(pub u64);

    impl DocId {
        #[inline]
        pub const fn new(id: u64) -> Self { Self(id) }
        #[inline]
        pub const fn inner(self) -> u64 { self.0 }
        pub fn from_key(key: &str) -> Result<Self, std::num::ParseIntError> {
            if key.is_empty() { "".parse::<u64>().map(Self) } else { Ok(Self(hash_key_u64(key))) }
        }
    }

    impl fmt::Display for DocId {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { write!(f, "DocId({})", self.0) }
    }

    impl From<u64> for DocId {
        #[inline]
        fn from(val: u64) -> Self { Self(val) }
    }

    impl From<DocId> for u64 {
        #[inline]
        fn from(id: DocId) -> Self { id.0 }
    }

    impl From<&str> for DocId {
        fn from(s: &str) -> Self {
            if let Ok(val) = s.parse::<u64>() { Self(val) } else { Self(hash_key_u64(s)) }
        }
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, Default)]
    #[repr(transparent)]
    pub struct TxId(pub u64);

    impl TxId {
        #[inline]
        pub const fn new(id: u64) -> Self { Self(id) }
        #[inline]
        pub const fn inner(self) -> u64 { self.0 }
    }

    impl fmt::Display for TxId {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { write!(f, "TxId({})", self.0) }
    }

    impl From<u64> for TxId {
        #[inline]
        fn from(val: u64) -> Self { Self(val) }
    }

    impl From<TxId> for u64 {
        #[inline]
        fn from(tx: TxId) -> Self { tx.0 }
    }

    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    pub struct Entity {
        pub id: EntityId,
        pub name: Arc<str>,
        pub entity_type: Arc<str>,
        #[serde(default)]
        pub attributes: ahash::AHashMap<String, serde_json::Value>,
    }

    impl Entity {
        pub fn new(id: EntityId, name: impl Into<String>, entity_type: impl Into<String>) -> Self {
            Self {
                id,
                name: name.into().into(),
                entity_type: entity_type.into().into(),
                attributes: Default::default(),
            }
        }
    }

    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    pub struct Edge {
        pub from: EntityId,
        pub to: EntityId,
        pub label: Arc<str>,
        pub weight: f32,
        #[serde(default, alias = "valid_from")]
        pub tx_valid_from: Option<TxId>,
        #[serde(default, alias = "valid_to")]
        pub tx_valid_to: Option<TxId>,
        #[serde(default)]
        pub business_valid_from: Option<i64>,
        #[serde(default)]
        pub business_valid_to: Option<i64>,
        #[serde(default)]
        pub source_doc_id: Option<DocId>,
    }

    impl Edge {
        pub fn new(from: EntityId, to: EntityId, label: impl Into<String>) -> Self {
            Self {
                from,
                to,
                label: label.into().into(),
                weight: 1.0,
                tx_valid_from: None,
                tx_valid_to: None,
                business_valid_from: None,
                business_valid_to: None,
                source_doc_id: None,
            }
        }
        pub fn with_weight(mut self, weight: f32) -> Self {
            self.weight = weight;
            self
        }
        pub fn with_source_doc_id(mut self, doc_id: Option<DocId>) -> Self {
            self.source_doc_id = doc_id;
            self
        }
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, Default)]
    #[repr(transparent)]
    pub struct HyperEdgeId(pub u64);

    impl HyperEdgeId {
        #[inline]
        pub const fn new(id: u64) -> Self { Self(id) }
        #[inline]
        pub const fn inner(self) -> u64 { self.0 }
    }

    impl From<u64> for HyperEdgeId {
        #[inline]
        fn from(id: u64) -> Self { Self(id) }
    }

    impl fmt::Display for HyperEdgeId {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { write!(f, "HyperEdgeId({})", self.0) }
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, Default)]
    #[repr(transparent)]
    pub struct RoleId(pub u32);

    impl RoleId {
        #[inline]
        pub const fn new(id: u32) -> Self { Self(id) }
        #[inline]
        pub const fn inner(self) -> u32 { self.0 }
    }

    impl From<u32> for RoleId {
        #[inline]
        fn from(id: u32) -> Self { Self(id) }
    }

    impl fmt::Display for RoleId {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { write!(f, "RoleId({})", self.0) }
    }

    #[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
    pub struct RoleBinding {
        pub role: RoleId,
        pub entity: EntityId,
    }

    impl RoleBinding {
        #[inline]
        pub fn new(role: RoleId, entity: EntityId) -> Self { Self { role, entity } }
    }

    #[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
    pub struct TlHfdParams {
        #[serde(default = "default_tl_hfd_sigma")]
        pub sigma: f32,
        #[serde(default = "default_tl_hfd_delta")]
        pub delta: f32,
        #[serde(default = "default_tl_hfd_gamma")]
        pub gamma: f32,
        #[serde(default = "default_tl_hfd_max_iterations")]
        pub max_iterations: u32,
        #[serde(default = "default_tl_hfd_max_top_k_expansion")]
        pub max_top_k_expansion: usize,
        #[serde(default = "default_tl_hfd_max_hyperedge_sort_size")]
        pub max_hyperedge_sort_size: usize,
    }

    fn default_tl_hfd_sigma() -> f32 { 0.1 }
    fn default_tl_hfd_delta() -> f32 { 2.0 }
    fn default_tl_hfd_gamma() -> f32 { 1.0 }
    fn default_tl_hfd_max_iterations() -> u32 { 50 }
    fn default_tl_hfd_max_top_k_expansion() -> usize { 10 }
    fn default_tl_hfd_max_hyperedge_sort_size() -> usize { 64 }

    impl Default for TlHfdParams {
        fn default() -> Self {
            Self {
                sigma: 0.1,
                delta: 2.0,
                gamma: 1.0,
                max_iterations: 50,
                max_top_k_expansion: 10,
                max_hyperedge_sort_size: 64,
            }
        }
    }

    #[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Default)]
    #[serde(rename_all = "snake_case")]
    #[non_exhaustive]
    pub enum PprAlgorithm {
        #[default]
        Auto,
        DensePowerIteration,
        ForwardPush,
        ShadowMode,
        TlHfd(TlHfdParams),
        ShadowModeTlHfd(TlHfdParams),
    }

    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    pub struct PprConfig {
        pub damping_factor: f32,
        pub max_iterations: u32,
        pub convergence_epsilon: f32,
        #[serde(default)]
        pub algorithm: PprAlgorithm,
        #[serde(default = "default_warn_on_non_convergence")]
        pub warn_on_non_convergence: bool,
    }

    fn default_warn_on_non_convergence() -> bool { true }

    impl Default for PprConfig {
        fn default() -> Self {
            Self {
                damping_factor: 0.85,
                max_iterations: 100,
                convergence_epsilon: 1e-6,
                algorithm: PprAlgorithm::Auto,
                warn_on_non_convergence: true,
            }
        }
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    pub struct ResourceBudget {
        pub memory_limit: u64,
    }

    impl Default for ResourceBudget {
        fn default() -> Self {
            Self { memory_limit: 1024 * 1024 * 1024 }
        }
    }

    #[derive(Debug)]
    pub struct ResourceTracker {
        pub budget: ResourceBudget,
        pub memory_used: AtomicU64,
    }

    impl Default for ResourceTracker {
        fn default() -> Self { Self::new(ResourceBudget::default()) }
    }

    impl ResourceTracker {
        pub fn new(budget: ResourceBudget) -> Self {
            Self {
                budget,
                memory_used: AtomicU64::new(0),
            }
        }
        pub fn unlimited() -> Self {
            Self {
                budget: ResourceBudget { memory_limit: u64::MAX },
                memory_used: AtomicU64::new(0),
            }
        }
        pub fn with_limit(max_bytes: usize) -> Self {
            Self {
                budget: ResourceBudget { memory_limit: max_bytes as u64 },
                memory_used: AtomicU64::new(0),
            }
        }
        pub fn consume_memory(&self, bytes: u64) -> Result<(), crate::error::GraphError> {
            loop {
                let current = self.memory_used.load(Ordering::Acquire);
                let next = current.saturating_add(bytes);
                if next > self.budget.memory_limit {
                    return Err(crate::error::GraphError::BudgetExhausted(format!(
                        "Memory budget exceeded: {} MB used, {} MB limit",
                        next / (1024 * 1024),
                        self.budget.memory_limit / (1024 * 1024)
                    )));
                }
                if self.memory_used.compare_exchange(current, next, Ordering::AcqRel, Ordering::Relaxed).is_ok() {
                    return Ok(());
                }
            }
        }
        pub fn record_allocation(&self, bytes: usize) {
            self.memory_used.fetch_add(bytes as u64, Ordering::Relaxed);
        }
        pub fn release_memory(&self, bytes: u64) {
            loop {
                let current = self.memory_used.load(Ordering::Acquire);
                let new_val = current.saturating_sub(bytes);
                if self.memory_used.compare_exchange(current, new_val, Ordering::AcqRel, Ordering::Relaxed).is_ok() {
                    break;
                }
            }
        }
        pub fn memory_used(&self) -> u64 {
            self.memory_used.load(Ordering::SeqCst)
        }
        pub fn has_memory_capacity(&self) -> bool {
            let threshold = self.budget.memory_limit.saturating_mul(95) / 100;
            self.memory_used() < threshold
        }
        pub fn is_exhausted(&self) -> bool {
            !self.has_memory_capacity()
        }
    }
}

#[cfg(feature = "workspace")]
pub use contextra_ports::graph::{HyperEdgeId, RoleBinding, RoleId};
#[cfg(feature = "workspace")]
pub use contextra_types::{
    DocId, Edge, Entity, EntityId, PprAlgorithm, PprConfig, ResourceTracker, TlHfdParams, TxId,
    MAX_SEARCH_K,
};

#[cfg(not(feature = "workspace"))]
pub use local::*;
