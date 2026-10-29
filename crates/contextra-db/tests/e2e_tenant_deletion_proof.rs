//! End-to-End Multi-Tenant Isolation & DSGVO-Art.-17 Deletion Proof Integration Test.
//!
//! Workflow:
//! 1. Paralleler Einsatz von 2 Mandanten (Tenant A = 101, Tenant B = 202).
//! 2. Dokumente einfügen für beide Mandanten.
//! 3. Hybride Suche ausführen — streng getrennte Ergebnisse je Mandant.
//! 4. Collection-Löschung für Mandant A via `drop_collection` mit Ed25519/HMAC Löschnachweis.
//! 5. Löschnachweis von Mandant A mathematisch verifizieren.
//! 6. Suche für Mandant A liefert nichts mehr.
//! 7. Wiederöffnen der Datenbank von Disk (Persistenz-Check).
//! 8. Mandant A ist weiterhin leer / gelöscht; Mandant B ist unverändert und sieht nie Daten/Beweise von Mandant A.

#![allow(deprecated)]

use contextra_db::prelude::*;
use serde_json::json;
use tempfile::TempDir;

#[tokio::test(flavor = "multi_thread")]
async fn test_e2e_multi_tenant_deletion_proof_persistence_isolation() -> contextra_types::Result<()>
{
    let tmp_dir = TempDir::new().expect("temp dir for e2e test");
    let db_path = tmp_dir.path().to_path_buf();

    let tenant_a = TenantId::try_new(101)?;
    let tenant_b = TenantId::try_new(202)?;

    let proof_key_a = b"proof-secret-key-for-tenant-a-101";
    let proof_key_b = b"proof-secret-key-for-tenant-b-202";

    let col_a_name = "col_a";
    let col_b_name = "col_b";

    let config = ContextraConfig {
        dimension: 4,
        embedding_backend: EmbeddingBackend::None,
        group_commit_window_micros: 0,
        ..Default::default()
    };

    // =========================================================================
    // STUFE 1: Öffnen, Einfügen & Multi-Tenant Hybrid-Suche
    // =========================================================================
    {
        let db = Contextra::open_with_config(&db_path, config.clone()).await?;

        let col_a = db.collection_for_tenant(col_a_name, tenant_a).await?;
        let col_b = db.collection_for_tenant(col_b_name, tenant_b).await?;

        // Mandant A schreibst sensitive Rust/DB-Dokumente
        col_a
            .insert(
                "doc_a_1",
                &[1.0, 0.0, 0.0, 0.0],
                Some(
                    json!({ "text": "rust memory safety and async runtime", "owner": "tenant_a" }),
                ),
            )
            .await?;
        col_a
            .insert(
                "doc_a_2",
                &[0.9, 0.1, 0.0, 0.0],
                Some(
                    json!({ "text": "database transaction isolation levels", "owner": "tenant_a" }),
                ),
            )
            .await?;

        // Mandant B schreibt ML-Dokumente
        col_b
            .insert(
                "doc_b_1",
                &[0.0, 1.0, 0.0, 0.0],
                Some(json!({ "text": "python machine learning models", "owner": "tenant_b" })),
            )
            .await?;
        col_b
            .insert(
                "doc_b_2",
                &[0.0, 0.9, 0.1, 0.0],
                Some(json!({ "text": "neural network gradient descent", "owner": "tenant_b" })),
            )
            .await?;

        db.flush().await?;

        // Verification: Mandant A sieht ausschließlich Mandant A-Daten
        let results_a = col_a
            .query()
            .text("rust memory")
            .embedding([1.0, 0.0, 0.0, 0.0])
            .k(5)
            .execute()
            .await?;
        assert!(!results_a.is_empty(), "Mandant A muss Treffer finden");
        for r in &results_a {
            assert!(
                r.id.starts_with("doc_a_"),
                "Mandant A darf nur doc_a_ sehen, sah: {}",
                r.id
            );
        }

        // Verification: Mandant B sieht ausschließlich Mandant B-Daten
        let results_b = col_b
            .query()
            .text("python machine")
            .embedding([0.0, 1.0, 0.0, 0.0])
            .k(5)
            .execute()
            .await?;
        assert!(!results_b.is_empty(), "Mandant B muss Treffer finden");
        for r in &results_b {
            assert!(
                r.id.starts_with("doc_b_"),
                "Mandant B darf nur doc_b_ sehen, sah: {}",
                r.id
            );
        }

        // Cross-Tenant Search Check: Mandant B führt Abfrage aus -> darf KEINE Mandant A Dokumente sehen
        let cross_search_b = col_b
            .query()
            .text("rust memory safety")
            .embedding([1.0, 0.0, 0.0, 0.0])
            .k(5)
            .execute()
            .await?;
        for r in &cross_search_b {
            assert!(
                !r.id.starts_with("doc_a_"),
                "Mandant B darf niemals Daten von Mandant A sehen, sah: {}",
                r.id
            );
        }

        // =========================================================================
        // STUFE 2: Physikalische Löschung für Mandant A & Löschnachweis-Prüfung
        // =========================================================================
        let proof_a = db
            .drop_collection(col_a_name, tenant_a, proof_key_a)
            .await?;

        // 1. Proof Eigenschaften prüfen
        assert_eq!(proof_a.tenant_id(), tenant_a);
        let verified_a = proof_a.verify(proof_key_a)?;
        assert!(
            verified_a,
            "Löschnachweis für Mandant A muss mit proof_key_a mathematisch verifiziert werden"
        );

        // 2. Falscher Key (Schlüssel von Mandant B) muss fehlschlagen
        let verified_with_wrong_key = proof_a.verify(proof_key_b)?;
        assert!(
            !verified_with_wrong_key,
            "Löschnachweis von Mandant A darf nicht mit dem Schlüssel von Mandant B verifiziert werden"
        );

        // 3. Nach Löschung: Suche für Mandant A liefert nichts mehr
        let col_a_reopened = db.collection_for_tenant(col_a_name, tenant_a).await?;
        assert_eq!(col_a_reopened.len().await, 0);
        let post_delete_search_a = col_a_reopened
            .query()
            .text("rust memory")
            .embedding([1.0, 0.0, 0.0, 0.0])
            .k(5)
            .execute()
            .await?;
        assert!(
            post_delete_search_a.is_empty(),
            "Suche für Mandant A muss nach Löschung leer sein"
        );

        // 4. Mandant B ist unberührt und funktioniert weiterhin
        let results_b_still_present = col_b
            .query()
            .text("python machine")
            .embedding([0.0, 1.0, 0.0, 0.0])
            .k(5)
            .execute()
            .await?;
        assert_eq!(
            results_b_still_present.len(),
            2,
            "Mandant B muss weiterhin alle seine Dokumente behalten"
        );

        db.close().await?;
    }

    // =========================================================================
    // STUFE 3: Wiederöffnen der Datenbank von Disk (Persistenz-Check)
    // =========================================================================
    {
        let db_reopened = Contextra::open_with_config(&db_path, config).await?;

        let col_a_after_reopen = db_reopened
            .collection_for_tenant(col_a_name, tenant_a)
            .await?;
        let col_b_after_reopen = db_reopened
            .collection_for_tenant(col_b_name, tenant_b)
            .await?;

        // Mandant A bleibt nach Reopen leer
        assert_eq!(col_a_after_reopen.len().await, 0);
        let search_a_reopen = col_a_after_reopen
            .query()
            .text("rust memory")
            .embedding([1.0, 0.0, 0.0, 0.0])
            .k(5)
            .execute()
            .await?;
        assert!(
            search_a_reopen.is_empty(),
            "Mandant A muss auch nach Neustart/Reopen gelöscht und leer bleiben"
        );

        // Isolations-Nachweis: Mandant B sieht weiterhin keinerlei Reste von Mandant A
        let search_b_for_a_reopen = col_b_after_reopen
            .query()
            .text("rust memory")
            .embedding([1.0, 0.0, 0.0, 0.0])
            .k(5)
            .execute()
            .await?;
        for r in &search_b_for_a_reopen {
            assert!(
                !r.id.starts_with("doc_a_"),
                "Mandant B darf auch nach Reopen keine Reste von Mandant A sehen, sah: {}",
                r.id
            );
        }

        db_reopened.close().await?;
    }

    Ok(())
}
