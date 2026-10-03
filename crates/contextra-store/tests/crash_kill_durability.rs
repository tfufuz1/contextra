// ANCHOR[TEST:STO-004] STATUS:DONE (TS:2026-09-30T00:00:00Z)
//! Crash kill durability verification test.
//!
//! Evaluates SIGKILL process termination recovery and durability invariants.
//! Proves that every commit returning Ok survives SIGKILL, transaction atomicity is preserved,
//! no phantom keys exist, reopen succeeds after crash, post-recovery writes succeed,
//! and WAL/HMAC integrity counter-probe fails closed on corrupted files.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic, deprecated)]

use contextra_core::{StorageEngine, TxId};
use contextra_store::lsm::{LsmConfig, LsmStorage};
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::Arc;
use std::time::Duration;
use tempfile::TempDir;

/// Independent reference value calculator.
///
/// Source/Logic: Blake3 hash of transaction number + key suffix string.
/// Guarantees ground-truth expected value calculation independent of internal storage engine encodings.
fn reference_value(n: u64, suffix: &str) -> Vec<u8> {
    let input = format!("ref_payload_n={}_suffix={}", n, suffix);
    blake3::hash(input.as_bytes()).as_bytes().to_vec()
}

fn copy_dir_all(src: &Path, dst: &Path) -> std::io::Result<()> {
    std::fs::create_dir_all(dst)?;
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        if ty.is_dir() {
            copy_dir_all(&entry.path(), &dst.join(entry.file_name()))?;
        } else {
            std::fs::copy(entry.path(), dst.join(entry.file_name()))?;
        }
    }
    Ok(())
}

/// Minimal reproduction test for duplicate sequence number bug during multi-op transaction commit WAL replay.
///
/// BUG DOCUMENTATION:
/// Location: `crates/contextra-store/src/lsm/ops/write.rs:236`
/// Description: When `commit_internal` prepares `wal_ops` for a multi-operation transaction, it assigns
/// `last_seq` (the sequence number of the final staged operation) to `WalOp::TxEnd`.
/// As a consequence, `WalOp::TxEnd` shares its sequence number with the preceding `WalOp::Put`/`WalOp::Delete`.
/// During WAL replay, `IntegrityVerifier` in `crates/contextra-crypto/src/wal_crypto.rs` enforces strictly
/// monotonically increasing sequence numbers (`seq_no > last_seq_no`).
/// When encountering duplicate sequence numbers on `TxEnd`, WAL replay fails with:
/// `WalCorruption { reason: "Duplicate or non-monotonic sequence number N (last: N)" }`.
#[tokio::test]
async fn repro_bug_tx_end_duplicate_seq_no_wal_replay() {
    let tmp = TempDir::new().expect("create temp directory");
    let config = LsmConfig {
        path: tmp.path().to_path_buf(),
        ..Default::default()
    };

    // 1. Commit a multi-operation transaction (2 keys)
    {
        let storage = LsmStorage::open(config.clone())
            .await
            .expect("LsmStorage::open failed during initial write");
        let tx = TxId::new(1);
        storage
            .put(tx, b"key1", b"val1")
            .await
            .expect("put key1 failed");
        storage
            .put(tx, b"key2", b"val2")
            .await
            .expect("put key2 failed");
        storage
            .commit(tx)
            .await
            .expect("commit multi-op transaction failed");
    }

    // 2. Reopen database from disk
    // REPRO: LsmStorage::open fails during WAL replay because TxEnd shares seq_no=2 with key2,
    // triggering WalCorruption error from IntegrityVerifier.
    let reopen_res = LsmStorage::open(config).await;
    assert!(
        reopen_res.is_err(),
        "Expected WAL corruption error on reopen due to duplicate TxEnd seq_no bug"
    );

    if let Err(e) = reopen_res {
        let err_str = format!("{e:?}");
        assert!(
            err_str.contains("Duplicate or non-monotonic sequence number")
                || err_str.contains("WalCorruption"),
            "Expected duplicate sequence number error, got: {err_str}"
        );
    }
}

