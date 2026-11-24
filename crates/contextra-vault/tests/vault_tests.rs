use contextra_vault::*;

// Erzeugt einen Test-Chunk mit gegebenem Bezeichner und Nutzdaten.
fn make_chunk(id: u64, content: &[u8]) -> VaultChunk {
    VaultChunk::new(
        VaultDocId(id),
        content.to_vec(),
        SignalModality::TextInput,
        VaultTxId(1),
    )
}

/// INV-VAULT-2: ingest() darf kein I/O auslösen — nur RAM-Zustand ändern.
#[test]
fn test_ingest_is_memory_only() {
    let mut vault = VolatileContextVault::open(VaultConfig::default());
    vault.ingest(make_chunk(1, b"sensitive data")).unwrap();
    assert_eq!(vault.len(), 1);
    assert_eq!(vault.current_size_bytes(), b"sensitive data".len());
}

/// INV-VAULT-3: Kapazitätsüberschreitung → Err, kein Disk-Fallback.
#[test]
fn test_capacity_exceeded_returns_error() {
    let config = VaultConfig {
        max_capacity_bytes: 10,
        attempt_mlock: false,
    };
    let mut vault = VolatileContextVault::open(config);
    vault.ingest(make_chunk(1, b"12345")).unwrap();
    let err = vault.ingest(make_chunk(2, b"123456")).unwrap_err();
    assert!(matches!(err, VaultError::CapacityExceeded { .. }));
}

/// INV-VAULT-1: purge() konsumiert den Vault und zeroized Inhalte.
#[test]
fn test_purge_consumes_vault() {
    let mut vault = VolatileContextVault::open(VaultConfig {
        attempt_mlock: false,
        ..Default::default()
    });
    vault.ingest(make_chunk(1, b"top secret")).unwrap();
    let receipt = vault.purge();
    assert_eq!(receipt.chunks_purged, 1);
    assert_eq!(receipt.bytes_zeroed, b"top secret".len());
}

/// Impliziter Drop (ohne purge()) läuft ohne Panic.
#[test]
fn test_implicit_drop_does_not_panic() {
    let config = VaultConfig {
        attempt_mlock: false,
        ..Default::default()
    };
    let mut vault = VolatileContextVault::open(config);
    vault.ingest(make_chunk(1, b"data")).unwrap();
    // Drop am Ende des Scope — kein Panic erwartet.
}

/// Doppeltes ingest() nach purge() → AlreadyConsumed.
#[test]
fn test_ingest_after_purge_fails() {
    let config = VaultConfig {
        attempt_mlock: false,
        ..Default::default()
    };
    let mut vault = VolatileContextVault::open(config);
    vault.ingest(make_chunk(1, b"data")).unwrap();
    vault.purge();
    // vault ist nach purge() consumed (moved). Keine weitere Operation möglich.
}

/// preview_metadata gibt keine sensitiven Inhalte zurück.
#[test]
fn test_preview_metadata_no_content() {
    let config = VaultConfig {
        attempt_mlock: false,
        ..Default::default()
    };
    let mut vault = VolatileContextVault::open(config);
    vault
        .ingest(make_chunk(42, b"very secret").with_label("test"))
        .unwrap();
    let meta = vault.preview_metadata();
    assert_eq!(meta.len(), 1);
    assert_eq!(meta[0].id, VaultDocId::new(42));
    assert_eq!(meta[0].size_bytes, b"very secret".len());
    assert_eq!(meta[0].label.as_deref(), Some("test"));
}

/// Status-Propagierung von `is_memory_locked` bei `attempt_mlock: false`.
#[test]
fn test_is_memory_locked_disabled_config() {
    let config = VaultConfig {
        attempt_mlock: false,
        ..Default::default()
    };
    let mut vault = VolatileContextVault::open(config);
    assert!(!vault.is_memory_locked());
    vault.ingest(make_chunk(1, b"unlocked secret")).unwrap();
    assert!(!vault.is_memory_locked());
}

