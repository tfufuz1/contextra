//! Aggregation Phase (Consolidation Stage 3) — Spec §21.4
//!
//! Implementiert Ring-3-Consolidation:
//! - Type-Denoising (self-supervised)
//! - Deterministisches Diagonal-GMM Clustering
//! - LLM-Synthese von Alpha-Knoten mit P12-Kostenschutz
//! - Super-Hyperkanten-Entwürfe via `SuperEdgeSink`-Trait

use crate::memory_consolidation::{CommunityStabilityTracker, ConsolidationPhaseResult};
use ahash::{AHashMap, AHashSet};
use contextra_graph::HyperEdgeId;
use contextra_ports::LlmTextGenerator;
use contextra_types::{ContextraError, EntityId, Result};

/// Konfiguration für die Aggregation-Phase.
#[derive(Debug, Clone)]
pub struct AggregationConfig {
    /// Maximal zulässiger Speicherverbrauch für Compaction/Aggregation in MB (Default: 512).
    pub max_compaction_peak_memory_mb: usize,
    /// Schwelle λ für die Superkanten-Erstellung (Default: 0.3).
    pub clustering_tau_threshold: f32,
    /// Deterministischer Seed für GMM SplitMix64 PRNG (Default: 42).
    pub gmm_deterministic_seed: u64,
    /// Maximale Anzahl von LLM-Synthese-Aufrufen pro Zylkus (Default: 10, P12-Kostenschutz).
    pub max_llm_calls_per_cycle: usize,
    /// Maximale Clusteranzahl K (Default: 16).
    pub max_clusters: usize,
    /// Mindestclustergröße (Default: 2).
    pub min_cluster_size: usize,
    /// Maximale Anzahl von EM-Iterationen im GMM (Default: 50).
    pub em_max_iterations: u32,
    /// Konvergenztoleranz für EM Log-Likelihood-Änderung (Default: 1e-4).
    pub em_tolerance: f32,
    /// Mindest-Prädikat-Typ-Kompatibilität für Denoising (Default: 0.1).
    pub min_type_compat_score: f32,
    /// Erforderliche aufeinanderfolgende Zyklen für Community-Stabilität (Default: 3).
    pub stability_cycles_required: u32,
}

impl Default for AggregationConfig {
    fn default() -> Self {
        Self {
            max_compaction_peak_memory_mb: 512,
            clustering_tau_threshold: 0.3,
            gmm_deterministic_seed: 42,
            max_llm_calls_per_cycle: 10,
            max_clusters: 16,
            min_cluster_size: 2,
            em_max_iterations: 50,
            em_tolerance: 1e-4,
            min_type_compat_score: 0.1,
            stability_cycles_required: 3,
        }
    }
}

impl AggregationConfig {
    /// Validiert die Konfigurationseinstellungen.
    pub fn validate(&self) -> Result<()> {
        if self.max_compaction_peak_memory_mb == 0 {
            return Err(ContextraError::InvalidInput(
                "max_compaction_peak_memory_mb must be > 0".into(),
            ));
        }
        if !self.clustering_tau_threshold.is_finite()
            || self.clustering_tau_threshold < 0.0
            || self.clustering_tau_threshold > 1.0
        {
            return Err(ContextraError::InvalidInput(
                "clustering_tau_threshold must be finite and in [0.0, 1.0]".into(),
            ));
        }
        if self.max_clusters == 0 {
            return Err(ContextraError::InvalidInput(
                "max_clusters must be > 0".into(),
            ));
        }
        if self.min_cluster_size == 0 {
            return Err(ContextraError::InvalidInput(
                "min_cluster_size must be > 0".into(),
            ));
        }
        if self.em_max_iterations == 0 {
            return Err(ContextraError::InvalidInput(
                "em_max_iterations must be > 0".into(),
            ));
        }
        if !self.em_tolerance.is_finite() || self.em_tolerance <= 0.0 {
            return Err(ContextraError::InvalidInput(
                "em_tolerance must be finite and > 0.0".into(),
            ));
        }
        if !self.min_type_compat_score.is_finite()
            || self.min_type_compat_score < 0.0
            || self.min_type_compat_score > 1.0
        {
            return Err(ContextraError::InvalidInput(
                "min_type_compat_score must be finite and in [0.0, 1.0]".into(),
            ));
        }
        Ok(())
    }
}

