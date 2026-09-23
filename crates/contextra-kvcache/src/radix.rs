// FILE-CONTEXT
// ZWECK: Prefix-Radix-Baum über Token-Sequenzen, RAII-Guards & KvReusePolicy (§9.2).
// STAND: TS:2026-09-15T00:00:00Z

//! # Prefix-Radix-Baum für KV-Block-Reuse
//!
//! Dieses Modul implementiert einen komprimierten Radix-Trie über Token-Sequenzen (`&[u32]`).
//! Es ermittelt das längste gemeinsame Präfix (LCP) zwischen einer Anfragesequenz und bereits
//! gecachten KV-Blöcken im Speicher.
//!
//! ## Hauptkomponenten
//! - **`KvReusePolicy`**: Steuert die Aggressivität von Prefix-Reuse (`Always`, `CostBased`, `Never`).
//! - **`KvBlockGuard`**: RAII-Guard mit atomarer Referenzzählung, der bei `Drop` den Block freigibt.
//! - **`PrefixRadixTree`**: Komprimierter Radix-Trie für schnelle O(L) LCP-Suche und Blockzuweisung.

use contextra_core::{ContextraError, TenantId};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use super::eviction_worker::EvictionWorker;

/// Policy zur Steuerung von Prefix-Reuse (§9.2).
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum KvReusePolicy {
    /// Wiederverwendung aller Treffer mit `matched_len > 0`.
    Always,
    /// Kostenbasiert: Treffer erst ab Mindestpräfixlänge `min_prefix_len`.
    CostBased { min_prefix_len: usize },
    /// Keinerlei Prefix-Reuse (immer Cache-Miss).
    Never,
}

impl Default for KvReusePolicy {
    fn default() -> Self {
        Self::CostBased { min_prefix_len: 4 }
    }
}

impl KvReusePolicy {
    /// Prüft, ob ein Treffer mit der Länge `matched_len` wiederverwendet werden soll.
    pub fn should_reuse(&self, matched_len: usize) -> bool {
        match self {
            KvReusePolicy::Always => matched_len > 0,
            KvReusePolicy::CostBased { min_prefix_len } => matched_len >= *min_prefix_len,
            KvReusePolicy::Never => false,
        }
    }
}

/// RAII-Guard für ausgeliehene/gematchte KV-Blöcke.
/// Dekrementiert bei `Drop` atomar die Referenzzählung des Blocks und benachrichtigt ggf. den `EvictionWorker`.
pub struct KvBlockGuard {
    pub block_id: u64,
    pub tenant_id: TenantId,
    ref_counter: Arc<AtomicUsize>,
    worker: Option<Arc<EvictionWorker>>,
}

impl KvBlockGuard {
    /// Erstellt einen neuen Guard und inkrementiert den Referenzzähler.
    pub fn new(
        block_id: u64,
        tenant_id: TenantId,
        ref_counter: Arc<AtomicUsize>,
        worker: Option<Arc<EvictionWorker>>,
    ) -> Self {
        ref_counter.fetch_add(1, Ordering::SeqCst);
        Self {
            block_id,
            tenant_id,
            ref_counter,
            worker,
        }
    }

    /// Gibt die Block-ID zurück.
    pub fn block_id(&self) -> u64 {
        self.block_id
    }

    /// Gibt die Mandanten-ID zurück.
    pub fn tenant_id(&self) -> TenantId {
        self.tenant_id
    }

    /// Gibt die aktuelle Anzahl aktiver Referenzen zurück.
    pub fn active_refs(&self) -> usize {
        self.ref_counter.load(Ordering::SeqCst)
    }
}

impl Drop for KvBlockGuard {
    fn drop(&mut self) {
        let prev = self.ref_counter.fetch_sub(1, Ordering::SeqCst);
        if prev == 1 {
            tracing::debug!(
                block_id = self.block_id,
                tenant_id = ?self.tenant_id,
                "KvBlockGuard dropped: ref count reached 0"
            );
            if let Some(worker) = &self.worker {
                worker.notify_block_released(self.block_id);
            }
        }
    }
}

