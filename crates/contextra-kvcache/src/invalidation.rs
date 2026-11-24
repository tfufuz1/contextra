//! Lokale Schnittstellen und Datenstrukturen für die Cache-Invalidierung.

use crate::types::TenantId;

// ANCHOR[contextra-kvcache:INV-SCOPE-001]
/// Definiert den Bereich (Scope) eines Agenten-Scratchpads für die Cache-Invalidierung.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScratchpadCacheScope {
    /// Mandanten-Identifier.
    pub tenant_id: TenantId,
    /// Token-Präfix, das den editierbaren Scratchpad-Bereich im Radix-Baum markiert.
    pub scratchpad_key_prefix: Vec<u32>,
}

impl ScratchpadCacheScope {
    /// Erstellt einen neuen [`ScratchpadCacheScope`].
    pub fn new(tenant_id: impl Into<TenantId>, scratchpad_key_prefix: Vec<u32>) -> Self {
        Self {
            tenant_id: tenant_id.into(),
            scratchpad_key_prefix,
        }
    }
}
