use contextra_text::morphology::{normalize_umlauts, GermanCompoundSplitter, MorphologicalTokenizer};
use contextra_text::tokenizer::{GermanMorphTokenizer, Tokenizer};

#[test]
fn inspect_20_compounds() {
    let splitter = GermanCompoundSplitter::new();
    let tokenizer = GermanMorphTokenizer::new();

    let words = vec![
        "Datenschutzgrundverordnung",
        "Bundesdatenschutzgesetz",
        "Softwarearchitektur",
        "Computerprogramm",
        "Kraftfahrzeugsteuer",
        "Bundesverfassungsgericht",
        "Informationstechnologie",
        "Datenbankmanagement",
        "Künstliche Intelligenz",
        "Maschinelles Lernen",
        "Arbeitnehmerüberlassungsgesetz",
        "Telekommunikationsgesetz",
        "Finanzdienstleistungsaufsicht",
        "Umweltschutzorganisation",
        "Qualitätsmanagementsystem",
        "Kundenbeziehungsmanagement",
        "Lieferkettensorgfaltspflichtengeschäft",
        "Kraftfahrzeug-Haftpflichtversicherung",
        "Betriebsratsvorsitzender",
        "Urheberrechtsreform",
    ];

    println!("\n=== 20 GERMAN COMPOUNDS EVALUATION ===");
    for word in words {
        let norm = normalize_umlauts(&word.to_lowercase());
        let decomp = splitter.decompose(&norm);
        let tokens = tokenizer.tokenize(word);
        println!("Word: '{}' | norm: '{}'", word, norm);
        println!("  Splitter decomp: {:?}", decomp);
        println!("  Tokenizer tokens: {:?}", tokens);
    }
}

#[test]
fn inspect_umlaut_bimap() {
    let tokenizer = GermanMorphTokenizer::new();

    let pairs = vec![
        ("Bär", "Baer"),
        ("Öl", "Oel"),
        ("Über", "Ueber"),
        ("Straße", "Strasse"),
    ];

    println!("\n=== UMLAUT BIMAP EVALUATION ===");
    for (umlaut_word, ae_word) in pairs {
        let tok_u = tokenizer.tokenize(umlaut_word);
        let tok_a = tokenizer.tokenize(ae_word);
        println!("Word: '{}' -> Tokens: {:?}", umlaut_word, tok_u);
        println!("Word: '{}' -> Tokens: {:?}", ae_word, tok_a);

        // Check if query for tok_a matches index of tok_u
        let match_u_in_a = tok_a.iter().any(|t| tok_u.contains(t));
        // Check if query for tok_u matches index of tok_a
        let match_a_in_u = tok_u.iter().any(|t| tok_a.contains(t));

        println!("  Match '{}' in indexed '{}': {}", ae_word, umlaut_word, match_u_in_a);
        println!("  Match '{}' in indexed '{}': {}", umlaut_word, ae_word, match_a_in_u);
    }
}

