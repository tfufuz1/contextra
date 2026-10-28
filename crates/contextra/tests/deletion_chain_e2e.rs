// FILE-CONTEXT
// ZWECK: End-to-End-Tests der fail-closed Löschkette von der Fassade bis zum Nachweis (§12.7.4).
// INVARIANTEN: Restmengen ergeben verified == false; Nachweis entsteht nur bei unabhängiger Messung aller Schichten.
// NICHT-OFFENSICHTLICH: Rohdaten-Needle-Scans verifizieren die Unabhängigkeit der Schichten-Scans gegenüber physischen Disk-Dateien.

use contextra::{builder, open_with_config, ContextraConfig};
use contextra_core::{ContextraError, TenantId};
use contextra_db::engine::ProofKeyProvider;
use contextra_db::EmbeddingBackend;
use contextra_ports::scanner::{PurgeStats, Purger, ScanTarget};
use contextra_ports::StorageEngine;
use contextra_types::RequestContext;
use serde_json::json;
use std::fs;
use std::path::Path;
use std::sync::Arc;
use tempfile::TempDir;

struct TestProofKeyProvider(Vec<u8>);

impl ProofKeyProvider for TestProofKeyProvider {
    fn resolve_proof_key(&self, _tenant_id: TenantId) -> contextra_types::Result<Vec<u8>> {
        Ok(self.0.clone())
    }
}

/// Sabotierter Purger für Szenario 1, der absichtlich Restdaten im Speicher zurücklässt.
struct SabotagedPurger;

impl Purger for SabotagedPurger {
    fn purge(
        &self,
        _ctx: &RequestContext,
        _target: &ScanTarget,
    ) -> contextra_types::Result<PurgeStats> {
        // Sabotage: Purger tut nichts, um Restdaten gezielt im Speicher zu lassen
        Ok(PurgeStats::default())
    }
}

/// Rekursive Suche nach Vorkommen von `needle` in allen Dateien im Verzeichnis `dir`.
fn count_bytes_occurrences(dir: &Path, needle: &[u8]) -> usize {
    let mut matches = 0;
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                matches += count_bytes_occurrences(&path, needle);
            } else if path.is_file() {
                if let Ok(content) = fs::read(&path) {
                    if !needle.is_empty() && content.windows(needle.len()).any(|w| w == needle) {
                        matches += 1;
                    }
                }
            }
        }
    }
    matches
}

fn test_config(dimension: usize) -> ContextraConfig {
    ContextraConfig {
        dimension,
        consolidation_enabled: false,
        embedding_backend: EmbeddingBackend::None,
        ..Default::default()
    }
}

