// FILE-CONTEXT
// STAND: 2026-10-15
// ZWECK: Tenant-Matrix-Runner for executing storage test cases over both LsmStorage and TenantScopedStorage (REG-15).
// INVARIANTEN: Zero-panic, fail-closed errors, runs test suite across direct LSM and tenant-scoped storage wrappers.

use contextra_ports::{StorageEngine, TenantId};
use contextra_store::{tenant_codec::TenantScopedStorage, LsmConfig, LsmStorage};
use std::future::Future;
use std::path::PathBuf;
use std::sync::Arc;
use tempfile::tempdir;

/// Storage mode under which a test iteration is executing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TenantMatrixMode {
    /// Direct, untenanted `LsmStorage` engine.
    LsmDirect,
    /// Tenant-isolated `TenantScopedStorage` engine wrapper.
    TenantScoped(TenantId),
}

/// Execution context provided to a tenant matrix test iteration.
pub struct TenantMatrixContext {
    /// Current mode of the storage engine.
    pub mode: TenantMatrixMode,
    /// Arc handle to the storage engine (implements `StorageEngine`).
    pub storage: Arc<dyn StorageEngine>,
    /// Root directory of the storage instance.
    pub dir_path: PathBuf,
    /// Assigned TenantId for this test iteration.
    pub tenant_id: TenantId,
}

/// Helper for executing test closures once over `LsmStorage` and once over `TenantScopedStorage`.
pub struct TenantMatrixRunner;

impl TenantMatrixRunner {
    /// Runs an async test function `f` twice:
    /// 1. Directly against `LsmStorage`
    /// 2. Against `TenantScopedStorage` wrapping `LsmStorage`
    pub async fn run_async<F, Fut>(test_fn: F)
    where
        F: Fn(TenantMatrixContext) -> Fut,
        Fut: Future<Output = ()>,
    {
        let tenant_id = TenantId(101);

        // Iteration 1: LsmStorage direct
        {
            let dir = tempdir().expect("Failed to create temp directory for LSM direct test");
            let mut config = LsmConfig::default();
            config.path = dir.path().to_path_buf();
            let lsm = Arc::new(
                LsmStorage::open(config)
                    .await
                    .expect("Failed to open LsmStorage for LSM direct test"),
            );
            let ctx = TenantMatrixContext {
                mode: TenantMatrixMode::LsmDirect,
                storage: lsm,
                dir_path: dir.path().to_path_buf(),
                tenant_id,
            };
            test_fn(ctx).await;
        }

        // Iteration 2: TenantScopedStorage wrapper
        {
            let dir = tempdir().expect("Failed to create temp directory for TenantScoped test");
            let mut config = LsmConfig::default();
            config.path = dir.path().to_path_buf();
            let lsm = Arc::new(
                LsmStorage::open(config)
                    .await
                    .expect("Failed to open LsmStorage for TenantScoped test"),
            );
            let scoped = Arc::new(TenantScopedStorage::new(lsm, tenant_id));
            let ctx = TenantMatrixContext {
                mode: TenantMatrixMode::TenantScoped(tenant_id),
                storage: scoped,
                dir_path: dir.path().to_path_buf(),
                tenant_id,
            };
            test_fn(ctx).await;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use contextra_types::TxId;

    #[tokio::test]
    async fn test_tenant_matrix_runner_executes_both_modes() {
        TenantMatrixRunner::run_async(|ctx| {
            let mode = ctx.mode;
            let storage = ctx.storage;
            async move {
                let tx = TxId::try_new(1).unwrap();
                storage.put(tx, b"key1", b"val1").await.unwrap();
                storage.commit(tx).await.unwrap();
                let res = storage.get(b"key1").await.unwrap();
                assert_eq!(res.as_deref(), Some(&b"val1"[..]));
                println!("Executed mode: {:?}", mode);
            }
        })
        .await;
    }
}
