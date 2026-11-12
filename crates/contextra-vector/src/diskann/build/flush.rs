// FILE-CONTEXT
// ZWECK: Inkrementelles Pruning und Index-Erweiterung (WP-VEC-07).
// INVARIANTEN: RobustPrune mit Alpha-Kriterium; NAN/INF-Filterung.
// STAND: 2026-11-11

use super::super::format::DiskAnnHeader;
use super::super::types::DiskAnnIndex;
use crate::distance::compute_distance_trusted;
use crate::error::{Result, VectorError};
use crate::types::DocId;

impl DiskAnnIndex {
    pub(crate) fn prune_in_memory(
        &self,
        candidates: &mut [SearchCandidate],
        vectors: &[Vec<f32>],
        max_degree: usize,
        alpha: f32,
    ) -> Result<Vec<u32>> {
        if candidates.is_empty() {
            return Ok(Vec::new());
        }
        candidates.sort_by(|a, b| a.distance.total_cmp(&b.distance));

        let mut pruned = Vec::with_capacity(max_degree);
        for cand in candidates.iter() {
            if pruned.len() >= max_degree {
                break;
            }

            let mut keep = true;
            for &p_idx in &pruned {
                let dist_p_cand = compute_distance_trusted(
                    &vectors[cand.index as usize],
                    &vectors[p_idx as usize],
                    self.inner.config.distance_metric,
                )?;
                if !dist_p_cand.is_finite() || !cand.distance.is_finite() {
                    tracing::warn!(
                        "DiskANN robust-prune: non-finite distance encountered, treating candidate as non-prunable (fail-open, candidate retained)"
                    );
                    continue;
                }
                if alpha * dist_p_cand < cand.distance {
                    keep = false;
                    break;
                }
            }

            if keep {
                pruned.push(cand.index);
            }
        }
        Ok(pruned)
    }

    /// Synchronous blocking helper to append a tombstone record to `tombstone.wal`.
    ///
    /// Aufrufer wählt spawn_blocking.
    pub(crate) fn prune_streaming(
        &self,
        cand_idx: u32,
        candidates: &mut [SearchCandidate],
        existing_count: usize,
        new_vecs: &[(DocId, Vec<f32>)],
        max_degree: usize,
        alpha: f32,
    ) -> Result<Vec<u32>> {
        if candidates.is_empty() {
            return Ok(Vec::new());
        }
        candidates.sort_by(|a, b| a.distance.total_cmp(&b.distance));

        let mut pruned = Vec::with_capacity(max_degree);
        let mut pruned_vecs: Vec<Vec<f32>> = Vec::with_capacity(max_degree);
        let _cand_v = self.get_vec_mixed(cand_idx, existing_count, new_vecs)?;

        for cand in candidates.iter() {
            if pruned.len() >= max_degree {
                break;
            }

            let cand_node_v = self.get_vec_mixed(cand.index, existing_count, new_vecs)?;
            let mut keep = true;

            for p_v in &pruned_vecs {
                let dist_p_cand =
                    compute_distance_trusted(&cand_node_v, p_v, self.inner.config.distance_metric)?;
                if !dist_p_cand.is_finite() || !cand.distance.is_finite() {
                    tracing::warn!(
                        "DiskANN robust-prune: non-finite distance encountered, treating candidate as non-prunable (fail-open, candidate retained)"
                    );
                    continue;
                }
                if alpha * dist_p_cand < cand.distance {
                    keep = false;
                    break;
                }
            }

            if keep {
                pruned.push(cand.index);
                pruned_vecs.push(cand_node_v);
            }
        }
        Ok(pruned)
    }

