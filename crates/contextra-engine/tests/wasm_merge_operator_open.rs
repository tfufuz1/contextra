// FILE-CONTEXT
// ZWECK: WASM Merge Operator opening path integration tests.

#[cfg(feature = "sandbox")]
use contextra_engine::{Contextra, ContextraConfig, WasmMergeOperator};
#[cfg(feature = "sandbox")]
use std::sync::Arc;
#[cfg(feature = "sandbox")]
use tempfile::tempdir;

#[cfg(feature = "sandbox")]
#[tokio::test]
async fn test_open_with_wasm_merge_operator() -> contextra_types::Result<()> {
    let tmp = tempdir().unwrap();

    let wat = r#"
        (module
            (memory (export "memory") 1)
            (func (export "_start"))
        )
    "#;
    let wasm_bytes = wat::parse_str(wat).expect("valid WAT");
    let op = WasmMergeOperator::new(wasm_bytes)?;

    let config = ContextraConfig {
        consolidation_enabled: false,
        merge_operator: Some(Arc::new(op)),
        ..Default::default()
    };

    let db = Contextra::open_with_config(tmp.path(), config).await?;
    db.close().await?;
    Ok(())
}
