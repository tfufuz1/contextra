// FILE-CONTEXT
// ZWECK: Mandantenisolierter KV-Prefix-Store mit Byte-Budget & LRU-Eviction (Spec §9.2).
// STAND: TS:2026-09-15T00:00:00Z

//! # Mandantenisolierter KV-Prefix-Store (`TenantPrefixKvStore`)
//!
//! Implementierung des Ports `contextra_ports::KvPrefixStore` für `contextra-kvcache`.
//! Verwendet `PrefixRadixTree` und `KvReusePolicy`, mit Byte-Budget pro Tenant und LRU-Eviction.

use ahash::AHashMap;
use contextra_ports::kv::{KvBlock, KvPrefixHit, KvPrefixStore, PrefixKey};
use contextra_types::{ContextraError, TenantId};
use parking_lot::RwLock;
use std::sync::atomic::{AtomicU64, Ordering};

use crate::radix::{KvReusePolicy, PrefixRadixTree};

/// Standard-Byte-Budget pro Tenant (256 MiB).
pub const DEFAULT_BYTE_BUDGET_PER_TENANT: usize = 256 * 1024 * 1024;

#[derive(Debug, Clone)]
struct BlockGroup {
    _group_id: u64,
    tokens: Vec<u32>,
    blocks: Vec<KvBlock>,
    bytes: usize,
    tick: u64,
}

struct Partition {
    tree: PrefixRadixTree,
    groups: AHashMap<u64, BlockGroup>,
    total_bytes: usize,
}

impl Partition {
    fn new(tenant_id: TenantId) -> Self {
        Self {
            tree: PrefixRadixTree::new(tenant_id),
            groups: AHashMap::new(),
            total_bytes: 0,
        }
    }
}

impl std::fmt::Debug for Partition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Partition")
            .field("tree_len", &self.tree.len())
            .field("groups_count", &self.groups.len())
            .field("total_bytes", &self.total_bytes)
            .finish()
    }
}

/// Mandantenisolierter KV-Prefix-Store.
///
/// Bietet strukturelle Isolation via `(TenantId, PrefixKey)`-Key in den Partitionen.
/// Unterstützt konfigurierbares Byte-Budget pro Tenant und LRU-Eviction bei Überschreitung.
pub struct TenantPrefixKvStore {
    partitions: RwLock<AHashMap<(TenantId, PrefixKey), Partition>>,
    byte_budget_per_tenant: usize,
    reuse_policy: KvReusePolicy,
    next_tick: AtomicU64,
    next_group_id: AtomicU64,
}

impl TenantPrefixKvStore {
    /// Erstellt einen neuen `TenantPrefixKvStore` mit Standard-Konfiguration (256 MiB Budget).
    pub fn new() -> Self {
        Self {
            partitions: RwLock::new(AHashMap::new()),
            byte_budget_per_tenant: DEFAULT_BYTE_BUDGET_PER_TENANT,
            reuse_policy: KvReusePolicy::default(),
            next_tick: AtomicU64::new(1),
            next_group_id: AtomicU64::new(1),
        }
    }

    /// Konfiguriert das maximale Byte-Budget pro Mandant über alle seine Präfix-Partitionen.
    pub fn with_byte_budget_per_tenant(mut self, budget: usize) -> Self {
        self.byte_budget_per_tenant = budget;
        self
    }

    /// Konfiguriert die Wiederverwendungs-Policy (`KvReusePolicy`).
    pub fn with_reuse_policy(mut self, policy: KvReusePolicy) -> Self {
        self.reuse_policy = policy;
        self
    }

    /// Sucht nach dem längsten übereinstimmenden Präfix in Blöcken für die angegebenen Tokens.
    pub fn lookup(&self, tenant: TenantId, key: &PrefixKey, tokens: &[u32]) -> Option<KvPrefixHit> {
        if tokens.is_empty() {
            return None;
        }

        let mut partitions = self.partitions.write();
        let partition = partitions.get_mut(&(tenant, key.clone()))?;

        let pm = partition.tree.find_longest_prefix(tokens, self.reuse_policy)?;
        let group = partition.groups.get_mut(&pm.block_id)?;

        group.tick = self.next_tick.fetch_add(1, Ordering::Relaxed);

        Some(KvPrefixHit {
            matched_tokens: pm.matched_len,
            blocks: group.blocks.clone(),
        })
    }

