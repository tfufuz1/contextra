//! Asynchrone, wiederaufnehmbare Textindex-Löschschicht.
#![cfg(feature = "types-compat")]

use crate::inverted::{InvertedIndex, TextScanner};
use crate::posting_list::PostingList;
use contextra_ports::erasure::{DocScope, ErasureLayer, LayerError, PurgeCx, PurgeOutcome, ResolvedScope};
use contextra_ports::gate::ReadGate;
use contextra_ports::scanner::{LayerScanner, WitnessLayer};
use contextra_ports::{BoxFuture, StorageEngine};
use contextra_types::{Cursor, DocId, ErasureError, LayerErrorKind, LayerProgress, Result, TxId};
use std::collections::BTreeSet;
use std::sync::{Arc, RwLock};

/// Liefert Transaktions-IDs; Commit und Rollback bleiben beim Storage-Port.
pub trait TextTxSource: Send + Sync {
    fn begin<'a>(&'a self) -> BoxFuture<'a, Result<TxId>>;
    fn rollback<'a>(&'a self, tx: TxId) -> BoxFuture<'a, Result<()>>;
    /// Registers posting-key version collapse after the commit sequence is known.
    fn register_collapse<'a>(&'a self, keys: Vec<Vec<u8>>, seq: contextra_types::Seq) -> BoxFuture<'a, Result<()>>;
}

/// Begrenzungen für Textindex-Purge-Ticks.
#[derive(Clone, Debug)]
pub struct TextErasureConfig {
    /// Maximale Zahl Terme pro Transaktion.
    pub terms_per_tx: usize,
    /// Dictionary-Fallback, wenn der Forward-Index bereits entfernt wurde.
    pub fallback_dictionary_scan: bool,
}

impl Default for TextErasureConfig {
    fn default() -> Self { Self { terms_per_tx: 256, fallback_dictionary_scan: true } }
}

/// ErasureLayer für genau einen Collection-Textindex.
pub struct TextErasure<S: StorageEngine> {
    index: Arc<InvertedIndex<S>>,
    tx: Arc<dyn TextTxSource>,
    cfg: TextErasureConfig,
    gate: Arc<dyn ReadGate>,
    scanner: TextScanner<S>,
    erased_docs: RwLock<Arc<BTreeSet<DocId>>>,
}

impl<S: StorageEngine> TextErasure<S> {
    /// Erzeugt eine Schicht für einen Collection-Index.
    pub fn new(index: Arc<InvertedIndex<S>>, tx: Arc<dyn TextTxSource>, cfg: TextErasureConfig, gate: Arc<dyn ReadGate>) -> Self {
        let scanner = TextScanner::new(index.clone());
        Self { index, tx, cfg, gate, scanner, erased_docs: RwLock::new(Arc::new(BTreeSet::new())) }
    }

    fn docs_for(&self, scope: &ResolvedScope) -> BTreeSet<DocId> {
        scope.docs.iter().filter_map(|d| match d {
            DocScope::Set { collection, docs, .. } if collection == self.index.namespace() => Some(docs.iter().copied()),
            _ => None,
        }).flatten().collect()
    }

    fn covers_index(&self, scope: &ResolvedScope) -> bool {
        scope.docs.iter().any(|d| match d {
            DocScope::Tenant { .. } => true,
            DocScope::Collection { collection, .. } => collection == self.index.namespace(),
            DocScope::Set { .. } => false,
        })
    }

    fn layer_error(kind: LayerErrorKind, detail: &'static str) -> LayerError {
        LayerError { kind, layer: WitnessLayer::from("TextIndex"), detail }
    }

    fn cursor(done: u64, hash: [u8; 8], phase: u8, stats_applied: bool) -> std::result::Result<Cursor, ErasureError> {
        let mut b = Vec::with_capacity(19); b.push(1); b.push(phase); b.extend_from_slice(&hash); b.extend_from_slice(&done.to_le_bytes()); b.push(u8::from(stats_applied));
        Cursor::new(b)
    }
}

