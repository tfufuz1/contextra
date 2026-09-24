use super::*;
use contextra_ports::{BoxFuture, LlmTextGenerator, StorageEngine, StorageStats};
use contextra_types::{
    ContextChunk, ContextraError, DocId, Result, TenantId, TokenBudget, TxId,
};
use contextra_engine::collection::Collection;
use contextra_engine::transaction::CommitIntent;
use contextra_engine::ProvenanceRecord;
use contextra_graph::CsrGraph;
use contextra_store::{LsmConfig, LsmStorage};
use contextra_vector::{HnswConfig, HnswIndex};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;

fn make_chunk(id: u64, content: &str, relevance: f32, is_tool: bool) -> ContextChunk {
    let metadata = if is_tool {
        Some(serde_json::json!({"tool_output": true}))
    } else {
        None
    };
    ContextChunk {
        doc_id: DocId::new(id),
        content: content.to_string(),
        relevance,
        token_count: content.len(),
        metadata,
        contextual_prefix: None,
        links: Vec::new(),
    }
}

#[test]
fn test_compactor_within_budget() {
    let budget = TokenBudget::new(100, 0);
    let compactor = ContextCompactor::new(budget, CompactionStrategy::StatusToken);

    let chunks = vec![
        make_chunk(1, "chunk 1", 0.9, false),
        make_chunk(2, "chunk 2", 0.8, false),
    ];

    let result = compactor.compact(chunks);
    assert_eq!(result.retained_chunks.len(), 2);
    assert!(result.status_tokens.is_empty());
    assert_eq!(result.tokens_used, "chunk 1".len() + "chunk 2".len());
}

#[test]
fn test_compactor_tool_output_compacted_first() {
    let budget = TokenBudget::new(20, 0);
    let compactor = ContextCompactor::new(budget, CompactionStrategy::StatusToken);

    let chunks = vec![
        make_chunk(1, "tool output data 12345", 0.95, true), // 22 bytes, tool output
        make_chunk(2, "important context", 0.8, false),      // 17 bytes, normal
    ];

    let result = compactor.compact(chunks);
    // Important context should be retained (17 bytes <= 20 max_tokens),
    // tool output should be sorted last and converted to status token.
    assert_eq!(result.retained_chunks.len(), 1);
    assert_eq!(result.retained_chunks[0].doc_id, DocId::new(2));
    assert_eq!(result.status_tokens.len(), 1);
    assert_eq!(
        result.status_tokens[0].replaced_doc_ids,
        vec![DocId::new(1)]
    );
}

#[test]
fn test_compactor_truncate_strategy() {
    let budget = TokenBudget::new(10, 0);
    let compactor = ContextCompactor::new(budget, CompactionStrategy::Truncate);

    let chunks = vec![
        make_chunk(1, "small", 0.9, false),            // 5 bytes
        make_chunk(2, "exceeding budget", 0.8, false), // 16 bytes
    ];

    let result = compactor.compact(chunks);
    assert_eq!(result.retained_chunks.len(), 1);
    assert_eq!(result.retained_chunks[0].doc_id, DocId::new(1));
    assert!(result.status_tokens.is_empty());
}

#[test]
fn test_compactor_summarize_strategy_fallback() {
    let budget = TokenBudget::new(10, 0);
    let compactor = ContextCompactor::new(budget, CompactionStrategy::Summarize);

    let chunks = vec![
        make_chunk(1, "small", 0.9, false),
        make_chunk(2, "large content for summarization", 0.8, false),
    ];

    let result = compactor.compact(chunks);
    assert_eq!(result.retained_chunks.len(), 1);
    assert_eq!(result.status_tokens.len(), 1);
    assert_eq!(
        result.status_tokens[0].replaced_doc_ids,
        vec![DocId::new(2)]
    );
}