    /// Fügt KV-Blöcke für ein Token-Präfix in den Store ein.
    pub fn insert(
        &self,
        tenant: TenantId,
        key: &PrefixKey,
        tokens: &[u32],
        blocks: Vec<KvBlock>,
    ) -> Result<(), ContextraError> {
        if tokens.is_empty() {
            return Err(ContextraError::InvalidInput(
                "Cannot insert with empty tokens".to_string(),
            ));
        }
        if blocks.is_empty() {
            return Err(ContextraError::InvalidInput(
                "Cannot insert with empty blocks".to_string(),
            ));
        }

        let group_bytes: usize = blocks.iter().map(|b| b.data.len()).sum();
        if group_bytes > self.byte_budget_per_tenant {
            return Err(ContextraError::PolicyViolation(format!(
                "Block group size ({group_bytes} bytes) exceeds tenant byte budget ({budget} bytes)",
                budget = self.byte_budget_per_tenant
            )));
        }

        let mut map = self.partitions.write();

        let partition = map
            .entry((tenant, key.clone()))
            .or_insert_with(|| Partition::new(tenant));

        if let Some(old_group_id) = partition.tree.remove(tokens) {
            if let Some(old_group) = partition.groups.remove(&old_group_id) {
                partition.total_bytes = partition.total_bytes.saturating_sub(old_group.bytes);
            }
        }

        let group_id = self.next_group_id.fetch_add(1, Ordering::Relaxed);
        let tick = self.next_tick.fetch_add(1, Ordering::Relaxed);

        partition.tree.insert(tokens, group_id)?;
        partition.groups.insert(
            group_id,
            BlockGroup {
                _group_id: group_id,
                tokens: tokens.to_vec(),
                blocks,
                bytes: group_bytes,
                tick,
            },
        );
        partition.total_bytes += group_bytes;

        Self::evict_to_budget_internal(&mut map, tenant, self.byte_budget_per_tenant);

        Ok(())
    }

    /// Evictiert KV-Präfix-Einträge für den angegebenen Mandanten und PrefixKey.
    /// Gibt die Anzahl der entfernten Blöcke zurück.
    pub fn evict(&self, tenant: TenantId, key: &PrefixKey) -> Result<u64, ContextraError> {
        let mut map = self.partitions.write();
        if let Some(partition) = map.remove(&(tenant, key.clone())) {
            let total_blocks: usize = partition.groups.values().map(|g| g.blocks.len()).sum();
            Ok(total_blocks as u64)
        } else {
            Ok(0)
        }
    }

    fn evict_to_budget_internal(
        map: &mut AHashMap<(TenantId, PrefixKey), Partition>,
        tenant: TenantId,
        budget: usize,
    ) {
        loop {
            let current_tenant_bytes: usize = map
                .iter()
                .filter(|(k, _)| k.0 == tenant)
                .map(|(_, p)| p.total_bytes)
                .sum();

            if current_tenant_bytes <= budget {
                break;
            }

            let mut candidate: Option<(PrefixKey, u64, u64)> = None; // (key, group_id, tick)

            for ((t, key), partition) in map.iter() {
                if *t != tenant {
                    continue;
                }
                for (&group_id, group) in &partition.groups {
                    match candidate {
                        None => {
                            candidate = Some((key.clone(), group_id, group.tick));
                        }
                        Some((_, best_gid, best_tick)) => {
                            if group.tick < best_tick
                                || (group.tick == best_tick && group_id < best_gid)
                            {
                                candidate = Some((key.clone(), group_id, group.tick));
                            }
                        }
                    }
                }
            }

            let (target_key, target_gid, _) = match candidate {
                Some(c) => c,
                None => break,
            };

            if let Some(partition) = map.get_mut(&(tenant, target_key.clone())) {
                if let Some(group) = partition.groups.remove(&target_gid) {
                    partition.tree.remove(&group.tokens);
                    partition.total_bytes = partition.total_bytes.saturating_sub(group.bytes);
                }
                if partition.groups.is_empty() {
                    map.remove(&(tenant, target_key));
                }
            }
        }
    }
}

impl Default for TenantPrefixKvStore {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for TenantPrefixKvStore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let guard = self.partitions.read();
        f.debug_struct("TenantPrefixKvStore")
            .field("partitions_count", &guard.len())
            .field("byte_budget_per_tenant", &self.byte_budget_per_tenant)
            .field("reuse_policy", &self.reuse_policy)
            .finish()
    }
}

impl KvPrefixStore for TenantPrefixKvStore {
    fn lookup(&self, tenant: TenantId, key: &PrefixKey, tokens: &[u32]) -> Option<KvPrefixHit> {
        self.lookup(tenant, key, tokens)
    }

    fn insert(
        &self,
        tenant: TenantId,
        key: &PrefixKey,
        tokens: &[u32],
        blocks: Vec<KvBlock>,
    ) -> Result<(), ContextraError> {
        self.insert(tenant, key, tokens, blocks)
    }

    fn evict(&self, tenant: TenantId, key: &PrefixKey) -> Result<u64, ContextraError> {
        self.evict(tenant, key)
    }
}
