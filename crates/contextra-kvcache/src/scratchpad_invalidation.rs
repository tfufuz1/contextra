// FILE-CONTEXT
// ZWECK: Targeted Invalidation von Agent-Scratchpad-Prefixes im Radix-Baum (Spec §9.2 / INV-CLM-SCRATCHPAD-1).
// STAND: TS:2026-10-01T00:00:00Z

//! # Scratchpad-Invalidierung (`ScratchpadCacheScope`, `ScratchpadInvalidator`)
//!
//! Stellt Mechanismen zur gezielten Invalidierung von Agenten-Scratchpad-Bereichen im KV-Prefix-Store
//! bereit, ohne den gepinnten Basiskontext anderer Partitionen oder Mandanten zu beeinträchtigen.

#![forbid(unsafe_code)]

use contextra_ports::kv::PrefixKey;
use contextra_types::error::Result;
use contextra_types::TenantId;

use crate::prefix_store::TenantPrefixKvStore;

/// Definiert den Bereich (Scope) eines Agenten-Scratchpads für die Cache-Invalidierung.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScratchpadCacheScope {
    /// Eindeutige Mandanten-ID (TenantId) des ausführenden Agenten.
    pub tenant_id: TenantId,
    /// Token-Präfix, das den editierbaren Scratchpad-Bereich im Radix-Baum markiert.
    pub scratchpad_key_prefix: Vec<u32>,
}

impl ScratchpadCacheScope {
    /// Erstellt einen neuen `ScratchpadCacheScope` für den angegebenen Mandanten und das Token-Präfix.
    pub fn new(tenant_id: TenantId, scratchpad_key_prefix: Vec<u32>) -> Self {
        Self {
            tenant_id,
            scratchpad_key_prefix,
        }
    }
}

/// Schnittstelle zur gezielten Invalidation von Scratchpad-Cache-Scopes.
pub trait ScratchpadInvalidator {
    /// Invalidiert alle Gruppen/Blöcke im Radix-Baum, die zum angegebenen `ScratchpadCacheScope` gehören.
    ///
    /// # Fehler
    /// Gibt einen `Result<usize>` zurück, wobei `usize` die Anzahl invalidierter Segmente/Blöcke darstellt.
    fn invalidate_scope(&self, scope: &ScratchpadCacheScope) -> Result<usize>;
}

/// Implementierung von `ScratchpadInvalidator` für `TenantPrefixKvStore`.
pub struct PrefixStoreScratchpadInvalidator<'a> {
    /// Referenz auf den unterliegenden `TenantPrefixKvStore`.
    pub store: &'a TenantPrefixKvStore,
    /// Liste von `PrefixKey`s, die im Store auf Scratchpad-Präfixe geprüft und evictiert werden sollen.
    pub keys: Vec<PrefixKey>,
}

impl<'a> PrefixStoreScratchpadInvalidator<'a> {
    /// Erstellt einen neuen `PrefixStoreScratchpadInvalidator` für den angegebenen Store.
    pub fn new(store: &'a TenantPrefixKvStore) -> Self {
        Self {
            store,
            keys: Vec::new(),
        }
    }

    /// Konfiguriert die zu prüfenden `PrefixKey`s für die Invalidation.
    pub fn with_keys(mut self, keys: Vec<PrefixKey>) -> Self {
        self.keys = keys;
        self
    }
}

