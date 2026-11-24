//! P2 Test-Suite: Swap-Sofortfix (WP-5a) und Rewrite mit Journal und Manifest (WP-5).
//!
//! Pflicht-Tests:
//! - `swap_error_leaves_no_plaintext`: Fault-Injection an Swap-Stellen verifiziert Zeroize und Poisoning.
//! - `rewrite_crash_at_every_step_never_exposes_plaintext`: Crash-Matrix über alle Einzelschritte.
//! - `rewrite_io_error_poisons_handle_permanently`: Erzwingt Invariante `INV-VAULT-FAILCLOSED-1`.
//! - `manifest_matches_journal_after_recovery`: Wiederanlauf-Konsistenz zwischen Journal und Manifest.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use contextra_erasure::vault::sealer::{SealerError, VaultSealer};

/// WP-5a: Test `swap_error_leaves_no_plaintext`
///
/// Injiziert Fehler an allen möglichen Stellen des Speicher-Swaps (vorher, mittendrin, danach)
/// und prüft, dass:
/// 1. Kein Klartext in beiden Puffern zurückbleibt (vollständig genullt/zeroized).
/// 2. Das Sealer-Handle dauerhaft vergiftet ist.
/// 3. Nachfolgende Operationen sofort mit `SealerError::Poisoned` abgelehnt werden.
#[test]
fn swap_error_leaves_no_plaintext() {
    let sealer = VaultSealer::new_in_memory();
    assert!(!sealer.is_poisoned());

    let secret_plain_a = b"super_secret_payload_buffer_a!!";
    let secret_plain_b = b"another_classified_secret_buf_b!";

    // Matrix aller Fehlerstellen: 0 = vor dem Swap, 1 = mitten im Swap, 2 = nach dem Swap, 3 = Längen-Mismatch
    for fault_step in [Some(0), Some(1), Some(2)] {
        let local_sealer = VaultSealer::new_in_memory();
        let mut buf_a = secret_plain_a.to_vec();
        let mut buf_b = secret_plain_b.to_vec();

        let res = local_sealer.safe_swap_buffers(&mut buf_a, &mut buf_b, fault_step);
        assert!(
            res.is_err(),
            "Swap should fail at fault step {fault_step:?}"
        );
        assert!(
            local_sealer.is_poisoned(),
            "Handle must be poisoned after swap failure"
        );

        // Verifikation: Kein Klartext im Speicher verblieben (alle Bytes müssen 0 sein)
        assert!(
            buf_a.iter().all(|&b| b == 0),
            "buf_a must be zeroized on fault {fault_step:?}"
        );
        assert!(
            buf_b.iter().all(|&b| b == 0),
            "buf_b must be zeroized on fault {fault_step:?}"
        );

        // Folgeaufrufe müssen abgewiesen werden
        let retry_res = local_sealer.safe_swap_buffers(&mut buf_a, &mut buf_b, None);
        assert_eq!(retry_res, Err(SealerError::Poisoned));
    }

    // Zusätzlicher Fall: Längen-Mismatch
    let mismatch_sealer = VaultSealer::new_in_memory();
    let mut short_buf = b"short".to_vec();
    let mut long_buf = b"longer_secret_text".to_vec();
    let mismatch_res = mismatch_sealer.safe_swap_buffers(&mut short_buf, &mut long_buf, None);
    assert!(mismatch_res.is_err());
    assert!(mismatch_sealer.is_poisoned());
    assert!(short_buf.iter().all(|&b| b == 0));
    assert!(long_buf.iter().all(|&b| b == 0));
}

#[cfg(feature = "async")]
mod async_rewrite_tests {

    use contextra_erasure::backend::StorageBackend;
    use contextra_erasure::error::ErasureError;
    use contextra_erasure::types_local::{LayerId, PurgeCx, PurgeOutcome, SystemClock};
    use contextra_erasure::vault::sealer::rewrite::{
        BlobRewriter, RewriteError, RewriteJournal, RewriteManifest, RewriteState,
    };
    use futures_util::future::BoxFuture;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::Arc;
    use zeroize::Zeroize;

    /// Test-Mock für StorageBackend zur präzisen Fehler- und Zustandsinjektion.
    struct MockRewriteBackend {
        fail_on_purge: AtomicBool,
        last_purged: parking_lot::RwLock<Option<u64>>,
        data_buffer: parking_lot::RwLock<Vec<u8>>,
    }

    impl MockRewriteBackend {
        fn new(initial_data: Vec<u8>) -> Self {
            Self {
                fail_on_purge: AtomicBool::new(false),
                last_purged: parking_lot::RwLock::new(None),
                data_buffer: parking_lot::RwLock::new(initial_data),
            }
        }

        fn set_fail(&self, fail: bool) {
            self.fail_on_purge.store(fail, Ordering::Release);
        }

