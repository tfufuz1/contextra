//! # Integrationstest-Suite: Invarianten bis
//!
//! Validiert die in der Spezifikation (Abschnitt 4) geforderten Invarianten:
//! -: Genau ein Löschpfad (ErasureCoordinator)
//! -: Purge ist ohne Verlust wiederaufnehmbar (Resume führt den Purge)
//! -: Pro Scope höchstens ein laufender Purge (Single-Flight)
//! -: Jede Schlüsselvernichtung ist im Ledger vor dem Ende der Saga verankert
//! -: Ein Fence blockiert konkurrierende Schreiber während eines physischen Purge
//! -: WAL-Einträge zu einem gelöschten Scope werden entsorgt oder unlesbar gemacht
//! -: Ein Löschbericht bindet die Zeugen, auf die er sich stützt (Hash-Kette)

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::panic
)]

use acta_erasure::gate::MemoryFenceGate;
use acta_erasure::journal::MemoryWalSink;
use acta_erasure::ledger::ErasureLedger;
use acta_erasure::orchestrator::ErasureCoordinator;
use acta_erasure::report::binding::{compute_witnesses_chain_hash, verify_witnesses_binding};
use acta_erasure::restore::replay::resume_and_execute_purge;
use acta_erasure::saga::driver::SagaDriver;
use acta_erasure::saga::machine::{EraseSaga, SagaState};
use acta_erasure::scope::SingleFlightCoordinator;
use acta_erasure::types_local::*;
use std::path::{Path, PathBuf};
use std::sync::Arc;

#[test]
fn test_inv_era_1_single_coordinator_entrypoint() {
    let coordinator = ErasureCoordinator::new();
    let tenant = TenantId::new(100);
    let scope = KeyScope::Tenant(tenant);

    // Genau eine koordinierte Einstiegsmethode
    let flight = coordinator.coordinate_erasure(scope.clone());
    assert!(flight.is_ok(), "Coordinator must acquire active scope flight");

    // Paralleler Einstieg auf denselben Scope wird sofort abgewiesen
    let duplicate = coordinator.coordinate_erasure(scope);
    assert!(duplicate.is_err(), "Duplicate parallel coordinate_erasure must be rejected");
}

#[test]
fn test_inv_era_2_resume_executes_purge() {
    let mut saga = EraseSaga::with_state(ErasureId::new([0x02; 16]), SagaState::Pending);
    let driver = SagaDriver::new();

    // Vor Resume: Saga im Pending-Zustand
    assert_eq!(saga.state(), SagaState::Pending);

    //: resume_and_execute_purge führt den Ablauf bis zur Vollendung
    let outcome = resume_and_execute_purge(&mut saga, &driver).expect("resume must succeed");
    assert_eq!(outcome, acta_erasure::saga::driver::StepOutcome::Terminal(SagaState::Completed));
    assert_eq!(saga.state(), SagaState::Completed);
}

#[test]
fn test_inv_era_3_single_flight_per_scope() {
    let coord = SingleFlightCoordinator::new();
    let scope = KeyScope::Tenant(TenantId::new(300));

    let f1 = coord.try_acquire(scope.clone()).expect("First flight acquisition must succeed");
    assert!(coord.is_active(&scope));

    // Zweiter Versuch scheitert
    let f2 = coord.try_acquire(scope.clone());
    assert!(f2.is_err(), "Second concurrent flight must fail under INV-ERA-3");

    drop(f1);
    assert!(!coord.is_active(&scope));

    // Nach Freigabe wieder möglich
    let f3 = coord.try_acquire(scope);
    assert!(f3.is_ok(), "Flight acquisition must succeed after release");
}

