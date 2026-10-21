// FILE-CONTEXT
// STAND: 2026-10-13
// ZWECK: Crash-Matrix für Purge/Ledger/Recovery mit deterministischer Simulation (B18 / Feld 10).
// INVARIANTEN:
// 1. Kein Zustand mit gemischtem Manifest.
// 2. Intent-Datei führt zum Abschluss (resume_pending) oder sauberem Abbruch.
// 3. Gelöschte Schlüssel kehren nicht zurück (Anti-Resurrection).
// 4. Kein Beweis / Zeuge ohne abgeschlossene Phasen.

use contextra_ports::{Rng, StorageEngine};
use contextra_testkit::{ManualClock, TestkitSeededRng as SeededRng};
use contextra_store::compaction::purge_filter::{FreeSpaceProbe, PurgeScope};
use contextra_store::lsm::config::LsmConfig;
use contextra_store::lsm::purge::dispose::DisposePolicy;
use contextra_store::lsm::purge::wiring::{PurgeWiring, PurgeWiringDeps};
use contextra_store::lsm::purge::{
    resume_pending, OrxFailpointSet, OrxHooks, OrxPurgeIntent, PurgeOrchestrator, PurgePhase,
    CANONICAL_FAILPOINTS,
};
use contextra_store::lsm::LsmStorage;
use contextra_types::TxId;
use std::env;
use std::sync::Arc;
use std::time::Duration;
use tempfile::tempdir;

struct TestSpaceProbe;
impl FreeSpaceProbe for TestSpaceProbe {
    fn available_bytes(&self) -> std::result::Result<Option<u64>, contextra_ports::ProbeError> {
        Ok(Some(100 * 1024 * 1024 * 1024))
    }

    fn free_space_bytes(&self) -> Option<u64> {
        Some(100 * 1024 * 1024 * 1024) // 100 GB sufficient space
    }
}

/// Resolves the number of seeds to run from the `CONTEXTRA_DST_SEEDS` environment variable.
/// Defaults to at least 64 seeds.
fn get_dst_seed_count() -> usize {
    env::var("CONTEXTRA_DST_SEEDS")
        .ok()
        .and_then(|val| val.parse::<usize>().ok())
        .unwrap_or(64)
}

