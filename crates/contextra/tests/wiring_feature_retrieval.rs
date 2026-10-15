// FILE-CONTEXT
// STAND: 2026-10-13
// ZWECK: Integration test for Router/Retrieval feature wiring.

#[cfg(feature = "router")]
use contextra::builder;

#[tokio::test]
async fn test_wiring_feature_retrieval() {
    #[cfg(feature = "router")]
    {
        let tmp_path = std::env::temp_dir().join(format!("retrieval_test_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&tmp_path);

        let b = builder(128).with_storage_path(&tmp_path);
        let clock = b.clock().clone();

        let bundle = contextra::wiring_retrieval::RetrievalWiringBundle::build(true, true, clock);
        assert!(bundle.is_ok(), "RetrievalWiringBundle build should succeed");

        let ret = bundle.unwrap();
        assert!(ret.graph_wiring.is_some());
        assert!(ret.router_wiring.is_some());

        let db = b.build().await;
        assert!(
            db.is_ok(),
            "Retrieval feature Contextra build should succeed"
        );

        let _ = std::fs::remove_dir_all(&tmp_path);
    }
}