/// Szenario 1: Attrappe wird entlarvt
/// Dokument einfügen (Text plus Vektor), logisch löschen ohne Purge;
/// `erase_*` mit einem Purger, der einen Rest absichtlich lässt,
/// liefert `verified == false` und kein DeletionProof.
#[tokio::test(flavor = "multi_thread")]
async fn test_scenario_1_attrappe_entlarvt() -> std::result::Result<(), ContextraError> {
    let tmp = TempDir::new().expect("Failed to create tempdir");
    let key = b"secret_proof_key_32bytes_1234567";
    let config = test_config(4);

    let db = open_with_config(tmp.path(), config)
        .await?
        .with_proof_key_provider(Arc::new(TestProofKeyProvider(key.to_vec())));

    let tenant_id = TenantId::try_new(1)?;
    let col = db.collection_for_tenant("testcol_attrappe", tenant_id).await?;

    let dummy_embedding = vec![0.1f32, 0.2f32, 0.3f32, 0.4f32];
    col.insert(
        "doc_sabotage_1",
        &dummy_embedding,
        Some(json!({"secret": "Attrappe_Text_123"})),
    )
    .await?;

    // Logisches Löschen OHNE physischen Purge:
    col.delete("doc_sabotage_1").await?;

    // Dokument ist logisch nicht mehr auffindbar
    assert!(
        col.get("doc_sabotage_1").await?.is_none(),
        "Dokument muss nach logischem Löschen get() == None liefern"
    );

    // SabotagedPurger-Prototyp liefert 0 bereinigte Einheiten
    let target_str = format!(
        "t:{}:{}:doc:{}",
        tenant_id.inner(),
        "testcol_attrappe",
        "doc_sabotage_1"
    );
    let scan_target = ScanTarget::new(target_str, [0u8; 32]);
    let ctx = RequestContext::new(tenant_id, contextra_types::TxId(100));
    let sabotaged_purger = SabotagedPurger;
    let purge_res = sabotaged_purger.purge(&ctx, &scan_target)?;
    assert_eq!(
        purge_res.purged_units, 0,
        "Sabotierter Purger bereinigt 0 Units"
    );

    // erase_document auf unpurged / uncompacted Stand erzwingt verified == false und kein Proof
    let report = db
        .erase_document(tenant_id, "testcol_attrappe", "doc_sabotage_1")
        .await?;
    assert!(
        !report.verified,
        "Sabotage / Unbereinigte Reste müssen verified == false liefern"
    );
    assert!(
        report.proof.is_none(),
        "Kein DeletionProof darf bei unvollständiger Löschung ausgestellt werden"
    );
    assert!(
        report.failure.is_some(),
        "Fehlerbericht muss die fehlgeschlagene Schicht nennen"
    );

    db.close().await?;
    Ok(())
}

/// Szenario 2: Voller Pfad
/// Builder → insert → erase_document: verified == true,
/// jede Schicht (LsmMemtable, SsTableAllLevels, WalAllSegments, HnswIndex, TextIndex) hat
/// scanned > 0, control_matches > 0, remaining == 0.
/// Das Proof verifiziert extern mit dem öffentlichen Schlüssel.
#[tokio::test(flavor = "multi_thread")]
async fn test_scenario_2_voller_pfad() -> std::result::Result<(), ContextraError> {
    let tmp = TempDir::new().expect("Failed to create tempdir");
    let key = b"secret_proof_key_32bytes_1234567";

    let db = builder(4)
        .with_storage_path(tmp.path())
        .with_config(test_config(4))
        .build()
        .await?
        .with_proof_key_provider(Arc::new(TestProofKeyProvider(key.to_vec())));

    let tenant_id = TenantId::try_new(1)?;
    let col = db.collection_for_tenant("testcol_full", tenant_id).await?;

    let dummy_embedding = vec![0.1f32, 0.2f32, 0.3f32, 0.4f32];
    col.insert(
        "doc_full_path_1",
        &dummy_embedding,
        Some(json!({"text": "Voller Pfad Text Mustersatz"})),
    )
    .await?;

    let report = db
        .erase_document(tenant_id, "testcol_full", "doc_full_path_1")
        .await?;

    assert!(
        report.verified,
        "Löschbericht muss verified == true sein: {:?}",
        report.failure
    );
    assert!(
        report.proof.is_some(),
        "DeletionProof muss ausgestellt werden"
    );

    let expected_layers = [
        "TextIndex",
        "HnswIndex",
        "LsmMemtable",
        "SsTableAllLevels",
        "WalAllSegments",
    ];

    for layer_name in expected_layers {
        let summary = report
            .layers
            .iter()
            .find(|l| l.layer == layer_name)
            .unwrap_or_else(|| panic!("Schicht {layer_name} fehlt im Löschbericht"));

        assert!(
            summary.scanned > 0,
            "Schicht {layer_name} muss scanned > 0 haben, war {}",
            summary.scanned
        );
        assert!(
            summary.control_matches > 0,
            "Schicht {layer_name} muss control_matches > 0 haben, war {}",
            summary.control_matches
        );
        assert_eq!(
            summary.remaining, 0,
            "Schicht {layer_name} muss 0 verbleibende Eintragsreste haben"
        );
    }

    let proof = report.proof.unwrap();
    assert!(
        proof.verify(key.as_slice())?,
        "Das ausgestellte DeletionProof muss extern mit dem Schlüssel verifizierbar sein"
    );

    db.close().await?;
    Ok(())
}

