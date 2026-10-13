// FILE-CONTEXT
// STAND: 2026-10-13
// ZWECK: Integrationstests zur Verifizierung der Trennung von Beweis-Signierschlüsseln von öffentlicher db_instance_id (CWE-330).
// INVARIANTEN: Signierschlüssel basieren auf CSPRNG/Keystore, nie auf db_instance_id. Identische db_instance_id erzeugt unterschiedliche Schlüssel. Fehlende Schlüsselquelle schlägt fail-closed fehl.

use contextra_crypto::deletion_proof::claim_text::{WitAssurance, WitClaimText};
use contextra_crypto::deletion_proof::evidence::WitLayerEvidence;
use contextra_crypto::deletion_proof::probe::{WitStoreGeneration, WitStoreProbe};
use contextra_crypto::deletion_proof::signer::{ProofKeyPairSigner, ProofSigner};
use contextra_crypto::deletion_proof::types::{DeletionLayer, DeletionScope};
use contextra_crypto::deletion_witness::{LayerWitness, WitIssueArgs};
use contextra_engine::deletion::forget_options::ForgetOptions;
use contextra_engine::deletion::proof_issue::{
    db_instance_id_to_secret_legacy_test_only, DpiDeletionRuntime, DpiLogicalInput,
    ProofSignerSource,
};
use contextra_types::{ContextraError, DocId, Result, TenantId, TxId};
use tempfile::TempDir;
use zeroize::Zeroizing;

struct DummyProbe;
impl WitStoreProbe for DummyProbe {
    fn current_generation(&self, _layer: &DeletionLayer) -> WitStoreGeneration {
        WitStoreGeneration(0)
    }

    fn scope_write_seq(&self, _scope_hash: &[u8; 32]) -> u64 {
        0
    }

    fn manifest_file_count(&self) -> Option<u32> {
        None
    }
}

/// Test-Schlüsselquelle, die kontrolliert einen Fehler liefert.
struct FailingSignerSource;

impl ProofSignerSource for FailingSignerSource {
    fn get_signer_seed(&self) -> Result<Zeroizing<[u8; 32]>> {
        Err(ContextraError::Crypto(
            "Schlüsselquelle nicht verfügbar (beabsichtigter Testfehler)".to_string(),
        ))
    }
}

/// (1) Zwei Instanzen mit identischer `db_instance_id` erzeugen verschiedene Signierschlüssel.
#[test]
fn test_identical_db_instance_id_generates_different_signing_keys() -> Result<()> {
    let dir1 = TempDir::new().map_err(|e| ContextraError::Crypto(e.to_string()))?;
    let dir2 = TempDir::new().map_err(|e| ContextraError::Crypto(e.to_string()))?;

    let key_path1 = dir1.path().join("signer.key");
    let key_path2 = dir2.path().join("signer.key");

    let db_instance_id = [0x42u8; 16];

    let rt1 = DpiDeletionRuntime::open(dir1.path(), Some(&key_path1), db_instance_id)?;
    let rt2 = DpiDeletionRuntime::open(dir2.path(), Some(&key_path2), db_instance_id)?;

    assert_eq!(rt1.db_instance_id(), db_instance_id);
    assert_eq!(rt2.db_instance_id(), db_instance_id);

    assert_ne!(
        rt1.signer().public_key(),
        rt2.signer().public_key(),
        "Zwei Instanzen mit identischer db_instance_id MÜSSEN unterschiedliche Public Keys erzeugen"
    );

    assert_ne!(
        rt1.signer().key_id(),
        rt2.signer().key_id(),
        "Zwei Instanzen mit identischer db_instance_id MÜSSEN unterschiedliche key_ids erzeugen"
    );

    Ok(())
}