/// Ergebnis der Aggregation-Phase.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AggregationPhaseResult {
    /// Anzahl tombstonierter Kanten wegen Inkompatibilität.
    pub raw_edges_tombstoned: usize,
    /// Anzahl erzeugter abstrakter Super-Hyperkanten.
    pub abstract_hyperedges_created: usize,
    /// Geschätzter Maximalverbrauch an Speicher in MB.
    pub peak_memory_used_mb: usize,
    /// Gesamtzahl der geschriebenen Child-Edge-IDs in Super-Hyperkanten.
    pub child_edge_ids_written: usize,
}

/// Gesamtergebnis der Konsolidierungspipeline (Spec §21.4).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConsolidationPipelineResult {
    /// Ergebnis des Structural Pass.
    pub structural: ConsolidationPhaseResult,
    /// Ergebnis des Synthesis Pass.
    pub synthesis: crate::memory_consolidation::SynthesisPhaseResult,
    /// Ergebnis der Aggregation Phase.
    pub aggregation: Option<AggregationPhaseResult>,
}

/// Eingabe-Knoten für die Aggregationsphase.
#[derive(Debug, Clone, PartialEq)]
pub struct AggregationNode {
    /// Eindeutige EntityId des Knotens.
    pub entity: EntityId,
    /// Merkmalsvektor/Embedding.
    pub embedding: Vec<f32>,
    /// Typ-ID des Entitätstyps.
    pub type_id: u32,
}

/// Eingabe-Kante für die Aggregationsphase.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AggregationEdge {
    /// Eindeutige ID der Hyperkante.
    pub id: HyperEdgeId,
    /// Prädikatstyp der Kante.
    pub predicate_type: u32,
    /// Beteiligte Entitäten der Kante.
    pub participants: Vec<EntityId>,
}

/// Entwurf einer abstrakten Super-Hyperkante.
#[derive(Debug, Clone, PartialEq)]
pub struct SuperEdgeDraft {
    /// Indizes der beiden beteiligten Cluster `(j, k)`.
    pub cluster_pair: (usize, usize),
    /// Alle beteiligten Entitäten aus den untergeordneten Kanten.
    pub participants: Vec<EntityId>,
    /// IDs der untergeordneten Kanten, die durch diese Superkante gebündelt werden.
    pub child_edge_ids: Vec<HyperEdgeId>,
    /// Gewicht / Dichte λ_jk der Verknüpfung.
    pub weight: f32,
}

/// Ein synthetisierter Alpha-Knoten (abstraktes Community-Konzept).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AlphaNode {
    /// Cluster-Index.
    pub cluster_id: usize,
    /// Mitglieds-Entitäten des Clusters.
    pub members: Vec<EntityId>,
    /// LLM-generierte Zusammenfassung/Synthese.
    pub summary: String,
}

/// Trait zur Abstraktion des Speicherziels für Superkanten und Tombstones.
pub trait SuperEdgeSink {
    /// Markiert eine Kante als tombstoned.
    fn tombstone_edge(&mut self, id: HyperEdgeId) -> Result<()>;
    /// Schreibt eine neue Super-Hyperkante und gibt deren zugewiesene ID zurück.
    fn write_super_edge(&mut self, draft: SuperEdgeDraft) -> Result<HyperEdgeId>;
    /// Atomarer Commit aller Änderungen.
    fn commit(&mut self) -> Result<()>;
}

/// Prüft, ob der geschätzte Compaction-Speicherbedarf das konfigurierte Budget überschreitet.
pub fn check_compaction_budget(peak_bytes: usize, cfg: &AggregationConfig) -> Result<()> {
    let used_mb = peak_bytes.div_ceil(1024 * 1024);
    if used_mb > cfg.max_compaction_peak_memory_mb {
        return Err(ContextraError::MemoryBudgetExceeded {
            used_mb: used_mb as u64,
            limit_mb: cfg.max_compaction_peak_memory_mb as u64,
        });
    }
    Ok(())
}

/// Deterministischer SplitMix64 PRNG.
struct SplitMix64 {
    state: u64,
}