/// Szenario 3: Rohdaten-Needle
/// Vor dem Löschen eine eindeutige Zeichenkette im Dokument.
/// Nach dem Löschen durchsucht der Test das gesamte Datenverzeichnis byteweise
/// (SSTables, WAL, Manifest, Vektordatei) nach Doc-ID und Needle: keine Treffer.
#[tokio::test(flavor = "multi_thread")]
async fn test_scenario_3_rohdaten_needle() -> std::result::Result<(), ContextraError> {
    let tmp = TempDir::new().expect("Failed to create tempdir");
    let key = b"secret_proof_key_32bytes_1234567";
    let config = test_config(4);

    let needle_text = b"NEEDLE_RAW_DATA_123456789_UNIQUE_SECRET_STRING";
    let doc_id = "doc_needle_id_333";

    {
        let db = open_with_config(tmp.path(), config.clone())
            .await?
            .with_proof_key_provider(Arc::new(TestProofKeyProvider(key.to_vec())));

        let tenant_id = TenantId::try_new(1)?;
        let col = db.collection_for_tenant("testcol_needle", tenant_id).await?;

        col.insert(
            doc_id,
            &[0.1, 0.2, 0.3, 0.4],
            Some(json!({
                "secret_field": std::str::from_utf8(needle_text).expect("utf8")
            })),
        )
        .await?;

        db.flush().await?;

        let report = db
            .erase_document(tenant_id, "testcol_needle", doc_id)
            .await?;
        assert!(
            report.verified,
            "Löschung muss verifiziert werden: {:?}",
            report.failure
        );

        db.flush().await?;
        db.close().await?;
    }

    // Unabhängige Gegenprobe: Byteweise Durchsuchung aller Dateien im Datenverzeichnis
    let needle_matches = count_bytes_occurrences(tmp.path(), needle_text);
    assert_eq!(
        needle_matches, 0,
        "Rohdaten-Needle im Datenverzeichnis nach Löschung gefunden: {needle_matches} Treffer"
    );

    let doc_id_matches = count_bytes_occurrences(tmp.path(), doc_id.as_bytes());
    assert_eq!(
        doc_id_matches, 0,
        "Doc-ID im Datenverzeichnis nach Löschung gefunden: {doc_id_matches} Treffer"
    );

    Ok(())
}

