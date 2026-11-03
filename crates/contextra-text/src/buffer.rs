//! Textpuffer und Purge/Merge-Logik (SPEC 9.7, D-49, D-71, E-L5, TX-09).

use crate::analyzer::{compute_term_tag, AnalyzedDocument, AnalyzerId};
use crate::segment::{SegmentBuilder, SegmentPosting, SegmentReader};
use contextra_crypto::hierarchy::IdxKey;
use contextra_types::{ClusterTag, ContextraError, DocTag, Result, TermTag};
use std::collections::{BTreeMap, HashMap, HashSet};

/// Identifikator eines Dokuments im Textindex (SPEC D-70).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TextDocHandle {
    pub cluster_tag: ClusterTag,
    pub doc_tag: DocTag,
}

/// Eintrag im Textpuffer vor dem Flush in ein Segment.
#[derive(Debug, Clone)]
pub struct TextBufferEntry {
    pub handle: TextDocHandle,
    pub doclen: u32,
    pub binding_envelope: Vec<u8>,
    pub analyzed: AnalyzedDocument,
}

/// Speicherpuffer für neue Dokumente vor der Segmentierung (SPEC 9.7).
#[derive(Default)]
pub struct TextBuffer {
    entries: HashMap<DocTag, TextBufferEntry>,
}

impl TextBuffer {
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
        }
    }

    pub fn insert(
        &mut self,
        handle: TextDocHandle,
        doclen: u32,
        binding_envelope: Vec<u8>,
        analyzed: AnalyzedDocument,
    ) {
        self.entries.insert(
            handle.doc_tag,
            TextBufferEntry {
                handle,
                doclen,
                binding_envelope,
                analyzed,
            },
        );
    }

    pub fn remove(&mut self, doc_tag: &DocTag) -> Option<TextBufferEntry> {
        self.entries.remove(doc_tag)
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Schreibt alle gepufferten Dokumente in ein neues Segment (SPEC F.7).
    pub fn flush_to_segment(
        &mut self,
        k_idx: &IdxKey,
        analyzer_id: AnalyzerId,
        bucket: u8,
        bucket_count: u8,
        positions: bool,
    ) -> Vec<u8> {
        let mut builder = SegmentBuilder::new(
            bucket,
            bucket_count,
            analyzer_id.id(),
            analyzer_id.version(),
            analyzer_id.lang(),
            positions,
        );

        let mut term_postings_map: BTreeMap<TermTag, Vec<SegmentPosting>> = BTreeMap::new();

        let mut sorted_entries: Vec<_> = self.entries.drain().map(|(_, v)| v).collect();
        sorted_entries.sort_by_key(|e| e.handle.doc_tag);

        for entry in sorted_entries {
            let doc_ord = builder.add_document(entry.doclen, entry.binding_envelope);

            let mut tf_and_pos: HashMap<String, (u32, Vec<u32>)> = HashMap::new();
            for tok in entry.analyzed.tokens {
                let entry_data = tf_and_pos.entry(tok.term).or_insert((0, Vec::new()));
                entry_data.0 += 1;
                entry_data.1.push(tok.position);
            }

            for (term_str, (tf, pos_list)) in tf_and_pos {
                let term_tag = compute_term_tag(k_idx, analyzer_id, &term_str);
                term_postings_map
                    .entry(term_tag)
                    .or_default()
                    .push(SegmentPosting {
                        doc_ord,
                        tf,
                        positions: pos_list,
                    });
            }
        }

        for (term_tag, postings) in term_postings_map {
            builder.add_term_postings(term_tag, postings);
        }

        builder.build()
    }
}

/// Ergebnis eines Purge-Laufs (SPEC TX-09).
#[derive(Debug, Clone, Default)]
pub struct TextPurgeOutcome {
    pub segments_rewritten: usize,
    pub bytes_rewritten: usize,
    pub documents_purged: usize,
}