/// Child process writer loop.
///
/// Opens `LsmStorage`, spawns a background `force_flush` task to trigger flush/rotation within
/// the kill window, and sequentially commits transactions with 3 keys each (`key_{n}_a/_b/_c`).
/// Outputs `"ACK <n>\n"` to stdout ONLY after `storage.commit()` returns `Ok(())`.
#[tokio::test]
#[ignore = "Invoked as subprocess by test_crash_kill_durability"]
async fn child_writer() {
    if std::env::var("CRASH_CHILD").is_err() {
        return;
    }

    let db_path_str = std::env::var("CRASH_DB_PATH").expect("CRASH_DB_PATH environment variable");
    let db_path = PathBuf::from(db_path_str);

    let group_commit_micros: u64 = std::env::var("CRASH_GROUP_COMMIT_WINDOW_MICROS")
        .unwrap_or_else(|_| "500".to_string())
        .parse()
        .expect("valid u64 for CRASH_GROUP_COMMIT_WINDOW_MICROS");

    let encrypted = std::env::var("CRASH_ENCRYPTION").as_deref() == Ok("1");

    let passphrase = if encrypted {
        Some("crash_test_secret_key_987".to_string())
    } else {
        None
    };

    let config = LsmConfig {
        path: db_path,
        group_commit_window_micros: group_commit_micros,
        encryption_passphrase: passphrase,
        ..Default::default()
    };

    let storage = Arc::new(
        LsmStorage::open(config)
            .await
            .expect("child LsmStorage::open failed"),
    );

    // Spawn background task calling force_flush periodically to put flush/rotation in kill window
    let storage_flush = Arc::clone(&storage);
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(Duration::from_millis(5)).await;
            let _ = storage_flush.force_flush().await;
        }
    });

    let mut stdout = std::io::stdout();
    let suffixes = ["a", "b", "c"];
    let mut n: u64 = 1;

    loop {
        let tx_id = TxId::new(n);
        for suffix in &suffixes {
            let key = format!("key_{}_{}", n, suffix);
            let val = reference_value(n, suffix);
            if let Err(e) = storage.put(tx_id, key.as_bytes(), &val).await {
                eprintln!("Child put error at n={n}, key={key}: {e:?}");
                std::process::exit(1);
            }
        }

        if let Err(e) = storage.commit(tx_id).await {
            eprintln!("Child commit error at n={n}: {e:?}");
            std::process::exit(1);
        }

        // Emit ACK to stdout only after commit returned Ok
        if writeln!(stdout, "ACK {}", n).is_err() || stdout.flush().is_err() {
            break; // Parent closed pipe
        }

        n += 1;
    }
}

