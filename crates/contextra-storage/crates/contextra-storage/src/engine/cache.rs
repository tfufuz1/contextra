//! Version-aware read cache (SPEC §7, WP-11.5, SP-READ).
#![forbid(unsafe_code)]

use contextra_storage_ports::{BranchId, ClusterId, Stamp};
use std::collections::HashMap;

pub const DEFAULT_CACHE_MAX_BYTES: usize = 16 * 1024 * 1024;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct CacheKey {
    cluster: ClusterId,
    branch: BranchId,
    user_key: Vec<u8>,
}

#[derive(Debug, Clone)]
struct CacheEntry {
    value: Vec<u8>,
    stamp: Stamp,
    size_bytes: usize,
}

pub struct ReadCache {
    entries: HashMap<CacheKey, CacheEntry>,
    max_bytes: usize,
    current_bytes: usize,
}

impl Default for ReadCache {
    fn default() -> Self {
        Self::new(DEFAULT_CACHE_MAX_BYTES)
    }
}

impl ReadCache {
    pub fn new(max_bytes: usize) -> Self {
        Self {
            entries: HashMap::new(),
            max_bytes: if max_bytes == 0 {
                DEFAULT_CACHE_MAX_BYTES
            } else {
                max_bytes
            },
            current_bytes: 0,
        }
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn current_bytes(&self) -> usize {
        self.current_bytes
    }

    /// Fetches a cached value if present and visible at `view_stamp`.
    pub fn get(
        &self,
        cluster: ClusterId,
        branch: BranchId,
        user_key: &[u8],
        view_stamp: Stamp,
    ) -> Option<Vec<u8>> {
        let key = CacheKey {
            cluster,
            branch,
            user_key: user_key.to_vec(),
        };
        if let Some(entry) = self.entries.get(&key) {
            if entry.stamp <= view_stamp {
                return Some(entry.value.clone());
            }
        }
        None
    }

    /// Inserts or updates a cached value under memory budget bounds.
    pub fn put(
        &mut self,
        cluster: ClusterId,
        branch: BranchId,
        user_key: Vec<u8>,
        value: Vec<u8>,
        stamp: Stamp,
    ) {
        let size_bytes = user_key.len() + value.len() + core::mem::size_of::<CacheEntry>();
        if size_bytes > self.max_bytes {
            return;
        }

        while self.current_bytes.saturating_add(size_bytes) > self.max_bytes
            && !self.entries.is_empty()
        {
            // Evict an arbitrary entry
            if let Some(evict_key) = self.entries.keys().next().cloned() {
                if let Some(removed) = self.entries.remove(&evict_key) {
                    self.current_bytes = self.current_bytes.saturating_sub(removed.size_bytes);
                }
            }
        }

        let key = CacheKey {
            cluster,
            branch,
            user_key,
        };
        if let Some(old) = self.entries.insert(
            key,
            CacheEntry {
                value,
                stamp,
                size_bytes,
            },
        ) {
            self.current_bytes = self.current_bytes.saturating_sub(old.size_bytes);
        }
        self.current_bytes = self.current_bytes.saturating_add(size_bytes);
    }

    /// Invalidates all cached entries for a cluster (e.g. upon gate or erase).
    pub fn invalidate_cluster(&mut self, cluster: ClusterId) {
        self.entries.retain(|k, v| {
            if k.cluster == cluster {
                self.current_bytes = self.current_bytes.saturating_sub(v.size_bytes);
                false
            } else {
                true
            }
        });
    }

    /// Invalidates all cached entries for a specific cluster and branch.
    pub fn invalidate_branch(&mut self, cluster: ClusterId, branch: BranchId) {
        self.entries.retain(|k, v| {
            if k.cluster == cluster && k.branch == branch {
                self.current_bytes = self.current_bytes.saturating_sub(v.size_bytes);
                false
            } else {
                true
            }
        });
    }

    /// Clears the entire cache.
    pub fn clear(&mut self) {
        self.entries.clear();
        self.current_bytes = 0;
    }
}