impl std::fmt::Debug for KvBlockGuard {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("KvBlockGuard")
            .field("block_id", &self.block_id)
            .field("tenant_id", &self.tenant_id)
            .field("active_refs", &self.active_refs())
            .finish()
    }
}

/// Ergebnis eines Präfix-Matches im Radix-Baum.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PrefixMatch {
    /// Übereinstimmende Token-Sequenz.
    pub matched_tokens: Vec<u32>,
    /// Anzahl der übereinstimmenden Tokens.
    pub matched_len: usize,
    /// Assoziierte KV-Block-ID.
    pub block_id: u64,
}

/// Knoten im komprimierten Prefix-Radix-Baum.
struct RadixNode {
    tokens: Vec<u32>,
    block_id: Option<u64>,
    children: Vec<RadixNode>,
}

impl RadixNode {
    fn new(tokens: Vec<u32>, block_id: Option<u64>) -> Self {
        Self {
            tokens,
            block_id,
            children: Vec::new(),
        }
    }

    fn lcp_len(a: &[u32], b: &[u32]) -> usize {
        a.iter().zip(b.iter()).take_while(|(&x, &y)| x == y).count()
    }
}

/// Komprimierter Prefix-Radix-Baum für Token-Sequenzen.
pub struct PrefixRadixTree {
    tenant_id: TenantId,
    children: Vec<RadixNode>,
    total_entries: usize,
}

impl PrefixRadixTree {
    /// Erstellt einen neuen Prefix-Radix-Baum für den angegebenen Tenant.
    pub fn new(tenant_id: TenantId) -> Self {
        Self {
            tenant_id,
            children: Vec::new(),
            total_entries: 0,
        }
    }

    /// Gibt die Tenant-ID des Radix-Baums zurück.
    pub fn tenant_id(&self) -> TenantId {
        self.tenant_id
    }

    /// Fügt eine Token-Sequenz mit assoziierter KV-Block-ID ein.
    pub fn insert(&mut self, tokens: &[u32], block_id: u64) -> Result<(), ContextraError> {
        if tokens.is_empty() {
            return Err(ContextraError::InvalidInput(
                "Cannot insert empty token sequence into PrefixRadixTree".to_string(),
            ));
        }

        Self::insert_into_vec(
            &mut self.children,
            tokens,
            block_id,
            &mut self.total_entries,
        );
        Ok(())
    }

    fn insert_into_vec(
        nodes: &mut Vec<RadixNode>,
        tokens: &[u32],
        block_id: u64,
        total_entries: &mut usize,
    ) {
        for node in nodes.iter_mut() {
            let lcp = RadixNode::lcp_len(&node.tokens, tokens);
            if lcp == 0 {
                continue;
            }

            if lcp == node.tokens.len() {
                let remaining = &tokens[lcp..];
                if remaining.is_empty() {
                    if node.block_id.is_none() {
                        *total_entries += 1;
                    }
                    node.block_id = Some(block_id);
                } else {
                    Self::insert_into_vec(&mut node.children, remaining, block_id, total_entries);
                }
                return;
            }

            // Node split required
            let child_remaining = node.tokens[lcp..].to_vec();
            let parent_tokens = node.tokens[..lcp].to_vec();

            let mut old_child = RadixNode::new(child_remaining, node.block_id);
            old_child.children = std::mem::take(&mut node.children);

            node.tokens = parent_tokens;
            node.block_id = None;
            node.children = vec![old_child];

            let query_remaining = &tokens[lcp..];
            if query_remaining.is_empty() {
                node.block_id = Some(block_id);
                *total_entries += 1;
            } else {
                let new_child = RadixNode::new(query_remaining.to_vec(), Some(block_id));
                node.children.push(new_child);
                *total_entries += 1;
            }
            return;
        }

        // No matching child prefix found -> add new branch
        nodes.push(RadixNode::new(tokens.to_vec(), Some(block_id)));
        *total_entries += 1;
    }