#[test]
fn test_compact_with_contextual_prefix_respects_budget() {
    let budget = TokenBudget::new(20, 0); // 20 tokens available
    let compactor = ContextCompactor::new(budget, CompactionStrategy::Truncate);

    // Chunk: token_count=10, prefix adds ~5 tokens → combined=15
    let chunk = ContextChunk {
        doc_id: DocId::new(1),
        content: "content".to_string(),
        relevance: 1.0,
        token_count: 10,
        metadata: None,
        contextual_prefix: Some("1234567890123456789012".to_string()), // 22 chars → +5 tokens
        links: Vec::new(),
    };

    // Without prefix: chunk fits (10 <= 20)
    // With prefix: chunk fits (15 <= 20)
    let result = compactor.compact(vec![chunk]);
    // combined_token_count=15 <= budget=20 → retained
    assert_eq!(result.retained_chunks.len(), 1);
    assert_eq!(result.tokens_used, 15); // combined_token_count, not raw token_count
}

struct UnreachableLlmGenerator;
impl LlmTextGenerator for UnreachableLlmGenerator {
    fn generate<'a>(&'a self, _prompt: &'a str) -> BoxFuture<'a, Result<String>> {
        Box::pin(async move {
            Err(ContextraError::Io(std::io::Error::new(
                std::io::ErrorKind::ConnectionRefused,
                "Unreachable LLM generator",
            )))
        })
    }
}

#[tokio::test]
async fn test_consolidate_via_llm_error_propagation_on_unreachable_client() {
    let budget = TokenBudget::new(100, 0);
    let compactor = ContextCompactor::new(
        budget,
        CompactionStrategy::LlmSummarize {
            max_input_chunks: 5,
        },
    );

    let dead_llm = UnreachableLlmGenerator;

    let chunks = vec![
        make_chunk(101, "First chunk content", 0.9, false),
        make_chunk(102, "Second chunk content", 0.8, false),
    ];

    let tenant = TenantId::try_new(1).unwrap();
    let res = compactor
        .consolidate_via_llm(tenant, &chunks, &dead_llm, "llama3.2")
        .await;
    // Must return an Error and NOT fall back silently to StatusToken inside compaction.rs
    assert!(res.is_err());
}

#[tokio::test]
async fn test_consolidate_via_llm_provenance_and_empty() {
    let budget = TokenBudget::new(100, 0);
    let compactor = ContextCompactor::new(budget, CompactionStrategy::Summarize);
    let dead_llm = UnreachableLlmGenerator;

    // Empty chunks slice test
    let tenant = TenantId::try_new(1).unwrap();
    let empty_res = compactor
        .consolidate_via_llm(tenant, &[], &dead_llm, "llama3.2")
        .await;
    assert!(empty_res.is_ok());
    let empty_ctx = empty_res.unwrap(); // unwrap allowed (in test)
    assert!(empty_ctx.retained_chunks.is_empty());
    assert!(empty_ctx.source_doc_ids.is_empty());
}

struct MockLlmGenerator;
impl LlmTextGenerator for MockLlmGenerator {
    fn generate<'a>(&'a self, _prompt: &'a str) -> BoxFuture<'a, Result<String>> {
        Box::pin(async move { Ok("Zusammenfassung der 3 Quelldokumente".to_string()) })
    }
}

#[tokio::test]
async fn test_consolidate_via_llm_provenance_3_source_docs() {
    let budget = TokenBudget::new(1000, 0);
    let compactor = ContextCompactor::new(budget, CompactionStrategy::Summarize);
    let mock_llm = MockLlmGenerator;

    let chunks = vec![
        make_chunk(10, "Erstes Quelldokument", 0.9, false),
        make_chunk(20, "Zweites Quelldokument", 0.8, false),
        make_chunk(30, "Drittes Quelldokument", 0.7, false),
    ];

    let tenant = TenantId::try_new(1).unwrap();
    let compacted = compactor
        .consolidate_via_llm(tenant, &chunks, &mock_llm, "mock-model")
        .await
        .expect("Consolidation should succeed");

    assert_eq!(compacted.source_doc_ids.len(), 3);
    assert_eq!(compacted.retained_chunks.len(), 1);

    let chunk = &compacted.retained_chunks[0];
    let meta = chunk.metadata.as_ref().expect("Metadata must be present");
    let prov_val = meta
        .get("provenance")
        .expect("Provenance must be present in metadata");
    let prov: ProvenanceRecord =
        serde_json::from_value(prov_val.clone()).expect("Valid ProvenanceRecord");

    assert_eq!(prov.index_type.as_deref(), Some("consolidated"));
    assert_eq!(prov.signal_ranks.len(), 3);
    assert!(prov
        .signal_ranks
        .contains_key(&DocId::new(10).0.to_string()));
    assert!(prov
        .signal_ranks
        .contains_key(&DocId::new(20).0.to_string()));
    assert!(prov
        .signal_ranks
        .contains_key(&DocId::new(30).0.to_string()));
}