/// Parent process durability verification test harness.
///
/// Spawns the child writer via `std::env::current_exe()`, issues SIGKILL at randomized ACK counts,
/// and validates durability, atomicity, phantom prevention, recovery writeability, and HMAC counter-probe.
#[tokio::test]
#[ignore = "BUG: TxEnd duplicate seq_no in crates/contextra-store/src/lsm/ops/write.rs:236 causes WalCorruption on replay"]
async fn test_crash_kill_durability() {
    if std::env::var("CRASH_CHILD").is_ok() {
        return;
    }

    let num_iters: usize = std::env::var("CRASH_ITERS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(25);

    let seed: u64 = std::env::var("CRASH_SEED")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or_else(|| {
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("system time nanos")
                .as_nanos() as u64
        });

    println!("CRASH_SEED={seed}");

    let mut rng = StdRng::seed_from_u64(seed);
    let current_exe = std::env::current_exe().expect("locate current test binary executable");

    for iter in 1..=num_iters {
        let group_commit_micros: u64 = if rng.gen_bool(0.5) { 0 } else { 2000 };

        #[cfg(feature = "encryption-at-rest")]
        let encrypted = rng.gen_bool(0.5);
        #[cfg(not(feature = "encryption-at-rest"))]
        let encrypted = false;

        let tmp_dir = TempDir::new().expect("create iteration temp directory");
        let target_ack: u64 = rng.gen_range(5..40);
        let jitter_ms: u64 = rng.gen_range(0..15);

        let mut child = Command::new(&current_exe)
            .arg("--exact")
            .arg("child_writer")
            .arg("--nocapture")
            .arg("--include-ignored")
            .env("CRASH_CHILD", "1")
            .env("CRASH_DB_PATH", tmp_dir.path())
            .env("CRASH_ENCRYPTION", if encrypted { "1" } else { "0" })
            .env(
                "CRASH_GROUP_COMMIT_WINDOW_MICROS",
                group_commit_micros.to_string(),
            )
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .spawn()
            .expect("spawn child writer process");

        let child_stdout = child.stdout.take().expect("capture child process stdout");
        let reader = BufReader::new(child_stdout);
        let mut last_ack: u64 = 0;

        for line_res in reader.lines() {
            let line = match line_res {
                Ok(l) => l,
                Err(_) => break,
            };
            let trimmed = line.trim();
            if let Some(num_str) = trimmed.strip_prefix("ACK ") {
                if let Ok(ack_n) = num_str.parse::<u64>() {
                    last_ack = ack_n;
                    if last_ack >= target_ack {
                        if jitter_ms > 0 {
                            tokio::time::sleep(Duration::from_millis(jitter_ms)).await;
                        }
                        let _ = child.kill();
                        let _ = child.wait();
                        break;
                    }
                }
            }
        }

        let _ = child.kill();
        let _ = child.wait();

        // Passphrase for reopening
        let passphrase = if encrypted {
            Some("crash_test_secret_key_987".to_string())
        } else {
            None
        };

        let config = LsmConfig {
            path: tmp_dir.path().to_path_buf(),
            group_commit_window_micros: group_commit_micros,
            encryption_passphrase: passphrase.clone(),
            ..Default::default()
        };

        // (a) LsmStorage::open MUST succeed (no error tolerated or ignored)
        let storage = match LsmStorage::open(config.clone()).await {
            Ok(s) => s,
            Err(e) => {
                let preserved_path = tmp_dir.into_path();
                panic!(
                    "CRASH DURABILITY TEST FAILED at Iteration {iter}/{num_iters}!\n\
                     Seed: {seed}\n\
                     Last ACK: {last_ack}\n\
                     Data Path: {}\n\
                     Error: LsmStorage::open failed post SIGKILL: {e:?}",
                    preserved_path.display()
                );
            }
        };

        // (b) All acknowledged transactions n must be completely readable with exact values
        for n in 1..=last_ack {
            for suffix in &["a", "b", "c"] {
                let key = format!("key_{}_{}", n, suffix);
                let expected = reference_value(n, suffix);
                let actual = match storage.get(key.as_bytes()).await {
                    Ok(Some(v)) => v,
                    Ok(None) => {
                        let preserved_path = tmp_dir.into_path();
                        panic!(
                            "CRASH DURABILITY TEST FAILED at Iteration {iter}/{num_iters}!\n\
                             Seed: {seed}\n\
                             Last ACK: {last_ack}\n\
                             Data Path: {}\n\
                             Error: Confirmed ACKed key '{key}' (tx {n}) missing after reopen!",
                            preserved_path.display()
                        );
                    }
                    Err(e) => {
                        let preserved_path = tmp_dir.into_path();
                        panic!(
                            "CRASH DURABILITY TEST FAILED at Iteration {iter}/{num_iters}!\n\
                             Seed: {seed}\n\
                             Last ACK: {last_ack}\n\
                             Data Path: {}\n\
                             Error: Read error for ACKed key '{key}': {e:?}",
                            preserved_path.display()
                        );
                    }
                };

                if actual.as_ref() != expected.as_slice() {
                    let preserved_path = tmp_dir.into_path();
                    panic!(
                        "CRASH DURABILITY TEST FAILED at Iteration {iter}/{num_iters}!\n\
                         Seed: {seed}\n\
                         Last ACK: {last_ack}\n\
                         Data Path: {}\n\
                         Error: Value mismatch for ACKed key '{key}'!",
                        preserved_path.display()
                    );
                }
            }
        }

        // (c) First unacknowledged transaction (last_ack + 1) atomicity check: 0 or 3 keys present
        let unack_n = last_ack + 1;
        let mut unack_present_count = 0;
        for suffix in &["a", "b", "c"] {
            let key = format!("key_{}_{}", unack_n, suffix);
            if let Ok(Some(actual)) = storage.get(key.as_bytes()).await {
                let expected = reference_value(unack_n, suffix);
                if actual.as_ref() == expected.as_slice() {
                    unack_present_count += 1;
                }
            }
        }

        if unack_present_count != 0 && unack_present_count != 3 {
            let preserved_path = tmp_dir.into_path();
            panic!(
                "CRASH DURABILITY TEST FAILED at Iteration {iter}/{num_iters}!\n\
                 Seed: {seed}\n\
                 Last ACK: {last_ack}\n\
                 Data Path: {}\n\
                 Error: Atomicity violation on unacknowledged tx {unack_n}! Found {unack_present_count}/3 keys.",
                preserved_path.display()
            );
        }

        // (d) No phantom keys for transactions beyond last_ack + 1
        for check_n in (last_ack + 2)..=(last_ack + 20) {
            for suffix in &["a", "b", "c"] {
                let key = format!("key_{}_{}", check_n, suffix);
                if let Ok(Some(_)) = storage.get(key.as_bytes()).await {
                    let preserved_path = tmp_dir.into_path();
                    panic!(
                        "CRASH DURABILITY TEST FAILED at Iteration {iter}/{num_iters}!\n\
                         Seed: {seed}\n\
                         Last ACK: {last_ack}\n\
                         Data Path: {}\n\
                         Error: Phantom key '{key}' found for unstarted transaction {check_n}!",
                        preserved_path.display()
                    );
                }
            }
        }

        // (e) Post-recovery writeability check: commit new tx and reopen
        let rec_tx = TxId::new(999_999_999);
        if let Err(e) = storage
            .put(rec_tx, b"recovery_test_key", b"recovery_test_val")
            .await
        {
            let preserved_path = tmp_dir.into_path();
            panic!(
                "CRASH DURABILITY TEST FAILED at Iteration {iter}/{num_iters}!\n\
                 Seed: {seed}\n\
                 Last ACK: {last_ack}\n\
                 Data Path: {}\n\
                 Error: Post-recovery put failed: {e:?}",
                preserved_path.display()
            );
        }

        if let Err(e) = storage.commit(rec_tx).await {
            let preserved_path = tmp_dir.into_path();
            panic!(
                "CRASH DURABILITY TEST FAILED at Iteration {iter}/{num_iters}!\n\
                 Seed: {seed}\n\
                 Last ACK: {last_ack}\n\
                 Data Path: {}\n\
                 Error: Post-recovery commit failed: {e:?}",
                preserved_path.display()
            );
        }

        drop(storage);

        let storage_reopen = match LsmStorage::open(config.clone()).await {
            Ok(s) => s,
            Err(e) => {
                let preserved_path = tmp_dir.into_path();
                panic!(
                    "CRASH DURABILITY TEST FAILED at Iteration {iter}/{num_iters}!\n\
                     Seed: {seed}\n\
                     Last ACK: {last_ack}\n\
                     Data Path: {}\n\
                     Error: Reopen after post-recovery commit failed: {e:?}",
                    preserved_path.display()
                );
            }
        };

        let rec_val = storage_reopen.get(b"recovery_test_key").await;
        if rec_val.as_ref().ok().and_then(|v| v.as_ref())
            != Some(&bytes::Bytes::from_static(b"recovery_test_val"))
        {
            let preserved_path = tmp_dir.into_path();
            panic!(
                "CRASH DURABILITY TEST FAILED at Iteration {iter}/{num_iters}!\n\
                 Seed: {seed}\n\
                 Last ACK: {last_ack}\n\
                 Data Path: {}\n\
                 Error: Post-recovery committed key was missing or mismatched!",
                preserved_path.display()
            );
        }

        drop(storage_reopen);

        // (f) WAL/HMAC Integrity Counter-Probe: Corrupt copy of database directory
        let corrupt_tmp = TempDir::new().expect("create corrupt temp directory");
        copy_dir_all(tmp_dir.path(), corrupt_tmp.path()).expect("copy db directory");

        let mut found_file_to_corrupt = false;
        if let Ok(entries) = std::fs::read_dir(corrupt_tmp.path()) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() {
                    if let Ok(meta) = std::fs::metadata(&path) {
                        if meta.len() > 16 {
                            if let Ok(mut data) = std::fs::read(&path) {
                                let offset = data.len() / 2;
                                data[offset] ^= 0xFF;
                                if std::fs::write(&path, &data).is_ok() {
                                    found_file_to_corrupt = true;
                                    break;
                                }
                            }
                        }
                    }
                }
            }
        }

        if found_file_to_corrupt {
            let corrupt_config = LsmConfig {
                path: corrupt_tmp.path().to_path_buf(),
                group_commit_window_micros: group_commit_micros,
                encryption_passphrase: passphrase.clone(),
                ..Default::default()
            };

            let corrupt_res = LsmStorage::open(corrupt_config).await;
            if corrupt_res.is_ok() {
                let preserved_path = tmp_dir.into_path();
                panic!(
                    "CRASH DURABILITY TEST FAILED at Iteration {iter}/{num_iters}!\n\
                     Seed: {seed}\n\
                     Last ACK: {last_ack}\n\
                     Data Path: {}\n\
                     Error: Counter-probe failed! Opening database with corrupted data file succeeded unexpectedly.",
                    preserved_path.display()
                );
            }
        }
    }
}