    /// Sucht das längste übereinstimmende Präfix unter Anwendung der `KvReusePolicy`.
    pub fn find_longest_prefix(
        &self,
        tokens: &[u32],
        policy: KvReusePolicy,
    ) -> Option<PrefixMatch> {
        if tokens.is_empty() || matches!(policy, KvReusePolicy::Never) {
            return None;
        }

        let mut current_nodes = &self.children;
        let mut remaining = tokens;
        let mut accumulated_tokens: Vec<u32> = Vec::new();

        let mut best_match: Option<PrefixMatch> = None;

        while !remaining.is_empty() {
            let mut matched_child = None;
            for node in current_nodes {
                let lcp = RadixNode::lcp_len(&node.tokens, remaining);
                if lcp > 0 {
                    matched_child = Some((node, lcp));
                    break;
                }
            }

            let (node, lcp) = match matched_child {
                Some(pair) => pair,
                None => break,
            };

            accumulated_tokens.extend_from_slice(&node.tokens[..lcp]);

            if lcp == node.tokens.len() {
                if let Some(block_id) = node.block_id {
                    let len = accumulated_tokens.len();
                    if policy.should_reuse(len) {
                        best_match = Some(PrefixMatch {
                            matched_tokens: accumulated_tokens.clone(),
                            matched_len: len,
                            block_id,
                        });
                    }
                }
                remaining = &remaining[lcp..];
                current_nodes = &node.children;
            } else {
                break;
            }
        }

        best_match
    }

    /// Entfernt eine Token-Sequenz aus dem Baum.
    pub fn remove(&mut self, tokens: &[u32]) -> Option<u64> {
        if tokens.is_empty() {
            return None;
        }
        Self::remove_from_vec(&mut self.children, tokens, &mut self.total_entries)
    }

    fn remove_from_vec(
        nodes: &mut [RadixNode],
        tokens: &[u32],
        total_entries: &mut usize,
    ) -> Option<u64> {
        for node in nodes.iter_mut() {
            let lcp = RadixNode::lcp_len(&node.tokens, tokens);
            if lcp == node.tokens.len() {
                let remaining = &tokens[lcp..];
                if remaining.is_empty() {
                    if let Some(block_id) = node.block_id.take() {
                        *total_entries = total_entries.saturating_sub(1);
                        return Some(block_id);
                    }
                    return None;
                }
                return Self::remove_from_vec(&mut node.children, remaining, total_entries);
            }
        }
        None
    }

    /// Gibt die Anzahl gespeicherter Präfixe zurück.
    pub fn len(&self) -> usize {
        self.total_entries
    }

    /// Prüft, ob der Baum leer ist.
    pub fn is_empty(&self) -> bool {
        self.total_entries == 0
    }

