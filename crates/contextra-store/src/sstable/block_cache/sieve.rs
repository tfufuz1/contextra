//! # Sieve-Cache Backend
//!
//! Zweck: Implementierung des SIEVE-Eviction-Algorithmus (effiziente FIFO/Clock-Variante).
// ANCHOR[contextra-store:SST-RDR-141]

use super::traits::BlockCacheBackend;
use bytes::Bytes;
use parking_lot::RwLock;
use std::sync::Arc;

pub struct SieveCacheBackend {
    state: RwLock<SieveCacheState>,
}

struct SieveNode {
    key: (u64, u64),
    value: Bytes,
    visited: std::sync::atomic::AtomicBool,
}

struct SieveCacheState {
    map: ahash::AHashMap<(u64, u64), Arc<SieveNode>>,
    nodes: std::collections::VecDeque<Arc<SieveNode>>,
    hand: usize,
    current_bytes: usize,
    capacity_bytes: usize,
}

impl BlockCacheBackend for SieveCacheBackend {
    fn new(capacity_bytes: usize) -> Self {
        let capacity_bytes = capacity_bytes.max(1);
        Self {
            state: RwLock::new(SieveCacheState {
                map: ahash::AHashMap::new(),
                nodes: std::collections::VecDeque::new(),
                hand: 0,
                current_bytes: 0,
                capacity_bytes,
            }),
        }
    }

    #[inline]
    fn get(&self, key: &(u64, u64)) -> Option<Bytes> {
        let s = self.state.read();
        if let Some(node) = s.map.get(key) {
            node.visited
                .store(true, std::sync::atomic::Ordering::Relaxed);
            return Some(node.value.clone());
        }
        None
    }

    #[inline]
    fn insert(&self, key: (u64, u64), value: Bytes) {
        let val_len = value.len();
        let mut s = self.state.write();

        if let Some(old_node) = s.map.get(&key).cloned() {
            let old_len = old_node.value.len();
            let new_node = Arc::new(SieveNode {
                key,
                value,
                visited: std::sync::atomic::AtomicBool::new(true),
            });
            s.map.insert(key, Arc::clone(&new_node));
            s.nodes.push_back(Arc::clone(&new_node));
            s.current_bytes = s.current_bytes.saturating_sub(old_len) + val_len;
        } else {
            let new_node = Arc::new(SieveNode {
                key,
                value,
                visited: std::sync::atomic::AtomicBool::new(false),
            });
            s.map.insert(key, Arc::clone(&new_node));
            s.nodes.push_back(Arc::clone(&new_node));
            s.current_bytes += val_len;
        }

        while s.current_bytes > s.capacity_bytes && !s.map.is_empty() {
            if s.nodes.is_empty() {
                break;
            }

            if s.hand >= s.nodes.len() {
                s.hand = 0;
            }

            let node = s.nodes[s.hand].clone();

            let hand = s.hand;
            if let Some(map_node) = s.map.get(&node.key) {
                if Arc::ptr_eq(map_node, &node) {
                    if node.visited.load(std::sync::atomic::Ordering::Relaxed) {
                        node.visited
                            .store(false, std::sync::atomic::Ordering::Relaxed);
                        s.hand += 1;
                    } else {
                        s.map.remove(&node.key);
                        s.current_bytes = s.current_bytes.saturating_sub(node.value.len());
                        s.nodes.remove(hand);
                    }
                } else {
                    s.nodes.remove(hand);
                }
            } else {
                s.nodes.remove(hand);
            }
        }
    }

    #[inline]
    fn len(&self) -> usize {
        self.state.read().map.len()
    }

    #[inline]
    fn current_bytes(&self) -> usize {
        self.state.read().current_bytes
    }

    #[inline]
    fn capacity_bytes(&self) -> usize {
        self.state.read().capacity_bytes
    }
}
