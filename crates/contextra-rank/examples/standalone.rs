//! Standalone-Minimalbeispiel für contextra-rank (R2/R3-Lieferung, <= 30 Zeilen).
use contextra_rank::fusion::{rrf_fuse, ScoredItem};

fn main() {
    let text = vec![ScoredItem::new("doc_1", 10.0), ScoredItem::new("doc_2", 5.0)];
    let vector = vec![ScoredItem::new("doc_2", 0.9), ScoredItem::new("doc_1", 0.8)];
    let fused = rrf_fuse(&[text, vector], 60).unwrap_or_default();
    if let Some(top) = fused.first() {
        println!("Top result: ID={}, score={:.6}", top.id, top.score);
        assert_eq!(top.id, "doc_1");
    }
}