#[tokio::test]
async fn test_b3_avgdl_consistency_100_insert_50_delete() -> contextra_types::Result<()> {
    use contextra_ports::{StorageEngine, TextIndex};
    use contextra_text::inverted::InvertedIndex;
    use contextra_text::tokenizer::DefaultTokenizer;
    use contextra_types::{DocId, TxId};
    use std::collections::HashMap;
    use std::sync::atomic::Ordering;
    use std::sync::Arc;

    // Use MockStorage from inverted tests or lightweight in-memory storage implementation
    #[derive(Default)]
    struct LocalMockStorage {
        store: parking_lot::RwLock<HashMap<Vec<u8>, Vec<(Vec<u8>, u64)>>>,
        next_seq: std::sync::atomic::AtomicU64,
    }

    impl StorageEngine for LocalMockStorage {
        fn get<'a>(&'a self, key: &'a [u8]) -> contextra_ports::BoxFuture<'a, contextra_types::Result<Option<bytes::Bytes>>> {
            Box::pin(async move { self.get_at_seq(key, u64::MAX).await })
        }
        fn put<'a>(&'a self, _tx: TxId, key: &'a [u8], val: &'a [u8]) -> contextra_ports::BoxFuture<'a, contextra_types::Result<()>> {
            Box::pin(async move {
                let seq = self.next_seq.fetch_add(1, Ordering::SeqCst);
                self.store.write().entry(key.to_vec()).or_default().push((val.to_vec(), seq));
                Ok(())
            })
        }
        fn delete<'a>(&'a self, _tx: TxId, key: &'a [u8]) -> contextra_ports::BoxFuture<'a, contextra_types::Result<()>> {
            Box::pin(async move {
                let seq = self.next_seq.fetch_add(1, Ordering::SeqCst);
                self.store.write().entry(key.to_vec()).or_default().push((Vec::new(), seq | contextra_types::TOMBSTONE_BIT));
                Ok(())
            })
        }
        fn commit<'a>(&'a self, _tx: TxId) -> contextra_ports::BoxFuture<'a, contextra_types::Result<()>> {
            Box::pin(async move { Ok(()) })
        }
        fn rollback<'a>(&'a self, _tx: TxId) -> contextra_ports::BoxFuture<'a, contextra_types::Result<()>> {
            Box::pin(async move { Ok(()) })
        }
        fn rollback_to_tx<'a>(&'a self, _tx: TxId) -> contextra_ports::BoxFuture<'a, contextra_types::Result<()>> {
            Box::pin(async move { Ok(()) })
        }
        fn get_at_seq<'a>(&'a self, key: &'a [u8], seq: u64) -> contextra_ports::BoxFuture<'a, contextra_types::Result<Option<bytes::Bytes>>> {
            Box::pin(async move {
                let store = self.store.read();
                if let Some(versions) = store.get(key) {
                    for (val, v_seq) in versions.iter().rev() {
                        let raw_seq = v_seq & !contextra_types::TOMBSTONE_BIT;
                        if raw_seq <= seq {
                            if (v_seq & contextra_types::TOMBSTONE_BIT) != 0 {
                                return Ok(None);
                            }
                            return Ok(Some(bytes::Bytes::from(val.clone())));
                        }
                    }
                }
                Ok(None)
            })
        }
        fn last_seq_no<'a>(&'a self) -> contextra_ports::BoxFuture<'a, contextra_types::Result<u64>> {
            Box::pin(async move { Ok(self.next_seq.load(Ordering::SeqCst)) })
        }
        fn last_tx_id<'a>(&'a self) -> contextra_ports::BoxFuture<'a, contextra_types::Result<TxId>> {
            Box::pin(async move { Ok(TxId::new(0)) })
        }
        fn flush<'a>(&'a self) -> contextra_ports::BoxFuture<'a, contextra_types::Result<()>> { Box::pin(async move { Ok(()) }) }
        fn stats<'a>(&'a self) -> contextra_ports::BoxFuture<'a, contextra_types::Result<contextra_ports::StorageStats>> {
            Box::pin(async move { Ok(contextra_ports::StorageStats { num_segments: 0, total_size_bytes: 0, memtable_size_bytes: 0 }) })
        }
        fn pin_checkpoint<'a>(&'a self, _id: u64) -> contextra_ports::BoxFuture<'a, contextra_types::Result<()>> { Box::pin(async move { Ok(()) }) }
        fn unpin_checkpoint<'a>(&'a self, _id: u64) -> contextra_ports::BoxFuture<'a, contextra_types::Result<()>> { Box::pin(async move { Ok(()) }) }
        fn scan<'a>(&'a self, _: std::ops::Bound<&'a [u8]>, _: std::ops::Bound<&'a [u8]>, _: Option<usize>) -> contextra_ports::BoxFuture<'a, contextra_types::Result<Vec<(Vec<u8>, Vec<u8>)>>> {
            Box::pin(async move { Ok(Vec::new()) })
        }
        fn scan_prefix<'a>(&'a self, prefix: &'a [u8]) -> contextra_ports::BoxFuture<'a, contextra_types::Result<Vec<(Vec<u8>, Vec<u8>)>>> {
            self.scan_prefix_at(prefix, u64::MAX)
        }
        fn scan_prefix_at<'a>(&'a self, prefix: &'a [u8], seq: u64) -> contextra_ports::BoxFuture<'a, contextra_types::Result<Vec<(Vec<u8>, Vec<u8>)>>> {
            Box::pin(async move {
                let store = self.store.read();
                let mut res = Vec::new();
                for (k, versions) in store.iter() {
                    if k.starts_with(prefix) {
                        for (val, v_seq) in versions.iter().rev() {
                            let raw_seq = v_seq & !contextra_types::TOMBSTONE_BIT;
                            if raw_seq <= seq {
                                if (v_seq & contextra_types::TOMBSTONE_BIT) == 0 {
                                    res.push((k.clone(), val.clone()));
                                }
                                break;
                            }
                        }
                    }
                }
                Ok(res)
            })
        }
    }

    let storage = Arc::new(LocalMockStorage::default());
    let index = InvertedIndex::new(storage.clone(), "avgdl_b3_test");

    // Insert 100 documents, each with exactly 10 non-stopword tokens
    let text = "alpha beta gamma delta epsilon zeta eta theta iota kappa";
    assert_eq!(DefaultTokenizer.tokenize(text).len(), 10);

    for i in 1..=100 {
        let doc_id = DocId::new(i);
        let tx = TxId::new(i);
        index.insert(tx, doc_id, text).await?;
        index.commit(tx).await?;
    }

    assert_eq!(index.len().await, 100);
    let avgdl_100 = index.search_bm25("alpha", 1, None).await?;
    let cached_avgdl_100 = index.search_bm25("beta", 1, None).await?;
    assert!(!avgdl_100.is_empty());
    assert!(!cached_avgdl_100.is_empty());

    // Check avgdl is 10.0
    let avgdl_x1000 = index.stats().await?.num_tokens as f64 / index.stats().await?.num_documents as f64;
    assert_eq!(avgdl_x1000, 10.0, "avgdl after 100 docs must be 10.0");

    // Delete 50 documents
    for i in 1..=50 {
        let doc_id = DocId::new(i);
        let tx = TxId::new(100 + i);
        index.delete(tx, doc_id).await?;
        index.commit(tx).await?;
    }

    assert_eq!(index.len().await, 50);
    let avgdl_50 = index.stats().await?.num_tokens as f64 / index.stats().await?.num_documents as f64;
    assert_eq!(avgdl_50, 10.0, "avgdl after deleting 50 docs must remain 10.0");

    Ok(())
}

