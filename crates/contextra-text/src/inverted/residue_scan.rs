// FILE-CONTEXT
// ZWECK: Schichteigener rein lesender Residue-Scanner für InvertedIndex (Text).
// INVARIANTEN: Safe Rust, single error root, pure read path.

use contextra_ports::StorageEngine;
use contextra_types::{DocId, Result};
use std::sync::atomic::Ordering;

/// Structural residue evidence collected during InvertedIndex text scan.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ResTextEvidence {
    pub terms_checked: usize,
    pub postings_found: usize,
    pub positions_found: usize,
    pub dict_entries_remaining_df0: usize,
    pub doc_count_before_after_consistent: bool,
    pub derived_stats_recomputed: bool,
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
            if let Ok(meta) = bincode::deserialize::<super::types::TextIndexMetadata>(&meta_bytes)
            {
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
