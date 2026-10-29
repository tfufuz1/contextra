// FILE-CONTEXT: TextIndex Storage & Memory Layer Scanner (§12.7.4, INV-WITNESS-MINT-1).
// ZWECK: Stellt den LayerScanner für WitnessLayer::TextIndex bereit.
// INVARIANTEN: Scan-Messungen laufen unabhängig vom Löschcode; Positivkontrolle vorab zwingend.
// NICHT-OFFENSICHTLICH: Scans read both LSM storage keys and in-memory ResidentPostingIndex entries directly.
// HOTSPOTS: scan_units, run_positive_control

use crate::inverted::index_struct::InvertedIndex;
use crate::posting_list::PostingList;
use contextra_ports::scanner::{LayerScanner, ScanSession, ScanTarget, WitnessLayer};
use contextra_ports::{StorageEngine, TextIndex};
use contextra_types::{ContextraError, DocId, RequestContext, Result, TxId};
use std::sync::Arc;

/// Parsed target type for text index scanner.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParsedTextTarget {
    Document(DocId),
    Collection(String),
}

impl ParsedTextTarget {
    /// Parses a target ID string into document ID or collection namespace.
    pub fn parse(target_id: &str) -> Self {
        if let Some(rest) = target_id.strip_prefix("doc:") {
            if let Ok(id) = rest.parse::<u64>() {
                return ParsedTextTarget::Document(DocId::from(id));
            }
        }
        if let Some(rest) = target_id.strip_prefix("doc_id:") {
            if let Ok(id) = rest.parse::<u64>() {
                return ParsedTextTarget::Document(DocId::from(id));
            }
        }
        #[cfg(not(feature = "docid-128"))]
        if let Ok(id) = target_id.parse::<u64>() {
            return ParsedTextTarget::Document(DocId::from(id));
        }
        #[cfg(feature = "docid-128")]
        if let Ok(id) = target_id.parse::<u128>() {
            return ParsedTextTarget::Document(DocId::from(id));
        }
        if let Some(rest) = target_id.strip_prefix("collection:") {
            return ParsedTextTarget::Collection(rest.to_string());
        }
        if let Some(rest) = target_id.strip_prefix("ns:") {
            return ParsedTextTarget::Collection(rest.to_string());
        }
        if let Some(rest) = target_id.strip_prefix("namespace:") {
            return ParsedTextTarget::Collection(rest.to_string());
        }
        ParsedTextTarget::Collection(target_id.to_string())
    }
}

/// Helper function to check if a byte slice contains a sub-slice.
fn slice_contains(haystack: &[u8], needle: &[u8]) -> bool {
    if needle.is_empty() || haystack.len() < needle.len() {
        return false;
    }
    haystack.windows(needle.len()).any(|w| w == needle)
}

/// Executes an async block safely inside synchronous scanner methods.
fn block_on_async<F: std::future::Future + Send + 'static>(f: F) -> F::Output
where
    F::Output: Send + 'static,
{
    futures::executor::block_on(f)
}

/// Scanner implementation for `WitnessLayer::TextIndex`.
pub struct TextScanner<S: StorageEngine + 'static> {
    index: Arc<InvertedIndex<S>>,
    target: Arc<parking_lot::Mutex<Option<ScanTarget>>>,
}

impl<S: StorageEngine + 'static> TextScanner<S> {
    /// Creates a new `TextScanner` for the given index.
    pub fn new(index: Arc<InvertedIndex<S>>) -> Self {
        Self {
            index,
            target: Arc::new(parking_lot::Mutex::new(None)),
        }
    }

    /// Creates a new `TextScanner` with an explicit target.
    pub fn new_with_target(index: Arc<InvertedIndex<S>>, target: ScanTarget) -> Self {
        Self {
            index,
            target: Arc::new(parking_lot::Mutex::new(Some(target))),
        }
    }

    /// Configures the scan target on this scanner.
    pub fn with_target(self, target: ScanTarget) -> Self {
        *self.target.lock() = Some(target);
        self
    }

    /// Sets the scan target.
    pub fn set_target(&self, target: ScanTarget) {
        *self.target.lock() = Some(target);
    }

    /// Executes the positive control canary check.
    fn run_positive_control(&self, session: &mut ScanSession) -> Result<()> {
        let canary_ns = "scratch_canary_pctl";
        let canary_doc_id = DocId::from(999_999_999u64);
        let scratch_index = InvertedIndex::new(self.index.storage().clone(), canary_ns);

        let tx = TxId::new(999_999_999);
        let scratch_index_arc = Arc::new(scratch_index);
        let scratch_c = scratch_index_arc.clone();
        block_on_async(async move {
            scratch_c
                .upsert_document(tx, canary_doc_id, "canary_positive_control_term")
                .await?;
            scratch_c.commit(tx).await
        })?;

        let mut found_canary = false;
        let prefix = scratch_index_arc.prefix().to_vec();
        let storage = scratch_index_arc.storage().clone();
        let raw_entries = block_on_async(async move { storage.scan_prefix(&prefix).await })?;

        let canary_suffix = format!(":{}", canary_doc_id.inner()).into_bytes();
        let canary_infix = format!(":{}:", canary_doc_id.inner()).into_bytes();

        for (k, v) in &raw_entries {
            if k.ends_with(&canary_suffix) || slice_contains(k, &canary_infix) {
                found_canary = true;
                break;
            }
            if k.starts_with(&scratch_index_arc.key("plb:")) {
                if let Ok(plist) = PostingList::decode_compact(v) {
                    if plist.as_slice().iter().any(|p| p.doc_id() == canary_doc_id) {
                        found_canary = true;
                        break;
                    }
                }
            }
        }

        // Clean up canary document from scratch namespace
        let scratch_c = scratch_index_arc.clone();
        let cleanup_res = block_on_async(async move {
            let tx_del = TxId::new(999_999_998);
            for (k, _) in raw_entries {
                let _ = scratch_c.storage().delete(tx_del, &k).await;
            }
            scratch_c.commit(tx_del).await
        });
        drop(cleanup_res);

        if found_canary {
            session.record_control_match();
            Ok(())
        } else {
            Err(ContextraError::Internal(
                "Positive control check failed: canary document was not detected".into(),
            ))
        }
    }
}

