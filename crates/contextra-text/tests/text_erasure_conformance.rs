#![cfg(feature = "types-compat")]

use contextra_text::TextErasureConfig;

#[test]
fn defaults_bound_term_commits_and_enable_dictionary_fallback() {
    let config = TextErasureConfig::default();
    assert_eq!(config.terms_per_tx, 256);
    assert!(config.fallback_dictionary_scan);
}

#[test]
fn erasure_layer_has_no_blocking_executor_path() {
    let source = include_str!("../src/erasure.rs");
    assert!(!source.contains("block_on"));
}