#[test]
fn test_b6_morphology_idempotency_and_proper_nouns() {
    let splitter = GermanCompoundSplitter::new();

    // 1. Idempotency test:
    // "bundesverfassungsgericht" -> ["bundes", "verfassungs", "gericht"]
    let norm = normalize_umlauts("bundesverfassungsgericht");
    let parts = splitter.decompose(&norm);
    assert_eq!(parts, vec!["bundes", "verfassungs", "gericht"]);

    // Calling decompose on individual parts must return each part unsplit (idempotency)
    for part in &parts {
        let second_decomp = splitter.decompose(part);
        assert_eq!(second_decomp, vec![*part], "Decomposing already split component '{}' must be idempotent", part);
    }

    // 2. Unknown Stems / Proper Nouns protection test:
    // Unknown proper noun or company name "contextrasystems"
    let unknown_noun = "contextrasystems";
    let decomp_unknown = splitter.decompose(unknown_noun);
    assert_eq!(decomp_unknown, vec![unknown_noun], "Unknown proper noun/word must fall back to unsplit original token");
}

#[tokio::test]
async fn test_b5_wand_vs_bruteforce_1000_docs_3_terms() -> contextra_types::Result<()> {
    use contextra_ports::{StorageEngine, TextIndex};
    use contextra_text::bm25::score_term;
    use contextra_text::inverted::InvertedIndex;
    use contextra_text::tokenizer::DefaultTokenizer;
    use contextra_types::{DocId, TxId};
    use std::collections::HashMap;
    use std::sync::atomic::Ordering;
    use std::sync::Arc;

    #[derive(Default)]
    struct LocalMockStorage {
        store: parking_lot::RwLock<HashMap<Vec<u8>, Vec<(Vec<u8>, u64)>>>,
        next_seq: std::sync::atomic::AtomicU64,
    }

    impl StorageEngine for LocalMockStorage {
        fn get<'a>(&'a self, key: &'a [u8]) -> contextra_ports::BoxFuture<'a, contextra_types::Result<Option<bytes::Bytes>>> {
            Box::pin(async move { self.get_at_seq(key, u64::MAX).await })
        }
        fn put<'a>(&'a self, _tx: TxId, key: &'a [u8], val: &'a [u8]) -> contextra_ports::BoxFuture<'a, contextra_types::Result<()>> {
            Box::pin(async move {
                let seq = self.next_seq.fetch_add(1, Ordering::SeqCst);
                self.store.write().entry(key.to_vec()).or_default().push((val.to_vec(), seq));
                Ok(())
            })
        }
        fn delete<'a>(&'a self, _tx: TxId, key: &'a [u8]) -> contextra_ports::BoxFuture<'a, contextra_types::Result<()>> {
            Box::pin(async move {
                let seq = self.next_seq.fetch_add(1, Ordering::SeqCst);
                self.store.write().entry(key.to_vec()).or_default().push((Vec::new(), seq | contextra_types::TOMBSTONE_BIT));
                Ok(())
            })
        }
        fn commit<'a>(&'a self, _tx: TxId) -> contextra_ports::BoxFuture<'a, contextra_types::Result<()>> { Box::pin(async move { Ok(()) }) }
        fn rollback<'a>(&'a self, _tx: TxId) -> contextra_ports::BoxFuture<'a, contextra_types::Result<()>> { Box::pin(async move { Ok(()) }) }
        fn rollback_to_tx<'a>(&'a self, _tx: TxId) -> contextra_ports::BoxFuture<'a, contextra_types::Result<()>> { Box::pin(async move { Ok(()) }) }
        fn get_at_seq<'a>(&'a self, key: &'a [u8], seq: u64) -> contextra_ports::BoxFuture<'a, contextra_types::Result<Option<bytes::Bytes>>> {
            Box::pin(async move {
                let store = self.store.read();
                if let Some(versions) = store.get(key) {
                    for (val, v_seq) in versions.iter().rev() {
                        let raw_seq = v_seq & !contextra_types::TOMBSTONE_BIT;
                        if raw_seq <= seq {
                            if (v_seq & contextra_types::TOMBSTONE_BIT) != 0 {
                                return Ok(None);
                            }
                            return Ok(Some(bytes::Bytes::from(val.clone())));
                        }
                    }
                }
                Ok(None)
            })
        }
        fn last_seq_no<'a>(&'a self) -> contextra_ports::BoxFuture<'a, contextra_types::Result<u64>> {
            Box::pin(async move { Ok(self.next_seq.load(Ordering::SeqCst)) })
        }
        fn last_tx_id<'a>(&'a self) -> contextra_ports::BoxFuture<'a, contextra_types::Result<TxId>> { Box::pin(async move { Ok(TxId::new(0)) }) }
        fn flush<'a>(&'a self) -> contextra_ports::BoxFuture<'a, contextra_types::Result<()>> { Box::pin(async move { Ok(()) }) }
        fn stats<'a>(&'a self) -> contextra_ports::BoxFuture<'a, contextra_types::Result<contextra_ports::StorageStats>> {
            Box::pin(async move { Ok(contextra_ports::StorageStats { num_segments: 0, total_size_bytes: 0, memtable_size_bytes: 0 }) })
        }
        fn pin_checkpoint<'a>(&'a self, _id: u64) -> contextra_ports::BoxFuture<'a, contextra_types::Result<()>> { Box::pin(async move { Ok(()) }) }
        fn unpin_checkpoint<'a>(&'a self, _id: u64) -> contextra_ports::BoxFuture<'a, contextra_types::Result<()>> { Box::pin(async move { Ok(()) }) }
        fn scan<'a>(&'a self, _: std::ops::Bound<&'a [u8]>, _: std::ops::Bound<&'a [u8]>, _: Option<usize>) -> contextra_ports::BoxFuture<'a, contextra_types::Result<Vec<(Vec<u8>, Vec<u8>)>>> {
            Box::pin(async move { Ok(Vec::new()) })
        }
        fn scan_prefix<'a>(&'a self, prefix: &'a [u8]) -> contextra_ports::BoxFuture<'a, contextra_types::Result<Vec<(Vec<u8>, Vec<u8>)>>> {
            self.scan_prefix_at(prefix, u64::MAX)
        }
        fn scan_prefix_at<'a>(&'a self, prefix: &'a [u8], seq: u64) -> contextra_ports::BoxFuture<'a, contextra_types::Result<Vec<(Vec<u8>, Vec<u8>)>>> {
            Box::pin(async move {
                let store = self.store.read();
                let mut res = Vec::new();
                for (k, versions) in store.iter() {
                    if k.starts_with(prefix) {
                        for (val, v_seq) in versions.iter().rev() {
                            let raw_seq = v_seq & !contextra_types::TOMBSTONE_BIT;
                            if raw_seq <= seq {
                                if (v_seq & contextra_types::TOMBSTONE_BIT) == 0 {
                                    res.push((k.clone(), val.clone()));
                                }
                                break;
                            }
                        }
                    }
                }
                Ok(res)
            })
        }
    }

    let storage = Arc::new(LocalMockStorage::default());
    let index = InvertedIndex::new(storage.clone(), "wand_b5_test");

    // Corpus construction: 1000 documents
    // Query terms: "alpha", "beta", "gamma"
    // Term frequencies and document lengths vary across 1000 documents
    let mut doc_tokens: HashMap<DocId, Vec<String>> = HashMap::new();

    let tx = TxId::new(1);
    for i in 1..=1000 {
        let doc_id = DocId::new(i);
        let mut text = String::new();
        if i % 2 == 0 {
            text.push_str("alpha ");
        }
        if i % 5 == 0 {
            text.push_str("beta beta ");
        }
        if i % 10 == 0 {
            text.push_str("gamma gamma gamma ");
        }
        if i % 3 == 0 {
            text.push_str("delta ");
        }
        if text.trim().is_empty() {
            text.push_str("epsilon zeta");
        }

        let tokens = DefaultTokenizer.tokenize(&text);
        doc_tokens.insert(doc_id, tokens);
        index.insert(tx, doc_id, &text).await?;
    }
    index.commit(tx).await?;

    // Execute Block-Max WAND Top-10 Search
    let wand_top_10 = index.search_bm25("alpha beta gamma", 10, None).await?;

    // Execute Full Brute-Force BM25 Calculation across all 1000 documents
    let total_docs = 1000u32;
    let total_tokens: u32 = doc_tokens.values().map(|t| t.len() as u32).sum();
    let avg_doc_len = total_tokens as f32 / total_docs as f32;

    // Document frequencies (DF) for query terms
    let query_terms = ["alpha", "beta", "gamma"];
    let mut dfs: HashMap<&str, u32> = HashMap::new();
    for term in &query_terms {
        let df = doc_tokens.values().filter(|toks| toks.contains(&term.to_string())).count() as u32;
        dfs.insert(term, df);
    }

    let mut brute_force_scores: Vec<(DocId, f32)> = Vec::new();
    for (&doc_id, toks) in &doc_tokens {
        let doc_len = toks.len() as u32;
        let mut total_score = 0.0f32;

        for &term in &query_terms {
            let tf = toks.iter().filter(|&t| t == term).count() as u32;
            if tf > 0 {
                let df = dfs[term];
                let score = score_term(tf, doc_len, avg_doc_len, df, total_docs);
                total_score += score;
            }
        }

        if total_score > 0.0 {
            brute_force_scores.push((doc_id, total_score));
        }
    }

    // Sort brute-force candidates by score desc, doc_id asc
    brute_force_scores.sort_by(|a, b| {
        b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| a.0.cmp(&b.0))
    });

    let brute_top_10: Vec<(DocId, f32)> = brute_force_scores.into_iter().take(10).collect();

    // Verify WAND Top-10 matches Brute-Force Top-10 exactly
    assert_eq!(wand_top_10.len(), brute_top_10.len());
    for (i, (wand_item, brute_item)) in wand_top_10.iter().zip(brute_top_10.iter()).enumerate() {
        assert_eq!(
            wand_item.0, brute_item.0,
            "Rank {} DocId mismatch: WAND {:?} vs Brute-Force {:?}",
            i, wand_item, brute_item
        );
        assert!(
            (wand_item.1 - brute_item.1).abs() < 1e-5,
            "Rank {} Score mismatch for doc {:?}: WAND {} vs Brute-Force {}",
            i, wand_item.0, wand_item.1, brute_item.1
        );
    }

    Ok(())
}
