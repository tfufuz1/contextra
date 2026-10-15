// FILE-CONTEXT
// STAND: 2026-10-13
// ZWECK: Integration test for sovereign feature wiring in contextra facade.

#[cfg(feature = "sovereign")]
use contextra::{builder, ContextraError};

#[tokio::test]
async fn test_wiring_feature_sovereign_fail_closed_without_passphrase() {
    #[cfg(feature = "sovereign")]
    {
        let tmp_path =
            std::env::temp_dir().join(format!("sovereign_test_nopass_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&tmp_path);

        let res = builder(128).with_storage_path(&tmp_path).build().await;

        assert!(
            res.is_err(),
            "Sovereign build without passphrase must fail closed"
        );
        if let Err(err) = res {
            assert!(matches!(err, ContextraError::PolicyViolation(_)));
        }

        let _ = std::fs::remove_dir_all(&tmp_path);
    }
}

#[tokio::test]
async fn test_wiring_feature_sovereign_success_with_passphrase() {
    #[cfg(feature = "sovereign")]
    {
        let tmp_path =
            std::env::temp_dir().join(format!("sovereign_test_pass_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&tmp_path);

        let res = builder(128)
            .with_storage_path(&tmp_path)
            .with_encryption_passphrase("sovereign_secret_passphrase")
            .build()
            .await;

        assert!(res.is_ok(), "Sovereign build with passphrase must succeed");
        let db = res.unwrap();
        assert_eq!(db.len().await.unwrap(), 0);

        let _ = std::fs::remove_dir_all(&tmp_path);
    }
}