    /// Leert den gesamten Baum.
    pub fn clear(&mut self) {
        self.children.clear();
        self.total_entries = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_kv_reuse_policy_evaluations() {
        assert!(KvReusePolicy::Always.should_reuse(1));
        assert!(KvReusePolicy::Always.should_reuse(100));
        assert!(!KvReusePolicy::Always.should_reuse(0));

        let cost_policy = KvReusePolicy::CostBased { min_prefix_len: 4 };
        assert!(!cost_policy.should_reuse(0));
        assert!(!cost_policy.should_reuse(3));
        assert!(cost_policy.should_reuse(4));
        assert!(cost_policy.should_reuse(10));

        assert!(!KvReusePolicy::Never.should_reuse(0));
        assert!(!KvReusePolicy::Never.should_reuse(10));
    }

    #[test]
    fn test_radix_tree_basic_insert_and_match() {
        let tenant = TenantId::try_new(1).unwrap();
        let mut tree = PrefixRadixTree::new(tenant);
        assert_eq!(tree.tenant_id(), tenant);

        let seq1 = vec![10, 20, 30, 40, 50];
        tree.insert(&seq1, 1001).unwrap();

        assert_eq!(tree.len(), 1);

        // Match with CostBased policy (min 4)
        let pm = tree
            .find_longest_prefix(&[10, 20, 30, 40, 50, 60], KvReusePolicy::default())
            .expect("Should match seq1");
        assert_eq!(pm.matched_len, 5);
        assert_eq!(pm.block_id, 1001);
        assert_eq!(pm.matched_tokens, vec![10, 20, 30, 40, 50]);

        // Partial match < min_prefix_len (3 < 4)
        let res = tree.find_longest_prefix(
            &[10, 20, 30],
            KvReusePolicy::CostBased { min_prefix_len: 4 },
        );
        assert!(res.is_none());

        // Same partial match with Always policy
        let pm_always = tree
            .find_longest_prefix(&[10, 20, 30, 40, 50], KvReusePolicy::Always)
            .expect("Always policy allows exact length 5 match");
        assert_eq!(pm_always.matched_len, 5);
    }

    #[test]
    fn test_radix_tree_node_splitting() {
        let tenant = TenantId::try_new(1).unwrap();
        let mut tree = PrefixRadixTree::new(tenant);

        // Insert common prefix
        tree.insert(&[1, 2, 3, 10, 20], 100).unwrap();
        tree.insert(&[1, 2, 3, 30, 40], 200).unwrap();

        assert_eq!(tree.len(), 2);

        // Search for query sharing prefix [1, 2, 3, 10, ...]
        let match1 = tree
            .find_longest_prefix(&[1, 2, 3, 10, 20, 99], KvReusePolicy::Always)
            .expect("match1");
        assert_eq!(match1.block_id, 100);
        assert_eq!(match1.matched_len, 5);

        // Search for query sharing prefix [1, 2, 3, 30, ...]
        let match2 = tree
            .find_longest_prefix(&[1, 2, 3, 30, 40, 88], KvReusePolicy::Always)
            .expect("match2");
        assert_eq!(match2.block_id, 200);
        assert_eq!(match2.matched_len, 5);
    }

    #[test]
    fn test_kv_block_guard_raii_refcounting() {
        let tenant = TenantId::try_new(42).unwrap();
        let ref_counter = Arc::new(AtomicUsize::new(0));

        assert_eq!(ref_counter.load(Ordering::SeqCst), 0);

        let guard1 = KvBlockGuard::new(999, tenant, Arc::clone(&ref_counter), None);
        assert_eq!(guard1.active_refs(), 1);
        assert_eq!(ref_counter.load(Ordering::SeqCst), 1);

        {
            let guard2 = KvBlockGuard::new(999, tenant, Arc::clone(&ref_counter), None);
            assert_eq!(guard2.active_refs(), 2);
            assert_eq!(ref_counter.load(Ordering::SeqCst), 2);
        }

        assert_eq!(ref_counter.load(Ordering::SeqCst), 1);
        drop(guard1);
        assert_eq!(ref_counter.load(Ordering::SeqCst), 0);
    }

    #[test]
    fn test_radix_tree_remove_and_clear() {
        let tenant = TenantId::try_new(10).unwrap();
        let mut tree = PrefixRadixTree::new(tenant);

        tree.insert(&[1, 2, 3, 4], 500).unwrap();
        assert_eq!(tree.len(), 1);

        let removed = tree.remove(&[1, 2, 3, 4]);
        assert_eq!(removed, Some(500));
        assert_eq!(tree.len(), 0);
        assert!(tree.is_empty());

        tree.insert(&[10, 20], 600).unwrap();
        tree.clear();
        assert!(tree.is_empty());
    }
}