/// (2) Aus einem ausgestellten Beweis lässt sich kein gültiger Fälschungsbeweis mit ID-Ableitung bilden.
#[test]
fn test_issued_proof_cannot_be_forged_using_id_derivation() -> Result<()> {
    let dir = TempDir::new().map_err(|e| ContextraError::Crypto(e.to_string()))?;
    let key_path = dir.path().join("signer.key");
    let db_instance_id = [0x99u8; 16];

    let rt = DpiDeletionRuntime::open(dir.path(), Some(&key_path), db_instance_id)?;

    let deleted_keys_hash = [0x11u8; 32];
    let footprint_hash = [0x22u8; 32];

    let witnesses = vec![LayerWitness::issue(WitIssueArgs {
        layer: DeletionLayer::LsmMemtable,
        scope_hash: deleted_keys_hash,
        scanned_at_tx: TxId::try_new(100).map_err(|e| ContextraError::Crypto(e.to_string()))?,
        as_of_tx: TxId::try_new(100).map_err(|e| ContextraError::Crypto(e.to_string()))?,
        assurance: WitAssurance::S0,
        claim: WitClaimText::S0,
        footprint_hash,
        scope_write_seq: 0,
        scanner_name: "test_scanner",
        scanner_version: 1,
        store_generation: WitStoreGeneration(0),
        layer_evidence: WitLayerEvidence::Memtable {
            memtables_scanned: 1,
            entries_found: 0,
            tombstones_found: 0,
        },
        key_identifiers_claimed: false,
    })?];

    let input = DpiLogicalInput {
        scope: DeletionScope::Document {
            doc_id: DocId::from(1001u64),
            tenant_id: TenantId(1),
        },
        storage_scope: "test_tenant".to_string(),
        deleted_keys_hash,
        footprint_hash,
        observed_empty: true,
        deleted_after_tx: TxId::try_new(100).map_err(|e| ContextraError::Crypto(e.to_string()))?,
        witnesses,
        scope_file_names: vec![],
    };

    let opts = ForgetOptions::new().with_assurance(WitAssurance::S0);
    let dummy_probe = DummyProbe;
    let probes: &[&dyn WitStoreProbe] = &[&dummy_probe];

    let (_handle, issued) = rt.finalize_logical(input, &opts, None, probes)?;

    let actual_key_id = rt.signer().key_id();
    assert_eq!(issued.proof.signer_key_id, actual_key_id);

    // Versuche, aus der öffentlichen db_instance_id einen vermuteten Fälschungs-Signierer abzuleiten.
    let legacy_secret = db_instance_id_to_secret_legacy_test_only(&db_instance_id);
    let legacy_signer = ProofKeyPairSigner::from_secret_bytes(legacy_secret);

    assert_ne!(
        issued.proof.signer_key_id,
        legacy_signer.key_id(),
        "Der echte Beweis darf NICHT mit der aus db_instance_id abgeleiteten Key-ID übereinstimmen"
    );

    assert_ne!(
        rt.signer().public_key(),
        legacy_signer.public_key(),
        "Der tatsächliche Signierschlüssel darf NICHT dem aus db_instance_id abgeleiteten Schlüssel entsprechen"
    );

    Ok(())
}

/// (3) Fehlende Schlüsselquelle liefert Fehler (fail-closed).
#[test]
fn test_failing_key_source_fails_closed() -> Result<()> {
    let dir = TempDir::new().map_err(|e| ContextraError::Crypto(e.to_string()))?;
    let db_instance_id = [0x77u8; 16];

    let failing_source = FailingSignerSource;
    let res = DpiDeletionRuntime::open_with_source(dir.path(), &failing_source, db_instance_id);

    assert!(
        res.is_err(),
        "Eine fehlerhafte/fehlende Schlüsselquelle MUSS fail-closed mit Err fehlschlagen"
    );

    if let Err(e) = res {
        let msg = e.to_string();
        assert!(
            msg.contains("Schlüsselquelle nicht verfügbar"),
            "Fehlermeldung sollte die Ursache der Schlüsselquelle enthalten: {msg}"
        );
    }

    Ok(())
}