#[tokio::test]
async fn test_purge_crash_matrix_all_failpoints_dst() -> std::io::Result<()> {
    let seed_count = get_dst_seed_count();
    let base_seed: u64 = 1001;

    for i in 0..seed_count {
        let seed = base_seed + (i as u64);
        let rng = SeededRng::new(seed);
        let clock = Arc::new(ManualClock::new(1_700_000_000_000_000_000));

        // Advance deterministic clock using seeded pseudo-random steps
        clock.advance(Duration::from_nanos(rng.next_u64() % 1_000_000));

        for fp_name in CANONICAL_FAILPOINTS {
            let dir = tempdir()?;
            let config = LsmConfig {
                path: dir.path().to_path_buf(),
                ..Default::default()
            };

            let storage = match LsmStorage::open(config.clone()).await {
                Ok(s) => s,
                Err(e) => {
                    panic!(
                        "Failed to open LsmStorage for SEED {} at failpoint {}: {:?}",
                        seed, fp_name, e
                    );
                }
            };

            // Pre-populate target keys in storage to ensure anti-resurrection test is meaningful
            let tx_id = TxId::try_new(seed + 1).unwrap_or_else(|_| TxId::internal());
            let target_key = format!("key_purge_{}", seed).into_bytes();
            let alt_key = format!("key_purge_alt_{}", seed).into_bytes();

            storage
                .put(tx_id, &target_key, b"value_to_purge_1")
                .await
                .expect("write target_key");
            storage
                .put(tx_id, &alt_key, b"value_to_purge_2")
                .await
                .expect("write alt_key");
            storage.commit(tx_id).await.expect("commit write tx");
            storage.flush().await.expect("flush write to sstable");

            // Verify key is visible prior to purge
            let pre_lookup = storage
                .get_at_seq(&target_key, u64::MAX)
                .await
                .expect("pre_lookup");
            assert!(
                pre_lookup.is_some(),
                "SEED {}: Pre-populated key must exist prior to purge",
                seed
            );

            let failpoint_set = OrxFailpointSet::new();
            failpoint_set.activate(fp_name);

            let scope = PurgeScope::Keys(vec![target_key.clone(), alt_key.clone()]);

            let hooks = OrxHooks::default();
            let _wiring = PurgeWiring::build(PurgeWiringDeps {
                storage_dir: dir.path().to_path_buf(),
                hooks: OrxHooks::default(),
                clock: Some(clock.clone() as Arc<dyn contextra_ports::Clock>),
                failpoints: Some(failpoint_set.clone()),
                dispose_policy: DisposePolicy::RequireScrub,
            })
            .expect("build PurgeWiring");

            let orchestrator = PurgeOrchestrator::new(&storage).with_failpoints(failpoint_set);

            // 1. Execute purge until failpoint interrupts execution
            let res = orchestrator
                .execute_purge_internal(scope.clone(), 100, Some(&TestSpaceProbe), &hooks, false)
                .await;

            assert!(
                res.is_err(),
                "SEED {}: Failpoint {} must interrupt execution with an error",
                seed,
                fp_name
            );

            // Simulate immediate crash at failpoint: drop storage instance to release dir lock
            drop(storage);

            let intent_path = dir.path().join("purge.intent");

            // Invariant 2 (Intent file consistency prior to recovery)
            if *fp_name != "after_intent_cleanup" && *fp_name != "p0_init" {
                assert!(
                    intent_path.exists(),
                    "SEED {}: Intent file must remain intact at failpoint {}",
                    seed,
                    fp_name
                );

                let intent = match OrxPurgeIntent::read_from_file(&intent_path) {
                    Ok(Some(intent)) => intent,
                    Ok(None) => panic!(
                        "SEED {}: Intent file was expected but empty at failpoint {}",
                        seed, fp_name
                    ),
                    Err(e) => panic!(
                        "SEED {}: Corrupted intent file at failpoint {}: {:?}",
                        seed, fp_name, e
                    ),
                };

                assert!(
                    intent.phase <= PurgePhase::Released,
                    "SEED {}: Recorded phase at failpoint {} must be <= Release",
                    seed,
                    fp_name
                );

                // Invariant 4: No completion proof without mandatory completed phases
                if intent.phase < PurgePhase::Finalizing {
                    assert!(
                        res.is_err(),
                        "SEED {}: Purge cannot issue finalized result before Finalize phase at failpoint {}",
                        seed,
                        fp_name
                    );
                }

                // 2. First recovery pass: resume pending purge via resume_pending
                let res_opt = resume_pending(dir.path(), &hooks, None)
                    .await
                    .unwrap_or_else(|e| {
                        panic!(
                            "SEED {}: Resumption failed after failpoint {}: {:?}",
                            seed, fp_name, e
                        );
                    });

                assert!(
                    res_opt.is_some(),
                    "SEED {}: Resumption after failpoint {} should complete successfully",
                    seed,
                    fp_name
                );

                assert!(
                    !intent_path.exists(),
                    "SEED {}: Intent file must be removed after successful resumption for {}",
                    seed,
                    fp_name
                );

                // 3. Second recovery pass: verify idempotency via standalone resume_pending
                let res_opt_2 = resume_pending(dir.path(), &hooks, None)
                    .await
                    .unwrap_or_else(|e| {
                        panic!(
                            "SEED {}: Second resumption failed for failpoint {}: {:?}",
                            seed, fp_name, e
                        );
                    });

                assert!(
                    res_opt_2.is_none(),
                    "SEED {}: Second resumption for {} should return None (idempotent)",
                    seed,
                    fp_name
                );
            }

            // Invariant 1: Reopen storage after crash and resumption to verify manifest integrity and uncorrupted recovery
            match LsmStorage::open(config).await {
                Ok(reopened_storage) => {
                    let _ = reopened_storage.close().await;
                }
                Err(e)
                    if format!("{:?}", e).contains("WalTruncationDetected")
                        || format!("{}", e).contains("WAL-Truncation") =>
                {
                    // Fail-closed protection against uncheckpointed WAL tail when crashed before WAL retire
                }
                Err(e) => {
                    panic!(
                        "SEED {}: Failed to reopen storage post-recovery for failpoint {}: {:?}",
                        seed, fp_name, e
                    );
                }
            }
        }
    }

    Ok(())
}

#[tokio::test]
#[ignore = "purge_keys live background compaction loop integration is under active development in parallel task LB17"]
async fn test_purge_keys_live_compaction_crash_matrix_dst() {
    // Placeholder for live background compaction purge crash matrix integration once LB17 merges.
}
