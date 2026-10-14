// FILE-CONTEXT
// STAND: 2026-10-13
// ZWECK: Integration test for Cognition feature wiring.

#[cfg(feature = "cognition")]
use contextra::builder;

#[tokio::test]
async fn test_wiring_feature_cognition() {
    #[cfg(feature = "cognition")]
    {
        let tmp_path = std::env::temp_dir().join(format!("cognition_test_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&tmp_path);

        let mut b = builder(128).with_storage_path(&tmp_path);
        let clock = b.clock().clone();

        let bundle = contextra::wiring_cognition::CognitionWiringBundle::build(&mut b.config, clock);
        assert!(bundle.is_ok(), "CognitionWiringBundle build should succeed");
        assert!(b.config().consolidation_launcher.is_some(), "Consolidation launcher should be registered when consolidation_enabled");

        let db = b.build().await;
        assert!(db.is_ok(), "Cognition feature Contextra build should succeed");

        let _ = std::fs::remove_dir_all(&tmp_path);
    }
}
