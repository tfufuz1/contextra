//! # Benchmark für den Durchsatz der RouterEngine
//!
//! **Ring:** Ring 3 (Applikations- und Testschicht).
//! **Zweck:** Benchmark für den Durchsatz der RouterEngine.

#![allow(
    clippy::type_complexity,
    clippy::cast_sign_loss,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    dead_code
)]

use contextra_ports::{BoxFuture, CommunityResolver, ContextPreparer, HybridSearchProvider};
use contextra_router::{RouterEngine, SlmProfile};
use contextra_types::{ContextChunk, ContextWindow, EntityId, Result, TokenBudget};
use criterion::{criterion_group, criterion_main, Criterion};
use std::sync::Arc;
use tokio::runtime::Runtime;

#[derive(Clone, Default)]
struct BenchCollection {
    docs: Arc<Vec<(String, Vec<f32>, String, u64)>>,
}

impl BenchCollection {
    fn new() -> Self {
        let mut docs = Vec::new();
        for i in 0..10 {
            let key = format!("doc_{i}");
            let vec = vec![(i as f32) * 0.1, 0.5, 0.2, 0.1];
            let text = format!("Document text content for entity {i}");
            let comm_id = (i as u64 % 5) + 1;
            docs.push((key, vec, text, comm_id));
        }
        Self {
            docs: Arc::new(docs),
        }
    }
}

struct BenchAdapter {
    collection: BenchCollection,
}

impl HybridSearchProvider for BenchAdapter {
    fn search_hybrid<'a>(
        &'a self,
        query_text: &'a str,
        query_embedding: &'a [f32],
        top_k: usize,
    ) -> BoxFuture<'a, Result<Vec<ContextChunk>>> {
        let docs = self.collection.docs.clone();
        Box::pin(async move {
            let mut scored: Vec<(f32, &str, &str)> = docs
                .iter()
                .map(|(key, vec, text, _comm_id)| {
                    let mut score = 0.0f32;
                    if !query_embedding.is_empty() && query_embedding.len() == vec.len() {
                        let dot: f32 = query_embedding
                            .iter()
                            .zip(vec.iter())
                            .map(|(a, b)| a * b)
                            .sum();
                        score += dot.clamp(0.0, 1.0);
                    }
                    if !query_text.is_empty() && text.contains(query_text) {
                        score += 0.5;
                    }
                    (score.clamp(0.0, 1.0), key.as_str(), text.as_str())
                })
                .collect();

            scored.sort_by(|a, b| b.0.total_cmp(&a.0));

            let mut chunks = Vec::new();
            for (i, (score, _key, text)) in scored.into_iter().take(top_k).enumerate() {
                chunks.push(ContextChunk {
                    doc_id: contextra_types::DocId::new((i + 1) as u64),
                    content: text.to_string(),
                    relevance: score,
                    token_count: 5,
                    metadata: None,
                    contextual_prefix: None,
                    links: vec![],
                });
            }
            Ok(chunks)
        })
    }
}

impl CommunityResolver for BenchAdapter {
    fn get_community<'a>(&'a self, entity_id: EntityId) -> BoxFuture<'a, Result<Option<u64>>> {
        let comm_id = (entity_id.inner() % 5) + 1;
        Box::pin(async move { Ok(Some(comm_id)) })
    }
}

struct BenchContextPreparer;

impl ContextPreparer for BenchContextPreparer {
    fn prepare_context(
        &self,
        chunks: Vec<ContextChunk>,
        budget: &TokenBudget,
        relevance_threshold: f32,
    ) -> Result<ContextWindow> {
        let mut filtered: Vec<ContextChunk> = chunks
            .into_iter()
            .filter(|c| c.relevance >= relevance_threshold)
            .collect();
        filtered.sort_by(|a, b| b.relevance.total_cmp(&a.relevance));

        let mut total_tokens = 0;
        let mut selected = Vec::new();
        let mut truncated = false;

        for chunk in filtered {
            if total_tokens + chunk.token_count <= budget.effective_limit() {
                total_tokens += chunk.token_count;
                selected.push(chunk);
            } else {
                truncated = true;
                break;
            }
        }

        Ok(ContextWindow {
            chunks: selected,
            total_tokens,
            truncated,
        })
    }
}

fn bench_router_engine(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let collection = BenchCollection::new();

    let query_vec = vec![0.5, 0.5, 0.2, 0.1];
    let query_text = "Document text content";

    for profile_count in [1, 10, 50, 500] {
        let mut profiles = Vec::with_capacity(profile_count);
        for i in 0..profile_count {
            profiles.push(SlmProfile::new(
                format!("slm-profile-{i}"),
                format!("http://127.0.0.1:9090/mcp/{i}"),
                vec![(i as u64 % 5) + 1],
                TokenBudget::new(1000, 100),
                0.01,
            ));
        }

        let adapter = Arc::new(BenchAdapter {
            collection: collection.clone(),
        });
        let preparer = Arc::new(BenchContextPreparer);

        let router = Arc::new(RouterEngine::new(
            adapter.clone(),
            adapter,
            preparer,
            profiles,
            None,
        ));

        c.bench_function(&format!("router_route_{}_profiles", profile_count), |b| {
            b.to_async(&rt).iter(|| {
                let router = router.clone();
                let query_vec = query_vec.clone();
                async move {
                    let _res = router.route(&query_vec, query_text).await;
                }
            });
        });
    }
}

criterion_group!(benches, bench_router_engine);
criterion_main!(benches);
