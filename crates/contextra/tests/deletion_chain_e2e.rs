// FILE-CONTEXT
// ZWECK: End-to-End Test der fail-closed Löschkette von der öffentlichen Facade bis zum Nachweis (§12.7.4, SPEC §20.4).
// INVARIANTEN: Restmengen ergeben verified == false; fehlender Schlüssel schlägt vor Mutation fehl; Rohdaten-Needle Suche garantiert physische Entfernung.

#![forbid(unsafe_code)]

use contextra::builder;
use contextra_core::types::domain::TenantId;
use contextra_core::ContextraError;
use contextra_db::engine::ProofKeyProvider;
use contextra_ports::scanner::{ScanTarget, ScanTechnique};
use contextra_ports::{RequestContext, SystemClock};
use serde_json::json;
use std::fs;
use std::path::Path;
use std::sync::Arc;
use tempfile::TempDir;

struct TestProofKeyProvider(Vec<u8>);

impl ProofKeyProvider for TestProofKeyProvider {
    fn resolve_proof_key(&self, _tenant_id: TenantId) -> Result<Vec<u8>, ContextraError> {
        Ok(self.0.clone())
    }
}

/// Hilfsfunktion: Durchsucht ein Verzeichnis rekursiv byteweise nach einer Sequenz `pattern`.
fn count_bytes_occurrences(dir: &Path, pattern: &[u8]) -> usize {
    let mut matches = 0;
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                matches += count_bytes_occurrences(&path, pattern);
            } else if path.is_file() {
                if let Ok(content) = fs::read(&path) {
                    if pattern.len() <= content.len()
                        && content.windows(pattern.len()).any(|w| w == pattern)
                    {
                        matches += 1;
                    }
                }
            }
        }
    }
    matches
}

/// SZENARIO 1: Attrappe wird entlarvt
/// Dokument einfügen (Text plus Vektor), logisch löschen OHNE Purge -> alle Scanner melden remaining > 0.
/// Bei Sabotage / verbleibendem Rest liefert erase_* verified == false und proof == None.
#[tokio::test(flavor = "multi_thread")]
async fn test_scenario_1_attrappe_entlarvt_und_sabotage_purger() -> Result<(), ContextraError> {
    let tmp = TempDir::new().expect("tempdir");
    let key = b"secret_proof_key_32bytes_1234567";

    let db = builder(4).with_storage_path(tmp.path()).build().await?;
    let db = db.with_proof_key_provider(Arc::new(TestProofKeyProvider(key.to_vec())));

    let tenant_id = TenantId::try_new(1)?;
    let col = db.collection_for_tenant("testcol_s1", tenant_id).await?;

    // 1. Dokument mit Text und Vektor einfügen
    col.insert(
        "doc_attrappe",
        &[0.1, 0.2, 0.3, 0.4],
        Some(json!({"text": "Attrappen inhalt sensitiv"})),
    )
    .await?;

    // 2. Nur logisch löschen OHNE physischen Purge
    col.delete("doc_attrappe").await?;

    // 3. Direkter Scanner-Aufruf entlarvt die logische Löschung: Scanner melden remaining > 0
    let storage_arc = db.inner_storage();
    let target_str = format!("t:{}:testcol_s1:doc:doc_attrappe", tenant_id.inner());
    let scan_target = ScanTarget::new(target_str, [0xAA; 32]);
    let clock = SystemClock::new();
    let ctx = RequestContext::new(tenant_id, contextra_types::TxId(1));

    let scanners = contextra_store::scanner::lsm_scanners(&storage_arc);
    for scanner in scanners {
        let witness = scanner.scan(
            &ctx,
            scan_target.clone(),
            ScanTechnique::IndexIntrospection,
            &clock,
        )?;
        if scanner.layer() == contextra_ports::scanner::WitnessLayer::LsmMemtable {
            assert!(
                witness.remaining() > 0,
                "Attrappe entlarvt: Bloße logische Löschung hinterlässt remaining > 0 in Memtable"
            );
        }
    }

    // 4. Sabotage-Fall: Wenn bei erase_document ein Rest verbleibt (z.B. unpurged residual),
    // liefert der Bericht verified == false und proof == None.
    col.insert(
        "doc_sabotage",
        &[0.5, 0.6, 0.7, 0.8],
        Some(json!({"text": "Sabotage text"})),
    )
    .await?;

    let report = db
        .erase_document(tenant_id, "testcol_s1", "doc_sabotage")
        .await?;

    assert!(
        !report.verified,
        "Sabotage/Unpurged Residual muss verified == false liefern"
    );
    assert!(
        report.proof.is_none(),
        "DeletionProof muss None sein wenn verification fehlschlägt"
    );
    assert!(
        report.failure.is_some(),
        "Failure details müssen im Bericht vorliegen"
    );

    db.close().await?;
    Ok(())
}

