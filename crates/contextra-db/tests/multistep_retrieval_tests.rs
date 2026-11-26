use contextra_db::multistep::{MultiStepConfig, MultiStepEngine};
use contextra_db::{Collection, QueryRewriter};
use contextra_graph::CsrGraph;
use contextra_ports::BoxFuture;
use contextra_store::{LsmConfig, LsmStorage};
use contextra_types::{Result, ScoredEntry};
use contextra_vector::{HnswConfig, HnswIndex};
use std::sync::atomic::AtomicU64;
use std::sync::Arc;
use tempfile::tempdir;

// Test-Rewriter, der vordefinierte Sub-Queries zurückgibt.
struct DummyRewriter {
    responses: std::sync::Mutex<Vec<Vec<String>>>,
}

impl QueryRewriter for DummyRewriter {
    fn rewrite<'a>(
        &'a self,
        _original_query: &'a str,
        _current_results: &'a [ScoredEntry],
    ) -> BoxFuture<'a, Result<Vec<String>>> {
        Box::pin(async move {
            let mut guard = self.responses.lock().unwrap();
            if !guard.is_empty() {
                Ok(guard.remove(0))
            } else {
                Ok(vec![])
            }
        })
    }
}

// Hilfsfunktion: Erzeugt eine Test-Kollektion mit In-Memory/TempDir-LSM und HNSW.
async fn create_test_collection() -> Arc<Collection<LsmStorage>> {
    let dir = tempdir().expect("tempdir");
    let lsm_config = LsmConfig {
        path: dir.path().to_path_buf(),
        ..Default::default()
    };
    let storage = Arc::new(LsmStorage::new(lsm_config).await.expect("lsm storage"));
    let hnsw_config = HnswConfig {
        dimension: 4,
        ..Default::default()
    };
    let index = Arc::new(HnswIndex::try_new(hnsw_config).expect("hnsw index"));
    let graph = Arc::new(CsrGraph::new());
    let next_tx = Arc::new(AtomicU64::new(1));

    let col = Collection::new(
        "default".to_string(),
        storage,
        index,
        graph,
        next_tx,
        4,
        contextra_text::Language::English,
    );

    Arc::new(col)
}

// Prüft, dass bei ausreichender Qualität in Runde 1 keine weiteren Runden gestartet werden.
#[tokio::test]
async fn test_multistep_single_round_sufficient() {
    let col = create_test_collection().await;
    col.insert(
        "doc1",
        &[1.0, 0.0, 0.0, 0.0],
        Some(serde_json::json!({"text": "rust programming"})),
    )
    .await
    .expect("insert");
    col.insert(
        "doc2",
        &[0.9, 0.1, 0.0, 0.0],
        Some(serde_json::json!({"text": "rust language"})),
    )
    .await
    .expect("insert");

    let config = MultiStepConfig {
        max_rounds: 3,
        quality_threshold: 0.001,
        min_quality_hits: 1,
        latency_budget_ms: 1000.0,
        ..Default::default()
    };
    let engine = MultiStepEngine::new(col, config);

    let rewriter = DummyRewriter {
        responses: std::sync::Mutex::new(vec![vec!["sub query 1".to_string()]]),
    };

    let result = engine
        .search("rust", &[1.0, 0.0, 0.0, 0.0], 5, Some(&rewriter))
        .await
        .expect("search");

    assert_eq!(result.rounds_executed, 1);
    assert!(result.sub_queries.is_empty());
    assert!(!result.results.is_empty());
}

// Prüft, dass bei zu geringer Qualität in Runde 1 eine Reformulierung via Rewriter ausgelöst wird.
#[tokio::test]
async fn test_multistep_query_rewriting_triggers() {
    let col = create_test_collection().await;
    col.insert(
        "doc1",
        &[1.0, 0.0, 0.0, 0.0],
        Some(serde_json::json!({"text": "rust programming"})),
    )
    .await
    .expect("insert");

    let config = MultiStepConfig {
        max_rounds: 3,
        quality_threshold: 0.99,
        min_quality_hits: 2,
        latency_budget_ms: 1000.0,
        ..Default::default()
    };
    let engine = MultiStepEngine::new(col, config);

    let rewriter = DummyRewriter {
        responses: std::sync::Mutex::new(vec![vec!["rust programming".to_string()], vec![]]),
    };

    let result = engine
        .search("rust", &[1.0, 0.0, 0.0, 0.0], 5, Some(&rewriter))
        .await
        .expect("search");

    assert_eq!(result.rounds_executed, 2);
    assert_eq!(result.sub_queries, vec!["rust programming"]);
    assert!(!result.results.is_empty());
}

// Prüft das Verhalten, wenn kein QueryRewriter bereitgestellt wird (nur 1 Runde).
#[tokio::test]
async fn test_multistep_no_rewriter_provided() {
    let col = create_test_collection().await;
    let config = MultiStepConfig::default();
    let engine = MultiStepEngine::new(col, config);

    let result = engine
        .search("query", &[1.0, 0.0, 0.0, 0.0], 5, None)
        .await
        .expect("search");

    assert_eq!(result.rounds_executed, 1);
    assert!(result.sub_queries.is_empty());
}