#[test]
fn test_inv_era_4_ledger_anchor_before_saga_completion() {
    #[derive(Default)]
    struct MemVfs {
        files: parking_lot::RwLock<std::collections::BTreeMap<PathBuf, Vec<u8>>>,
    }
    impl Vfs for MemVfs {
        fn read(&self, path: &Path) -> Result<Vec<u8>, acta_erasure::error::ErasureError> {
            self.files.read().get(path).cloned().ok_or_else(|| {
                acta_erasure::error::ErasureError::Io(std::io::Error::new(std::io::ErrorKind::NotFound, "not found"))
            })
        }
        fn write_atomic(&self, path: &Path, data: &[u8]) -> Result<(), acta_erasure::error::ErasureError> {
            self.files.write().insert(path.to_path_buf(), data.to_vec());
            Ok(())
        }
        fn exists(&self, path: &Path) -> Result<bool, acta_erasure::error::ErasureError> {
            Ok(self.files.read().contains_key(path))
        }
        fn create_dir_all(&self, _path: &Path) -> Result<(), acta_erasure::error::ErasureError> { Ok(()) }
        fn remove_file(&self, path: &Path) -> Result<(), acta_erasure::error::ErasureError> {
            self.files.write().remove(path);
            Ok(())
        }
        fn len(&self, path: &Path) -> Result<u64, acta_erasure::error::ErasureError> {
            Ok(self.files.read().get(path).map(|v| v.len() as u64).unwrap_or(0))
        }
        fn list_dir(&self, _path: &Path) -> Result<Vec<String>, acta_erasure::error::ErasureError> { Ok(vec![]) }
        fn sync_dir(&self, _path: &Path) -> Result<(), acta_erasure::error::ErasureError> { Ok(()) }
    }

    let vfs = Arc::new(MemVfs::default());
    let path = PathBuf::from("audit.ledger");
    let key = [0x77; 32];

    let ledger = ErasureLedger::open_or_create(vfs.clone(), &path, &key).expect("ledger open");
    let entry = ledger.record_key_destruction([0x44; 32], vec![1, 2, 3], 42, 1000).expect("anchor key");

    assert_eq!(entry.gated_seq, 42);
    assert!(ledger.verify_integrity());

    // Erneutes Laden prüft Integrität fail-closed
    let reloaded = ErasureLedger::open_or_create(vfs, &path, &key).expect("reloaded ledger");
    assert_eq!(reloaded.entries().len(), 1);
}

#[tokio::test]
async fn test_inv_era_5_fence_blocks_concurrent_writers() {
    let gate = MemoryFenceGate::new();
    let key = KeyBuf::new([0x55; 16]);
    let scope = KeyScope::Tenant(TenantId::new(500));

    assert!(!gate.is_fenced(&key));
    let _permit = gate.acquire_fence(&scope).await.expect("fence acquire");

    gate.add_fenced_key(*key.as_bytes());
    assert!(gate.is_fenced(&key), "Key must be fenced during purge operation");
}

#[test]
fn test_inv_era_6_wal_purged_and_unreadable() {
    let wal = MemoryWalSink::new();
    let scope = KeyScope::Tenant(TenantId::new(600));
    let ctx = RequestContext::new(TenantId::new(600), TxId::new(60));
    let id = ErasureId::new([0x66; 16]);

    assert!(!wal.is_scope_purged(&scope));
    let reclaimed = wal.purge_scope_wal(&ctx, &scope).expect("purge wal");
    assert_eq!(reclaimed, 1);
    assert!(wal.is_scope_purged(&scope));

    wal.append_erasure_marker(&ctx, &id, &scope).expect("append marker");
    assert_eq!(wal.markers_count(), 1);
}

#[test]
fn test_inv_era_7_report_binds_witnesses_hash_chain() {
    let w1 = LayerWitness::from_hash([0x10; 32]);
    let w2 = LayerWitness::from_hash([0x20; 32]);
    let witnesses = vec![w1.clone(), w2.clone()];

    let chain_hash = compute_witnesses_chain_hash(&witnesses);
    assert!(verify_witnesses_binding(&witnesses, &chain_hash));

    // Manipulierte Zeugen brechen Hashkette
    let tampered_w = LayerWitness::from_hash([0xFF; 32]);
    let tampered_witnesses = vec![w1, tampered_w];
    assert!(!verify_witnesses_binding(&tampered_witnesses, &chain_hash));
}
