#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use contextra::compliance::{AvvParams, TimeRange, REVIEW_HEADER};
use contextra::ContextraBuilder;
use contextra_core::error::ContextraError;
use contextra_types::TenantId;

#[tokio::test]
async fn test_compliance_avv_draft_generation_and_escaping() {
    let tmp_path = std::env::temp_dir().join(format!(
        "contextra_test_compliance_avv_{}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&tmp_path);

    let db = ContextraBuilder::new(16)
        .with_storage_path(&tmp_path)
        .build()
        .await
        .expect("Database open should succeed");

    // Hostile input string containing Markdown headings (# ), links ([x](y)), and table pipes (|)
    let hostile_controller = "# Kanzlei & Partner [evil_link](http://hostile.test) | Col1";
    let hostile_processor = "Contextra Cloud | #1 Operator";
    let hostile_subprocessor = "- [subprocessor](http://sub.test) | Cloud";

    let params = AvvParams {
        controller_name: hostile_controller.to_string(),
        processor_name: hostile_processor.to_string(),
        tenant_id: TenantId::try_new(777).unwrap(),
        technical_measures: None,
        subprocessors: vec![hostile_subprocessor.to_string()],
        deletion_sla_days: 14,
    };

    let draft = db
        .generate_avv_draft(&params)
        .expect("AVV draft generation should succeed");

    // 1. Assert review header presence in draft structure and markdown output
    assert!(
        draft.markdown.contains("Contextra AVV Draft Review Header"),
        "Markdown must contain review header"
    );
    assert_eq!(draft.review_header, REVIEW_HEADER.trim());

    // 2. Assert hostile input escaping:
    // "# Kanzlei" must be escaped to "\# Kanzlei"
    assert!(
        draft.markdown.contains("\\# Kanzlei"),
        "Heading # must be escaped to \\#"
    );
    // "[evil_link](...)" must be escaped to "\[evil_link\]\(...\)"
    assert!(
        draft
            .markdown
            .contains("\\[evil_link\\]\\(http://hostile.test\\)"),
        "Link markdown syntax must be escaped"
    );
    // "|" must be escaped to "\|"
    assert!(
        draft.markdown.contains("Col1"),
        "Sanitized text must contain field contents"
    );
    assert!(
        draft.markdown.contains("\\|"),
        "Table pipe | must be escaped to \\|"
    );

    let _ = std::fs::remove_dir_all(&tmp_path);
}

#[tokio::test]
async fn test_export_processing_register_returns_compliance_source_unavailable() {
    let tmp_path = std::env::temp_dir().join(format!(
        "contextra_test_compliance_register_{}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&tmp_path);

    let db = ContextraBuilder::new(16)
        .with_storage_path(&tmp_path)
        .build()
        .await
        .expect("Database open should succeed");

    let range = TimeRange {
        start_nanos: 0,
        end_nanos: 1000,
    };

    let res = db.export_processing_register(range).await;
    assert!(
        res.is_err(),
        "Export must return error when no production source exists"
    );

    match res.err().unwrap() {
        ContextraError::CapabilityUnsupported { capability, reason } => {
            assert_eq!(capability, "export_processing_register");
            assert!(
                reason.contains("ComplianceSourceUnavailable"),
                "Expected ComplianceSourceUnavailable in reason, got: {reason}"
            );
        }
        other => panic!("Expected CapabilityUnsupported error, got: {:?}", other),
    }

    let _ = std::fs::remove_dir_all(&tmp_path);
}