impl SplitMix64 {
    fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9e3779b97f4a7c15);
        let mut z = self.state;
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d049bb133111eb);
        z ^ (z >> 31)
    }

    fn next_f64(&mut self) -> f64 {
        (self.next_u64() >> 11) as f64 / (1u64 << 53) as f64
    }
}

/// Berechnet einen deterministischen Hash über eine Liste von `EntityId`s.
///
/// Im Gegensatz zu `compute_community_hash(&[DocId])` verwendet diese Funktion
/// `EntityId`s, da in Stage 3 Entitäts-Knoten verarbeitet werden.
pub fn compute_entity_community_hash(members: &[EntityId]) -> u64 {
    let mut sorted_ids: Vec<u64> = members.iter().map(|e| e.inner()).collect();
    sorted_ids.sort_unstable();
    let mut bytes = Vec::with_capacity(sorted_ids.len() * 8);
    for id in sorted_ids {
        bytes.extend_from_slice(&id.to_le_bytes());
    }
    let hash = blake3::hash(&bytes);
    let mut hash_bytes = [0u8; 8];
    hash_bytes.copy_from_slice(&hash.as_bytes()[0..8]);
    u64::from_le_bytes(hash_bytes)
}

#[allow(clippy::type_complexity)]
fn validate_and_sort_aggregation_inputs(
    nodes: &[AggregationNode],
    edges: &[AggregationEdge],
) -> Result<(
    Vec<AggregationNode>,
    Vec<AggregationEdge>,
    usize,
    AHashMap<EntityId, u32>,
)> {
    let mut sorted_nodes = nodes.to_vec();
    sorted_nodes.sort_unstable_by_key(|n| n.entity.inner());

    let mut sorted_edges = edges.to_vec();
    sorted_edges.sort_unstable_by_key(|e| e.id.0);

    let dim = sorted_nodes.first().map(|n| n.embedding.len()).unwrap_or(0);
    if dim == 0 {
        return Err(ContextraError::InvalidInput(
            "Node embedding dimension must be > 0".into(),
        ));
    }

    let node_type_map: AHashMap<EntityId, u32> =
        sorted_nodes.iter().map(|n| (n.entity, n.type_id)).collect();

    for node in &sorted_nodes {
        if node.embedding.len() != dim {
            return Err(ContextraError::InvalidInput(
                "Inconsistent embedding dimensions across nodes".into(),
            ));
        }
        for &val in &node.embedding {
            if !val.is_finite() {
                return Err(ContextraError::InvalidInput(
                    "NaN or Infinity detected in node embedding".into(),
                ));
            }
        }
    }

    Ok((sorted_nodes, sorted_edges, dim, node_type_map))
}

fn perform_type_denoising(
    sorted_edges: Vec<AggregationEdge>,
    node_type_map: &AHashMap<EntityId, u32>,
    cfg: &AggregationConfig,
    sink: &mut dyn SuperEdgeSink,
) -> Result<(Vec<AggregationEdge>, usize)> {
    let mut pred_type_counts: AHashMap<u32, AHashMap<u32, usize>> = AHashMap::new();
    let mut pred_total_counts: AHashMap<u32, usize> = AHashMap::new();

    for edge in &sorted_edges {
        let entry_map = pred_type_counts.entry(edge.predicate_type).or_default();
        let total = pred_total_counts.entry(edge.predicate_type).or_default();

        for part in &edge.participants {
            if let Some(&t_id) = node_type_map.get(part) {
                *entry_map.entry(t_id).or_default() += 1;
                *total += 1;
            }
        }
    }

    let mut active_edges = Vec::new();
    let mut raw_edges_tombstoned = 0;

    for edge in sorted_edges {
        let mut min_compat = 1.0f32;
        let total = pred_total_counts
            .get(&edge.predicate_type)
            .copied()
            .unwrap_or(0);
        if total > 0 {
            let type_counts = pred_type_counts.get(&edge.predicate_type);
            for part in &edge.participants {
                if let Some(&t_id) = node_type_map.get(part) {
                    let count = type_counts.and_then(|m| m.get(&t_id)).copied().unwrap_or(0);
                    let compat = (count as f32) / (total as f32);
                    if compat < min_compat {
                        min_compat = compat;
                    }
                }
            }
        }

        if min_compat < cfg.min_type_compat_score {
            sink.tombstone_edge(edge.id)?;
            raw_edges_tombstoned += 1;
        } else {
            active_edges.push(edge);
        }
    }

    Ok((active_edges, raw_edges_tombstoned))
}