    /// Inkrementeller Vamana-Insert für neue Vektoren (Streaming DiskANN; arXiv:2602.21514 §4).
    ///
    /// Phase 1: Lade bestehende Graph-Topologie read-only aus Mmap via `load_node`.
    /// Phase 2: Für jeden neuen Vektor: Greedy Beam-Search über den aktuellen Graphen.
    /// Phase 3: RNG-Pruning (α = 1.2) zur Auswahl von maximal `max_degree` Nachbarn.
    /// Phase 4: Rückwärts-Kanten hinzufügen und ggf. Nachbarschaft überschrittener Nachbarn re-prunen.
    /// Phase 5: Geänderte Graph-Struktur und neue Knoten atomar auf Disk schreiben.
    ///
    /// AI-TAG[RESOLVED] Echte inkrementelle Streaming-DiskANN Implementierung mit Beam-Search, RNG-Pruning und Rückwärts-Kanten-Kompression. (TS:2026-09-07T06:15:00Z) (SESSION: f04imm01)
    pub(crate) fn write_incremental_to_file_sync(
        &self,
        tmp_path: &std::path::Path,
        new_vecs: &[(DocId, Vec<f32>)],
    ) -> Result<()> {
        if new_vecs.is_empty() {
            return Ok(());
        }

        let existing_count = self.len_sync();
        if existing_count == 0 {
            let mut all_vecs = Vec::with_capacity(new_vecs.len());
            let mut all_ids = Vec::with_capacity(new_vecs.len());
            for (id, vec) in new_vecs {
                all_vecs.push(vec.clone());
                all_ids.push(*id);
            }
            return self.build_to_path_sync(tmp_path, &all_vecs, &all_ids);
        }

        let total_nodes = existing_count + new_vecs.len();
        let mut graph: Vec<Vec<u32>> = vec![vec![]; total_nodes];

        // Phase 1: Adjazenzlisten bestehender Knoten aus Mmap laden
        #[allow(clippy::needless_range_loop)]
        for i in 0..existing_count {
            let node = self.load_node(i as u32)?;
            graph[i] = node.neighbors;
        }

        let mut all_ids = self.inner.doc_ids.read().clone();
        for (id, _) in new_vecs {
            all_ids.push(*id);
        }

        let entry_point = self.inner.header.read().map(|h| h.entry_point).unwrap_or(0);
        let alpha = 1.2f32;

        // Phase 2, 3 & 4: Inkrementelles Einfügen jedes neuen Vektors
        for (j, (_id, vec)) in new_vecs.iter().enumerate() {
            let new_node_idx = (existing_count + j) as u32;

            // Phase 2: Beam-Search über den bisherigen Graphen (0..new_node_idx)
            let mut candidates = self.search_streaming(
                vec,
                &graph,
                existing_count,
                new_vecs,
                entry_point,
                self.inner.config.beam_width,
            )?;

            // Phase 3: RNG-Pruning
            let pruned = self.prune_streaming(
                new_node_idx,
                &mut candidates,
                existing_count,
                new_vecs,
                self.inner.config.max_degree,
                alpha,
            )?;

            // Phase 4: Rückwärts-Kanten & Re-Pruning bei Grad-Überschreitung
            for &neighbor in &pruned {
                let neighbor_idx = neighbor as usize;
                if !graph[neighbor_idx].contains(&new_node_idx) {
                    graph[neighbor_idx].push(new_node_idx);
                    if graph[neighbor_idx].len() > self.inner.config.max_degree {
                        let nbr_v = self.get_vec_mixed(neighbor, existing_count, new_vecs)?;
                        let mut cand_vec: Vec<SearchCandidate> =
                            Vec::with_capacity(graph[neighbor_idx].len());
                        for &idx in &graph[neighbor_idx] {
                            let idx_v = self.get_vec_mixed(idx, existing_count, new_vecs)?;
                            let dist = compute_distance_trusted(
                                &nbr_v,
                                &idx_v,
                                self.inner.config.distance_metric,
                            )?;
                            cand_vec.push(SearchCandidate {
                                index: idx,
                                distance: dist,
                            });
                        }
                        graph[neighbor_idx] = self.prune_streaming(
                            neighbor,
                            &mut cand_vec,
                            existing_count,
                            new_vecs,
                            self.inner.config.max_degree,
                            alpha,
                        )?;
                    }
                }
            }
            graph[new_node_idx as usize] = pruned;
        }

        // Phase 5: Vektoren für alle Knoten zusammenstellen und in tmp_path serialisieren
        let (mut all_vecs, _) = self.load_all_vectors_from_mmap()?;
        for (_, vec) in new_vecs {
            all_vecs.push(vec.clone());
        }

        self.write_to_path_sync(tmp_path, &graph, &all_vecs, &all_ids)
    }
}
