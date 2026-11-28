//! F-04 regression: compaction output file names must not collide across process restarts.
//!
//! `CompactionEngine::generate_sst_path` draws its sequence number from a process-global
//! counter that restarts at 1 on every process start. Two sessions therefore produce the
//! same `sst-compact-…-0000.sst` name. The test runs each session in a fresh child process
//! (the only way to get a real restart, since the counter is a static) and verifies that
//! every key written in every session is still readable afterwards.

use contextra_ports::StorageEngine;
use contextra_store::lsm::{LsmConfig, LsmStorage};
use contextra_types::{Result, TxId};
use std::path::Path;
use tempfile::TempDir;

const TEST_NAME: &str = "compaction_output_names_survive_restart";
const SESSION_DIR_ENV: &str = "CONTEXTRA_F04_SESSION_DIR";
const SESSION_TAG_ENV: &str = "CONTEXTRA_F04_SESSION_TAG";
const SESSIONS: u64 = 3;
const FLUSHES_PER_SESSION: u64 = 4;
const KEYS_PER_FLUSH: u64 = 4;

fn key(tag: u64, flush: u64, k: u64) -> Vec<u8> {
    format!("s{tag}-f{flush}-k{k}").into_bytes()
}

fn value(tag: u64, flush: u64, k: u64) -> Vec<u8> {
    format!("value-s{tag}-f{flush}-k{k}").into_bytes()
}

fn config(dir: &Path) -> LsmConfig {
    LsmConfig {
        path: dir.to_path_buf(),
        ..Default::default()
    }
}

/// One process lifetime: write, flush several times, compact, close.
async fn run_session(dir: &Path, tag: u64) -> Result<()> {
    let storage = LsmStorage::new(config(dir)).await?;
    for flush in 0..FLUSHES_PER_SESSION {
        for k in 0..KEYS_PER_FLUSH {
            // Monotonic across sessions: session `tag` uses ids above all earlier sessions.
            let tx = TxId::new(tag * 1_000_000 + flush * 1_000 + k + 1);
            storage
                .put(tx, &key(tag, flush, k), &value(tag, flush, k))
                .await?;
            storage.commit(tx).await?;
        }
        storage.force_flush().await?;
    }
    storage.maybe_compact().await?;
    storage.close().await?;
    Ok(())
}

/// Re-opens the database in this process and checks every key from every session.
async fn verify_all_sessions(dir: &Path) -> Result<()> {
    let storage = LsmStorage::new(config(dir)).await?;
    let mut missing = Vec::new();
    for tag in 1..=SESSIONS {
        for flush in 0..FLUSHES_PER_SESSION {
            for k in 0..KEYS_PER_FLUSH {
                let got = storage.get(&key(tag, flush, k)).await?;
                if got.as_deref() != Some(value(tag, flush, k).as_slice()) {
                    missing.push(format!("s{tag}-f{flush}-k{k}"));
                }
            }
        }
    }
    storage.close().await?;
    assert!(
        missing.is_empty(),
        "data lost after restart, {} key(s) unreadable: {:?}",
        missing.len(),
        missing
    );
    Ok(())
}

fn run_child_session(dir: &Path, tag: u64) {
    let exe = std::env::current_exe().expect("current test binary");
    let status = std::process::Command::new(exe)
        .args(["--exact", TEST_NAME, "--test-threads=1", "--nocapture"])
        .env(SESSION_DIR_ENV, dir)
        .env(SESSION_TAG_ENV, tag.to_string())
        .status()
        .expect("spawn child session");
    assert!(status.success(), "child session {tag} failed");
}

#[tokio::test]
async fn compaction_output_names_survive_restart() -> Result<()> {
    if let Ok(dir) = std::env::var(SESSION_DIR_ENV) {
        let tag: u64 = std::env::var(SESSION_TAG_ENV)
            .expect("session tag")
            .parse()
            .expect("session tag is a number");
        return run_session(Path::new(&dir), tag).await;
    }

    let tmp = TempDir::new().expect("temp dir");
    for tag in 1..=SESSIONS {
        run_child_session(tmp.path(), tag);
    }
    verify_all_sessions(tmp.path()).await
}
