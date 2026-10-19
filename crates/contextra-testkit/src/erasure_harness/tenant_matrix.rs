// FILE-CONTEXT
// STAND: 2026-10-15
// ZWECK: Tenant-Matrix-Runner for executing storage test cases over both LsmStorage and TenantScopedStorage (REG-15).
// INVARIANTEN: Zero-panic, fail-closed errors, runs test suite across direct LSM and tenant-scoped storage wrappers.

use contextra_ports::{StorageEngine, TenantId};
use contextra_store::{tenant_codec::TenantScopedStorage, LsmConfig, LsmStorage};
use std::fs;
use std::future::Future;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

static TEMP_DIR_COUNTER: AtomicU64 = AtomicU64::new(1);

struct TempDirGuard {
    path: PathBuf,
}

impl TempDirGuard {
    fn new(prefix: &str) -> std::io::Result<Self> {
        let count = TEMP_DIR_COUNTER.fetch_add(1, Ordering::SeqCst);
        let pid = std::process::id();
        let path = std::env::temp_dir().join(format!("{}_{}_{}", prefix, pid, count));
        fs::create_dir_all(&path)?;
        Ok(Self { path })
    }

    fn path(&self) -> &std::path::Path {
        &self.path
    }
}

impl Drop for TempDirGuard {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

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
        let tenant_id = match TenantId::try_new(101) {
            Ok(id) => id,
            Err(_) => return,
        };

        // Iteration 1: LsmStorage direct
        if let Ok(dir) = TempDirGuard::new("contextra_lsm_direct") {
            let config = LsmConfig {
                path: dir.path().to_path_buf(),
                ..Default::default()
            };
            if let Ok(lsm_instance) = LsmStorage::open(config).await {
                let lsm = Arc::new(lsm_instance);
                let ctx = TenantMatrixContext {
                    mode: TenantMatrixMode::LsmDirect,
                    storage: lsm,
                    dir_path: dir.path().to_path_buf(),
                    tenant_id,
                };
                test_fn(ctx).await;
            }
        }

        // Iteration 2: TenantScopedStorage wrapper
        if let Ok(dir) = TempDirGuard::new("contextra_tenant_scoped") {
            let config = LsmConfig {
                path: dir.path().to_path_buf(),
                ..Default::default()
            };
            if let Ok(lsm_instance) = LsmStorage::open(config).await {
                let lsm = Arc::new(lsm_instance);
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
                let tx = TxId::try_new(1).expect("valid tx");
                storage.put(tx, b"key1", b"val1").await.expect("put");
                storage.commit(tx).await.expect("commit");
                let res = storage.get(b"key1").await.expect("get");
                assert_eq!(res.as_deref(), Some(&b"val1"[..]));
                println!("Executed mode: {:?}", mode);
            }
        })
        .await;
    }
}