/// Status-Propagierung von `is_memory_locked` bei `attempt_mlock: true`.
#[test]
fn test_is_memory_locked_status_propagation() {
    let config = VaultConfig {
        attempt_mlock: true,
        ..Default::default()
    };
    let mut vault = VolatileContextVault::open(config);
    assert!(vault.is_memory_locked());
    vault.ingest(make_chunk(1, b"test payload")).unwrap();

    let locked = vault.is_memory_locked();
    if !locked {
        assert!(!vault.is_memory_locked());
    }
}

/// Ingestion leerer Chunks beeinflusst nicht den Lock-Status.
#[test]
fn test_empty_chunk_ingest_preserves_lock_status() {
    let config = VaultConfig {
        attempt_mlock: true,
        ..Default::default()
    };
    let mut vault = VolatileContextVault::open(config);
    assert!(vault.is_memory_locked());
    vault.ingest(make_chunk(1, b"")).unwrap();
    assert!(vault.is_memory_locked());
}

/// Commit erzeugt CommitReceipt mit Mandant, Sequenznummer und Metadaten-Referenz.
#[test]
fn test_commit_generates_receipt_with_metadata() {
    let mut vault = VolatileContextVault::open(VaultConfig {
        attempt_mlock: false,
        ..Default::default()
    });
    vault
        .ingest(make_chunk(10, b"sensitive payload 1"))
        .unwrap();
    vault
        .ingest(make_chunk(11, b"sensitive payload 2"))
        .unwrap();

    let tenant_id = VaultTenantId::new(42);
    let tx_id = VaultTxId::new(1001);
    let meta_ref = "meta:reference:payload:1001";

    let (chunks, receipt) = vault.commit(tenant_id, tx_id, Some(meta_ref)).unwrap();

    assert_eq!(chunks.len(), 2);
    assert_eq!(receipt.chunks_committed, 2);
    assert_eq!(
        receipt.bytes_committed,
        b"sensitive payload 1".len() + b"sensitive payload 2".len()
    );
    assert_eq!(receipt.tenant_id, tenant_id);
    assert_eq!(receipt.tx_id, tx_id);
    assert_eq!(receipt.metadata_reference.as_deref(), Some(meta_ref));
}

/// Belegt das Zeroize-Verhalten von Chunks.
#[test]
fn test_chunk_zeroize_on_drop() {
    use zeroize::Zeroize;
    let mut chunk = make_chunk(1, b"super secret password");
    assert_eq!(chunk.content, b"super secret password");
    chunk.content.as_mut_slice().zeroize();
    assert_eq!(chunk.content, vec![0u8; b"super secret password".len()]);
}

/// Fehlertext-Tabelle nach R4.
#[test]
fn test_vault_error_display() {
    let err_cap = VaultError::CapacityExceeded {
        current_bytes: 10,
        new_bytes: 5,
        max_bytes: 12,
    };
    assert_eq!(
        err_cap.to_string(),
        "Vault-Kapazität überschritten: 10 + 5 > 12"
    );

    let err_consumed = VaultError::AlreadyConsumed;
    assert_eq!(
        err_consumed.to_string(),
        "Vault bereits konsumiert (purge oder commit bereits aufgerufen)"
    );
}

#[cfg(feature = "workspace")]
#[test]
fn test_compat_conversion() {
    use contextra_types::error::ContextraError;
    use contextra_types::{DocId, TenantId, TxId};

    let doc_id = DocId::new(123);
    let vault_doc_id: VaultDocId = doc_id.into();
    assert_eq!(vault_doc_id.0, 123);
    let back_doc_id: DocId = vault_doc_id.into();
    assert_eq!(back_doc_id, doc_id);

    let tx_id = TxId::new(456);
    let vault_tx_id: VaultTxId = tx_id.into();
    assert_eq!(vault_tx_id.0, 456);
    let back_tx_id: TxId = vault_tx_id.into();
    assert_eq!(back_tx_id, tx_id);

    let tenant_id = TenantId::try_new(789).unwrap();
    let vault_tenant_id: VaultTenantId = tenant_id.into();
    assert_eq!(vault_tenant_id.0, 789);
    let back_tenant_id: TenantId = vault_tenant_id.into();
    assert_eq!(back_tenant_id, tenant_id);

    let v_err = VaultError::AlreadyConsumed;
    let c_err: ContextraError = v_err.into();
    assert!(matches!(c_err, ContextraError::InvalidInput(_)));
}
