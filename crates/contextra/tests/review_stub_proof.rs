// FILE-CONTEXT
// ZWECK: Review-Test (T-DEL-2) für Stub-Löschbeweise in Contextra.
// INVARIANTEN: Prüft, dass Löschbeweise mit Restzahlen (> 0) oder falschem Schlüssel als Err abgelehnt werden.
// NICHT-OFFENSICHTLICH: Beinhaltet compile_fail Doctests zur Absicherung, dass keine DeletionProof-Erzeugung ohne LayerWitness möglich ist.

use contextra_crypto::deletion_proof::{CustodyLevel, DeletionProof, ExcludedScope};
use contextra_ports::scanner::ScanTechnique;
use contextra_ports::SystemClock;
use contextra_testkit::mint_witness_for_tests;
use contextra_types::{RequestContext, SagaRecord, TenantId, TxId};

/// T-DEL-2: Bericht/Witness mit Restbestand (remaining > 0) muss Err liefern.
#[test]
fn test_proof_with_remaining_items_must_fail() {
    let clock = SystemClock::new();
    let ctx = RequestContext::new(TenantId::try_new(1).unwrap(), TxId(100));
    let saga = SagaRecord::new([0xAA; 32]);
    let hmac_key = b"secret_hmac_key_32_bytes_long!!";

    let witnesses = vec![
        mint_witness_for_tests("LsmMemtable", 10, 5, ScanTechnique::Structural),
        mint_witness_for_tests("SsTableAllLevels", 50, 0, ScanTechnique::Structural),
        mint_witness_for_tests("WalAllSegments", 100, 0, ScanTechnique::Structural),
    ];

    let res = DeletionProof::issue(
        &ctx,
        &saga,
        &witnesses,
        &[ExcludedScope::LlmParameterMemory],
        CustodyLevel::Local,
        hmac_key.as_slice(),
        &clock,
    );

    assert!(
        res.is_err(),
        "T-DEL-2: DeletionProof::issue mit remaining > 0 muss Err liefern"
    );
}

/// T-DEL-2: Verifikation mit falschem Schlüssel muss Err liefern bzw. false ergeben.
#[test]
fn test_proof_verification_with_wrong_key_fails() {
    let clock = SystemClock::new();
    let ctx = RequestContext::new(TenantId::try_new(1).unwrap(), TxId(100));
    let saga = SagaRecord::new([0xAA; 32]);
    let hmac_key = b"secret_hmac_key_32_bytes_long!!";

    let witnesses = vec![
        mint_witness_for_tests("LsmMemtable", 10, 0, ScanTechnique::Structural),
        mint_witness_for_tests("SsTableAllLevels", 50, 0, ScanTechnique::Structural),
        mint_witness_for_tests("WalAllSegments", 100, 0, ScanTechnique::Structural),
    ];

    let proof = DeletionProof::issue(
        &ctx,
        &saga,
        &witnesses,
        &[ExcludedScope::LlmParameterMemory],
        CustodyLevel::Local,
        hmac_key.as_slice(),
        &clock,
    )
    .expect("issue succeeds");

    let wrong_key = b"wrong_hmac_key_32_bytes_long!!!";
    assert!(
        !proof.verify(wrong_key.as_slice()).unwrap(),
        "T-DEL-2: Verifikation mit falschem Schlüssel muss false zurückgeben"
    );
}

/// T-DEL-2: Nachweis ohne Scanner-Witness darf nicht kompilieren.
///
/// ```compile_fail
/// use contextra_crypto::deletion_proof::{DeletionProof, ExcludedScope, CustodyLevel};
/// use contextra_types::{RequestContext, SagaRecord, TenantId, TxId};
/// use contextra_ports::SystemClock;
///
/// let clock = SystemClock::new();
/// let ctx = RequestContext::new(TenantId::try_new(1).unwrap(), TxId(100));
/// let saga = SagaRecord::new([0xAA; 32]);
///
/// // Versuche DeletionProof::issue ohne LayerWitness-Instanzen aufzurufen
/// let _ = DeletionProof::issue(
///     &ctx,
///     &saga,
///     &["DummyWitnessWithoutType"], // Kompiliert nicht, da &[LayerWitness] gefordert ist
///     &[ExcludedScope::LlmParameterMemory],
///     CustodyLevel::Local,
///     b"key",
///     &clock,
/// );
/// ```
fn _doctest_compile_fail_without_witness() {}