/// Führt das Umschreiben von Segmenten für gelöschte Dokumente aus (E-L5, TX-09).
pub fn purge_segments(
    segments: &[Vec<u8>],
    target_docs: &HashSet<DocTag>,
    valid_deks: &HashSet<DocTag>,
    _k_idx: &IdxKey,
) -> Result<(Vec<Vec<u8>>, TextPurgeOutcome)> {
    let mut rewritten_segments = Vec::new();
    let mut outcome = TextPurgeOutcome::default();

    for segment_bytes in segments {
        let reader = SegmentReader::open(segment_bytes)?;
        let mut contains_target = false;
        let mut surviving_docs = Vec::new();

        for doc_ord in 0..reader.header.doc_count {
            if let Some((doclen, env)) = reader.get_binding(doc_ord)? {
                let is_target = if env.len() >= 32 {
                    if let Some(tag_slice) = env.get(16..32).and_then(|s| s.try_into().ok()) {
                        let doc_tag = DocTag::new(tag_slice);
                        target_docs.contains(&doc_tag) || !valid_deks.contains(&doc_tag)
                    } else {
                        false
                    }
                } else {
                    false
                };

                if is_target {
                    contains_target = true;
                    outcome.documents_purged += 1;
                } else {
                    surviving_docs.push((doc_ord, doclen, env.to_vec()));
                }
            }
        }

        if !contains_target {
            rewritten_segments.push(segment_bytes.clone());
            continue;
        }

        outcome.segments_rewritten += 1;
        outcome.bytes_rewritten += segment_bytes.len();

        if surviving_docs.is_empty() {
            continue;
        }

        let _analyzer_id = AnalyzerId::from_id(reader.header.analyzer_id)?;
        let mut builder = SegmentBuilder::new(
            reader.header.bucket,
            reader.header.bucket_count,
            reader.header.analyzer_id,
            reader.header.analyzer_version,
            reader.header.lang,
            reader.header.positions,
        );

        let mut old_to_new_ord: HashMap<u32, u32> = HashMap::new();
        for (old_ord, doclen, env) in surviving_docs {
            let new_ord = builder.add_document(doclen, env);
            old_to_new_ord.insert(old_ord, new_ord);
        }

        let mut cursor = 28usize;
        let dc_slice: [u8; 4] = segment_bytes
            .get(cursor..cursor + 4)
            .and_then(|s| s.try_into().ok())
            .ok_or_else(|| {
                ContextraError::InvalidInput("Truncated segment bytes at doc count".into())
            })?;
        let doc_count = u32::from_be_bytes(dc_slice) as usize;
        cursor += 4;

        for _ in 0..doc_count {
            cursor += 4;
            let bl_slice: [u8; 2] = segment_bytes
                .get(cursor..cursor + 2)
                .and_then(|s| s.try_into().ok())
                .ok_or_else(|| {
                    ContextraError::InvalidInput("Truncated doc table entry in segment".into())
                })?;
            let b_len = u16::from_be_bytes(bl_slice) as usize;
            cursor += 2;
            cursor += b_len;
        }
        let bitmap_bytes = (reader.header.doc_count as usize + 7) / 8;
        cursor += bitmap_bytes;

        let tc_slice: [u8; 4] = segment_bytes
            .get(cursor..cursor + 4)
            .and_then(|s| s.try_into().ok())
            .ok_or_else(|| {
                ContextraError::InvalidInput("Truncated term count in segment".into())
            })?;
        let term_count = u32::from_be_bytes(tc_slice) as usize;
        cursor += 4;

        for _ in 0..term_count {
            let tag_slice: [u8; 16] = segment_bytes
                .get(cursor..cursor + 16)
                .and_then(|s| s.try_into().ok())
                .ok_or_else(|| ContextraError::InvalidInput("Truncated tag in segment".into()))?;
            let current_tag = TermTag::new(tag_slice);
            cursor += 16;

            cursor += 4; // skip df

            let bc_slice: [u8; 4] = segment_bytes
                .get(cursor..cursor + 4)
                .and_then(|s| s.try_into().ok())
                .ok_or_else(|| {
                    ContextraError::InvalidInput("Truncated block count in segment".into())
                })?;
            let block_count = u32::from_be_bytes(bc_slice) as usize;
            cursor += 4;

            let mut surviving_postings = Vec::new();

            for _ in 0..block_count {
                cursor += 6;
                let count = *segment_bytes.get(cursor).ok_or_else(|| {
                    ContextraError::InvalidInput("Truncated block posting count".into())
                })? as usize;
                cursor += 1;

                let mut prev_doc_ord = 0u32;
                for _ in 0..count {
                    let d_slice: [u8; 4] = segment_bytes
                        .get(cursor..cursor + 4)
                        .and_then(|s| s.try_into().ok())
                        .ok_or_else(|| ContextraError::InvalidInput("Truncated delta".into()))?;
                    let delta = u32::from_be_bytes(d_slice);
                    cursor += 4;

                    let tf_slice: [u8; 4] = segment_bytes
                        .get(cursor..cursor + 4)
                        .and_then(|s| s.try_into().ok())
                        .ok_or_else(|| ContextraError::InvalidInput("Truncated tf".into()))?;
                    let tf = u32::from_be_bytes(tf_slice);
                    cursor += 4;

                    let doc_ord = prev_doc_ord.saturating_add(delta);
                    prev_doc_ord = doc_ord;

                    let mut positions = Vec::new();
                    if reader.header.positions {
                        let pc_slice: [u8; 2] = segment_bytes
                            .get(cursor..cursor + 2)
                            .and_then(|s| s.try_into().ok())
                            .ok_or_else(|| {
                                ContextraError::InvalidInput("Truncated pos count".into())
                            })?;
                        let pos_count = u16::from_be_bytes(pc_slice) as usize;
                        cursor += 2;

                        let mut prev_pos = 0u32;
                        for _ in 0..pos_count {
                            let pd_slice: [u8; 4] = segment_bytes
                                .get(cursor..cursor + 4)
                                .and_then(|s| s.try_into().ok())
                                .ok_or_else(|| {
                                    ContextraError::InvalidInput("Truncated pos delta".into())
                                })?;
                            let p_delta = u32::from_be_bytes(pd_slice);
                            cursor += 4;
                            let pos = prev_pos.saturating_add(p_delta);
                            prev_pos = pos;
                            positions.push(pos);
                        }
                    }

                    if let Some(&new_ord) = old_to_new_ord.get(&doc_ord) {
                        surviving_postings.push(SegmentPosting {
                            doc_ord: new_ord,
                            tf,
                            positions,
                        });
                    }
                }
            }

            if !surviving_postings.is_empty() {
                builder.add_term_postings(current_tag, surviving_postings);
            }
        }

        let new_bytes = builder.build();
        rewritten_segments.push(new_bytes);
    }

    Ok((rewritten_segments, outcome))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::analyzer::AnalyzedToken;
    use contextra_crypto::secret::SecretKey;

    #[test]
    fn test_buffer_flush_and_purge() {
        let key = IdxKey(SecretKey::new([7u8; 32]));
        let mut buffer = TextBuffer::new();

        let doc1 = DocTag::new([1u8; 16]);
        let doc2 = DocTag::new([2u8; 16]);

        let mut env1 = vec![0u8; 32];
        if let Some(dst) = env1.get_mut(16..32) {
            dst.copy_from_slice(doc1.as_bytes());
        }

        let mut env2 = vec![0u8; 32];
        if let Some(dst) = env2.get_mut(16..32) {
            dst.copy_from_slice(doc2.as_bytes());
        }

        buffer.insert(
            TextDocHandle {
                cluster_tag: ClusterTag::new([0u8; 16]),
                doc_tag: doc1,
            },
            100,
            env1,
            AnalyzedDocument {
                tokens: vec![AnalyzedToken {
                    term: "testterm".into(),
                    position: 0,
                }],
                token_count: 1,
            },
        );

        buffer.insert(
            TextDocHandle {
                cluster_tag: ClusterTag::new([0u8; 16]),
                doc_tag: doc2,
            },
            120,
            env2,
            AnalyzedDocument {
                tokens: vec![AnalyzedToken {
                    term: "testterm".into(),
                    position: 0,
                }],
                token_count: 1,
            },
        );

        let seg_bytes = buffer.flush_to_segment(&key, AnalyzerId::De1, 0, 1, true);

        let mut target_docs = HashSet::new();
        target_docs.insert(doc1);

        let mut valid_deks = HashSet::new();
        valid_deks.insert(doc2);

        let (rewritten, outcome) =
            purge_segments(&[seg_bytes], &target_docs, &valid_deks, &key).unwrap();
        assert_eq!(outcome.segments_rewritten, 1);
        assert_eq!(outcome.documents_purged, 1);
        assert_eq!(rewritten.len(), 1);

        let reader = SegmentReader::open(&rewritten[0]).unwrap();
        assert_eq!(reader.header.doc_count, 1);
        assert_eq!(reader.header.total_len, 120);

        let term_tag = compute_term_tag(&key, AnalyzerId::De1, "testterm");
        let postings = reader.get_postings(&term_tag).unwrap().unwrap();
        assert_eq!(postings[0].postings.len(), 1);
        assert_eq!(postings[0].postings[0].doc_ord, 0);
    }
}
