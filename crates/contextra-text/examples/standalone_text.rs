//! Autarkes Anwendungsbeispiel für `contextra-text` (Klasse A).
//!
//! Dieses Beispiel demonstriert die vollständig isolierte Nutzung von `contextra-text`:
//! 1. Text-Normalisierung und deutsche morphologische Komposita-Zerlegung.
//! 2. Mathematische Relevanzberechnung mit Standard-BM25.
//! 3. Thread-sicherer In-Memory Volltextindex mit Snapshot-Isolation (`Index<DocKey, str>` & `Maintain`).

use contextra_text::bm25::BM25;
use contextra_text::contract::DocKey;
use contextra_text::memory_index::MemoryTextIndex;
use contextra_text::morphology::{
    normalize_umlauts, GermanCompoundSplitter, MorphologicalTokenizer,
};
use contextra_text::tokenizer::{GermanMorphTokenizer, Tokenizer};
use contextra_text::traits::{Index, Maintain};

fn main() {
    println!("=== Contextra-Text Autarkes Baukasten-Beispiel (Klasse A) ===\n");

    // 1. Textanalyse & deutsche Morphologie
    let splitter = GermanCompoundSplitter::default();
    let sample = "Donaudampfschifffahrt";
    let normalized = normalize_umlauts(sample);
    let parts = splitter.decompose(&normalized);
    println!("1. Morphologische Komposita-Zerlegung:");
    println!("   Eingabe:      {sample}");
    println!("   Normalisiert: {normalized}");
    println!("   Bestandteile: {parts:?}\n");

    let tokenizer = GermanMorphTokenizer::default();
    let text = "Schnelle Informationsrückgewinnung mit hochgradig optimierter Volltextsuche.";
    let tokens = tokenizer.tokenize(text);
    println!("2. Tokenisierung (GermanMorphTokenizer):");
    println!("   Text:   {text}");
    println!("   Tokens: {tokens:?}\n");

    // 2. BM25 Relevanzbewertung
    let bm25 = BM25::default();
    let tf = 3u32;
    let doc_len = 50u32;
    let avg_doc_len = 45.0f32;
    let df = 12u32;
    let total_docs = 1_000u32;

    let score = bm25.score_term(tf, doc_len, avg_doc_len, df, total_docs);
    println!("3. BM25 Relevanzberechnung:");
    println!("   Parameter: tf={tf}, doc_len={doc_len}, avg_doc_len={avg_doc_len}, df={df}, N={total_docs}");
    println!("   Errechneter BM25-Score: {score:.4}\n");

    // 3. Thread-sicherer In-Memory Volltextindex (Snapshot-Isolation)
    let mut index = MemoryTextIndex::new();
    let doc1 = DocKey::from(1u64);
    let doc2 = DocKey::from(2u64);

    Index::<DocKey, str>::insert(&mut index, doc1, b"Donaudampfschifffahrt und Volltextsuche")
        .unwrap();
    Index::<DocKey, str>::insert(&mut index, doc2, b"Volltextsuche in Rust mit BM25").unwrap();

    let hits = Index::<DocKey, str>::search(&index, "Volltextsuche", 10).unwrap();
    println!("4. In-Memory Volltextsuche Treffer: {}", hits.len());
    for (i, hit) in hits.iter().enumerate() {
        println!(
            "   Rang {}: DocId={:?}, Score={:.4}",
            i + 1,
            hit.doc_id,
            hit.score
        );
    }

    // Löschung und Kompaktierung
    Index::<DocKey, str>::remove(&mut index, &doc1).unwrap();
    let stats = Maintain::compact(&mut index).unwrap();
    println!(
        "5. Index-Kompaktierung: {} Dokumente bereinigt, {} aktiv.\n",
        stats.purged_docs, stats.active_docs
    );

    println!("Baukasten contextra-text erfolgreich autark ausgeführt!");
}