impl<'a> ScratchpadInvalidator for PrefixStoreScratchpadInvalidator<'a> {
    fn invalidate_scope(&self, scope: &ScratchpadCacheScope) -> Result<usize> {
        if scope.scratchpad_key_prefix.is_empty() {
            return Ok(0);
        }

        self.store.invalidate_prefix_scope_keys(
            scope.tenant_id,
            &scope.scratchpad_key_prefix,
            &self.keys,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::radix::KvReusePolicy;
    use bytes::Bytes;
    use contextra_ports::kv::{KvBlock, KvLayout, RopeConfig};
    use contextra_types::model_fingerprint::ModelFingerprint;

    fn create_test_prefix_key(name: &str) -> PrefixKey {
        PrefixKey {
            model: ModelFingerprint::new([0xcd; 32], name, "F16"),
            tokenizer_hash: [0x11; 32],
            layout: KvLayout {
                n_layer: 16,
                n_kv_head: 4,
                head_dim: 64,
                dtype: "f16".to_string(),
            },
            rope: RopeConfig {
                base: 10000.0,
                scaling: None,
            },
        }
    }

    #[test]
    fn test_scratchpad_invalidation_matching_prefix() {
        let store = TenantPrefixKvStore::new().with_reuse_policy(KvReusePolicy::Always);
        let tenant_a = TenantId::try_new(10).unwrap();

        let key_base = create_test_prefix_key("model-base");
        let key_scratchpad = create_test_prefix_key("model-scratchpad");

        let base_tokens = vec![1, 2, 3, 4];
        let base_block = KvBlock {
            block_id: 1,
            data: Bytes::from_static(b"base_context_data"),
        };
        store
            .insert(tenant_a, &key_base, &base_tokens, vec![base_block])
            .unwrap();

        let scratchpad_tokens = vec![100, 101, 102, 103];
        let scratchpad_block = KvBlock {
            block_id: 2,
            data: Bytes::from_static(b"scratchpad_data"),
        };
        store
            .insert(
                tenant_a,
                &key_scratchpad,
                &scratchpad_tokens,
                vec![scratchpad_block],
            )
            .unwrap();

        let invalidator = PrefixStoreScratchpadInvalidator::new(&store)
            .with_keys(vec![key_base.clone(), key_scratchpad.clone()]);

        let scope = ScratchpadCacheScope::new(tenant_a, scratchpad_tokens.clone());

        let count = invalidator.invalidate_scope(&scope).unwrap();
        assert_eq!(count, 1);

        assert!(store
            .lookup(tenant_a, &key_scratchpad, &scratchpad_tokens)
            .is_none());
        let base_hit = store.lookup(tenant_a, &key_base, &base_tokens).unwrap();
        assert_eq!(base_hit.matched_tokens, 4);
    }

    #[test]
    fn test_pinned_base_context_remains_untouched() {
        let store = TenantPrefixKvStore::new().with_reuse_policy(KvReusePolicy::Always);
        let tenant = TenantId::try_new(20).unwrap();

        let key_pinned = create_test_prefix_key("pinned-context");
        let pinned_tokens = vec![10, 20, 30, 40, 50];
        let pinned_block = KvBlock {
            block_id: 10,
            data: Bytes::from_static(b"pinned_base_block"),
        };
        store
            .insert(tenant, &key_pinned, &pinned_tokens, vec![pinned_block])
            .unwrap();

        let invalidator =
            PrefixStoreScratchpadInvalidator::new(&store).with_keys(vec![key_pinned.clone()]);

        let unmatching_scope = ScratchpadCacheScope::new(tenant, vec![999, 888, 777]);
        let invalidated = invalidator.invalidate_scope(&unmatching_scope).unwrap();
        assert_eq!(invalidated, 0);

        let hit = store
            .lookup(tenant, &key_pinned, &pinned_tokens)
            .expect("Pinned base context must remain retrievable");
        assert_eq!(hit.matched_tokens, 5);
    }

    #[test]
    fn test_tenant_isolation_scratchpad_invalidation() {
        let store = TenantPrefixKvStore::new().with_reuse_policy(KvReusePolicy::Always);
        let tenant_a = TenantId::try_new(100).unwrap();
        let tenant_b = TenantId::try_new(200).unwrap();

        let key = create_test_prefix_key("shared-model");
        let scratchpad_tokens = vec![50, 51, 52, 53];

        let block_a = KvBlock {
            block_id: 1,
            data: Bytes::from_static(b"tenant_a_scratchpad"),
        };
        let block_b = KvBlock {
            block_id: 2,
            data: Bytes::from_static(b"tenant_b_scratchpad"),
        };

        store
            .insert(tenant_a, &key, &scratchpad_tokens, vec![block_a])
            .unwrap();
        store
            .insert(tenant_b, &key, &scratchpad_tokens, vec![block_b])
            .unwrap();

        let invalidator =
            PrefixStoreScratchpadInvalidator::new(&store).with_keys(vec![key.clone()]);

        let scope_a = ScratchpadCacheScope::new(tenant_a, scratchpad_tokens.clone());
        let count = invalidator.invalidate_scope(&scope_a).unwrap();
        assert_eq!(count, 1);

        assert!(store.lookup(tenant_a, &key, &scratchpad_tokens).is_none());
        let hit_b = store
            .lookup(tenant_b, &key, &scratchpad_tokens)
            .expect("Tenant B's scratchpad must remain untouched");
        assert_eq!(hit_b.matched_tokens, 4);
    }
}