impl<S: StorageEngine + 'static> LayerScanner for TextScanner<S> {
    fn layer(&self) -> WitnessLayer {
        WitnessLayer::from("TextIndex")
    }

    fn scan_units(&self, session: &mut ScanSession, ctx: &RequestContext) -> Result<()> {
        // Step 1: Run mandatory positive control
        self.run_positive_control(session)?;

        // Record structural index layer unit
        let mut hasher = blake3::Hasher::new();
        hasher.update(b"index_structure:");
        hasher.update(self.index.prefix());
        let index_unit_hash: [u8; 32] = *hasher.finalize().as_bytes();
        session.record_unit(index_unit_hash, false);

        // Step 2: Determine target
        let target_opt = self.target.lock().clone();
        let target_str = match target_opt {
            Some(ref t) => t.target_id.clone(),
            None => format!("doc:{}", ctx.tx_id.inner()),
        };

        let parsed_target = ParsedTextTarget::parse(&target_str);

        // Step 3: Scan physical LSM storage entries for index namespace
        let index_prefix = self.index.prefix().to_vec();
        let storage = self.index.storage().clone();
        let raw_entries = block_on_async(async move { storage.scan_prefix(&index_prefix).await })?;

        match parsed_target {
            ParsedTextTarget::Document(target_doc_id) => {
                let doc_suffix = format!(":{}", target_doc_id.inner()).into_bytes();
                let doc_infix = format!(":{}:", target_doc_id.inner()).into_bytes();

                for (k, v) in &raw_entries {
                    let mut hasher = blake3::Hasher::new();
                    hasher.update(k);
                    hasher.update(v);
                    let unit_hash: [u8; 32] = *hasher.finalize().as_bytes();

                    let mut is_residual = false;

                    if k.ends_with(&doc_suffix) || slice_contains(k, &doc_infix) {
                        is_residual = true;
                    } else if k.starts_with(&self.index.key("plb:")) {
                        if let Ok(plist) = PostingList::decode_compact(v) {
                            if plist.as_slice().iter().any(|p| p.doc_id() == target_doc_id) {
                                is_residual = true;
                            }
                        }
                    }

                    session.record_unit(unit_hash, is_residual);
                }

                // Step 4: Scan in-memory ResidentPostingIndex cache
                let resident = &self.index.resident_index;
                let plb_prefix = self.index.key("plb:");
                for (k, _) in &raw_entries {
                    if k.starts_with(&plb_prefix) {
                        let term_bytes = &k[plb_prefix.len()..];
                        if let Ok(term) = std::str::from_utf8(term_bytes) {
                            if let Some(plist) = resident.get(term) {
                                if plist.as_slice().iter().any(|p| p.doc_id() == target_doc_id) {
                                    let mut hasher = blake3::Hasher::new();
                                    hasher.update(b"resident:");
                                    hasher.update(term.as_bytes());
                                    hasher.update(&target_doc_id.inner().to_le_bytes());
                                    let unit_hash: [u8; 32] = *hasher.finalize().as_bytes();
                                    session.record_unit(unit_hash, true);
                                }
                            }
                        }
                    }
                }
            }
            ParsedTextTarget::Collection(ref col_ns) => {
                let col_prefix = if col_ns == "default" {
                    b"__txt:default:".to_vec()
                } else if col_ns.starts_with("__txt:") {
                    col_ns.as_bytes().to_vec()
                } else {
                    format!("__txt:{}:", col_ns).into_bytes()
                };

                let storage = self.index.storage().clone();
                let col_entries =
                    block_on_async(async move { storage.scan_prefix(&col_prefix).await })?;

                for (k, v) in col_entries {
                    let mut hasher = blake3::Hasher::new();
                    hasher.update(&k);
                    hasher.update(&v);
                    let unit_hash: [u8; 32] = *hasher.finalize().as_bytes();
                    session.record_unit(unit_hash, true);
                }
            }
        }

        Ok(())
    }
}

/// Factory function to construct a `LayerScanner` for text index.
pub fn text_scanner<S: StorageEngine + 'static>(
    index: Arc<InvertedIndex<S>>,
) -> Arc<dyn LayerScanner> {
    Arc::new(TextScanner::new(index))
}
