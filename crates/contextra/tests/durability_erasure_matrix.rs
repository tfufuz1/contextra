// FILE-CONTEXT
// ZWECK: E2E Integration tests for DurabilityMode x ErasureMode matrix (§4.6, E20, M5.10).
// INVARIANTEN: All 9 matrix cases must be explicitly tested.

use contextra::builder::ContextraBuilder;
use contextra_ports::erasure::ErasureMode;
use contextra_store::lsm::DurabilityMode;
use contextra_types::error::ContextraError;
use tempfile::TempDir;

#[tokio::test]
async fn test_durability_erasure_matrix_all_9_cases() -> Result<(), Box<dyn std::error::Error>> {
    let base_tmp = TempDir::new()?;

    // 1. (Full, Off) => OK
    let path1 = base_tmp.path().join("case1");
    let mut config = contextra_db::ContextraConfig::default();
    config.durability_mode = DurabilityMode::Full;
    let db1 = ContextraBuilder::new(4)
        .with_storage_path(&path1)
        .with_config(config)
        .with_erasure_mode(ErasureMode::Off)
        .build()
        .await;
    assert!(db1.is_ok(), "Full x Off must succeed");

    // 2. (Full, Assured) => OK if feature erasure is compiled or fails with FeatureMissing
    let path2 = base_tmp.path().join("case2");
    let mut config = contextra_db::ContextraConfig::default();
    config.durability_mode = DurabilityMode::Full;
    let db2 = ContextraBuilder::new(4)
        .with_storage_path(&path2)
        .with_config(config)
        .with_erasure_mode(ErasureMode::Assured)
        .build()
        .await;
    if cfg!(feature = "erasure") {
        assert!(
            db2.is_ok(),
            "Full x Assured must succeed with erasure feature"
        );
    } else {
        assert!(
            db2.is_err(),
            "Full x Assured must fail when erasure feature is missing"
        );
        if let Err(ContextraError::PolicyViolation(msg)) = db2 {
            assert!(
                msg.contains("erasure"),
                "Error must mention erasure feature"
            );
        }
    }

    // 3. (Full, Attested) => OK if feature erasure is compiled or fails with FeatureMissing
    let path3 = base_tmp.path().join("case3");
    let mut config = contextra_db::ContextraConfig::default();
    config.durability_mode = DurabilityMode::Full;
    let db3 = ContextraBuilder::new(4)
        .with_storage_path(&path3)
        .with_config(config)
        .with_erasure_mode(ErasureMode::Attested)
        .build()
        .await;
    if cfg!(feature = "erasure") {
        assert!(
            db3.is_ok(),
            "Full x Attested must succeed with erasure feature"
        );
    } else {
        assert!(
            db3.is_err(),
            "Full x Attested must fail when erasure feature is missing"
        );
    }

    // 4. (WalNoHmac, Off) => OK
    let path4 = base_tmp.path().join("case4");
    let mut config = contextra_db::ContextraConfig::default();
    config.durability_mode = DurabilityMode::WalNoHmac;
    let db4 = ContextraBuilder::new(4)
        .with_storage_path(&path4)
        .with_config(config)
        .with_erasure_mode(ErasureMode::Off)
        .build()
        .await;
    assert!(db4.is_ok(), "WalNoHmac x Off must succeed");

    // 5. (WalNoHmac, Assured) => OK (with warning) or fails FeatureMissing
    let path5 = base_tmp.path().join("case5");
    let mut config = contextra_db::ContextraConfig::default();
    config.durability_mode = DurabilityMode::WalNoHmac;
    let db5 = ContextraBuilder::new(4)
        .with_storage_path(&path5)
        .with_config(config)
        .with_erasure_mode(ErasureMode::Assured)
        .build()
        .await;
    if cfg!(feature = "erasure") {
        assert!(db5.is_ok(), "WalNoHmac x Assured must succeed with warning");
    } else {
        assert!(
            db5.is_err(),
            "WalNoHmac x Assured must fail when erasure feature is missing"
        );
    }

    // 6. (WalNoHmac, Attested) => Forbidden (INV-DURABILITY-RING)
    let path6 = base_tmp.path().join("case6");
    let mut config = contextra_db::ContextraConfig::default();
    config.durability_mode = DurabilityMode::WalNoHmac;
    let db6 = ContextraBuilder::new(4)
        .with_storage_path(&path6)
        .with_config(config)
        .with_erasure_mode(ErasureMode::Attested)
        .build()
        .await;
    assert!(db6.is_err(), "WalNoHmac x Attested must fail");
    if let Err(err) = db6 {
        let msg = err.to_string();
        assert!(
            msg.contains("INV-DURABILITY-RING")
                || msg.contains("Incompatible")
                || msg.contains("erasure"),
            "Expected INV-DURABILITY-RING error, got: {msg}"
        );
    }

    // 7. (MemoryOnly, Off) => OK
    let path7 = base_tmp.path().join("case7");
    let mut config = contextra_db::ContextraConfig::default();
    config.durability_mode = DurabilityMode::MemoryOnly;
    let db7 = ContextraBuilder::new(4)
        .with_storage_path(&path7)
        .with_config(config)
        .with_erasure_mode(ErasureMode::Off)
        .build()
        .await;
    assert!(db7.is_ok(), "MemoryOnly x Off must succeed");

    // 8. (MemoryOnly, Assured) => Forbidden (INV-DURABILITY-RING)
    let path8 = base_tmp.path().join("case8");
    let mut config = contextra_db::ContextraConfig::default();
    config.durability_mode = DurabilityMode::MemoryOnly;
    let db8 = ContextraBuilder::new(4)
        .with_storage_path(&path8)
        .with_config(config)
        .with_erasure_mode(ErasureMode::Assured)
        .build()
        .await;
    assert!(db8.is_err(), "MemoryOnly x Assured must fail");
    if let Err(err) = db8 {
        let msg = err.to_string();
        assert!(
            msg.contains("INV-DURABILITY-RING")
                || msg.contains("Incompatible")
                || msg.contains("erasure"),
            "Expected INV-DURABILITY-RING error, got: {msg}"
        );
    }

    // 9. (MemoryOnly, Attested) => Forbidden (INV-DURABILITY-RING)
    let path9 = base_tmp.path().join("case9");
    let mut config = contextra_db::ContextraConfig::default();
    config.durability_mode = DurabilityMode::MemoryOnly;
    let db9 = ContextraBuilder::new(4)
        .with_storage_path(&path9)
        .with_config(config)
        .with_erasure_mode(ErasureMode::Attested)
        .build()
        .await;
    assert!(db9.is_err(), "MemoryOnly x Attested must fail");
    if let Err(err) = db9 {
        let msg = err.to_string();
        assert!(
            msg.contains("INV-DURABILITY-RING")
                || msg.contains("Incompatible")
                || msg.contains("erasure"),
            "Expected INV-DURABILITY-RING error, got: {msg}"
        );
    }

    Ok(())
}