#[test]
fn test_compact_empty_chunks_returns_empty_compacted_context() {
    let budget = TokenBudget::new(100, 0);
    let compactor = ContextCompactor::new(budget, CompactionStrategy::Truncate);
    let result = compactor.compact(vec![]);
    assert!(result.retained_chunks.is_empty());
    assert!(result.status_tokens.is_empty());
    assert_eq!(result.tokens_used, 0);
    assert!(result.source_doc_ids.is_empty());
}

#[derive(Clone)]
struct FaultyDeleteStorage {
    inner: Arc<LsmStorage>,
    fail_delete: Arc<AtomicBool>,
}

impl StorageEngine for FaultyDeleteStorage {
    fn get<'a>(
        &'a self,
        key: &'a [u8],
    ) -> BoxFuture<'a, Result<Option<bytes::Bytes>>> {
        Box::pin(async move { self.inner.get(key).await })
    }

    fn get_at_seq<'a>(
        &'a self,
        key: &'a [u8],
        seq: u64,
    ) -> BoxFuture<'a, Result<Option<bytes::Bytes>>> {
        Box::pin(async move { self.inner.get_at_seq(key, seq).await })
    }

    fn put<'a>(
        &'a self,
        tx_id: TxId,
        key: &'a [u8],
        value: &'a [u8],
    ) -> BoxFuture<'a, Result<()>> {
        Box::pin(async move { self.inner.put(tx_id, key, value).await })
    }

    fn put_if_absent<'a>(
        &'a self,
        tx_id: TxId,
        key: &'a [u8],
        value: &'a [u8],
    ) -> BoxFuture<'a, Result<bool>> {
        Box::pin(async move { self.inner.put_if_absent(tx_id, key, value).await })
    }

    fn delete<'a>(
        &'a self,
        tx_id: TxId,
        key: &'a [u8],
    ) -> BoxFuture<'a, Result<()>> {
        Box::pin(async move {
            if self.fail_delete.load(Ordering::SeqCst) {
                return Err(ContextraError::Transaction(
                    "INJECTED FAULT: Storage delete failure".into(),
                ));
            }
            self.inner.delete(tx_id, key).await
        })
    }

    fn commit<'a>(&'a self, tx_id: TxId) -> BoxFuture<'a, Result<()>> {
        Box::pin(async move { self.inner.commit(tx_id).await })
    }

    fn rollback<'a>(&'a self, tx_id: TxId) -> BoxFuture<'a, Result<()>> {
        Box::pin(async move { self.inner.rollback(tx_id).await })
    }

    fn rollback_to_tx<'a>(&'a self, tx_id: TxId) -> BoxFuture<'a, Result<()>> {
        Box::pin(async move { self.inner.rollback_to_tx(tx_id).await })
    }

    fn flush<'a>(&'a self) -> BoxFuture<'a, Result<()>> {
        Box::pin(async move { self.inner.flush().await })
    }

    fn stats<'a>(&'a self) -> BoxFuture<'a, Result<StorageStats>> {
        Box::pin(async move { self.inner.stats().await })
    }

    fn last_seq_no<'a>(&'a self) -> BoxFuture<'a, Result<u64>> {
        Box::pin(async move { self.inner.last_seq_no().await })
    }

    fn last_tx_id<'a>(&'a self) -> BoxFuture<'a, Result<TxId>> {
        Box::pin(async move { self.inner.last_tx_id().await })
    }

    fn pin_checkpoint<'a>(&'a self, seq_no: u64) -> BoxFuture<'a, Result<()>> {
        Box::pin(async move { self.inner.pin_checkpoint(seq_no).await })
    }

    fn unpin_checkpoint<'a>(&'a self, seq_no: u64) -> BoxFuture<'a, Result<()>> {
        Box::pin(async move { self.inner.unpin_checkpoint(seq_no).await })
    }

    fn scan_prefix<'a>(
        &'a self,
        prefix: &'a [u8],
    ) -> BoxFuture<'a, Result<Vec<(Vec<u8>, Vec<u8>)>>> {
        Box::pin(async move { self.inner.scan_prefix(prefix).await })
    }

    fn scan_prefix_at<'a>(
        &'a self,
        prefix: &'a [u8],
        seq_no: u64,
    ) -> BoxFuture<'a, Result<Vec<(Vec<u8>, Vec<u8>)>>> {
        Box::pin(async move { self.inner.scan_prefix_at(prefix, seq_no).await })
    }

    fn scan<'a>(
        &'a self,
        start: std::ops::Bound<&'a [u8]>,
        end: std::ops::Bound<&'a [u8]>,
        limit: Option<usize>,
    ) -> BoxFuture<'a, Result<Vec<(Vec<u8>, Vec<u8>)>>> {
        Box::pin(async move { self.inner.scan(start, end, limit).await })
    }
}

