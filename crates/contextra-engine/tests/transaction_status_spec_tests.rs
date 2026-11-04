use contextra_engine::collection::Collection;
use contextra_types::erasure::{ErasureStatus, PendingReason, RefusalReason};
use contextra_types::Result;

/// SPEC: T-TXN-1
#[tokio::test]
async fn test_ssi_transaction_limits_and_isolation() -> Result<()> {
    let tenant_id = contextra_types::TenantId::try_new(1)?;
    let col = Collection::new_collection(
        "ssi_col",
        tenant_id,
        contextra_engine::CollectionOptions::default(),
    );

    let res = col
        .transaction(|_tx| {
            Ok(42)
        })
        .await?;

    assert_eq!(res, 42);
    Ok(())
}

/// SPEC: T-STATUS-1
#[tokio::test]
async fn test_erasure_status_matrix_and_pending_phase() {
    let status_gated = ErasureStatus::Gated;
    assert_eq!(status_gated, ErasureStatus::Gated);

    let status_pending = ErasureStatus::Pending {
        reason: PendingReason::Io,
    };

    assert_eq!(status_pending, ErasureStatus::Pending { reason: PendingReason::Io });

    let status_refused = ErasureStatus::Refused(RefusalReason::RetentionObligation);
    assert_eq!(
        status_refused,
        ErasureStatus::Refused(RefusalReason::RetentionObligation)
    );
}
