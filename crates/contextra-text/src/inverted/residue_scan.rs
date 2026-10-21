// FILE-CONTEXT
// ZWECK: Schichteigener rein lesender Residue-Scanner für InvertedIndex (Text).
// INVARIANTEN: Safe Rust, single error root, pure read path.

use contextra_ports::StorageEngine;
use contextra_types::{DocId, Result};
use std::sync::atomic::Ordering;

/// Sweep depth level for independent text residue scans.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TextSweepDepth {
    /// Standard scan: checks terms from forward index if available.
    #[default]
    Standard,
    /// Full sweep: scans entire index storage prefixes (plb:, pl:, pos:) independently of caller terms.
    FullSweep,
}

/// Structural residue evidence collected during InvertedIndex text scan.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ResTextEvidence {
    pub terms_checked: usize,
    pub postings_found: usize,
    pub positions_found: usize,
    pub dict_entries_remaining_df0: usize,
    pub doc_count_before_after_consistent: bool,
    pub derived_stats_recomputed: bool,
    pub sweep_depth: TextSweepDepth,
}

fn parse_term_and_doc_suffix(prefix_len: usize, key: &[u8]) -> Option<(&str, &str)> {
    if key.len() <= prefix_len {
        return None;
    }
    let suffix = &key[prefix_len..];
    let pos = suffix.iter().rposition(|&b| b == b':')?;
    let term = std::str::from_utf8(&suffix[..pos]).ok()?;
    let doc_id_str = std::str::from_utf8(&suffix[pos + 1..]).ok()?;
    Some((term, doc_id_str))
}

impl<S: StorageEngine> super::index_struct::InvertedIndex<S> {
    /// Scans the inverted index for residual posting entries, positions, df==0 dictionary terms,
    /// and stats consistency for a given `doc_id` and set of `terms`.
    pub async fn scan_residue(
        &self,
        doc_id: DocId,
        terms: &[impl AsRef<[u8]>],
    ) -> Result<ResTextEvidence> {
        let mut evidence = ResTextEvidence {
            terms_checked: terms.len(),
            sweep_depth: TextSweepDepth::Standard,
            ..Default::default()
        };

        for term_bytes in terms {
            let term_bytes_ref = term_bytes.as_ref();
            let term_str = match std::str::from_utf8(term_bytes_ref) {
                Ok(s) => s,
                Err(_) => continue,
            };

            // 1. Check posting existence in resident index
            let mut found_in_resident = false;
            if let Some(plist) = self.resident_index.get(term_str) {
                if plist.as_slice().iter().any(|p| p.doc_id() == doc_id) {
                    evidence.postings_found += 1;
                    found_in_resident = true;
                }
                if plist.is_empty() {
                    evidence.dict_entries_remaining_df0 += 1;
                }
            }

            // 2. Check posting existence in storage
            if !found_in_resident {
                let pl_doc_key = self.key_with_term_doc(term_str, doc_id);
                if let Some(bytes) = self.storage.get(&pl_doc_key).await? {
                    if !bytes.is_empty() {
                        evidence.postings_found += 1;
                    }
                }

                let plb_key = self.key_batch_posting_list(term_str);
                if let Some(bytes) = self.storage.get(&plb_key).await? {
                    if let Ok(plist) = crate::posting_list::PostingList::decode_compact(&bytes) {
                        if plist.is_empty() {
                            evidence.dict_entries_remaining_df0 += 1;
                        } else if plist.as_slice().iter().any(|p| p.doc_id() == doc_id) {
                            evidence.postings_found += 1;
                        }
                    }
                }
            }

            // 3. Check position records
            let pos_key = self.key_with_id(&format!("pos:{}:", term_str), doc_id.inner());
            if let Some(bytes) = self.storage.get(&pos_key).await? {
                if !bytes.is_empty() {
                    evidence.positions_found += 1;
                }
            }
        }

        // 4. Check document counter / stats consistency
        let docs = self.total_docs.load(Ordering::SeqCst);
        let tokens = self.total_tokens.load(Ordering::SeqCst);
        let cached_avg = self.avg_doc_len_x1000.load(Ordering::SeqCst);

        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let expected_avg = if docs > 0 {
            (tokens as f64 / docs as f64 * 1000.0) as u64
        } else {
            0
        };

        evidence.derived_stats_recomputed = cached_avg == expected_avg;

        // Verify total_docs against metadata key if present
        let meta_key = self.key("meta:stats");
        if let Some(meta_bytes) = self.storage.get(&meta_key).await? {
            if let Ok(meta) = bincode::deserialize::<super::types::TextIndexMetadata>(&meta_bytes) {
                evidence.doc_count_before_after_consistent = meta.total_docs == docs;
            } else {
                evidence.doc_count_before_after_consistent = true;
            }
        } else {
            evidence.doc_count_before_after_consistent = true;
        }

        Ok(evidence)
    }