impl<S: StorageEngine> ErasureLayer for TextErasure<S> {
    fn layer(&self) -> WitnessLayer { WitnessLayer::from("TextIndex") }

    fn on_gate<'a>(&'a self, scope: &'a ResolvedScope) -> BoxFuture<'a, std::result::Result<(), LayerError>> {
        Box::pin(async move {
            let tenant = scope.docs.iter().find_map(|d| match d {
                DocScope::Set { tenant, collection, .. } if collection == self.index.namespace() => Some(*tenant),
                DocScope::Collection { tenant, collection } if collection == self.index.namespace() => Some(*tenant),
                DocScope::Tenant { tenant } => Some(*tenant),
                _ => None,
            });
            if let Some(tenant) = tenant { self.index.install_erasure_gate(self.gate.clone(), tenant); }
            let mut next = (**self.erased_docs.read().map_err(|_| Self::layer_error(LayerErrorKind::Permanent, "mask_lock_poisoned"))?).clone();
            next.extend(self.docs_for(scope));
            if self.covers_index(scope) { self.index.erased_all.store(true, std::sync::atomic::Ordering::Release); }
            let next = Arc::new(next);
            *self.erased_docs.write().map_err(|_| Self::layer_error(LayerErrorKind::Permanent, "mask_lock_poisoned"))? = next.clone();
            *self.index.erased_docs.write() = next;
            self.scanner.set_erasure_scope(self.docs_for(scope), BTreeSet::new(), self.covers_index(scope));
            Ok(())
        })
    }