        fn read_data(&self) -> Vec<u8> {
            self.data_buffer.read().clone()
        }
    }

    impl StorageBackend for MockRewriteBackend {
        fn purge<'a>(
            &'a self,
            cx: &'a PurgeCx,
        ) -> BoxFuture<'a, Result<PurgeOutcome, ErasureError>> {
            Box::pin(async move {
                if self.fail_on_purge.load(Ordering::Acquire) {
                    return Err(ErasureError::Storage(
                        "injected storage purge failure".to_string(),
                    ));
                }

                // Physisches Überschreiben (Zeroizing) des Puffers
                let mut buf = self.data_buffer.write();
                buf.zeroize();
                *self.last_purged.write() = Some(cx.deadline_ms);

                Ok(PurgeOutcome::new(1, 1024))
            })
        }

        fn layer_id(&self) -> LayerId {
            LayerId::new([10u8; 16])
        }
    }

    /// WP-5: Test `rewrite_crash_at_every_step_never_exposes_plaintext`
    ///
    /// Vollständige Crash-Matrix über alle 5 Einzelschritte der Rewrite-Pipeline.
    /// Nach jedem Crash wird sichergestellt, dass kein Klartext exponiert wird und
    /// der Wiederanlauf konsistent vollendet oder sicher zurückrollt.
    #[tokio::test]
    async fn rewrite_crash_at_every_step_never_exposes_plaintext() {
        let clock = Arc::new(SystemClock::new());

        for crash_step in 1..=5 {
            let initial_secret = b"sensitive_vault_blob_data_must_be_erased".to_vec();
            let backend = Arc::new(MockRewriteBackend::new(initial_secret.clone()));
            let journal = RewriteJournal::new();
            let manifest = RewriteManifest::new();

            let rewriter = BlobRewriter::new(
                journal.clone(),
                manifest.clone(),
                backend.clone(),
                clock.clone(),
            );

            let rewrite_id = 100 + crash_step as u64;
            let target_id = format!("doc_target_{crash_step}");
            let digest = [0xabu8; 32];

            // Ausführung mit gezieltem Crash nach `crash_step`
            let exec_res = rewriter
                .execute_rewrite(rewrite_id, &target_id, digest, Some(crash_step))
                .await;
            assert!(exec_res.is_err(), "Step {crash_step} must simulate failure");

            // Nach Crash/Neustart wird ein frischer Rewriter über dieselben persistenten Zustände (Journal/Manifest) geöffnet
            let recovery_rewriter = BlobRewriter::new(
                journal.clone(),
                manifest.clone(),
                backend.clone(),
                clock.clone(),
            );
            let recovered_count = recovery_rewriter
                .recover_and_reconcile()
                .await
                .expect("recovery must succeed");
            assert_eq!(recovered_count, 1, "Must reconcile the uncommitted intent");

            // Prüfen des Zustands nach Recovery:
            let intent = journal.get_intent(rewrite_id).expect("intent must exist");
            let manifest_entry = manifest.get_entry(&target_id);

            if crash_step < 4 {
                // Vor Phase 4 (Manifest-Update): Wurde sicher zurückgerollt, Backend-Daten wurden unschädlich gemacht / genullt
                assert_eq!(intent.state, RewriteState::RolledBack);
                assert!(
                    manifest_entry.is_none(),
                    "Manifest must not contain uncommitted target before step 4"
                );
                // Bei RolledBack sorgt recover_and_reconcile für Nullung im Backend
                let data = backend.read_data();
                assert!(
                    data.iter().all(|&b| b == 0),
                    "Backend data must be zeroized after rollback"
                );
            } else {
                // Ab Phase 4 (ManifestUpdated / Committed): Manifest war aktuell -> Rewrite wird committed
                assert_eq!(intent.state, RewriteState::Committed);
                assert!(
                    manifest_entry.is_some(),
                    "Manifest must contain target after step 4"
                );
                assert!(manifest_entry.unwrap().is_erased);
                let data = backend.read_data();
                assert!(
                    data.iter().all(|&b| b == 0),
                    "Backend data must remain zeroized after commit"
                );
            }
        }
    }

    /// WP-5: Test `rewrite_io_error_poisons_handle_permanently`
    ///
    /// Erzwingt Invariante `INV-VAULT-FAILCLOSED-1`:
    /// Jeder E/A- oder Sync-Fehler beim Überschreiben vergiftet das Vault/Rewriter-Handle dauerhaft.
    #[tokio::test]
    async fn rewrite_io_error_poisons_handle_permanently() {
        let clock = Arc::new(SystemClock::new());
        let backend = Arc::new(MockRewriteBackend::new(b"initial_vault_data".to_vec()));
        let journal = RewriteJournal::new();
        let manifest = RewriteManifest::new();

        let rewriter = BlobRewriter::new(
            journal.clone(),
            manifest.clone(),
            backend.clone(),
            clock.clone(),
        );
        assert!(!rewriter.is_poisoned());

        // 1. Test: E/A-Fehler beim Backend-Purge/Overwrite
        backend.set_fail(true);
        let res_io = rewriter
            .execute_rewrite(1, "scope_target_io", [0u8; 32], None)
            .await;
        assert!(res_io.is_err());
        assert!(matches!(
            res_io.unwrap_err(),
            RewriteError::BlobOverwrite(_)
        ));
        assert!(
            rewriter.is_poisoned(),
            "INV-VAULT-FAILCLOSED-1: Handle must be permanently poisoned on overwrite IO error"
        );

        // Weitere Aufrufe müssen sofort mit Poisoned abgewiesen werden
        let res_retry = rewriter
            .execute_rewrite(2, "scope_target_io_2", [0u8; 32], None)
            .await;
        assert_eq!(res_retry.unwrap_err(), RewriteError::Poisoned);

        // 2. Test: Sync-Fehler vergiftet ebenfalls dauerhaft
        let sync_rewriter = BlobRewriter::new(
            RewriteJournal::new(),
            RewriteManifest::new(),
            backend.clone(),
            clock.clone(),
        );
        backend.set_fail(false);
        let res_sync = sync_rewriter
            .execute_rewrite(3, "scope_target_sync", [0u8; 32], Some(3))
            .await;
        assert!(res_sync.is_err());
        assert!(matches!(
            res_sync.unwrap_err(),
            RewriteError::SyncFailure(_)
        ));
        assert!(
            sync_rewriter.is_poisoned(),
            "INV-VAULT-FAILCLOSED-1: Handle must be permanently poisoned on sync failure"
        );
    }

    /// WP-5: Test `manifest_matches_journal_after_recovery`
    ///
    /// Verifiziert die Wiederanlauf-Konsistenz:
    /// Nach Crash und Recovery stimmen Journal und Manifest deterministisch überein.
    #[tokio::test]
    async fn manifest_matches_journal_after_recovery() {
        let clock = Arc::new(SystemClock::new());
        let backend = Arc::new(MockRewriteBackend::new(b"data_blob_123".to_vec()));
        let journal = RewriteJournal::new();
        let manifest = RewriteManifest::new();

        let rewriter = BlobRewriter::new(
            journal.clone(),
            manifest.clone(),
            backend.clone(),
            clock.clone(),
        );

        // Führe erfolgreichen Rewrite durch
        let res_ok = rewriter
            .execute_rewrite(42, "target_alpha", [1u8; 32], None)
            .await;
        assert!(res_ok.is_ok());

        // Führe Rewrite durch, der bei Phase 4 (nach Manifest-Update) abstürzt
        let res_crash_phase4 = rewriter
            .execute_rewrite(43, "target_beta", [2u8; 32], Some(4))
            .await;
        assert!(res_crash_phase4.is_err());

        // Führe Rewrite durch, der bei Phase 2 abstürzt
        let res_crash_phase2 = rewriter
            .execute_rewrite(44, "target_gamma", [3u8; 32], Some(2))
            .await;
        assert!(res_crash_phase2.is_err());

        // Vor Recovery: Uncommitted Intents vorhanden
        assert_eq!(journal.uncommitted_intents().len(), 2);

        // Recovery ausführen
        let recovered = rewriter
            .recover_and_reconcile()
            .await
            .expect("reconcile must succeed");
        assert_eq!(recovered, 2);

        // Nach Recovery: Keine uncommitted Intents mehr
        assert_eq!(journal.uncommitted_intents().len(), 0);

        // Manifest und Journal abgleichen:
        // target_alpha: Committed und im Manifest
        let alpha_intent = journal.get_intent(42).unwrap();
        let alpha_manifest = manifest.get_entry("target_alpha").unwrap();
        assert_eq!(alpha_intent.state, RewriteState::Committed);
        assert!(alpha_manifest.is_erased);
        assert_eq!(alpha_manifest.rewrite_id, 42);

        // target_beta: Durch Recovery vollendet (Committed) und im Manifest
        let beta_intent = journal.get_intent(43).unwrap();
        let beta_manifest = manifest.get_entry("target_beta").unwrap();
        assert_eq!(beta_intent.state, RewriteState::Committed);
        assert!(beta_manifest.is_erased);
        assert_eq!(beta_manifest.rewrite_id, 43);

        // target_gamma: Zurückgerollt (RolledBack) und NICHT im Manifest aktiv
        let gamma_intent = journal.get_intent(44).unwrap();
        let gamma_manifest = manifest.get_entry("target_gamma");
        assert_eq!(gamma_intent.state, RewriteState::RolledBack);
        assert!(gamma_manifest.is_none());
    }
}