fn build_clusters_from_assignments(
    sorted_nodes: &[AggregationNode],
    assignments: &[usize],
    min_cluster_size: usize,
) -> (Vec<Vec<EntityId>>, AHashMap<EntityId, usize>) {
    let mut raw_clusters: AHashMap<usize, Vec<EntityId>> = AHashMap::new();
    for (i, node) in sorted_nodes.iter().enumerate() {
        if let Some(&c_idx) = assignments.get(i) {
            raw_clusters.entry(c_idx).or_default().push(node.entity);
        }
    }

    let mut cluster_list: Vec<(EntityId, Vec<EntityId>)> = raw_clusters
        .into_values()
        .filter_map(|mut members| {
            members.sort_unstable_by_key(|e| e.inner());
            members.first().copied().map(|min_id| (min_id, members))
        })
        .collect();

    cluster_list.sort_unstable_by_key(|(min_id, _)| min_id.inner());

    let final_clusters: Vec<Vec<EntityId>> = cluster_list
        .into_iter()
        .map(|(_, members)| members)
        .filter(|members| members.len() >= min_cluster_size)
        .collect();

    let mut entity_to_cluster: AHashMap<EntityId, usize> = AHashMap::new();
    for (c_idx, members) in final_clusters.iter().enumerate() {
        for member in members {
            entity_to_cluster.insert(*member, c_idx);
        }
    }

    (final_clusters, entity_to_cluster)
}

async fn synthesize_alpha_nodes(
    final_clusters: &[Vec<EntityId>],
    cfg: &AggregationConfig,
    llm: &dyn LlmTextGenerator,
    tracker: &mut CommunityStabilityTracker,
) -> Vec<AlphaNode> {
    let mut alpha_nodes = Vec::new();
    let mut llm_calls_made = 0;

    for (c_idx, members) in final_clusters.iter().enumerate() {
        let comm_hash = compute_entity_community_hash(members);
        let cycles = tracker.observe(comm_hash);

        if cycles >= cfg.stability_cycles_required {
            if llm_calls_made < cfg.max_llm_calls_per_cycle {
                let prompt = format!(
                    "Synthesize the abstract memory alpha-node concept for cluster {} with entities: {:?}",
                    c_idx,
                    members.iter().map(|e| e.inner()).collect::<Vec<_>>()
                );

                match llm.generate(&prompt).await {
                    Ok(summary) => {
                        alpha_nodes.push(AlphaNode {
                            cluster_id: c_idx,
                            members: members.clone(),
                            summary,
                        });
                    }
                    Err(e) => {
                        tracing::warn!(
                            cluster_id = c_idx,
                            error = %e,
                            "Aggregation LLM synthesis failed for cluster; skipping cluster"
                        );
                    }
                }
                llm_calls_made += 1;
            } else {
                tracing::info!(
                    cluster_id = c_idx,
                    max_calls = cfg.max_llm_calls_per_cycle,
                    "LLM calls budget reached for current cycle; deferring cluster synthesis"
                );
            }
        }
    }

    alpha_nodes
}

fn collect_child_participants(
    active_edges: &[AggregationEdge],
    child_edge_ids: &[HyperEdgeId],
) -> Vec<EntityId> {
    let mut participant_set = AHashSet::new();
    for edge in active_edges {
        if child_edge_ids.contains(&edge.id) {
            for part in &edge.participants {
                participant_set.insert(*part);
            }
        }
    }
    let mut participants: Vec<EntityId> = participant_set.into_iter().collect();
    participants.sort_unstable_by_key(|e| e.inner());
    participants
}

