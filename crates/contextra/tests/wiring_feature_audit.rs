// FILE-CONTEXT
// STAND: 2026-10-13
// ZWECK: Integration test for audit-export and avv-generator feature wiring.

#[cfg(feature = "audit-export")]
use contextra::builder;
#[cfg(feature = "audit-export")]
use contextra_types::TenantId;

#[tokio::test]
async fn test_wiring_feature_audit_export_and_avv() {
    #[cfg(feature = "audit-export")]
    {
        let tmp_path = std::env::temp_dir().join(format!("audit_test_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&tmp_path);

        let db = builder(64)
            .with_storage_path(&tmp_path)
            .build()
            .await
            .expect("Build with audit export should succeed");

        let audit_bundle = contextra::wiring_audit::AuditWiringBundle::build(
            TenantId::SYSTEM,
            db.license_gate().clock(),
        )
        .expect("AuditWiringBundle build should succeed");

        #[cfg(feature = "avv-generator")]
        {
            let markdown = audit_bundle.render_avv_markdown();
            assert!(markdown.is_ok(), "AVV markdown rendering should succeed");
            let text = markdown.unwrap();
            assert!(
                text.contains("Auftragsverarbeitungsvertrag"),
                "AVV markdown should contain title header"
            );
        }

        let _ = std::fs::remove_dir_all(&tmp_path);
    }
}
