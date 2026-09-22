use criterion::{criterion_group, criterion_main, Criterion};
use memfuse_core::{EntityId, StorageEngine, TokenBudget};
use memfuse_db::{MemFuse, MemFuseConfig};
use memfuse_router::{RouterEngine, SlmProfile};
use serde_json::json;
use std::sync::Arc;
use tokio::runtime::Runtime;

fn bench_router_engine(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    let (collection, db, _dir) = rt.block_on(async {
        let dir = tempfile::tempdir().unwrap();
        let config = MemFuseConfig {
            dimension: 4,
            ..Default::default()
        };
        let db = MemFuse::open_with_config(dir.path(), config).await.unwrap();
        let collection = db.collection("default").await.unwrap();

        for i in 0..10 {
            let key = format!("doc_{i}");
            let vec = vec![(i as f32) * 0.1, 0.5, 0.2, 0.1];
            collection
                .insert(
                    &key,
                    &vec,
                    Some(json!({"text": format!("Document text content for entity {i}")})),
                )
                .await
                .unwrap();

            let eid = EntityId::from_key(&key).unwrap();
            let tx = db.allocate_tx().unwrap();
            let comm_key = format!("__graph:community:{}", eid.inner()).into_bytes();
            let comm_id = (i as u64) % 5 + 1;
            db.inner_storage()
                .put(tx, &comm_key, &serde_json::to_vec(&comm_id).unwrap())
                .await
                .unwrap();
            db.inner_storage().commit(tx).await.unwrap();
        }

        (collection, db, dir)
    });

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

        struct BenchCollectionAdapter<S: StorageEngine + 'static> {
            collection: Arc<memfuse_db::Collection<S>>,
        }

        impl<S: StorageEngine + 'static> memfuse_ports::HybridSearchProvider for BenchCollectionAdapter<S> {
            fn search_hybrid<'a>(
                &'a self,
                query_text: &'a str,
                query_embedding: &'a [f32],
                top_k: usize,
            ) -> memfuse_core::BoxFuture<'a, memfuse_core::Result<Vec<memfuse_core::ContextChunk>>>
            {
                Box::pin(async move {
                    let search_results = self
                        .collection
                        .query()
                        .text(query_text)
                        .embedding(query_embedding)
                        .k(top_k)
                        .execute()
                        .await?;

                    let mut chunks = Vec::with_capacity(search_results.len());
                    for res in search_results {
                        if let Ok(mut chunk) = memfuse_core::ContextChunk::try_from(res) {
                            chunk.content = chunk.combined_text_owned();
                            chunks.push(chunk);
                        }
                    }
                    Ok(chunks)
                })
            }
        }

        impl<S: StorageEngine + 'static> memfuse_ports::CommunityResolver for BenchCollectionAdapter<S> {
            fn get_community<'a>(
                &'a self,
                entity_id: EntityId,
            ) -> memfuse_core::BoxFuture<'a, memfuse_core::Result<Option<u64>>> {
                Box::pin(async move { self.collection.get_community(entity_id).await })
            }
        }

        struct BenchContextPreparer;

        impl memfuse_ports::ContextPreparer for BenchContextPreparer {
            fn prepare_context(
                &self,
                chunks: Vec<memfuse_core::ContextChunk>,
                budget: &TokenBudget,
                relevance_threshold: f32,
            ) -> memfuse_core::Result<memfuse_core::ContextWindow> {
                let mut manager = memfuse_db::context::ContextManager::new(budget.clone());
                manager.set_relevance_threshold(relevance_threshold);
                manager.prepare_context(chunks)
            }
        }

        let adapter = Arc::new(BenchCollectionAdapter {
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

    drop(db);
}

criterion_group!(benches, bench_router_engine);
criterion_main!(benches);
