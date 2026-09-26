//! Global Fusion Strategy for Corpus-Wide Topic Retrieval (§B.3.4).
//!
//! Executes community-aware Reciprocal Rank Fusion (RRF) over LeanRAG-aggregated nodes
//! grouped by Leiden community assignments.

use ahash::{AHashMap, AHashSet};
use contextra_types::ContextraError;
use serde::{Deserialize, Serialize};

use super::rrf::weighted_reciprocal_rank_fusion_with_options;
use super::signal::MetadataMergePriority;
use super::types::SearchResult;

/// Configuration for `GlobalFusionStrategy`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GlobalFusionConfig {
    /// Maximum number of community nodes to include in the final result set.
    pub max_community_nodes: Option<usize>,
    /// Minimum size (member count) required for a community to be considered.
    pub min_community_size: Option<usize>,
    /// RRF smoothing parameter k (default: 60.0).
    pub k_rrf: f32,
}

impl Default for GlobalFusionConfig {
    fn default() -> Self {
        Self {
            max_community_nodes: Some(50),
            min_community_size: Some(3),
            k_rrf: 60.0,
        }
    }
}

/// Community-aware fusion strategy for corpus-wide topic retrieval (LeanRAG-backed).
#[derive(Debug, Clone, Default)]
pub struct GlobalFusionStrategy {
    config: GlobalFusionConfig,
}

impl GlobalFusionStrategy {
    /// Creates a new `GlobalFusionStrategy` with the given configuration.
    pub fn new(config: GlobalFusionConfig) -> Self {
        Self { config }
    }

    /// Returns a reference to the configuration.
    pub fn config(&self) -> &GlobalFusionConfig {
        &self.config
    }

    /// Fuses multi-signal search results using community-aware RRF.
    ///
    /// Candidates are grouped by community ID (extracted from candidate metadata `"community_id"`
    /// or `"community"`). Communities smaller than `min_community_size` are filtered out.
    /// Intra-community RRF is computed per community, and communities are ranked by aggregated
    /// relevance before selecting the top nodes up to `max_community_nodes`.
    ///
    /// If no community assignments are present in candidate metadata (empty community index),
    /// this method degrades controlledly with `ContextraError::InvalidInput`.
    pub fn fuse(
        &self,
        result_sets: Vec<(String, Vec<SearchResult>, f32)>,
        max_results: usize,
    ) -> Result<Vec<SearchResult>, ContextraError> {
        if max_results == 0 {
            return Ok(Vec::new());
        }

        // 1. Extract community assignment for each candidate document across all signal sets.
        let mut doc_community: AHashMap<String, u64> = AHashMap::new();
        let mut total_candidates = 0usize;

        for (_, result_set, _) in &result_sets {
            total_candidates += result_set.len();
            for doc in result_set {
                if let Some(ref meta) = doc.metadata {
                    let comm_id = meta
                        .get("community_id")
                        .or_else(|| meta.get("community"))
                        .and_then(|v| v.as_u64().or_else(|| v.as_i64().map(|i| i as u64)));
                    if let Some(cid) = comm_id {
                        doc_community.insert(doc.id.clone(), cid);
                    }
                }
            }
        }

        // Degradation check: If candidates exist but no community assignment was found,
        // degrade controlledly with ContextraError::InvalidInput.
        if total_candidates > 0 && doc_community.is_empty() {
            return Err(ContextraError::InvalidInput(
                "Community index is empty or no community assignments present in search candidates; RetrievalStrategy::Global requires prior consolidation (contextra_consolidate)".to_string(),
            ));
        }

        if total_candidates == 0 {
            return Ok(Vec::new());
        }

        // Group candidate doc IDs by community ID.
        let mut community_members: AHashMap<u64, AHashSet<String>> = AHashMap::new();
        for (doc_id, &cid) in &doc_community {
            community_members
                .entry(cid)
                .or_default()
                .insert(doc_id.clone());
        }

        // Filter communities by min_community_size.
        let min_size = self.config.min_community_size.unwrap_or(1);
        community_members.retain(|_, members| members.len() >= min_size);

        if community_members.is_empty() {
            return Err(ContextraError::InvalidInput(
                format!("No communities satisfy min_community_size requirement ({min_size})"),
            ));
        }

        // 2. Intra-community RRF per community.
        struct CommunityResult {
            community_id: u64,
            fused_candidates: Vec<SearchResult>,
            aggregate_score: f32,
        }

        let mut community_results: Vec<CommunityResult> = Vec::new();

        for (&cid, members) in &community_members {
            // Filter result sets for members of this community
            let filtered_sets: Vec<(String, Vec<SearchResult>, f32)> = result_sets
                .iter()
                .map(|(sig, list, w)| {
                    let filtered: Vec<SearchResult> = list
                        .iter()
                        .filter(|doc| members.contains(&doc.id))
                        .cloned()
                        .collect();
                    (sig.clone(), filtered, *w)
                })
                .collect();

            let fused = weighted_reciprocal_rank_fusion_with_options(
                filtered_sets,
                members.len(),
                MetadataMergePriority::default(),
                true,
                None,
            );

            let aggregate_score: f32 = fused.iter().map(|res| res.score).sum();

            community_results.push(CommunityResult {
                community_id: cid,
                fused_candidates: fused,
                aggregate_score,
            });
        }

        // 3. Rank communities by aggregated relevance (descending score, tie-break by community_id).
        community_results.sort_by(|a, b| {
            b.aggregate_score
                .partial_cmp(&a.aggregate_score)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| a.community_id.cmp(&b.community_id))
        });

        // 4. Flatten ordered candidates up to max_community_nodes and max_results limit.
        let max_nodes = self
            .config
            .max_community_nodes
            .unwrap_or(usize::MAX)
            .min(max_results);

        let mut final_results: Vec<SearchResult> = Vec::with_capacity(max_nodes);

        for comm in community_results {
            for mut doc in comm.fused_candidates {
                if final_results.len() >= max_nodes {
                    break;
                }
                // Mark result explicitly as aggregated topic result
                let mut meta_obj = match doc.metadata.take() {
                    Some(serde_json::Value::Object(map)) => map,
                    Some(other) => {
                        let mut map = serde_json::Map::new();
                        map.insert("raw_metadata".to_string(), other);
                        map
                    }
                    None => serde_json::Map::new(),
                };
                meta_obj.insert(
                    "aggregated_topic_result".to_string(),
                    serde_json::Value::Bool(true),
                );
                meta_obj.insert(
                    "community_id".to_string(),
                    serde_json::Value::Number(comm.community_id.into()),
                );
                doc.metadata = Some(serde_json::Value::Object(meta_obj));

                final_results.push(doc);
            }
            if final_results.len() >= max_nodes {
                break;
            }
        }

        Ok(final_results)
    }
}