    /// Scans the inverted index for ANY residual posting entries, positions, df==0 dictionary terms,
    /// and stats consistency for a given `doc_id` independently without relying on caller-provided terms.
    pub async fn scan_residue_independent(
        &self,
        doc_id: DocId,
        depth: TextSweepDepth,
    ) -> Result<ResTextEvidence> {
        let mut evidence = ResTextEvidence {
            sweep_depth: depth,
            ..Default::default()
        };

        // Standard depth: try forward index terms if available
        if depth == TextSweepDepth::Standard {
            let fw_key = self.key_with_id("fw:", doc_id.inner());
            if let Some(fw_bytes) = self.storage.get(&fw_key).await? {
                if let Ok(terms) = bincode::deserialize::<Vec<String>>(&fw_bytes) {
                    let term_refs: Vec<&str> = terms.iter().map(|s| s.as_str()).collect();
                    let mut caller_ev = self.scan_residue(doc_id, &term_refs).await?;
                    caller_ev.sweep_depth = depth;
                    return Ok(caller_ev);
                }
            }
        }

        // FullSweep or missing forward index: independent prefix sweep across all stored terms
        let mut found_terms = std::collections::HashSet::new();

        // 1. Scan plb: batch posting lists
        let plb_prefix = {
            let mut k = Vec::with_capacity(self.prefix.len() + 4);
            k.extend_from_slice(&self.prefix);
            k.extend_from_slice(b"plb:");
            k
        };
        let plb_entries = self.storage.scan_prefix(&plb_prefix).await?;
        for (key, _) in &plb_entries {
            if key.len() > plb_prefix.len() {
                if let Ok(term) = std::str::from_utf8(&key[plb_prefix.len()..]) {
                    found_terms.insert(term.to_string());
                }
            }
        }

        // 2. Scan pl: single posting entries
        let pl_prefix = {
            let mut k = Vec::with_capacity(self.prefix.len() + 3);
            k.extend_from_slice(&self.prefix);
            k.extend_from_slice(b"pl:");
            k
        };
        let pl_entries = self.storage.scan_prefix(&pl_prefix).await?;
        for (key, _) in &pl_entries {
            if let Some((term, _)) = parse_term_and_doc_suffix(pl_prefix.len(), key) {
                found_terms.insert(term.to_string());
            }
        }

        // 3. Scan pos: position entries
        let pos_prefix = {
            let mut k = Vec::with_capacity(self.prefix.len() + 4);
            k.extend_from_slice(&self.prefix);
            k.extend_from_slice(b"pos:");
            k
        };
        let pos_entries = self.storage.scan_prefix(&pos_prefix).await?;
        for (key, _) in &pos_entries {
            if let Some((term, _)) = parse_term_and_doc_suffix(pos_prefix.len(), key) {
                found_terms.insert(term.to_string());
            }
        }

        evidence.terms_checked = found_terms.len();

        for term in &found_terms {
            // Check posting existence in resident index
            let mut found_in_resident = false;
            if let Some(plist) = self.resident_index.get(term) {
                if plist.as_slice().iter().any(|p| p.doc_id() == doc_id) {
                    evidence.postings_found += 1;
                    found_in_resident = true;
                }
                if plist.is_empty() {
                    evidence.dict_entries_remaining_df0 += 1;
                }
            }

            // Check posting existence in storage if not found in resident
            if !found_in_resident {
                let pl_doc_key = self.key_with_term_doc(term, doc_id);
                if let Some(bytes) = self.storage.get(&pl_doc_key).await? {
                    if !bytes.is_empty() {
                        evidence.postings_found += 1;
                    }
                }

                let plb_key = self.key_batch_posting_list(term);
                if let Some(bytes) = self.storage.get(&plb_key).await? {
                    if let Ok(plist) = crate::posting_list::PostingList::decode_compact(&bytes) {
                        if plist.is_empty() {
                            evidence.dict_entries_remaining_df0 += 1;
                        } else if plist.as_slice().iter().any(|p| p.doc_id() == doc_id) {
                            evidence.postings_found += 1;
                        }
                    }
                }
            }

            // Check position records
            let pos_key = self.key_with_id(&format!("pos:{}:", term), doc_id.inner());
            if let Some(bytes) = self.storage.get(&pos_key).await? {
                if !bytes.is_empty() {
                    evidence.positions_found += 1;
                }
            }
        }

        // Check document counter / stats consistency
        let docs = self.total_docs.load(Ordering::SeqCst);
        let tokens = self.total_tokens.load(Ordering::SeqCst);
        let cached_avg = self.avg_doc_len_x1000.load(Ordering::SeqCst);

        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let expected_avg = if docs > 0 {
            (tokens as f64 / docs as f64 * 1000.0) as u64
        } else {
            0
        };

        evidence.derived_stats_recomputed = cached_avg == expected_avg;

        // Verify total_docs against metadata key if present
        let meta_key = self.key("meta:stats");
        if let Some(meta_bytes) = self.storage.get(&meta_key).await? {
            if let Ok(meta) = bincode::deserialize::<super::types::TextIndexMetadata>(&meta_bytes) {
                evidence.doc_count_before_after_consistent = meta.total_docs == docs;
            } else {
                evidence.doc_count_before_after_consistent = true;
            }
        } else {
            evidence.doc_count_before_after_consistent = true;
        }

        Ok(evidence)
    }
}

