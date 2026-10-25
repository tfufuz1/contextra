// FILE-CONTEXT
// ZWECK: Review-Test (T-DEL-2) für Stub-Löschbeweise in Contextra (Ring 4 / Engine).
// INVARIANTEN: Prüft, dass Löschbeweise mit Restzahlen (> 0) zwingend als Err abgelehnt werden.

use contextra_core::{TenantId, TxId};
use contextra_db::engine::{DeletionLayer, DeletionProof, DeletionScope, LayerCleanupProof};

/// T-DEL-2: Ein Beweis mit remaining_count > 0 muss Err liefern.
/// Auf dem aktuellen Stand (no_crypto_stubs) verwirft `verify_and_create` das Ergebnis des Verifiers
/// und `new_after_verified_empty` akzeptiert jede Restzahl, weshalb dieser Test rot sein muss.
#[test]
fn test_tdel_2_proof_with_remaining_items_must_fail() {
    // 1. `new_after_verified_empty` mit remaining_count > 0 (z. B. 5) muss Err liefern
    let res_direct = LayerCleanupProof::new_after_verified_empty(DeletionLayer::LsmMemtable, 5);
    assert!(
        res_direct.is_err(),
        "T-DEL-2: LayerCleanupProof::new_after_verified_empty mit remaining_count = 5 muss Err liefern, lieferte aber {:?}",
        res_direct
    );

    // 2. `verify_and_create` mit Verifier, der false (nicht leer) liefert, muss Err liefern
    let res_verify = LayerCleanupProof::verify_and_create(
        DeletionLayer::LsmMemtable,
        || Ok(false), // Verifier signalisiert, dass noch Restelemente existieren
    );
    assert!(
        res_verify.is_err(),
        "T-DEL-2: LayerCleanupProof::verify_and_create muss Err liefern, wenn Verifier false liefert, lieferte aber {:?}",
        res_verify
    );

    // 3. DeletionProof::create mit einem LayerCleanupProof, der remaining_count > 0 hat, muss Err liefern
    let scope = DeletionScope::Tenant {
        tenant_id: TenantId::try_new(1).expect("valid tenant_id"),
    };
    let invalid_layer_proof = LayerCleanupProof {
        layer: DeletionLayer::LsmMemtable,
        remaining_count: 5,
    };
    let proof_res = DeletionProof::create(
        scope,
        vec![],
        TxId::new(1),
        vec![invalid_layer_proof],
        vec![],
        b"proof_key_32_bytes_1234567890123",
    );
    assert!(
        proof_res.is_err(),
        "T-DEL-2: DeletionProof::create mit remaining_count > 0 in LayerCleanupProof muss Err liefern, lieferte aber {:?}",
        proof_res
    );
}