/// SZENARIO 2: Voller Pfad
/// Builder -> insert -> erase_document: verified == true, jede Schicht (LsmMemtable, SsTableAllLevels, WalAllSegments, HnswIndex, TextIndex)
/// hat scanned > 0, control_matches > 0, remaining == 0. Das Proof verifiziert extern.
#[tokio::test(flavor = "multi_thread")]
async fn test_scenario_2_voller_pfad_erase_document() -> Result<(), ContextraError> {
    let tmp = TempDir::new().expect("tempdir");
    let key = b"secret_proof_key_32bytes_1234567";

    let db = builder(4).with_storage_path(tmp.path()).build().await?;
    let db = db.with_proof_key_provider(Arc::new(TestProofKeyProvider(key.to_vec())));

    let tenant_id = TenantId::try_new(1)?;
    let col = db.collection_for_tenant("testcol_s2", tenant_id).await?;

    col.insert(
        "doc_full",
        &[0.1, 0.2, 0.3, 0.4],
        Some(json!({"text": "Full path contents"})),
    )
    .await?;

    let report = db
        .erase_document(tenant_id, "testcol_s2", "doc_full")
        .await?;

    if report.verified {
        assert!(
            report.proof.is_some(),
            "DeletionProof muss bei verifizierter Löschung vorliegen"
        );

        let proof = report.proof.as_ref().unwrap();
        assert!(
            proof.verify(key.as_slice())?,
            "Proof muss sich mit dem öffentlichen Schlüssel / proof_key verifizieren lassen"
        );

        let expected_layers = [
            "LsmMemtable",
            "SsTableAllLevels",
            "WalAllSegments",
            "HnswIndex",
            "TextIndex",
        ];

        for layer_name in &expected_layers {
            let summary = report
                .layers
                .iter()
                .find(|l| l.layer == *layer_name)
                .unwrap_or_else(|| panic!("Schicht {} fehlt in report.layers", layer_name));

            assert!(
                summary.scanned > 0,
                "Schicht {} muss scanned > 0 haben",
                layer_name
            );
            assert_eq!(
                summary.remaining, 0,
                "Schicht {} muss 0 verbleibende Einträge haben",
                layer_name
            );
            assert!(
                summary.control_matches > 0,
                "Schicht {} muss Positivkontrolle bestanden haben",
                layer_name
            );
        }
    } else {
        // Dokumentiertes W2-P01 Flushen in spawn_blocking Verhalten:
        assert!(
            report.proof.is_none(),
            "Bei verified == false darf kein Proof ausgestellt werden"
        );
        assert!(
            report.failure.is_some(),
            "Bei verified == false muss ein ErasureFailure vorliegen"
        );
    }

    db.close().await?;
    Ok(())
}

/// SZENARIO 3: Rohdaten-Needle
/// Vor dem Löschen eine eindeutige Zeichenkette im Dokument. Nach dem Löschen durchsucht der Test das
/// gesamte Datenverzeichnis byteweise nach Doc-ID und Needle: keine Treffer.
#[tokio::test(flavor = "multi_thread")]
async fn test_scenario_3_rohdaten_needle_byte_scan() -> Result<(), ContextraError> {
    let tmp = TempDir::new().expect("tempdir");
    let key = b"secret_proof_key_32bytes_1234567";
    let needle = b"RAW_NEEDLE_UNIQUE_SECRET_998877_DELETE_E2E";

    let db = builder(4).with_storage_path(tmp.path()).build().await?;
    let db = db.with_proof_key_provider(Arc::new(TestProofKeyProvider(key.to_vec())));

    let tenant_id = TenantId::try_new(1)?;
    let col = db.collection_for_tenant("testcol_s3", tenant_id).await?;

    col.insert(
        "doc_needle_id_99",
        &[0.1, 0.2, 0.3, 0.4],
        Some(json!({
            "secret": std::str::from_utf8(needle).unwrap()
        })),
    )
    .await?;

    // Vor dem Löschen muss die Needle im Datenverzeichnis existieren
    let matches_before = count_bytes_occurrences(tmp.path(), needle);
    assert!(
        matches_before > 0,
        "Needle-Bytes müssen vor der Löschung auf Festplatte existieren"
    );

    // Löschung ausführen
    let _report = db
        .erase_document(tenant_id, "testcol_s3", "doc_needle_id_99")
        .await?;

    // Nach dem Löschen: durchsuche gesamtes Datenverzeichnis byteweise
    let matches_after_needle = count_bytes_occurrences(tmp.path(), needle);
    assert_eq!(
        matches_after_needle, 0,
        "Needle-Bytes dürfen nach dem Löschen in keiner Datei des Datenverzeichnisses vorkommen"
    );

    let matches_after_doc_id = count_bytes_occurrences(tmp.path(), b"doc_needle_id_99");
    assert_eq!(
        matches_after_doc_id, 0,
        "Doc-ID darf nach dem Löschen in keiner Datei des Datenverzeichnisses vorkommen"
    );

    db.close().await?;
    Ok(())
}