impl<S: StorageEngine> contextra_ports::LayerScan for super::index_struct::InvertedIndex<S> {
    fn layer_id(&self) -> u16 {
        20
    }

    fn scan(&self, scope: &contextra_ports::PhysicalScope) -> std::result::Result<contextra_ports::LayerObservation, contextra_ports::ScanError> {
        let total_docs = self.total_docs.load(Ordering::SeqCst);
        let total_tokens = self.total_tokens.load(Ordering::SeqCst);
        let item_count = total_docs;

        if item_count == 0 {
            let digest = blake3::hash(b"text_empty").into();
            return Ok(contextra_ports::LayerObservation {
                layer_id: 20,
                clean: true,
                state_digest: digest,
                item_count: 0,
            });
        }

        let mut matches = 0u64;
        let mut hasher = blake3::Hasher::new();
        hasher.update(b"contextra/text/v1");
        hasher.update(&total_docs.to_le_bytes());
        hasher.update(&total_tokens.to_le_bytes());

        self.resident_index.visit_entries(|term, plist| {
            hasher.update(term.as_bytes());
            for p in plist.as_slice() {
                let doc_str = p.doc_id().inner().to_string();
                let key_0 = format!("t:0:{}", doc_str).into_bytes();
                let key_1 = format!("t:1:{}", doc_str).into_bytes();
                if scope.matches_key(&key_0)
                    || scope.matches_key(&key_1)
                    || scope.keys().iter().any(|k| {
                        std::str::from_utf8(k)
                            .map(|s| s.ends_with(&doc_str))
                            .unwrap_or(false)
                    })
                {
                    matches += 1;
                }
            }
        });

        Ok(contextra_ports::LayerObservation {
            layer_id: 20,
            clean: matches == 0,
            state_digest: hasher.finalize().into(),
            item_count,
        })
    }
}