    fn purge<'a>(&'a self, scope: &'a ResolvedScope, cx: &'a PurgeCx<'a>) -> BoxFuture<'a, std::result::Result<PurgeOutcome, LayerError>> {
        Box::pin(async move {
            if cx.cancel.is_cancelled() { return Err(Self::layer_error(LayerErrorKind::Cancelled, "cancelled")); }
            let mut docs = self.docs_for(scope);
            let remove_all = self.covers_index(scope);
            if docs.is_empty() && remove_all {
                let prefix = self.index.key("fw:");
                let mut after: Option<Vec<u8>> = None;
                loop {
                    let rows = cx.reader.scan_prefix(&prefix, after.as_deref(), 1024).await
                        .map_err(|e| Self::layer_error(e.kind(), "forward_index_scan_failed"))?;
                    if rows.is_empty() { break; }
                    for (key, _) in &rows {
                        let raw = std::str::from_utf8(&key[prefix.len()..]).map_err(|_| Self::layer_error(LayerErrorKind::Permanent, "forward_index_key_corrupt"))?;
                        #[cfg(not(feature = "docid-128"))]
                        let id = match raw.parse::<u64>() { Ok(id) => id as u128, Err(_) => return Err(Self::layer_error(LayerErrorKind::Permanent, "forward_index_key_corrupt")) };
                        #[cfg(feature = "docid-128")]
                        let id = match raw.parse::<u128>() { Ok(id) => id, Err(_) => return Err(Self::layer_error(LayerErrorKind::Permanent, "forward_index_key_corrupt")) };
                        docs.insert(DocId::new(id as _));
                    }
                    after = rows.last().map(|(k, _)| k.to_vec());
                    if rows.len() < 1024 { break; }
                }
            }
            let mut terms = BTreeSet::<String>::new();
            for doc in &docs {
                let fw = self.index.key_with_id("fw:", doc.inner());
                match cx.reader.get(&fw).await {
                    Ok(Some(bytes)) => {
                        let decoded: Vec<String> = bincode::deserialize(&bytes).map_err(|_| Self::layer_error(LayerErrorKind::Permanent, "forward_index_corrupt"))?;
                        terms.extend(decoded);
                    }
                    Ok(None) => {}
                    Err(e) => return Err(Self::layer_error(e.kind(), "privileged_read_failed")),
                }
            }
            if terms.is_empty() && self.cfg.fallback_dictionary_scan && (!docs.is_empty() || remove_all) {
                let prefix = self.index.key("plb:");
                let mut after: Option<Vec<u8>> = None;
                loop {
                    let rows = cx.reader.scan_prefix(&prefix, after.as_deref(), 1024).await
                        .map_err(|e| Self::layer_error(e.kind(), "dictionary_scan_failed"))?;
                    if rows.is_empty() { break; }
                    for (k, _) in &rows {
                        let term = std::str::from_utf8(&k[prefix.len()..]).map_err(|_| Self::layer_error(LayerErrorKind::Permanent, "dictionary_key_corrupt"))?;
                        terms.insert(term.to_owned());
                    }
                    after = rows.last().map(|(k, _)| k.to_vec());
                    if rows.len() < 1024 { break; }
                }
                let legacy_prefix = self.index.key("pl:");
                let mut after: Option<Vec<u8>> = None;
                loop {
                    let rows = cx.reader.scan_prefix(&legacy_prefix, after.as_deref(), 1024).await
                        .map_err(|e| Self::layer_error(e.kind(), "legacy_dictionary_scan_failed"))?;
                    if rows.is_empty() { break; }
                    for (key, _) in &rows {
                        if let Some(split) = key.iter().rposition(|b| *b == b':') {
                            if split > legacy_prefix.len() {
                                    let term = std::str::from_utf8(&key[legacy_prefix.len()..split]).map_err(|_| Self::layer_error(LayerErrorKind::Permanent, "legacy_dictionary_key_corrupt"))?;
                                    terms.insert(term.to_owned());
                            }
                        }
                    }
                    after = rows.last().map(|(k, _)| k.to_vec());
                    if rows.len() < 1024 { break; }
                }
            }
            self.scanner.set_erasure_scope(docs.clone(), terms.clone(), remove_all);
            let hash = blake3::hash(&terms.iter().flat_map(|s| s.as_bytes().iter().copied().chain(std::iter::once(0))).collect::<Vec<_>>());
            let resume = cx.cursor.as_ref().and_then(|c| {
                let b = c.as_bytes();
                if b.len() != 19 || b[0] != 1 || b[2..10] != hash.as_bytes()[..8] { return None; }
                let raw: [u8; 8] = match b[10..18].try_into() { Ok(v) => v, Err(_) => return None };
                Some((u64::from_le_bytes(raw), b[18] == 1 && b[1] == 1))
            });
            let mut done = resume.map_or(0, |(n, _)| n) as usize;
            let stats_applied = resume.is_some_and(|(_, applied)| applied);
            let start = done.min(terms.len());
            let page_size = self.cfg.terms_per_tx.max(1).min(cx.max_units.max(1));
            let page: Vec<_> = terms.iter().skip(start).take(page_size).cloned().collect();
            let mut bytes_freed = 0u64;
            if !page.is_empty() {
                let mut writes = Vec::new(); let mut io_bytes = 0u64;
                for term in &page {
                    if cx.cancel.is_cancelled() { return Err(Self::layer_error(LayerErrorKind::Cancelled, "cancelled")); }
                    let key = self.index.key_batch_posting_list(term);
                    if let Some(raw) = cx.reader.get(&key).await.map_err(|e| Self::layer_error(e.kind(), "posting_read_failed"))? {
                        io_bytes = io_bytes.saturating_add(raw.len() as u64);
                        let source = raw.to_vec();
                        let target_docs: Vec<DocId> = docs.iter().copied().collect();
                        let transformed = tokio::task::spawn_blocking(move || -> std::result::Result<(Option<Vec<u8>>, u64), ()> {
                            let mut list = PostingList::decode_compact(&source).map_err(|_| ())?;
                            if remove_all { list = PostingList::empty(); }
                            else { for doc in target_docs { list = list.remove(doc); } }
                            if list.is_empty() { Ok((None, source.len() as u64)) }
                            else {
                                let encoded = list.encode_compact().map_err(|_| ())?;
                                let freed = source.len().saturating_sub(encoded.len()) as u64;
                                Ok((Some(encoded), freed))
                            }
                        }).await.map_err(|_| Self::layer_error(LayerErrorKind::Retryable, "posting_worker_failed"))?
                            .map_err(|_| Self::layer_error(LayerErrorKind::Permanent, "posting_corrupt"))?;
                        io_bytes = io_bytes.saturating_add(transformed.0.as_ref().map_or(0, |v| v.len() as u64));
                        bytes_freed = bytes_freed.saturating_add(transformed.1).saturating_add(if transformed.0.is_none() { key.len() as u64 } else { 0 });
                        writes.push((key, transformed.0));
                    }
                    let legacy_prefix = self.index.key_term_prefix(term);
                    let mut after: Option<Vec<u8>> = None;
                    loop {
                        let rows = cx.reader.scan_prefix(&legacy_prefix, after.as_deref(), 1024).await
                            .map_err(|e| Self::layer_error(e.kind(), "legacy_posting_scan_failed"))?;
                        if rows.is_empty() { break; }
                        for (legacy_key, legacy_value) in &rows {
                            let raw_id = std::str::from_utf8(&legacy_key[legacy_prefix.len()..]).map_err(|_| Self::layer_error(LayerErrorKind::Permanent, "legacy_document_key_corrupt"))?;
                            #[cfg(not(feature = "docid-128"))]
                            let id = match raw_id.parse::<u64>() { Ok(v) => v as u128, Err(_) => return Err(Self::layer_error(LayerErrorKind::Permanent, "legacy_document_key_corrupt")) };
                            #[cfg(feature = "docid-128")]
                            let id = match raw_id.parse::<u128>() { Ok(v) => v, Err(_) => return Err(Self::layer_error(LayerErrorKind::Permanent, "legacy_document_key_corrupt")) };
                            if remove_all || docs.contains(&DocId::new(id as _)) {
                                bytes_freed = bytes_freed.saturating_add((legacy_key.len() + legacy_value.len()) as u64);
                                io_bytes = io_bytes.saturating_add((legacy_key.len() + legacy_value.len()) as u64);
                                writes.push((legacy_key.to_vec(), None));
                            }
                        }
                        after = rows.last().map(|(k, _)| k.to_vec());
                        if rows.len() < 1024 { break; }
                    }
                }
                cx.budget.acquire(contextra_ports::erasure::IoClass::Erase, io_bytes).await.map_err(|e| Self::layer_error(e.kind(), "io_budget_failed"))?;
                let tx = self.tx.begin().await.map_err(|_| Self::layer_error(LayerErrorKind::Retryable, "tx_begin_failed"))?;
                let mut failure = None;
                for (key, value) in &writes {
                    let result = match value { Some(v) => self.index.storage().put(tx, key, v).await, None => self.index.storage().delete(tx, key).await };
                    if result.is_err() { failure = Some(()); break; }
                }
                if failure.is_some() || self.index.storage().commit(tx).await.is_err() {
                    self.tx.rollback(tx).await.map_err(|_| Self::layer_error(LayerErrorKind::Retryable, "rollback_failed"))?;
                    return Err(Self::layer_error(LayerErrorKind::Retryable, "posting_commit_failed"));
                }
                let seq = self.index.storage().last_seq_no().await.map_err(|_| Self::layer_error(LayerErrorKind::Retryable, "commit_sequence_read_failed"))?;
                let plb_prefix = self.index.key("plb:");
                let collapse_keys = writes.iter().filter(|(key, _)| key.starts_with(&plb_prefix)).map(|(key, _)| key.clone()).collect();
                self.tx.register_collapse(collapse_keys, contextra_types::Seq::new(seq)).await
                    .map_err(|_| Self::layer_error(LayerErrorKind::Retryable, "collapse_registration_failed"))?;
                done += page.len();
            }
            let complete = done >= terms.len();
            if complete && !stats_applied {
                let mut removed = crate::inverted::RemovedDocStats::default();
                for doc in &docs {
                    let key = self.index.key_with_id("dl:", doc.inner());
                    let Some(raw) = cx.reader.get(&key).await.map_err(|e| Self::layer_error(e.kind(), "document_length_read_failed"))? else { continue };
                    let bytes: [u8; 4] = raw.as_ref().try_into().map_err(|_| Self::layer_error(LayerErrorKind::Permanent, "document_length_corrupt"))?;
                    removed.docs = removed.docs.saturating_add(1);
                    removed.tokens = removed.tokens.saturating_add(u32::from_le_bytes(bytes) as u64);
                }
                if removed.docs > 0 {
                    let stats_key = self.index.key("meta:stats");
                    let mut meta = match cx.reader.get(&stats_key).await.map_err(|e| Self::layer_error(e.kind(), "statistics_read_failed"))? {
                        Some(raw) => bincode::deserialize::<crate::inverted::TextIndexMetadata>(&raw).map_err(|_| Self::layer_error(LayerErrorKind::Permanent, "statistics_corrupt"))?,
                        None => crate::inverted::TextIndexMetadata {
                            total_docs: self.index.total_docs.load(std::sync::atomic::Ordering::Acquire),
                            total_tokens: self.index.total_tokens.load(std::sync::atomic::Ordering::Acquire),
                            avg_doc_len_x1000: self.index.avg_doc_len_x1000.load(std::sync::atomic::Ordering::Acquire),
                        },
                    };
                    meta.total_docs = meta.total_docs.saturating_sub(removed.docs);
                    meta.total_tokens = meta.total_tokens.saturating_sub(removed.tokens);
                    meta.avg_doc_len_x1000 = if meta.total_docs == 0 { 0 } else { meta.total_tokens.saturating_mul(1000) / meta.total_docs };
                    let encoded = bincode::serialize(&meta).map_err(|_| Self::layer_error(LayerErrorKind::Permanent, "statistics_encode_failed"))?;
                    cx.budget.acquire(contextra_ports::erasure::IoClass::Erase, encoded.len() as u64).await.map_err(|e| Self::layer_error(e.kind(), "io_budget_failed"))?;
                    let tx = self.tx.begin().await.map_err(|_| Self::layer_error(LayerErrorKind::Retryable, "stats_tx_begin_failed"))?;
                    if self.index.storage().put(tx, &stats_key, &encoded).await.is_err() || self.index.storage().commit(tx).await.is_err() {
                        self.tx.rollback(tx).await.map_err(|_| Self::layer_error(LayerErrorKind::Retryable, "rollback_failed"))?;
                        return Err(Self::layer_error(LayerErrorKind::Retryable, "statistics_commit_failed"));
                    }
                    self.index.recompute_stats_after(&removed);
                }
                let hash8: [u8; 8] = hash.as_bytes()[..8].try_into().map_err(|_| Self::layer_error(LayerErrorKind::Permanent, "work_hash_invalid"))?;
                let stats_cursor = Self::cursor(done as u64, hash8, 1, true).map_err(|_| Self::layer_error(LayerErrorKind::Permanent, "cursor_invalid"))?;
                let stats_progress = LayerProgress { cursor: Some(stats_cursor), units_done: done as u64, bytes_freed };
                cx.progress.record(&self.layer(), &stats_progress).await.map_err(|e| Self::layer_error(e.kind(), "stats_progress_record_failed"))?;
            }
            if complete { self.index.drop_resident_terms(&terms.iter().cloned().collect::<Vec<_>>()); }
            let hash8: [u8; 8] = hash.as_bytes()[..8].try_into().map_err(|_| Self::layer_error(LayerErrorKind::Permanent, "work_hash_invalid"))?;
            let cursor = if complete { None } else { Some(Self::cursor(done as u64, hash8, 0, false).map_err(|_| Self::layer_error(LayerErrorKind::Permanent, "cursor_invalid"))?) };
            let progress = LayerProgress { cursor, units_done: done as u64, bytes_freed };
            cx.progress.record(&self.layer(), &progress).await.map_err(|e| Self::layer_error(e.kind(), "progress_record_failed"))?;
            Ok(PurgeOutcome { progress, units_committed: page.len() as u64, bytes_freed })
        })
    }

    fn scanner(&self) -> &dyn LayerScanner { &self.scanner }
}

