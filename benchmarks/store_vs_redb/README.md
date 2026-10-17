# Contextra vs redb Storage Benchmark Suite

Standalone benchmark suite for evaluating embedded storage engine performance under strictly equal durability guarantees.

## Durability Configuration Parity

To ensure durability-equal comparison:
- **Contextra (`LsmStorage`)**: Configured with `group_commit_window_micros = 0` and `durability_mode = Full`. Every `commit()` forces an immediate `fsync` / `fdatasync` call on the HMAC-chained Write-Ahead Log (WAL).
- **redb (`Database`)**: Configured with `write_txn.set_durability(Durability::Immediate)`. Every commit flushes and fsyncs database state to disk.

## How to Run on Bare Metal

For accurate performance evaluation and decision signals, run on bare metal (or dedicated isolated NVMe instances) using release optimizations.

```bash
cd benchmarks/store_vs_redb
cargo build --release
```

### Fast Smoke Test (< 60s)

Verify harness execution and workload pipelines:

```bash
cargo run --release -- --smoke
```

### Full Benchmark Execution

Run full benchmark suite with 1,000,000 prefilled keys, 128-byte values, and 5 repetitions:

```bash
cargo run --release -- --prefill-keys 1000000 --value-bytes 128 --runs 5 --output-json results.json --output-md summary.md
```

### Decision F5 Threshold Verdict Evaluation

To run benchmarks and automatically evaluate the project decision threshold (Contextra / redb <= 1.5x ratio on random reads, range scans, and commit latencies):

```bash
cargo run --release -- --verdict
```

A non-zero exit code will be returned if any critical performance metric exceeds 1.5x redb.

## Bare Metal Profiling Tips

### 1. Dropping Kernel Page Caches

To ensure cold read measurements reflect real disk I/O rather than Linux page cache hits, drop OS caches before cold workloads (requires root/sudo):

```bash
sudo sysctl -w vm.drop_caches=3
```

### 2. Pinning CPU Cores

To eliminate cross-core migration overhead and NUMA effects:

```bash
taskset -c 0-3 cargo run --release -- --prefill-keys 1000000
```

## Interpreting Decision F5 Results

Project Decision F5 requires that under **equal durability guarantees**:
1. Random Read latency (cold & warm) p99 is at most **1.5x** redb.
2. Range Scan latency p99 is at most **1.5x** redb.
3. Commit latency p99 (single and batched) is at most **1.5x** redb.

If Contextra passes all threshold checks (`--verdict` prints `PASS`), Contextra's LSM storage engine meets the performance criteria for core production deployment.
