//! Autarkes Anwendungsbeispiel für `contextra-text` (M10).
//!
//! Dieses Beispiel demonstriert die isolierte Nutzung der Komponenten von `contextra-text`:
//! 1. Text-Normalisierung und deutsche morphologische Komposita-Zerlegung.
//! 2. Mathematische Relevanzberechnung mit Standard-BM25.
//! 3. Posting-Listen-Erzeugung und dynamisches Pruning mit BoundedTopK.

use contextra_text::bm25::BM25;
use contextra_text::morphology::{
    normalize_umlauts, GermanCompoundSplitter, MorphologicalTokenizer,
};
use contextra_text::posting_list::{Posting, PostingList};
use contextra_text::tokenizer::{GermanMorphTokenizer, Tokenizer};
use contextra_text::wand::BoundedTopK;
use contextra_types::DocId;

fn main() {
    println!("=== Contextra-Text Autarkes Baukasten-Beispiel (M10) ===\n");

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

    // 3. Posting-Listen & Block-Info
    let mut postings = Vec::new();
    postings.push(Posting::new(DocId::new(1), 2, 40));
    postings.push(Posting::new(DocId::new(2), 5, 60));
    postings.push(Posting::new(DocId::new(3), 1, 35));

    let posting_list = PostingList::new(postings);
    println!("4. Posting-Liste:");
    println!("   Anzahl Postings: {}", posting_list.len());
    if let Some(first_block) = posting_list.blocks().first() {
        println!(
            "   Block-Info: Max TF = {}, Min Doc Len = {}\n",
            first_block.max_tf, first_block.min_doc_len
        );
    }

    // 4. BoundedTopK Min-Heap Pruning
    let mut top_k = BoundedTopK::new(2);
    top_k.push(DocId::new(1), 1.25);
    top_k.push(DocId::new(2), 3.80);
    top_k.push(DocId::new(3), 2.10);

    let ranked = top_k.into_sorted_vec();
    println!("5. BoundedTopK Ergebnisse (k=2):");
    for (rank, (doc_id, s)) in ranked.iter().enumerate() {
        println!("   Rang {}: DocId={:?}, Score={:.2}", rank + 1, doc_id, s);
    }

    println!("\nBaukasten contextra-text erfolgreich autark ausgeführt!");
}
