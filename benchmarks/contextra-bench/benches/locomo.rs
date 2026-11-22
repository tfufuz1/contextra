use std::path::PathBuf;
use std::time::Duration;

use contextra_bench::locomo::{load_locomo_dataset, run_locomo_eval};
use contextra_bench::long_mem_eval::ScoredChunk;
use contextra_db::{Contextra, ContextraConfig};
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use tempfile::TempDir;
use tokio::runtime::Runtime;

fn get_fixture_path(relative: &str) -> PathBuf {
    if let Ok(manifest_dir) = std::env::var("CARGO_MANIFEST_DIR") {
        let p = PathBuf::from(manifest_dir).join(relative);
        if p.exists() {
            return p;
        }
    }
    let p = PathBuf::from("benchmarks/contextra-bench").join(relative);
    if p.exists() {
        return p;
    }
    PathBuf::from(relative)
}

fn ensure_fd_limit(min_fds: u64) {
    contextra_sys::ensure_fd_limit(min_fds);
}

fn bench_locomo_eval(c: &mut Criterion) {
    ensure_fd_limit(65536);
    let rt = Runtime::new().expect("Tokio runtime creation failed");
    let mut group = c.benchmark_group("locomo");
    group.sample_size(10);
    group.measurement_time(Duration::from_secs(5));

    let fixture_path = get_fixture_path("tests/fixtures/locomo_fixture.json");
    let dataset =
        load_locomo_dataset(&fixture_path).expect("Failed to load LoCoMo fixture dataset");

    group.bench_function("hybrid_retrieval_fixture", |b| {
        b.to_async(&rt).iter_custom(|iters| {
            let cases = &dataset.cases;
            let conversation_index = &dataset.conversation_index;
            async move {
                let mut total_duration = std::time::Duration::ZERO;
                for _ in 0..iters {
                    let db_cfg = ContextraConfig {
                        dimension: 768,
                        ..Default::default()
                    };
                    let temp_dir = TempDir::new().expect("TempDir failed");
                    let db = Contextra::open_with_config(temp_dir.path(), db_cfg)
                        .await
                        .expect("Contextra open failed");
                    let col = db
                        .collection("locomo_bench_col")
                        .await
                        .expect("Collection creation failed");

                    let dummy_vec = vec![0.5f32; 768];
                    let mut batch = Vec::new();
                    for (case_idx, case) in cases.iter().enumerate() {
                        for (ev_idx, ev) in case.evidence.iter().enumerate() {
                            let doc_id = format!("locomo_doc_{}_{}", case_idx, ev_idx);
                            let metadata = serde_json::json!({
                                "text": ev,
                                "case_id": case.question_id,
                                "sample_id": case.sample_id,
                            });
                            batch.push((doc_id, dummy_vec.clone(), Some(metadata)));
                        }
                    }
                    if !batch.is_empty() {
                        col.insert_many(&batch).await.expect("insert_many failed");
                    }

                    let start = std::time::Instant::now();
                    let report = run_locomo_eval(cases, conversation_index, |q| {
                        let q_owned = q.to_string();
                        let col_ref = &col;
                        Box::pin(async move {
                            let res = col_ref.query().text(&q_owned).k(5).execute().await?;
                            let chunks = res
                                .into_iter()
                                .map(|r| {
                                    let text = r
                                        .metadata
                                        .as_ref()
                                        .and_then(|m| m.get("text"))
                                        .and_then(|v| v.as_str())
                                        .unwrap_or_default()
                                        .to_string();
                                    ScoredChunk {
                                        id: r.id,
                                        text,
                                        score: r.score,
                                    }
                                })
                                .collect();
                            Ok(chunks)
                        })
                    })
                    .await
                    .expect("run_locomo_eval failed");

                    total_duration += start.elapsed();
                    let _ = db.close().await;
                    black_box(report);
                }
                total_duration
            }
        });
    });

    group.finish();
}

criterion_group!(benches, bench_locomo_eval);
criterion_main!(benches);