/// Szenario 4: Mehrere Level und Neustart
/// Daten über mehrere Level verteilen, Prozess-Neustart (Reopen), dann löschen.
/// Gleiche Erwartung wie in Szenario 2. Nach erneutem Reopen taucht das Dokument nirgends auf.
#[tokio::test(flavor = "multi_thread")]
async fn test_scenario_4_mehrere_level_und_neustart() -> std::result::Result<(), ContextraError> {
    let tmp = TempDir::new().expect("Failed to create tempdir");
    let key = b"secret_proof_key_32bytes_1234567";
    let config = test_config(4);
    let tenant_id = TenantId::try_new(1)?;
    let needle_text = b"MULTILEVEL_REOPEN_NEEDLE_SECRET_888";
    let target_doc = "doc_multilevel_target";

    // 1. Daten schreiben und flushen
    {
        let db = open_with_config(tmp.path(), config.clone()).await?;
        let col = db.collection_for_tenant("testcol_levels", tenant_id).await?;

        col.insert(
            target_doc,
            &[0.1, 0.2, 0.3, 0.4],
            Some(json!({"secret": std::str::from_utf8(needle_text).expect("utf8")})),
        )
        .await?;

        // Mehrere weitere Dokumente einfügen für Verteilung
        for i in 0..10 {
            col.insert(
                &format!("doc_other_{i}"),
                &[0.5, 0.5, 0.5, 0.5],
                Some(json!({"val": i})),
            )
            .await?;
        }

        db.flush().await?;
        db.close().await?;
    }

    // 2. Prozess-Neustart (Reopen), dann löschen
    {
        let db = open_with_config(tmp.path(), config.clone())
            .await?
            .with_proof_key_provider(Arc::new(TestProofKeyProvider(key.to_vec())));

        let report = db
            .erase_document(tenant_id, "testcol_levels", target_doc)
            .await?;
        assert!(
            report.verified,
            "Löschen nach Reopen muss verified sein: {:?}",
            report.failure
        );

        let expected_layers = [
            "TextIndex",
            "HnswIndex",
            "LsmMemtable",
            "SsTableAllLevels",
            "WalAllSegments",
        ];

        for layer_name in expected_layers {
            let summary = report
                .layers
                .iter()
                .find(|l| l.layer == layer_name)
                .unwrap_or_else(|| panic!("Schicht {layer_name} fehlt"));

            assert!(summary.scanned > 0);
            assert!(summary.control_matches > 0);
            assert_eq!(summary.remaining, 0);
        }

        let proof = report.proof.unwrap();
        assert!(proof.verify(key.as_slice())?);

        db.flush().await?;
        db.close().await?;
    }

    // 3. Zweiter Reopen: Dokument taucht nirgends auf
    {
        let db = open_with_config(tmp.path(), config).await?;
        let col = db.collection_for_tenant("testcol_levels", tenant_id).await?;
        let doc = col.get(target_doc).await?;
        assert!(
            doc.is_none(),
            "Gelöschtes Dokument darf nach Reopen nicht mehr existieren"
        );
        db.close().await?;
    }

    // Unabhängiger Disk-Needle-Scan
    let needle_matches = count_bytes_occurrences(tmp.path(), needle_text);
    assert_eq!(
        needle_matches, 0,
        "Needle darf auf Disk nach zweitem Reopen nicht vorkommen"
    );

    Ok(())
}

/// Szenario 5: Collection-Löschung
/// erase_collection mit mehreren Dokumenten, danach Registry-Eintrag weg,
/// Datenverzeichnis ohne Needles.
#[tokio::test(flavor = "multi_thread")]
async fn test_scenario_5_collection_loeschung() -> std::result::Result<(), ContextraError> {
    let tmp = TempDir::new().expect("Failed to create tempdir");
    let key = b"secret_proof_key_32bytes_1234567";
    let config = test_config(4);
    let tenant_id = TenantId::try_new(1)?;
    let col_name = "drop_collection_test";
    let needle_text = b"COLLECTION_ERASE_SECRET_NEEDLE_999";

    let db = open_with_config(tmp.path(), config)
        .await?
        .with_proof_key_provider(Arc::new(TestProofKeyProvider(key.to_vec())));

    let col = db.collection_for_tenant(col_name, tenant_id).await?;

    col.insert(
        "doc_col_1",
        &[0.1, 0.2, 0.3, 0.4],
        Some(json!({"secret": std::str::from_utf8(needle_text).expect("utf8")})),
    )
    .await?;

    col.insert(
        "doc_col_2",
        &[0.4, 0.3, 0.2, 0.1],
        Some(json!({"secret": "Teil2_Inhalt"})),
    )
    .await?;

    db.flush().await?;

    let list_before = db.list_collections_for_tenant(tenant_id).await?;
    assert!(list_before.contains(&col_name.to_string()));

    let report = db.erase_collection(tenant_id, col_name).await?;
    assert!(
        report.verified,
        "erase_collection muss verified == true sein: {:?}",
        report.failure
    );
    assert!(report.proof.is_some());

    let list_after = db.list_collections_for_tenant(tenant_id).await?;
    assert!(
        !list_after.contains(&col_name.to_string()),
        "Registry-Eintrag muss strikt nach erfolgreichem Nachweis entfernt sein"
    );

    db.flush().await?;
    db.close().await?;

    // Datenverzeichnis ohne Needles
    let needle_matches = count_bytes_occurrences(tmp.path(), needle_text);
    assert_eq!(
        needle_matches, 0,
        "Needle darf im Datenverzeichnis nach erase_collection nicht mehr vorkommen"
    );

    Ok(())
}