#[tokio::test]
async fn test_consolidation_aborts_on_delete_failure(
) -> std::result::Result<(), Box<dyn std::error::Error>> {
    let tmp = tempfile::tempdir()?;
    let lsm_config = LsmConfig {
        path: tmp.path().to_path_buf(),
        ..Default::default()
    };
    let lsm = Arc::new(LsmStorage::new(lsm_config).await?);
    let fail_delete = Arc::new(AtomicBool::new(false));
    let faulty_storage = FaultyDeleteStorage {
        inner: lsm,
        fail_delete: fail_delete.clone(),
    };

    let dim = 4;
    let hnsw_config = HnswConfig {
        dimension: dim,
        ..Default::default()
    };
    let hnsw = Arc::new(HnswIndex::try_new(hnsw_config)?);
    let graph = Arc::new(CsrGraph::with_storage(Arc::new(faulty_storage.clone())));
    let next_tx = Arc::new(AtomicU64::new(1));

    let col = Collection::new(
        "test_col".to_string(),
        Arc::new(faulty_storage),
        hnsw,
        graph,
        next_tx,
        dim,
        contextra_engine::Language::English,
    );

    // 1. Insert source document
    let src_id_str = "source_doc_1";
    let src_doc_id = DocId::from_key(src_id_str)?;
    col.insert(
        src_id_str,
        &[0.1, 0.2, 0.3, 0.4],
        Some(serde_json::json!({"text": "source text"})),
    )
    .await?;

    // 2. Start consolidation session
    let target_str_id = "summary_target_doc";
    let target_doc_id = DocId::from_key(target_str_id)?;
    let session = ConsolidationSession::start(&col, &[src_doc_id], target_doc_id).await?;

    // 3. Configure delete_op to fail via storage delete failure
    fail_delete.store(true, Ordering::SeqCst);

    // 4. Call commit
    let res = session
        .commit(
            target_str_id,
            &[0.1, 0.2, 0.3, 0.4],
            "Summary of source doc",
            None,
        )
        .await;

    // 5. Assert commit returned Err
    assert!(
        res.is_err(),
        "commit() must return Err when delete_op fails"
    );

    // 6. Assert summary target doc was not persisted
    let summary_doc = col.get(target_str_id).await?;
    assert!(
        summary_doc.is_none(),
        "Target summary document must not be persisted if commit aborts"
    );

    Ok(())
}

