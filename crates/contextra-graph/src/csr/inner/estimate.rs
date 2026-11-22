use std::collections::HashSet;
use std::sync::Arc;

use contextra_types::{DocId, Entity, EntityId, TxId};

use super::super::types::EdgePayload;
use super::GraphInner;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct MemoryEstimate {
    pub shared_payload_bytes: usize,
    pub private_bytes: usize,
}

impl MemoryEstimate {
    #[inline]
    pub fn total_bytes(&self) -> usize {
        self.shared_payload_bytes + self.private_bytes
    }
}

#[inline]
fn table_bytes<K, V>(capacity: usize) -> usize {
    capacity * (std::mem::size_of::<(K, V)>() + 1)
}

#[inline]
fn vec_bytes<T>(capacity: usize) -> usize {
    capacity * std::mem::size_of::<T>()
}

impl GraphInner {
    fn estimate_primary_tables_memory(&self, presized: bool) -> usize {
        let mut p = 0usize;

        let cap = if presized {
            self.id_map.len()
        } else {
            self.id_map.capacity()
        };
        p += table_bytes::<EntityId, usize>(cap);

        let cap = if presized {
            self.communities.len()
        } else {
            self.communities.capacity()
        };
        p += table_bytes::<EntityId, u64>(cap);

        let cap = if presized {
            self.staged_entities.len()
        } else {
            self.staged_entities.capacity()
        };
        p += table_bytes::<(TxId, EntityId), Entity>(cap);

        let cap = if presized {
            self.staged_edges.len()
        } else {
            self.staged_edges.capacity()
        };
        p += table_bytes::<(TxId, EntityId, EntityId), EdgePayload>(cap);

        let cap = if presized {
            self.staged_removals.len()
        } else {
            self.staged_removals.capacity()
        };
        p += table_bytes::<(TxId, EntityId, EntityId), ()>(cap);

        let cap = if presized {
            self.tombstoned_edges.len()
        } else {
            self.tombstoned_edges.capacity()
        };
        p += table_bytes::<(EntityId, EntityId), ()>(cap);

        p
    }

    fn estimate_csr_vectors_memory(&self, presized: bool) -> usize {
        let mut p = 0usize;
        p += vec_bytes::<EntityId>(if presized {
            self.reverse_map.len()
        } else {
            self.reverse_map.capacity()
        });
        p += vec_bytes::<Entity>(if presized {
            self.entities.len()
        } else {
            self.entities.capacity()
        });
        p += vec_bytes::<usize>(if presized {
            self.offsets.len()
        } else {
            self.offsets.capacity()
        });
        p += vec_bytes::<usize>(if presized {
            self.targets.len()
        } else {
            self.targets.capacity()
        });
        p += vec_bytes::<f32>(if presized {
            self.weights.len()
        } else {
            self.weights.capacity()
        });
        p += vec_bytes::<TxId>(if presized {
            self.tx_valid_froms.len()
        } else {
            self.tx_valid_froms.capacity()
        });
        p += vec_bytes::<TxId>(if presized {
            self.tx_valid_tos.len()
        } else {
            self.tx_valid_tos.capacity()
        });
        p += vec_bytes::<i64>(if presized {
            self.business_valid_froms.len()
        } else {
            self.business_valid_froms.capacity()
        });
        p += vec_bytes::<i64>(if presized {
            self.business_valid_tos.len()
        } else {
            self.business_valid_tos.capacity()
        });
        p += vec_bytes::<DocId>(if presized {
            self.source_doc_ids.len()
        } else {
            self.source_doc_ids.capacity()
        });
        p += vec_bytes::<f32>(if presized {
            self.out_weight_sums.len()
        } else {
            self.out_weight_sums.capacity()
        });
        p
    }

    fn estimate_pending_and_hyperedge_memory(&self, presized: bool) -> usize {
        let mut p = 0usize;

        let cap = if presized {
            self.pending_edges.len()
        } else {
            self.pending_edges.capacity()
        };
        p += table_bytes::<(EntityId, EntityId), Vec<EdgePayload>>(cap);
        for v in self.pending_edges.values() {
            p += vec_bytes::<EdgePayload>(if presized { v.len() } else { v.capacity() });
        }

        let cap = if presized {
            self.doc_to_edges.len()
        } else {
            self.doc_to_edges.capacity()
        };
        p += table_bytes::<DocId, HashSet<(EntityId, EntityId)>>(cap);
        for set in self.doc_to_edges.values() {
            p += table_bytes::<(EntityId, EntityId), ()>(if presized {
                set.len()
            } else {
                set.capacity()
            });
        }

        let cap = if presized {
            self.hyperedges.len()
        } else {
            self.hyperedges.capacity()
        };
        p += table_bytes::<crate::hyperedge::HyperEdgeId, Arc<crate::hyperedge::HyperEdge>>(cap);

        let cap = if presized {
            self.doc_to_hyperedges.len()
        } else {
            self.doc_to_hyperedges.capacity()
        };
        p += table_bytes::<DocId, HashSet<crate::hyperedge::HyperEdgeId>>(cap);
        for set in self.doc_to_hyperedges.values() {
            p += table_bytes::<crate::hyperedge::HyperEdgeId, ()>(if presized {
                set.len()
            } else {
                set.capacity()
            });
        }

        let cap = if presized {
            self.child_to_parents.len()
        } else {
            self.child_to_parents.capacity()
        };
        p += table_bytes::<crate::hyperedge::HyperEdgeId, HashSet<crate::hyperedge::HyperEdgeId>>(
            cap,
        );
        for set in self.child_to_parents.values() {
            p += table_bytes::<crate::hyperedge::HyperEdgeId, ()>(if presized {
                set.len()
            } else {
                set.capacity()
            });
        }

        let cap = if presized {
            self.hyperedge_index.len()
        } else {
            self.hyperedge_index.capacity()
        };
        p += table_bytes::<EntityId, HashSet<crate::hyperedge::HyperEdgeId>>(cap);
        for set in self.hyperedge_index.values() {
            p += table_bytes::<crate::hyperedge::HyperEdgeId, ()>(if presized {
                set.len()
            } else {
                set.capacity()
            });
        }

        p
    }

    pub(crate) fn estimate_memory(&self, presized: bool) -> MemoryEstimate {
        let private = self.estimate_primary_tables_memory(presized)
            + self.estimate_csr_vectors_memory(presized)
            + self.estimate_pending_and_hyperedge_memory(presized);

        let shared_payload_bytes = self
            .hyperedges
            .values()
            .map(|e| {
                std::mem::size_of::<crate::hyperedge::HyperEdge>()
                    + (e.participants.len() * std::mem::size_of::<crate::hyperedge::RoleBinding>())
            })
            .sum::<usize>();

        MemoryEstimate {
            shared_payload_bytes,
            private_bytes: private,
        }
    }

    pub(crate) fn estimate_memory_bytes(&self) -> usize {
        self.estimate_memory(false).total_bytes()
    }

    pub(crate) fn estimate_compaction_peak_bytes(&self) -> usize {
        let current = self.estimate_memory(false);
        let rebuild_private = self.estimate_memory(true).private_bytes;
        current.total_bytes() + rebuild_private
    }
}
