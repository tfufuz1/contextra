//! Adversarial- und Grenzwert-Tests für Embedder und Reranker.

use contextra_infer_onnx::traits::{Embedder, Reranker};
use contextra_infer_onnx::{FakeEmbedder, FakeReranker};

#[test]
fn test_adversarial_empty_inputs() {
    let embedder = FakeEmbedder::new(32).unwrap();
    let empty_embed = embedder.embed("").unwrap();
    assert_eq!(empty_embed.len(), 32);

    let empty_batch = embedder.embed_batch(&[]).unwrap();
    assert!(empty_batch.is_empty());

    let reranker = FakeReranker::new();
    let scores = reranker.score("", &[]).unwrap();
    assert!(scores.is_empty());

    let scores_empty_docs = reranker.score("query", &["", ""]).unwrap();
    assert_eq!(scores_empty_docs.len(), 2);
    assert_eq!(scores_empty_docs[0], 0.0);
    assert_eq!(scores_empty_docs[1], 0.0);
}

#[test]
fn test_adversarial_very_long_input() {
    let embedder = FakeEmbedder::new(64).unwrap();
    let long_text = "a".repeat(100_000);
    let res = embedder.embed(&long_text);
    assert!(res.is_ok());
    assert_eq!(res.unwrap().len(), 64);
}

#[test]
fn test_zero_dimension_rejected() {
    let res = FakeEmbedder::new(0);
    assert!(res.is_err());
}
