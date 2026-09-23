// FILE-CONTEXT
// STAND: 2026-09-15
// ZWECK: BEIR (Benchmarking Information Retrieval) Evaluation für Contextra BM25 + Hybrid Search.

use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;
use std::time::Instant;

use anyhow::{Context, Result};
use hdrhistogram::Histogram;
use contextra_db::{Contextra, ContextraConfig};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BeirDocument {
    pub id: String,
    pub title: String,
    pub text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BeirQuery {
    pub id: String,
    pub query: String,
    pub relevant_docs: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BeirMetrics {
    pub dataset_name: String,
    pub num_docs: usize,
    pub num_queries: usize,
    pub ndcg_at_10: f64,
    pub recall_at_10: f64,
    pub map_at_100: f64,
    pub latency_p50_ms: f64,
    pub latency_p99_ms: f64,
}

/// Lädt BEIR-Corpus aus JSONL-Datei
pub fn load_beir_corpus(path: &Path) -> Result<Vec<BeirDocument>> {
    let file = File::open(path)
        .with_context(|| format!("Konnte Corpus-Datei nicht öffnen: {:?}", path))?;
    let reader = BufReader::new(file);
    let mut docs = Vec::new();

    for line in reader.lines() {
        let l = line?;
        if l.trim().is_empty() {
            continue;
        }
        let val: serde_json::Value = serde_json::from_str(&l)?;
        let id = val
            .get("_id")
            .or_else(|| val.get("id"))
            .and_then(|v| v.as_str())
            .unwrap_or_default()
            .to_string();
        let title = val
            .get("title")
            .and_then(|v| v.as_str())
            .unwrap_or_default()
            .to_string();
        let text = val
            .get("text")
            .and_then(|v| v.as_str())
            .unwrap_or_default()
            .to_string();

        if !id.is_empty() {
            docs.push(BeirDocument { id, title, text });
        }
    }

    Ok(docs)
}

/// Lädt BEIR-Queries + QRels aus JSONL / TSV-Dateien
pub fn load_beir_queries(queries_path: &Path, qrels_path: &Path) -> Result<Vec<BeirQuery>> {
    let q_file = File::open(queries_path)
        .with_context(|| format!("Konnte Queries-Datei nicht öffnen: {:?}", queries_path))?;
    let q_reader = BufReader::new(q_file);
    let mut queries_map: HashMap<String, String> = HashMap::new();

    for line in q_reader.lines() {
        let l = line?;
        if l.trim().is_empty() {
            continue;
        }
        let val: serde_json::Value = serde_json::from_str(&l)?;
        let id = val
            .get("_id")
            .or_else(|| val.get("id"))
            .and_then(|v| v.as_str())
            .unwrap_or_default()
            .to_string();
        let query = val
            .get("text")
            .or_else(|| val.get("query"))
            .and_then(|v| v.as_str())
            .unwrap_or_default()
            .to_string();

        if !id.is_empty() {
            queries_map.insert(id, query);
        }
    }

    // Load QRels (support both JSONL and TSV)
    let qr_file = File::open(qrels_path)
        .with_context(|| format!("Konnte QRels-Datei nicht öffnen: {:?}", qrels_path))?;
    let qr_reader = BufReader::new(qr_file);
    let mut qrels: HashMap<String, Vec<String>> = HashMap::new();

    for line in qr_reader.lines() {
        let l = line?;
        let trimmed = l.trim();
        if trimmed.is_empty() {
            continue;
        }

        if trimmed.starts_with('{') {
            // JSONL format
            if let Ok(val) = serde_json::from_str::<serde_json::Value>(trimmed) {
                let qid = val
                    .get("qid")
                    .or_else(|| val.get("query-id"))
                    .and_then(|v| v.as_str())
                    .unwrap_or_default()
                    .to_string();
                let docid = val
                    .get("docid")
                    .or_else(|| val.get("corpus-id"))
                    .and_then(|v| v.as_str())
                    .unwrap_or_default()
                    .to_string();
                let rel = val
                    .get("relevance")
                    .or_else(|| val.get("score"))
                    .and_then(|v| v.as_i64())
                    .unwrap_or(0);

                if rel > 0 && !qid.is_empty() && !docid.is_empty() {
                    qrels.entry(qid).or_default().push(docid);
                }
            }
        } else {
            // TSV format: query-id corpus-id score (or query-id \t 0 \t corpus-id \t score)
            let parts: Vec<&str> = trimmed.split_whitespace().collect();
            if parts.len() >= 3 {
                let (qid, docid, score_str) = if parts.len() >= 4 {
                    (parts[0], parts[2], parts[3])
                } else {
                    (parts[0], parts[1], parts[2])
                };

                let rel: i64 = score_str.parse().unwrap_or(0);
                if rel > 0 {
                    qrels
                        .entry(qid.to_string())
                        .or_default()
                        .push(docid.to_string());
                }
            }
        }
    }

    let mut result = Vec::new();
    for (qid, query_text) in queries_map {
        if let Some(relevant_docs) = qrels.remove(&qid) {
            result.push(BeirQuery {
                id: qid,
                query: query_text,
                relevant_docs,
            });
        }
    }

    Ok(result)
}

/// Berechnet NDCG@k aus Ranked Result List und Relevance Set (strikt log2-Standard)
pub fn ndcg_at_k(ranked: &[String], relevant: &HashSet<String>, k: usize) -> f64 {
    if relevant.is_empty() || k == 0 {
        return 0.0;
    }

    let limit = k.min(ranked.len());
    let mut dcg = 0.0;
    for i in 0..limit {
        if relevant.contains(&ranked[i]) {
            let rank = (i + 1) as f64;
            dcg += 1.0 / (rank + 1.0).log2();
        }
    }

    let idcg_limit = k.min(relevant.len());
    let mut idcg = 0.0;
    for i in 0..idcg_limit {
        let rank = (i + 1) as f64;
        idcg += 1.0 / (rank + 1.0).log2();
    }

    if idcg > 0.0 {
        dcg / idcg
    } else {
        0.0
    }
}

/// Berechnet MAP@k
pub fn map_at_k(ranked: &[String], relevant: &HashSet<String>, k: usize) -> f64 {
    if relevant.is_empty() || k == 0 {
        return 0.0;
    }

    let limit = k.min(ranked.len());
    let mut hits = 0usize;
    let mut sum_precision = 0.0;

    for i in 0..limit {
        if relevant.contains(&ranked[i]) {
            hits += 1;
            sum_precision += (hits as f64) / ((i + 1) as f64);
        }
    }

    let denom = k.min(relevant.len());
    if denom > 0 {
        sum_precision / (denom as f64)
    } else {
        0.0
    }
}

/// Berechnet Recall@k
pub fn recall_at_k(ranked: &[String], relevant: &HashSet<String>, k: usize) -> f64 {
    if relevant.is_empty() || k == 0 {
        return 0.0;
    }

    let limit = k.min(ranked.len());
    let mut hits = 0usize;
    for item in &ranked[..limit] {
        if relevant.contains(item) {
            hits += 1;
        }
    }

    let denom = k.min(relevant.len());
    if denom > 0 {
        (hits as f64) / (denom as f64)
    } else {
        0.0
    }
}

/// Führt vollständige BEIR-Evaluation auf einer Contextra-Collection durch
pub async fn run_beir_eval(
    db_path: &Path,
    corpus: &[BeirDocument],
    queries: &[BeirQuery],
    dataset_name: &str,
    use_hybrid: bool,
) -> Result<BeirMetrics> {
    let db_cfg = ContextraConfig {
        dimension: 768,
        ..Default::default()
    };

    let db = Contextra::open_with_config(db_path, db_cfg).await?;
    let col = db.collection(&format!("beir_{}", dataset_name)).await?;

    let dummy_vec = vec![0.0f32; 768];
    let mut batch = Vec::new();

    for doc in corpus {
        let content = if doc.title.is_empty() {
            doc.text.clone()
        } else {
            format!("{} {}", doc.title, doc.text)
        };
        let metadata = serde_json::json!({
            "title": doc.title,
            "text": content,
        });
        batch.push((doc.id.clone(), dummy_vec.clone(), Some(metadata)));

        if batch.len() >= 100 {
            col.insert_many(&batch).await?;
            batch.clear();
        }
    }
    if !batch.is_empty() {
        col.insert_many(&batch).await?;
    }

    let mut hist = Histogram::<u64>::new_with_bounds(1, 60_000_000, 3)?;
    let mut ndcg_sum = 0.0;
    let mut recall_sum = 0.0;
    let mut map_sum = 0.0;

    for q in queries {
        let relevant_set: HashSet<String> = q.relevant_docs.iter().cloned().collect();
        let start = Instant::now();

        let search_res = if use_hybrid {
            col.query()
                .text(&q.query)
                .embedding(&dummy_vec)
                .k(100)
                .execute()
                .await?
        } else {
            col.query().text(&q.query).k(100).execute().await?
        };

        let elapsed_us = start.elapsed().as_micros() as u64;
        let _ = hist.record(elapsed_us.max(1));

        let retrieved_ids: Vec<String> = search_res.into_iter().map(|r| r.id).collect();

        ndcg_sum += ndcg_at_k(&retrieved_ids, &relevant_set, 10);
        recall_sum += recall_at_k(&retrieved_ids, &relevant_set, 10);
        map_sum += map_at_k(&retrieved_ids, &relevant_set, 100);
    }

    let num_q = queries.len().max(1) as f64;
    let metrics = BeirMetrics {
        dataset_name: dataset_name.to_string(),
        num_docs: corpus.len(),
        num_queries: queries.len(),
        ndcg_at_10: ndcg_sum / num_q,
        recall_at_10: recall_sum / num_q,
        map_at_100: map_sum / num_q,
        latency_p50_ms: (hist.value_at_quantile(0.50) as f64) / 1000.0,
        latency_p99_ms: (hist.value_at_quantile(0.99) as f64) / 1000.0,
    };

    // Save to benchmarks/results/beir_results.json per APM-5
    let results_dir = Path::new("benchmarks/results");
    if !results_dir.exists() {
        std::fs::create_dir_all(results_dir)?;
    }
    let json = serde_json::to_string_pretty(&metrics)?;
    std::fs::write(results_dir.join("beir_results.json"), json)?;

    Ok(metrics)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ir_metrics_calculations() {
        let ranked = vec![
            "doc1".to_string(),
            "doc2".to_string(),
            "doc3".to_string(),
            "doc4".to_string(),
        ];
        let mut relevant = HashSet::new();
        relevant.insert("doc1".to_string());
        relevant.insert("doc3".to_string());

        let ndcg = ndcg_at_k(&ranked, &relevant, 10);
        assert!(ndcg > 0.0 && ndcg <= 1.0);

        let map = map_at_k(&ranked, &relevant, 10);
        assert!(map > 0.0 && map <= 1.0);

        let recall = recall_at_k(&ranked, &relevant, 2);
        assert_eq!(recall, 0.5);
    }
}
