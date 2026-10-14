// FILE-CONTEXT
// STAND: 2026-10-13
// ZWECK: Integration test for WASM sandbox feature wiring.

#[cfg(feature = "sandbox")]
use contextra::builder;

#[tokio::test]
async fn test_wiring_feature_sandbox() {
    #[cfg(feature = "sandbox")]
    {
        let tmp_path = std::env::temp_dir().join(format!("sandbox_test_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&tmp_path);

        let db = builder(128)
            .with_storage_path(&tmp_path)
            .with_sandbox(true)
            .build()
            .await;

        assert!(db.is_ok(), "Sandbox feature build should succeed");

        let bundle = contextra::wiring_sandbox::SandboxWiringBundle::build(true)
            .expect("SandboxWiringBundle build should succeed");

        assert!(bundle.is_enabled());

        let _ = std::fs::remove_dir_all(&tmp_path);
    }
}