// Prüft, dass Sub-Queries bei Bedarf auch nur BM25-Volltext verwenden.
#[tokio::test]
async fn test_multistep_subquery_uses_bm25_only() {
    let col = create_test_collection().await;
    col.insert(
        "doc1",
        &[1.0, 0.0, 0.0, 0.0],
        Some(serde_json::json!({"text": "rust programming language"})),
    )
    .await
    .expect("insert");

    let config = MultiStepConfig {
        max_rounds: 2,
        quality_threshold: 0.99,
        min_quality_hits: 2,
        latency_budget_ms: 1000.0,
        ..Default::default()
    };
    let engine = MultiStepEngine::new(col, config);

    let rewriter = DummyRewriter {
        responses: std::sync::Mutex::new(vec![vec!["rust programming".to_string()]]),
    };

    let result = engine
        .search("original", &[0.1, 0.2, 0.3, 0.4], 5, Some(&rewriter))
        .await
        .expect("search");

    assert_eq!(result.rounds_executed, 2);
    assert_eq!(result.sub_queries, vec!["rust programming"]);
    assert!(!result.results.is_empty());
}

// Test-Rewriter, der absichtlich Fehler meldet.
struct FailingRewriter;

impl QueryRewriter for FailingRewriter {
    fn rewrite<'a>(
        &'a self,
        _original_query: &'a str,
        _current_results: &'a [ScoredEntry],
    ) -> BoxFuture<'a, Result<Vec<String>>> {
        Box::pin(async move {
            Err(contextra_types::ContextraError::Internal(
                "Rewriter error".into(),
            ))
        })
    }
}

// Prüft, dass bei überschrittenem Latenzbudget Teilergebnisse ohne Abbruch zurückgegeben werden.
#[tokio::test]
async fn test_multistep_latency_budget_exceeded_returns_partial_result() {
    let col = create_test_collection().await;
    col.insert(
        "doc1",
        &[1.0, 0.0, 0.0, 0.0],
        Some(serde_json::json!({"text": "rust programming"})),
    )
    .await
    .expect("insert");

    let config = MultiStepConfig {
        max_rounds: 3,
        quality_threshold: 0.99,
        min_quality_hits: 2,
        latency_budget_ms: 0.0001,
        ..Default::default()
    };
    let engine = MultiStepEngine::new(col, config);

    let rewriter = DummyRewriter {
        responses: std::sync::Mutex::new(vec![vec!["sub query 1".to_string()]]),
    };

    tokio::time::sleep(std::time::Duration::from_millis(10)).await;

    let result = engine
        .search("rust", &[1.0, 0.0, 0.0, 0.0], 5, Some(&rewriter))
        .await
        .expect("search should succeed gracefully with partial results");

    assert_eq!(result.rounds_executed, 1);
    assert!(result.sub_queries.is_empty());
    assert!(!result.results.is_empty());
}

// Prüft, dass ein scheiternder Rewriter auf die Originalquery zurückfällt und keinen Absturz verursacht.
#[tokio::test]
async fn test_multistep_failing_rewriter_gracefully_stops() {
    let col = create_test_collection().await;
    col.insert(
        "doc1",
        &[1.0, 0.0, 0.0, 0.0],
        Some(serde_json::json!({"text": "rust programming"})),
    )
    .await
    .expect("insert");

    let config = MultiStepConfig {
        max_rounds: 3,
        quality_threshold: 0.99,
        min_quality_hits: 2,
        latency_budget_ms: 1000.0,
        ..Default::default()
    };
    let engine = MultiStepEngine::new(col, config);
    let rewriter = FailingRewriter;

    let result = engine
        .search("rust", &[1.0, 0.0, 0.0, 0.0], 5, Some(&rewriter))
        .await
        .expect("search should succeed gracefully even if rewriter fails");

    assert_eq!(result.rounds_executed, 3);
    assert_eq!(result.sub_queries, vec!["rust", "rust"]);
}

// Test-Rewriter mit künstlicher Verzögerung.
struct SlowRewriter;

impl QueryRewriter for SlowRewriter {
    fn rewrite<'a>(
        &'a self,
        _original_query: &'a str,
        _current_results: &'a [ScoredEntry],
    ) -> BoxFuture<'a, Result<Vec<String>>> {
        Box::pin(async move {
            tokio::time::sleep(std::time::Duration::from_millis(20)).await;
            Ok(vec!["sub query 1".to_string()])
        })
    }
}

// Prüft die Latenzbudget-Schranke bei langsamem Rewriting.
#[tokio::test]
async fn test_multistep_latency_budget_guard_exceeded() {
    let col = create_test_collection().await;
    col.insert(
        "doc1",
        &[1.0, 0.0, 0.0, 0.0],
        Some(serde_json::json!({"text": "rust programming"})),
    )
    .await
    .expect("insert");

    let config = MultiStepConfig {
        max_rounds: 5,
        quality_threshold: 0.99,
        min_quality_hits: 2,
        latency_budget_ms: 10.0,
        ..Default::default()
    };
    let engine = MultiStepEngine::new(col, config);
    let rewriter = SlowRewriter;

    let result = engine
        .search("rust", &[1.0, 0.0, 0.0, 0.0], 5, Some(&rewriter))
        .await
        .expect("search should succeed with partial results under budget exhaustion");

    assert_eq!(result.rounds_executed, 2);
    assert!(!result.results.is_empty());
}

// Prüft Latenzbudget-Erschöpfung auf einer leeren Datenbank.
#[tokio::test]
async fn test_multistep_latency_budget_exhaustion_on_empty_db() {
    let col = create_test_collection().await;
    let config = MultiStepConfig {
        max_rounds: 5,
        quality_threshold: 0.99,
        min_quality_hits: 2,
        latency_budget_ms: 1.0,
        ..Default::default()
    };
    let engine = MultiStepEngine::new(col, config);
    let rewriter = SlowRewriter;

    let result = engine
        .search("nonexistent", &[1.0, 0.0, 0.0, 0.0], 5, Some(&rewriter))
        .await
        .expect("search on empty DB over budget must return empty result without panic");

    assert!(result.results.is_empty());
}