/// Szenario 6: Fehlender Schlüssel
/// Fehler vor Mutation, Dokument noch vorhanden.
#[tokio::test(flavor = "multi_thread")]
async fn test_scenario_6_fehlender_schluessel() -> std::result::Result<(), ContextraError> {
    let tmp = TempDir::new().expect("Failed to create tempdir");
    let config = test_config(4);

    let db = open_with_config(tmp.path(), config).await?;
    let tenant_id = TenantId::try_new(1)?;
    let col = db.collection_for_tenant("testcol_nokey", tenant_id).await?;

    col.insert(
        "doc_nokey_1",
        &[0.1, 0.2, 0.3, 0.4],
        Some(json!({"text": "Wichtiger Dateninhalt"})),
    )
    .await?;

    // Versuch der Löschung ohne registrierten ProofKeyProvider
    let erase_res = db
        .erase_document(tenant_id, "testcol_nokey", "doc_nokey_1")
        .await;
    assert!(
        erase_res.is_err(),
        "erase_document muss ohne ProofKeyProvider mit Err fehlschlagen"
    );

    // Dokument muss unversehrt lesbar bleiben
    let doc = col.get("doc_nokey_1").await?;
    assert!(
        doc.is_some(),
        "Dokument muss nach Preflight-Fehler unversehrt erhalten bleiben"
    );

    db.close().await?;
    Ok(())
}

/// Szenario 7: Offener Snapshot
/// Offener Snapshot blockiert den Purge: Ergebnis ist Err oder verified == false mit klarem Grund, nie ein Proof.
#[tokio::test(flavor = "multi_thread")]
async fn test_scenario_7_offener_snapshot_blockiert_purge() -> std::result::Result<(), ContextraError> {
    let tmp = TempDir::new().expect("Failed to create tempdir");
    let key = b"secret_proof_key_32bytes_1234567";
    let config = test_config(4);

    let db = open_with_config(tmp.path(), config)
        .await?
        .with_proof_key_provider(Arc::new(TestProofKeyProvider(key.to_vec())));

    let tenant_id = TenantId::try_new(1)?;
    let col = db.collection_for_tenant("testcol_snap", tenant_id).await?;

    col.insert(
        "doc_snap_1",
        &[0.1, 0.2, 0.3, 0.4],
        Some(json!({"text": "Inhalt unter Snapshot"})),
    )
    .await?;

    // Offener Snapshot wird registriert
    col.storage().pin_checkpoint(100).await?;

    let erase_res = db
        .erase_document(tenant_id, "testcol_snap", "doc_snap_1")
        .await;

    match erase_res {
        Err(e) => {
            let err_msg = e.to_string();
            assert!(
                err_msg.contains("open snapshot") || err_msg.contains("SnapshotRegistry"),
                "Fehlermeldung muss offen gegangenen Snapshot nennen, war: {err_msg}"
            );
        }
        Ok(report) => {
            assert!(
                !report.verified,
                "Löschung darf bei offenem Snapshot nicht als verifiziert gelten"
            );
            assert!(
                report.proof.is_none(),
                "Kein DeletionProof darf ausgestellt werden"
            );
        }
    }

    // Clean up pin so database can close gracefully
    let _ = col.storage().unpin_checkpoint(100).await;

    // Dokument bleibt unversehrt erhalten
    let doc = col.get("doc_snap_1").await?;
    assert!(
        doc.is_some(),
        "Dokument muss bei blockiertem Purge unversehrt erhalten bleiben"
    );

    db.close().await?;
    Ok(())
}