#[allow(clippy::indexing_slicing)]
fn build_and_write_super_edges(
    active_edges: &[AggregationEdge],
    num_final_clusters: usize,
    entity_to_cluster: &AHashMap<EntityId, usize>,
    cfg: &AggregationConfig,
    sink: &mut dyn SuperEdgeSink,
) -> Result<(usize, usize)> {
    let mut abstract_hyperedges_created = 0;
    let mut child_edge_ids_written = 0;

    let mut cluster_touching_edges: Vec<AHashSet<HyperEdgeId>> =
        vec![AHashSet::new(); num_final_clusters];
    let mut pair_spanning_edges: AHashMap<(usize, usize), Vec<HyperEdgeId>> = AHashMap::new();

    for edge in active_edges {
        let mut touched_clusters: AHashSet<usize> = AHashSet::new();
        for part in &edge.participants {
            if let Some(&c_idx) = entity_to_cluster.get(part) {
                touched_clusters.insert(c_idx);
            }
        }

        for &c_idx in &touched_clusters {
            if let Some(set) = cluster_touching_edges.get_mut(c_idx) {
                set.insert(edge.id);
            }
        }

        let mut sorted_touched: Vec<usize> = touched_clusters.into_iter().collect();
        sorted_touched.sort_unstable();

        let len = sorted_touched.len();
        for i in 0..len {
            for j in (i + 1)..len {
                if let (Some(&c1), Some(&c2)) = (sorted_touched.get(i), sorted_touched.get(j)) {
                    let pair = (c1, c2);
                    pair_spanning_edges.entry(pair).or_default().push(edge.id);
                }
            }
        }
    }

    let mut pair_keys: Vec<(usize, usize)> = pair_spanning_edges.keys().copied().collect();
    pair_keys.sort_unstable();

    for (j, k) in pair_keys {
        if let Some(spanning) = pair_spanning_edges.get(&(j, k)) {
            let e_jk = spanning.len() as f32;
            let e_j = cluster_touching_edges.get(j).map(|s| s.len()).unwrap_or(0) as f32;
            let e_k = cluster_touching_edges.get(k).map(|s| s.len()).unwrap_or(0) as f32;

            let min_ej_ek = e_j.min(e_k);
            let lambda = if min_ej_ek > 0.0 {
                e_jk / min_ej_ek
            } else {
                0.0
            };

            if lambda > cfg.clustering_tau_threshold {
                let mut child_edge_ids = spanning.clone();
                child_edge_ids.sort_unstable_by_key(|id| id.0);
                child_edge_ids.dedup();

                let participants = collect_child_participants(active_edges, &child_edge_ids);

                let draft = SuperEdgeDraft {
                    cluster_pair: (j, k),
                    participants,
                    child_edge_ids: child_edge_ids.clone(),
                    weight: lambda,
                };

                sink.write_super_edge(draft)?;
                child_edge_ids_written += child_edge_ids.len();
                abstract_hyperedges_created += 1;
            }
        }
    }

    Ok((abstract_hyperedges_created, child_edge_ids_written))
}

/// Führt die Aggregation (Consolidation Stage 3) aus.
pub async fn run_aggregation_pass(
    nodes: &[AggregationNode],
    edges: &[AggregationEdge],
    cfg: &AggregationConfig,
    llm: &dyn LlmTextGenerator,
    tracker: &mut CommunityStabilityTracker,
    sink: &mut dyn SuperEdgeSink,
) -> Result<(AggregationPhaseResult, Vec<AlphaNode>)> {
    cfg.validate()?;

    if nodes.is_empty() {
        sink.commit()?;
        return Ok((
            AggregationPhaseResult {
                raw_edges_tombstoned: 0,
                abstract_hyperedges_created: 0,
                peak_memory_used_mb: 0,
                child_edge_ids_written: 0,
            },
            Vec::new(),
        ));
    }

    let (sorted_nodes, sorted_edges, dim, node_type_map) =
        validate_and_sort_aggregation_inputs(nodes, edges)?;

    let (active_edges, raw_edges_tombstoned) =
        perform_type_denoising(sorted_edges, &node_type_map, cfg, sink)?;

    let n = sorted_nodes.len();
    let k_num = cfg.max_clusters.min(n);
    let assignments = run_deterministic_diagonal_gmm(
        &sorted_nodes,
        k_num,
        cfg.gmm_deterministic_seed,
        cfg.em_max_iterations,
        cfg.em_tolerance,
    );

    let (final_clusters, entity_to_cluster) =
        build_clusters_from_assignments(&sorted_nodes, &assignments, cfg.min_cluster_size);

    let alpha_nodes = synthesize_alpha_nodes(&final_clusters, cfg, llm, tracker).await;

    let (abstract_hyperedges_created, child_edge_ids_written) = build_and_write_super_edges(
        &active_edges,
        final_clusters.len(),
        &entity_to_cluster,
        cfg,
        sink,
    )?;

    sink.commit()?;

    let node_mem = n * dim * 4 + n * 32;
    let edge_mem = edges.len() * 64;
    let total_bytes = node_mem + edge_mem;
    let peak_memory_used_mb = total_bytes.div_ceil(1024 * 1024);

    Ok((
        AggregationPhaseResult {
            raw_edges_tombstoned,
            abstract_hyperedges_created,
            peak_memory_used_mb,
            child_edge_ids_written,
        },
        alpha_nodes,
    ))
}