#[tokio::test]
async fn test_cleanup_orphaned_consolidation_intents_uses_next_tx_allocator_monotonically(
) -> std::result::Result<(), Box<dyn std::error::Error>> {
    let tmp = tempfile::tempdir()?;
    let lsm_config = LsmConfig {
        path: tmp.path().to_path_buf(),
        ..Default::default()
    };
    let lsm = Arc::new(LsmStorage::new(lsm_config).await?);
    let next_tx = Arc::new(AtomicU64::new(10));

    let target_doc_id = DocId::new(42);
    let intent = CommitIntent::Consolidation {
        source_docs: vec![(DocId::new(1), TxId::new(1))],
        target_id: target_doc_id,
        base_tx: TxId::new(5),
    };
    let intent_bytes = serde_json::to_vec(&intent)?;

    let setup_tx = TxId::new(1);
    lsm.put(setup_tx, b"consolidation_intent:1", &intent_bytes)
        .await?;
    lsm.commit(setup_tx).await?;
    lsm.put(setup_tx, b"consolidation_intent:2", &intent_bytes)
        .await?;
    lsm.commit(setup_tx).await?;

    let prev_counter = next_tx.load(Ordering::SeqCst);
    let cleaned = cleanup_orphaned_consolidation_intents(lsm.as_ref(), &next_tx).await?;
    assert_eq!(cleaned, 2, "Should clean up exactly 2 orphaned intents");

    let post_counter = next_tx.load(Ordering::SeqCst);
    assert_eq!(
        post_counter,
        prev_counter + 2,
        "next_tx counter must increment monotonically by the number of cleaned intents"
    );

    let remaining1 = lsm.get(b"consolidation_intent:1").await?;
    let remaining2 = lsm.get(b"consolidation_intent:2").await?;
    assert!(remaining1.is_none());
    assert!(remaining2.is_none());

    Ok(())
}

#[tokio::test]
async fn test_concurrent_mutation_aborts_consolidation() -> Result<()> {
    let tmp = tempfile::tempdir().unwrap();
    let lsm_config = LsmConfig {
        path: tmp.path().to_path_buf(),
        ..Default::default()
    };
    let storage = Arc::new(LsmStorage::new(lsm_config).await.unwrap());
    let index = Arc::new(
        HnswIndex::try_new(HnswConfig {
            dimension: 4,
            ..Default::default()
        })
        .unwrap(),
    );
    let col = Collection::new(
        "default".to_string(),
        storage,
        index,
        Arc::new(CsrGraph::new()),
        Arc::new(AtomicU64::new(1)),
        4,
        contextra_engine::Language::English,
    );

    col.insert(
        "source_1",
        &[1.0, 0.0, 0.0, 0.0],
        Some(serde_json::json!({"text": "Fact 1"})),
    )
    .await?;
    col.insert(
        "source_2",
        &[0.0, 1.0, 0.0, 0.0],
        Some(serde_json::json!({"text": "Fact 2"})),
    )
    .await?;

    let d1 = DocId::from_key("source_1")?;
    let d2 = DocId::from_key("source_2")?;
    let target_id = DocId::from_key("summary_12")?;

    let session = ConsolidationSession::start(&col, &[d1, d2], target_id).await?;

    col.update(
        "source_2",
        &[0.0, 1.0, 0.0, 0.0],
        Some(serde_json::json!({"text": "Fact 2 updated by agent"})),
    )
    .await?;

    let commit_res = session
        .commit(
            "summary_12",
            &[0.5, 0.5, 0.0, 0.0],
            "Summary of 1 and 2",
            None,
        )
        .await;
    assert!(
        commit_res.is_err(),
        "Consolidation commit must fail under concurrent mutation"
    );
    match commit_res.unwrap_err() {
        ContextraError::StaleRead(msg) => {
            assert!(msg.contains("OCC conflict"));
        }
        other => panic!("Expected StaleRead error, got: {:?}", other),
    }

    let doc1 = col.get("source_1").await?;
    let doc2 = col.get("source_2").await?;
    assert!(
        doc1.is_some(),
        "source_1 must not be deleted on aborted consolidation"
    );
    assert!(
        doc2.is_some(),
        "source_2 must not be deleted on aborted consolidation"
    );

    Ok(())
}
