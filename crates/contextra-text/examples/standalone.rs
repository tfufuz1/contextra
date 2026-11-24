//! Standalone-Beispiel für `contextra-text` (Klasse A, autark ohne I/O).

use contextra_text::contract::DocKey;
use contextra_text::memory_index::MemoryTextIndex;
use contextra_text::traits::Index;

fn main() {
    let mut index = MemoryTextIndex::new();
    let doc1 = DocKey::from(1u64);
    let doc2 = DocKey::from(2u64);

    Index::<DocKey, str>::insert(&mut index, doc1, b"Donaudampfschifffahrt und Volltextsuche").unwrap();
    Index::<DocKey, str>::insert(&mut index, doc2, b"Volltextsuche in Rust mit BM25").unwrap();

    let hits = Index::<DocKey, str>::search(&index, "Volltextsuche", 5).unwrap();
    println!("Gefundene Treffer: {}", hits.len());
    for hit in hits {
        println!("Doc: {:?}, Score: {:.4}", hit.doc_id, hit.score);
    }
}
