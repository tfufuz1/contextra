// FILE-CONTEXT
// ZWECK: Lifecycle wiring integration tests for contextra-engine facade initialization.

use contextra_engine::{ConsolidationLauncher, Contextra, ContextraConfig, EngineWiring};
use contextra_ports::DriftStatusProvider;
use contextra_types::ContextraError;
use tempfile::tempdir;

struct MockDriftProvider;
impl DriftStatusProvider for MockDriftProvider {
    fn overall_drift_status(&self) -> String {
        "InLimits".to_string()
    }
}

#[tokio::test]
async fn test_open_with_config_and_wiring_populates_controllers() -> contextra_types::Result<()> {
    let tmp = tempdir().unwrap();

    let router = std::sync::Arc::new(MockDriftProvider);
    let calibrator = std::sync::Arc::new(parking_lot::Mutex::new(
        contextra_rank::IsotonicCalibrator::new(10, 100),
    ));
    let router_weak = std::sync::Arc::downgrade(&router) as std::sync::Weak<dyn DriftStatusProvider>;
    let calibrator_weak = std::sync::Arc::downgrade(&calibrator);

    let wiring = EngineWiring::build(
        Some(router_weak),
        Some(calibrator_weak),
    )?;

    let config = ContextraConfig {
        consolidation_enabled: false,
        ..Default::default()
    };
    let db = Contextra::open_with_config_and_wiring(tmp.path(), config, wiring).await?;

    let stats = db.stats().await?;
    assert_eq!(stats.drift_status, "InLimits");

    db.close().await?;
    Ok(())
}

#[tokio::test]
async fn test_open_with_consolidation_enabled_without_launcher_returns_policy_violation() {
    let tmp = tempdir().unwrap();
    let config = ContextraConfig {
        consolidation_enabled: true,
        consolidation_launcher: None,
        ..Default::default()
    };

    match Contextra::open_with_config(tmp.path(), config).await {
        Ok(_) => panic!("Expected open_with_config to fail"),
        Err(ContextraError::PolicyViolation(msg)) => {
            assert!(msg.contains("no consolidation_launcher is configured"));
        }
        Err(other) => panic!("Expected PolicyViolation, got: {:?}", other),
    }
}

#[tokio::test]
async fn test_open_with_consolidation_enabled_with_launcher_succeeds() -> contextra_types::Result<()> {
    let tmp = tempdir().unwrap();
    let launcher: ConsolidationLauncher = std::sync::Arc::new(|_col, _interval, _max_llm, _cancel| {
        tokio::spawn(async {})
    });

    let config = ContextraConfig {
        consolidation_enabled: true,
        consolidation_launcher: Some(launcher),
        ..Default::default()
    };

    let db = Contextra::open_with_config(tmp.path(), config).await?;
    db.close().await?;
    Ok(())
}
