// FILE-CONTEXT
// ZWECK: Integrationstests fuer die Isolation und das Clone-Verhalten der Auto-Extraction-Konfiguration in Collections.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use contextra_engine::collection::crud::auto_extraction::ExtractorMode;
use contextra_engine::collection::crud::{
    AutoExtractionConfig, AutoExtractionMode, EntityExtractionConfig,
};
use contextra_ports::{BoxFuture, LlmTextGenerator};
use contextra_types::Result;
use std::sync::Arc;
use tempfile::TempDir;

#[derive(Debug)]
struct TestLlmGenerator {
    id: String,
}

impl TestLlmGenerator {
    fn new(id: impl Into<String>) -> Self {
        Self { id: id.into() }
    }
}

impl LlmTextGenerator for TestLlmGenerator {
    fn generate<'a>(&'a self, _prompt: &'a str) -> BoxFuture<'a, Result<String>> {
        let resp = format!("[{{\"id\": \"{}\"}}]", self.id);
        Box::pin(async move { Ok(resp) })
    }
}

/// Test 1: Collection ohne Konfiguration liefert `None`.
#[tokio::test]
async fn test_unconfigured_collection_returns_none() {
    let tmp = TempDir::new().expect("temp dir");
    let db = contextra_engine::Contextra::open_with_config(
        tmp.path(),
        contextra_engine::ContextraConfig {
            dimension: 4,
            max_elements: 100,
            ..Default::default()
        },
    )
    .await
    .expect("open db");

    let col = db.collection("col_unconfig").await.expect("collection");
    assert!(
        col.auto_extraction_config().is_none(),
        "Fresh collection must return None for auto_extraction_config"
    );
    assert!(
        col.auto_extraction_config_with_extractor().is_none(),
        "Fresh collection must return None for auto_extraction_config_with_extractor"
    );
}

/// Test 2: Collections mit verschiedenen Konfigurationen werden in einer Schleife erzeugt und gedroppt.
/// Keine neue Collection sieht fremde Konfiguration.
#[tokio::test]
async fn test_auto_extraction_loop_creation_isolation() {
    let tmp = TempDir::new().expect("temp dir");
    let db = contextra_engine::Contextra::open_with_config(
        tmp.path(),
        contextra_engine::ContextraConfig {
            dimension: 4,
            max_elements: 100,
            ..Default::default()
        },
    )
    .await
    .expect("open db");

    for i in 0..50 {
        let col_name = format!("col_loop_{i}");
        let gen_id = format!("gen_{i}");
        let gen = Arc::new(TestLlmGenerator::new(gen_id));
        let cfg = AutoExtractionConfig::new(AutoExtractionMode::Enabled);

        {
            let col = db.collection(&col_name).await.expect("create collection");
            col.set_auto_extraction_with_mode(gen, cfg, ExtractorMode::OpenIe);

            let extracted = col
                .auto_extraction_config_with_extractor()
                .expect("auto extraction should be configured");
            assert_eq!(extracted.1.mode, AutoExtractionMode::Enabled);
            assert_eq!(extracted.2, ExtractorMode::OpenIe);
        } // `col` dropped here

        // Open a new collection with a different name
        let new_col_name = format!("col_new_{i}");
        let new_col = db.collection(&new_col_name).await.expect("create new collection");
        assert!(
            new_col.auto_extraction_config().is_none(),
            "New collection {new_col_name} must not inherit configuration from dropped collection"
        );
    }
}

/// Test 3: Klon sieht nachträglich gesetzte Konfiguration des Originals und umgekehrt.
#[tokio::test]
async fn test_auto_extraction_clone_shares_configuration() {
    let tmp = TempDir::new().expect("temp dir");
    let db = contextra_engine::Contextra::open_with_config(
        tmp.path(),
        contextra_engine::ContextraConfig {
            dimension: 4,
            max_elements: 100,
            ..Default::default()
        },
    )
    .await
    .expect("open db");

    let col_original = db.collection("col_clone_test").await.expect("collection");
    let col_clone = col_original.clone();

    // Initial state: both return None
    assert!(col_original.auto_extraction_config().is_none());
    assert!(col_clone.auto_extraction_config().is_none());

    // 1. Config set on original is visible in clone
    let gen_1 = Arc::new(TestLlmGenerator::new("gen_1"));
    let cfg_1 = AutoExtractionConfig::new(AutoExtractionMode::Enabled);
    col_original.set_auto_extraction(gen_1, cfg_1);

    let from_clone = col_clone
        .auto_extraction_config()
        .expect("clone should see config set on original");
    assert_eq!(from_clone.1.mode, AutoExtractionMode::Enabled);

    // 2. Config updated on clone is visible in original
    let gen_2 = Arc::new(TestLlmGenerator::new("gen_2"));
    let cfg_2 = AutoExtractionConfig {
        mode: AutoExtractionMode::Disabled,
        enabled: false,
        entity_config: EntityExtractionConfig::default(),
    };
    col_clone.set_auto_extraction_with_mode(gen_2, cfg_2, ExtractorMode::RuleBased);

    let from_original = col_original
        .auto_extraction_config_with_extractor()
        .expect("original should see config updated on clone");
    assert_eq!(from_original.1.mode, AutoExtractionMode::Disabled);
    assert_eq!(from_original.2, ExtractorMode::RuleBased);
}
