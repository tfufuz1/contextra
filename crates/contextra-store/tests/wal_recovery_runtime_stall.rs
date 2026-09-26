// FILE-CONTEXT
// ZWECK: Runtime stall test for WAL recovery replay to detect non-linear (O(n²)) complexity regressions.
// INVARIANTEN: Replay throughput across subsequent windows MUST NOT drop below 50% of the baseline throughput (first 10%).
// STAND: TS:2026-09-26T00:00:00Z

use contextra_core::TxId;
use contextra_store::wal::{
    ReplayProgressSink, Wal, WalEntry, WalOp, WalSeq, WAL_V3_HEADER,
};
use std::io::Write;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;
use std::time::Instant;
use tempfile::tempdir;

const TOTAL_ENTRIES: u64 = 1_000_000;
const WINDOW_SIZE: u64 = 100_000; // 10% of total entries

struct ThroughputProgressSink {
    start_time: Instant,
    replayed_count: AtomicU64,
    window_timestamps: Mutex<Vec<(u64, Instant)>>,
}

impl ThroughputProgressSink {
    fn new() -> Self {
        Self {
            start_time: Instant::now(),
            replayed_count: AtomicU64::new(0),
            window_timestamps: Mutex::new(Vec::new()),
        }
    }
}

impl ReplayProgressSink for ThroughputProgressSink {
    fn on_entry_replayed(&self, _seq: WalSeq, _entries_total_estimate: Option<u64>) {
        let count = self.replayed_count.fetch_add(1, Ordering::Relaxed) + 1;
        if count % WINDOW_SIZE == 0 {
            let mut guard = self.window_timestamps.lock().unwrap();
            guard.push((count, Instant::now()));
        }
    }
}

#[tokio::test]
async fn test_wal_recovery_linear_throughput_no_runtime_stall() {
    let dir = tempdir().expect("create tempdir");
    let wal_path = dir.path().join("stall_test.wal");

    // 1. Synthesize a 1,000,000 entry WAL file on disk
    {
        let file = std::fs::File::create(&wal_path).expect("create WAL file");
        let mut writer = std::io::BufWriter::with_capacity(8 * 1024 * 1024, file);

        writer.write_all(&WAL_V3_HEADER).expect("write header");

        let integrity_key = [0u8; 32];
        let mut prev_hmac = [0u8; 32];

        for i in 1..=TOTAL_ENTRIES {
            let op = WalOp::Put {
                tx_id: TxId::new(i),
                key: b"k".to_vec(),
                value: b"v".to_vec(),
            };
            let entry = WalEntry::try_new(op, i, &integrity_key, prev_hmac)
                .expect("construct entry");
            prev_hmac = entry.checksum;

            let bytes = entry.to_bytes().expect("serialize entry");
            writer.write_all(&bytes).expect("write entry");
        }
        writer.flush().expect("flush synthetic WAL");
    }

    // 2. Open WAL read-only and replay with progress sink
    let wal = Wal::open_read_only(&wal_path, None)
        .await
        .expect("open read-only WAL");

    let sink = ThroughputProgressSink::new();
    let replayed_entries = wal
        .replay_with_sink(&sink)
        .await
        .expect("replay WAL with sink");

    assert_eq!(
        replayed_entries.len() as u64,
        TOTAL_ENTRIES,
        "Replay must decode all synthesized entries"
    );

    // 3. Analyze window throughput
    let timestamps = sink.window_timestamps.lock().unwrap();
    assert_eq!(
        timestamps.len(),
        10,
        "Must record timestamps for all 10 10% windows"
    );

    let start_time = sink.start_time;
    let first_window_duration = timestamps[0].1.duration_since(start_time);
    let first_window_secs = first_window_duration.as_secs_f64();

    // Baseline throughput (entries/sec for first 10%)
    let baseline_throughput = (WINDOW_SIZE as f64) / first_window_secs.max(1e-6);

    let min_allowed_throughput = baseline_throughput * 0.50;

    let mut prev_instant = timestamps[0].1;
    for (idx, (count, instant)) in timestamps.iter().enumerate().skip(1) {
        let window_duration = instant.duration_since(prev_instant);
        let window_secs = window_duration.as_secs_f64();
        let window_throughput = (WINDOW_SIZE as f64) / window_secs.max(1e-6);

        // Print debug window performance
        println!(
            "Window {} (up to entry {}): {:.2?} ({:.0} entries/s vs baseline {:.0} entries/s)",
            idx + 1,
            count,
            window_duration,
            window_throughput,
            baseline_throughput
        );

        // Allow microsecond-level timing noise if window completed in under 2ms
        if window_duration.as_millis() > 2 {
            assert!(
                window_throughput >= min_allowed_throughput,
                "WAL replay runtime stall detected in window {} (up to entry {}): throughput {:.0} entries/s dropped below 50% of baseline throughput {:.0} entries/s",
                idx + 1,
                count,
                window_throughput,
                baseline_throughput
            );
        }

        prev_instant = *instant;
    }
}