#[allow(
    clippy::indexing_slicing,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_precision_loss
)]
fn initialize_gmm_centroids(
    nodes: &[AggregationNode],
    k: usize,
    rng: &mut SplitMix64,
) -> Vec<Vec<f32>> {
    let n = nodes.len();
    let mut centroids: Vec<Vec<f32>> = Vec::with_capacity(k);
    let first_idx = (rng.next_u64() % (n as u64)) as usize;
    centroids.push(nodes[first_idx].embedding.clone());

    for _ in 1..k {
        let mut dists = vec![f64::MAX; n];
        let mut sum_dist = 0.0f64;

        for (i, node) in nodes.iter().enumerate() {
            let mut min_d = f64::MAX;
            for centroid in &centroids {
                let mut d = 0.0f64;
                for (x, c) in node.embedding.iter().zip(centroid.iter()) {
                    let diff = (*x - *c) as f64;
                    d += diff * diff;
                }
                if d < min_d {
                    min_d = d;
                }
            }
            dists[i] = min_d;
            sum_dist += min_d;
        }

        if sum_dist <= 1e-12 {
            centroids.push(nodes[centroids.len() % n].embedding.clone());
        } else {
            let target = rng.next_f64() * sum_dist;
            let mut accum = 0.0f64;
            let mut chosen = 0;
            for (i, &d) in dists.iter().enumerate() {
                accum += d;
                if accum >= target {
                    chosen = i;
                    break;
                }
            }
            centroids.push(nodes[chosen].embedding.clone());
        }
    }

    centroids
}

#[allow(
    clippy::indexing_slicing,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_precision_loss
)]
fn gmm_e_step(
    nodes: &[AggregationNode],
    k: usize,
    dim: usize,
    pi: &[f64],
    mu: &[Vec<f64>],
    sigma_sq: &[Vec<f64>],
) -> (Vec<Vec<f64>>, f64) {
    let n = nodes.len();
    let mut log_resp = vec![vec![0.0f64; k]; n];
    let mut current_log_likelihood = 0.0f64;

    for (i, node) in nodes.iter().enumerate() {
        let mut max_log_val = -f64::INFINITY;
        for j in 0..k {
            let mut log_p = 0.0f64;
            for m in 0..dim {
                let diff = (node.embedding[m] as f64) - mu[j][m];
                log_p += -(0.5 * (2.0 * std::f64::consts::PI * sigma_sq[j][m]).ln())
                    - ((diff * diff) / (2.0 * sigma_sq[j][m]));
            }
            let val = pi[j].ln() + log_p;
            log_resp[i][j] = val;
            if val > max_log_val {
                max_log_val = val;
            }
        }

        let mut sum_exp = 0.0f64;
        for j in 0..k {
            sum_exp += (log_resp[i][j] - max_log_val).exp();
        }

        let log_sum = max_log_val + sum_exp.ln();
        current_log_likelihood += log_sum;

        for j in 0..k {
            log_resp[i][j] = (log_resp[i][j] - log_sum).exp();
        }
    }

    (log_resp, current_log_likelihood)
}