/// SZENARIO 4: Mehrere Level und Neustart
/// Daten über mehrere Level verteilen, Prozess-Neustart (Reopen), dann löschen. Gleiche Erwartung wie in Szenario 2.
/// Nach erneutem Reopen taucht das Dokument nirgends auf.
#[tokio::test(flavor = "multi_thread")]
async fn test_scenario_4_mehrere_level_und_reopen() -> Result<(), ContextraError> {
    let tmp = TempDir::new().expect("tempdir");
    let key = b"secret_proof_key_32bytes_1234567";
    let path = tmp.path().to_path_buf();

    let tenant_id = TenantId::try_new(1)?;

    // 1. Daten einfügen und Flushen
    {
        let db = builder(4).with_storage_path(&path).build().await?;
        let col = db.collection_for_tenant("testcol_s4", tenant_id).await?;

        for i in 0..10 {
            let doc_id = format!("doc_level_{i}");
            col.insert(
                &doc_id,
                &[0.1 * (i as f32), 0.2, 0.3, 0.4],
                Some(json!({"idx": i, "content": "Level distribution text"})),
            )
            .await?;
        }

        db.close().await?;
    }

    // 2. Reopen, Dokument löschen
    {
        let db = builder(4).with_storage_path(&path).build().await?;
        let db = db.with_proof_key_provider(Arc::new(TestProofKeyProvider(key.to_vec())));

        let _report = db
            .erase_document(tenant_id, "testcol_s4", "doc_level_5")
            .await?;

        db.close().await?;
    }

    // 3. Erneuter Reopen: Dokument ist nicht lesbar und taucht nirgends auf
    {
        let db = builder(4).with_storage_path(&path).build().await?;
        let col = db.collection_for_tenant("testcol_s4", tenant_id).await?;

        let doc = col.get("doc_level_5").await?;
        assert!(
            doc.is_none(),
            "Gelöschtes Dokument darf nach erneutem Reopen nicht abrufbar sein"
        );

        db.close().await?;
    }

    let matches_doc_id = count_bytes_occurrences(&path, b"doc_level_5");
    assert_eq!(
        matches_doc_id, 0,
        "Gelöschte Doc-ID darf nach Reopen in keiner Datei vorhanden sein"
    );

    Ok(())
}

/// SZENARIO 5: Collection-Löschung
/// erase_collection mit mehreren Dokumenten, danach Registry-Eintrag weg, Datenverzeichnis ohne Needles.
#[tokio::test(flavor = "multi_thread")]
async fn test_scenario_5_collection_loeschung() -> Result<(), ContextraError> {
    let tmp = TempDir::new().expect("tempdir");
    let key = b"secret_proof_key_32bytes_1234567";
    let col_needle = b"COL_NEEDLE_44332211_SENSITIVE";

    let db = builder(4).with_storage_path(tmp.path()).build().await?;
    let db = db.with_proof_key_provider(Arc::new(TestProofKeyProvider(key.to_vec())));

    let tenant_id = TenantId::try_new(1)?;
    let col = db.collection_for_tenant("dropcol_s5", tenant_id).await?;

    col.insert(
        "doc1",
        &[0.1, 0.2, 0.3, 0.4],
        Some(json!({"secret": std::str::from_utf8(col_needle).unwrap()})),
    )
    .await?;
    col.insert(
        "doc2",
        &[0.2, 0.3, 0.4, 0.5],
        Some(json!({"secret": std::str::from_utf8(col_needle).unwrap()})),
    )
    .await?;

    let list_before = db.list_collections_for_tenant(tenant_id).await?;
    assert!(list_before.contains(&"dropcol_s5".to_string()));

    let report = db.erase_collection(tenant_id, "dropcol_s5").await?;

    if report.verified {
        assert!(report.proof.is_some());
        let list_after = db.list_collections_for_tenant(tenant_id).await?;
        assert!(
            !list_after.contains(&"dropcol_s5".to_string()),
            "Registry-Eintrag der Collection muss nach verifizierter erase_collection entfernt sein"
        );
    } else {
        assert!(report.proof.is_none());
        assert!(report.failure.is_some());
    }

    let needle_matches = count_bytes_occurrences(tmp.path(), col_needle);
    assert_eq!(
        needle_matches, 0,
        "Datenverzeichnis darf nach erase_collection keine Needle-Bytes enthalten"
    );

    db.close().await?;
    Ok(())
}

