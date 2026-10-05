// FILE-CONTEXT: J31 closure verification test for contextra-text symbols
// ZWECK: Direct integration verification for extend_vocabulary, new_with_all_domains, and remove_terms.

use contextra_text::morphology::{GermanCompoundSplitter, MorphologicalTokenizer};
use contextra_text::posting_list::{Posting, ResidentPostingIndex};
use contextra_types::DocId;

#[test]
fn test_j31closure_extend_vocabulary_dynamic_addition() {
    let mut splitter = GermanCompoundSplitter::new();

    let term = "customsubsystemtestword";
    // Before extension: unknown custom term remains unsplit
    let initial_decomp = splitter.decompose(term);
    assert_eq!(initial_decomp, vec![term]);

    // Extend dictionary with constituent morphemes
    splitter.extend_vocabulary(["customsubsystem", "testword"]);

    let extended_decomp = splitter.decompose(term);
    assert_eq!(
        extended_decomp,
        vec!["customsubsystem", "testword"],
        "extend_vocabulary should register new terms into Trie enabling compound decomposition"
    );
}

#[test]
fn test_j31closure_new_with_all_domains_decomposition() {
    let splitter = GermanCompoundSplitter::new_with_all_domains();

    // Legal domain compound
    let legal_term = "schadenersatzanspruch";
    let legal_decomp = splitter.decompose(legal_term);
    assert!(
        legal_decomp.len() >= 2,
        "new_with_all_domains should load legal terms for compound decomposition, got: {:?}",
        legal_decomp
    );

    // Medical domain compound
    let med_term = "bandscheibenvorfall";
    let med_decomp = splitter.decompose(med_term);
    assert!(
        med_decomp.len() >= 2,
        "new_with_all_domains should load medical terms for compound decomposition, got: {:?}",
        med_decomp
    );
}

#[test]
fn test_j31closure_remove_terms_cache_eviction() {
    let index = ResidentPostingIndex::new();
    let term_a = "query".to_string();
    let term_b = "filter".to_string();

    let doc_1 = DocId::new(101);
    let doc_2 = DocId::new(102);

    index.upsert_posting(&term_a, Posting::new(doc_1, 3, 50));
    index.upsert_posting(&term_a, Posting::new(doc_2, 1, 30));
    index.upsert_posting(&term_b, Posting::new(doc_1, 2, 50));

    // Remove doc_1 across terms
    index.remove_terms(doc_1, &[term_a.clone(), term_b.clone()]);

    let list_a = index.get(&term_a).expect("term_a should still exist for doc_2");
    assert_eq!(list_a.len(), 1);
    assert_eq!(list_a.as_slice()[0].doc_id(), doc_2);

    // term_b had only doc_1, so list should now be completely evicted
    assert!(
        index.get(&term_b).is_none(),
        "term_b posting list should be removed once empty"
    );
}