#[allow(
    clippy::indexing_slicing,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_precision_loss
)]
fn gmm_m_step(
    nodes: &[AggregationNode],
    k: usize,
    dim: usize,
    log_resp: &[Vec<f64>],
    pi: &mut [f64],
    mu: &mut [Vec<f64>],
    sigma_sq: &mut [Vec<f64>],
) {
    let n = nodes.len();
    for j in 0..k {
        let n_k: f64 = log_resp.iter().map(|r| r[j]).sum();
        let n_k_safe = n_k.max(1e-8);
        pi[j] = n_k_safe / (n as f64);

        for m in 0..dim {
            let mut mu_jm = 0.0f64;
            for (i, node) in nodes.iter().enumerate() {
                mu_jm += log_resp[i][j] * (node.embedding[m] as f64);
            }
            mu[j][m] = mu_jm / n_k_safe;

            let mut var_jm = 0.0f64;
            for (i, node) in nodes.iter().enumerate() {
                let diff = (node.embedding[m] as f64) - mu[j][m];
                var_jm += log_resp[i][j] * diff * diff;
            }
            sigma_sq[j][m] = (var_jm / n_k_safe).max(1e-6);
        }
    }
}

#[allow(
    clippy::indexing_slicing,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_precision_loss
)]
fn run_gmm_em_loop(
    nodes: &[AggregationNode],
    k: usize,
    dim: usize,
    centroids: &[Vec<f32>],
    max_iterations: u32,
    tolerance: f32,
) -> (Vec<f64>, Vec<Vec<f64>>, Vec<Vec<f64>>) {
    let mut pi = vec![1.0f64 / (k as f64); k];
    let mut mu = vec![vec![0.0f64; dim]; k];
    for j in 0..k {
        for m in 0..dim {
            mu[j][m] = centroids[j][m] as f64;
        }
    }
    let mut sigma_sq = vec![vec![1.0f64; dim]; k];
    let mut prev_log_likelihood = -f64::INFINITY;

    for _iter in 0..max_iterations {
        let (log_resp, current_log_likelihood) = gmm_e_step(nodes, k, dim, &pi, &mu, &sigma_sq);
        gmm_m_step(nodes, k, dim, &log_resp, &mut pi, &mut mu, &mut sigma_sq);

        let diff_ll = (current_log_likelihood - prev_log_likelihood).abs();
        if diff_ll < (tolerance as f64) {
            break;
        }
        prev_log_likelihood = current_log_likelihood;
    }

    (pi, mu, sigma_sq)
}

#[allow(
    clippy::indexing_slicing,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_precision_loss
)]
fn assign_clusters_from_gmm(
    nodes: &[AggregationNode],
    k: usize,
    dim: usize,
    pi: &[f64],
    mu: &[Vec<f64>],
    sigma_sq: &[Vec<f64>],
) -> Vec<usize> {
    let n = nodes.len();
    let mut assignments = vec![0; n];
    for (i, node) in nodes.iter().enumerate() {
        let mut best_k = 0;
        let mut max_log_resp = -f64::INFINITY;

        for j in 0..k {
            let mut log_p = 0.0f64;
            for m in 0..dim {
                let diff = (node.embedding[m] as f64) - mu[j][m];
                log_p += -(0.5 * (2.0 * std::f64::consts::PI * sigma_sq[j][m]).ln())
                    - ((diff * diff) / (2.0 * sigma_sq[j][m]));
            }
            let log_resp = pi[j].ln() + log_p;
            if log_resp > max_log_resp {
                max_log_resp = log_resp;
                best_k = j;
            }
        }
        assignments[i] = best_k;
    }
    assignments
}

/// Deterministischer Diagonal-GMM via SplitMix64 PRNG.
#[allow(
    clippy::indexing_slicing,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_precision_loss
)]
fn run_deterministic_diagonal_gmm(
    nodes: &[AggregationNode],
    k: usize,
    seed: u64,
    max_iterations: u32,
    tolerance: f32,
) -> Vec<usize> {
    let n = nodes.len();
    if n == 0 {
        return Vec::new();
    }
    if k <= 1 || n == 1 {
        return vec![0; n];
    }

    let dim = nodes[0].embedding.len();
    let mut rng = SplitMix64::new(seed);

    let centroids = initialize_gmm_centroids(nodes, k, &mut rng);
    let (pi, mu, sigma_sq) = run_gmm_em_loop(nodes, k, dim, &centroids, max_iterations, tolerance);

    assign_clusters_from_gmm(nodes, k, dim, &pi, &mu, &sigma_sq)
}