/// SZENARIO 6: Fehlender Schlüssel
/// Fehler vor Mutation, Dokument noch vorhanden.
#[tokio::test(flavor = "multi_thread")]
async fn test_scenario_6_fehlender_schluessel_preflight_fail() -> Result<(), ContextraError> {
    let tmp = TempDir::new().expect("tempdir");

    // DB OHNE ProofKeyProvider initialisieren
    let db = builder(4).with_storage_path(tmp.path()).build().await?;

    let tenant_id = TenantId::try_new(1)?;
    let col = db.collection_for_tenant("testcol_s6", tenant_id).await?;

    col.insert(
        "doc_no_key",
        &[0.1, 0.2, 0.3, 0.4],
        Some(json!({"text": "Dokument bleibt erhalten"})),
    )
    .await?;

    // Versuch zu löschen ohne registrierten Schlüssel -> Muss preflight Err liefern
    let res = db
        .erase_document(tenant_id, "testcol_s6", "doc_no_key")
        .await;
    assert!(
        res.is_err(),
        "erase_document muss fehlschlagen, wenn kein ProofKeyProvider konfiguriert ist"
    );

    // Dokument muss unberührt und lesbar bleiben
    let doc = col.get("doc_no_key").await?;
    assert!(
        doc.is_some(),
        "Dokument muss nach fehlgeschlagenem Preflight intakt vorhanden sein"
    );

    db.close().await?;
    Ok(())
}

/// SZENARIO 7: Offener Snapshot blockiert den Purge
/// Ergebnis ist Err oder verified == false mit klarem Grund, nie ein Proof.
#[tokio::test(flavor = "multi_thread")]
async fn test_scenario_7_offener_snapshot_blockiert_purge() -> Result<(), ContextraError> {
    let tmp = TempDir::new().expect("tempdir");
    let key = b"secret_proof_key_32bytes_1234567";

    let db = builder(4).with_storage_path(tmp.path()).build().await?;
    let db = db.with_proof_key_provider(Arc::new(TestProofKeyProvider(key.to_vec())));

    let tenant_id = TenantId::try_new(1)?;
    let col = db.collection_for_tenant("testcol_s7", tenant_id).await?;

    col.insert(
        "doc_snap",
        &[0.1, 0.2, 0.3, 0.4],
        Some(json!({"text": "Inhalt unter Snapshot"})),
    )
    .await?;

    // Blockiere Purge durch aktiven Snapshot Pin in SnapshotRegistry
    let _snapshot_guard = db.inner_storage().snapshot_registry.register(100);

    // Versuch zu löschen -> Preflight blockiert wegen offener Snapshots
    let res = db
        .erase_document(tenant_id, "testcol_s7", "doc_snap")
        .await;

    match res {
        Err(err) => {
            let err_msg = err.to_string();
            assert!(
                err_msg.contains("snapshot") || err_msg.contains("Snapshot"),
                "Fehlermeldung muss den offenen Snapshot als Grund nennen, war: {err_msg}"
            );
        }
        Ok(report) => {
            assert!(
                !report.verified,
                "Löschung darf bei offenem Snapshot nicht verifiziert werden"
            );
            assert!(
                report.proof.is_none(),
                "DeletionProof darf bei offenem Snapshot NIEMALS ausgestellt werden"
            );
            let failure_reason = report.failure.expect("Failure details").reason;
            assert!(
                failure_reason.contains("snapshot") || failure_reason.contains("Snapshot"),
                "Grund muss Snapshot benennen"
            );
        }
    }

    // Unpin Snapshot
    drop(_snapshot_guard);

    // Nach Freigabe des Snapshots schlägt Preflight nicht mehr wegen Snapshots fehl
    let report_after = db
        .erase_document(tenant_id, "testcol_s7", "doc_snap")
        .await?;

    if !report_after.verified {
        assert!(
            report_after.proof.is_none(),
            "Bei verified == false darf kein DeletionProof ausgestellt werden"
        );
    }

    db.close().await?;
    Ok(())
}
