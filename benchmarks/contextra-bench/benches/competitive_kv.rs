use std::ops::Bound;
use std::path::Path;
use std::time::{Duration, Instant};

use contextra_core::{StorageEngine, TxId};
use contextra_store::{LsmConfig, LsmStorage};
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use redb::{Database as RedbDb, TableDefinition as RedbTableDefinition};
use sled::Db as SledDb;
use tempfile::TempDir;
use tokio::runtime::Runtime;

const REDB_TABLE: RedbTableDefinition<&[u8], &[u8]> = RedbTableDefinition::new("competitive_kv");

// Matrix dataset sizes
const WRITE_KEY_COUNT: usize = 1_000_000;
const READ_KEY_COUNT: usize = 100_000;
const MIXED_OP_COUNT: usize = 100_000;
const SCAN_RANGE_LIMIT: usize = 10_000;
const CONTEXTRA_TX_BATCH_SIZE: usize = 5_000;

#[derive(Clone)]
enum KvOp {
    Read(Vec<u8>),
    Write(Vec<u8>, Vec<u8>),
}

/// Generates a synthetic dataset of key-value pairs using a deterministic seed.
fn generate_kv_pairs(count: usize, seed: u64) -> Vec<(Vec<u8>, Vec<u8>)> {
    let mut rng = StdRng::seed_from_u64(seed);
    let mut pairs = Vec::with_capacity(count);
    for i in 0..count {
        let key = format!("key_{:08}", i).into_bytes();
        let mut val = Vec::with_capacity(100);
        val.extend_from_slice(format!("val_{:08}_", i).as_bytes());
        let padding: Vec<u8> = (0..80).map(|_| rng.gen::<u8>()).collect();
        val.extend_from_slice(&padding);
        pairs.push((key, val));
    }
    pairs
}

/// Generates random keys selected from an existing dataset.
fn generate_random_read_keys(dataset: &[(Vec<u8>, Vec<u8>)], count: usize, seed: u64) -> Vec<Vec<u8>> {
    let mut rng = StdRng::seed_from_u64(seed);
    let mut keys = Vec::with_capacity(count);
    let n = dataset.len();
    for _ in 0..count {
        let idx = rng.gen_range(0..n);
        keys.push(dataset[idx].0.clone());
    }
    keys
}

/// Generates a 50/50 mix of read and write operations.
fn generate_mixed_ops(dataset: &[(Vec<u8>, Vec<u8>)], count: usize, seed: u64) -> Vec<KvOp> {
    let mut rng = StdRng::seed_from_u64(seed);
    let mut ops = Vec::with_capacity(count);
    let n = dataset.len();
    for i in 0..count {
        if rng.gen_bool(0.5) {
            let idx = rng.gen_range(0..n);
            ops.push(KvOp::Read(dataset[idx].0.clone()));
        } else {
            let key = format!("mixed_key_{:08}", i).into_bytes();
            let val = format!("mixed_val_{:08}", i).into_bytes();
            ops.push(KvOp::Write(key, val));
        }
    }
    ops
}

async fn create_contextra_db(dir: &Path) -> LsmStorage {
    let config = LsmConfig {
        path: dir.to_path_buf(),
        ..Default::default()
    };
    LsmStorage::new(config)
        .await
        .expect("Failed to create Contextra LsmStorage")
}

async fn populate_contextra(db: &LsmStorage, dataset: &[(Vec<u8>, Vec<u8>)]) {
    let mut tx_counter = 1u64;
    for chunk in dataset.chunks(CONTEXTRA_TX_BATCH_SIZE) {
        let tx = TxId(tx_counter);
        tx_counter += 1;
        for (key, val) in chunk {
            db.put(tx, key, val)
                .await
                .expect("Contextra put failed during populate");
        }
        db.commit(tx)
            .await
            .expect("Contextra commit failed during populate");
    }
}

fn create_redb(dir: &Path) -> RedbDb {
    RedbDb::create(dir.join("redb.db")).expect("Failed to create redb database")
}

fn populate_redb(db: &RedbDb, dataset: &[(Vec<u8>, Vec<u8>)]) {
    let write_txn = db.begin_write().expect("redb begin_write failed");
    {
        let mut table = write_txn
            .open_table(REDB_TABLE)
            .expect("redb open_table failed");
        for (k, v) in dataset {
            table
                .insert(k.as_slice(), v.as_slice())
                .expect("redb insert failed");
        }
    }
    write_txn.commit().expect("redb commit failed");
}

fn create_sled(dir: &Path) -> SledDb {
    sled::open(dir.join("sled_db")).expect("Failed to create sled database")
}

fn populate_sled(db: &SledDb, dataset: &[(Vec<u8>, Vec<u8>)]) {
    for (k, v) in dataset {
        db.insert(k.as_slice(), v.as_slice())
            .expect("sled insert failed");
    }
    db.flush().expect("sled flush failed");
}

// ---------------------------------------------------------------------------
// 3.a) Sequential Write 1M keys: put(tx, k, v) / insert(k, v) / insert(k, v)
// ---------------------------------------------------------------------------
fn bench_sequential_write_1m(c: &mut Criterion) {
    let rt = Runtime::new().expect("Tokio runtime creation failed");
    let mut group = c.benchmark_group("a_sequential_write_1m");
    group.sample_size(10);
    group.measurement_time(Duration::from_secs(10));
    group.throughput(Throughput::Elements(WRITE_KEY_COUNT as u64));

    let dataset = generate_kv_pairs(WRITE_KEY_COUNT, 42);

    // Contextra-LSM
    group.bench_function(BenchmarkId::new("Contextra-LSM", WRITE_KEY_COUNT), |b| {
        b.to_async(&rt).iter_custom(|iters| {
            let dataset = &dataset;
            async move {
                let mut total_duration = Duration::ZERO;
                for i in 0..iters {
                    let tmp = TempDir::new().expect("TempDir failed");
                    let db = create_contextra_db(tmp.path()).await;
                    let start = Instant::now();
                    let mut tx_counter = i * 1_000_000 + 1;
                    for chunk in dataset.chunks(CONTEXTRA_TX_BATCH_SIZE) {
                        let tx = TxId(tx_counter);
                        tx_counter += 1;
                        for (k, v) in chunk {
                            db.put(tx, k, v).await.expect("Contextra put failed");
                        }
                        db.commit(tx).await.expect("Contextra commit failed");
                    }
                    total_duration += start.elapsed();
                    black_box(db);
                }
                total_duration
            }
        });
    });

    // redb
    group.bench_function(BenchmarkId::new("redb", WRITE_KEY_COUNT), |b| {
        b.iter_custom(|iters| {
            let mut total_duration = Duration::ZERO;
            for _ in 0..iters {
                let tmp = TempDir::new().expect("TempDir failed");
                let db = create_redb(tmp.path());
                let start = Instant::now();
                let write_txn = db.begin_write().expect("redb begin_write failed");
                {
                    let mut table = write_txn
                        .open_table(REDB_TABLE)
                        .expect("redb open_table failed");
                    for (k, v) in &dataset {
                        table
                            .insert(k.as_slice(), v.as_slice())
                            .expect("redb insert failed");
                    }
                }
                write_txn.commit().expect("redb commit failed");
                total_duration += start.elapsed();
                black_box(db);
            }
            total_duration
        });
    });

    // sled
    group.bench_function(BenchmarkId::new("sled", WRITE_KEY_COUNT), |b| {
        b.iter_custom(|iters| {
            let mut total_duration = Duration::ZERO;
            for _ in 0..iters {
                let tmp = TempDir::new().expect("TempDir failed");
                let db = create_sled(tmp.path());
                let start = Instant::now();
                for (k, v) in &dataset {
                    db.insert(k.as_slice(), v.as_slice())
                        .expect("sled insert failed");
                }
                db.flush().expect("sled flush failed");
                total_duration += start.elapsed();
                black_box(db);
            }
            total_duration
        });
    });

    group.finish();
}

// ---------------------------------------------------------------------------
// 3.b) Random Read 100k keys: get(&k) / get(&k) / get(&k)
// ---------------------------------------------------------------------------
fn bench_random_read_100k(c: &mut Criterion) {
    let rt = Runtime::new().expect("Tokio runtime creation failed");
    let mut group = c.benchmark_group("b_random_read_100k");
    group.sample_size(10);
    group.measurement_time(Duration::from_secs(10));
    group.throughput(Throughput::Elements(READ_KEY_COUNT as u64));

    let dataset = generate_kv_pairs(WRITE_KEY_COUNT, 42);
    let read_keys = generate_random_read_keys(&dataset, READ_KEY_COUNT, 99);

    // Setup pre-populated stores
    let tmp_ctx = TempDir::new().expect("TempDir failed");
    let db_ctx = rt.block_on(async {
        let db = create_contextra_db(tmp_ctx.path()).await;
        populate_contextra(&db, &dataset).await;
        db
    });

    let tmp_redb = TempDir::new().expect("TempDir failed");
    let db_redb = create_redb(tmp_redb.path());
    populate_redb(&db_redb, &dataset);

    let tmp_sled = TempDir::new().expect("TempDir failed");
    let db_sled = create_sled(tmp_sled.path());
    populate_sled(&db_sled, &dataset);

    // Contextra-LSM
    group.bench_function(BenchmarkId::new("Contextra-LSM", READ_KEY_COUNT), |b| {
        b.to_async(&rt).iter(|| async {
            for k in &read_keys {
                let res = db_ctx.get(k).await.expect("Contextra get failed");
                black_box(res);
            }
        });
    });

    // redb
    group.bench_function(BenchmarkId::new("redb", READ_KEY_COUNT), |b| {
        b.iter(|| {
            let read_txn = db_redb.begin_read().expect("redb begin_read failed");
            let table = read_txn
                .open_table(REDB_TABLE)
                .expect("redb open_table failed");
            for k in &read_keys {
                let res = table.get(k.as_slice()).expect("redb get failed");
                black_box(res);
            }
        });
    });

    // sled
    group.bench_function(BenchmarkId::new("sled", READ_KEY_COUNT), |b| {
        b.iter(|| {
            for k in &read_keys {
                let res = db_sled.get(k.as_slice()).expect("sled get failed");
                black_box(res);
            }
        });
    });

    group.finish();
}

// ---------------------------------------------------------------------------
// 3.c) Mixed 50/50 Read/Write, interleaved
// ---------------------------------------------------------------------------
fn bench_mixed_50_50(c: &mut Criterion) {
    let rt = Runtime::new().expect("Tokio runtime creation failed");
    let mut group = c.benchmark_group("c_mixed_50_50_100k");
    group.sample_size(10);
    group.measurement_time(Duration::from_secs(10));
    group.throughput(Throughput::Elements(MIXED_OP_COUNT as u64));

    let base_dataset = generate_kv_pairs(100_000, 42);
    let ops = generate_mixed_ops(&base_dataset, MIXED_OP_COUNT, 123);

    // Contextra-LSM
    group.bench_function(BenchmarkId::new("Contextra-LSM", MIXED_OP_COUNT), |b| {
        b.to_async(&rt).iter_custom(|iters| {
            let base_dataset = &base_dataset;
            let ops = &ops;
            async move {
                let mut total_duration = Duration::ZERO;
                for iter in 0..iters {
                    let tmp = TempDir::new().expect("TempDir failed");
                    let db = create_contextra_db(tmp.path()).await;
                    populate_contextra(&db, base_dataset).await;

                    let start = Instant::now();
                    for (op_idx, op) in ops.iter().enumerate() {
                        match op {
                            KvOp::Read(k) => {
                                let res = db.get(k).await.expect("Contextra get failed");
                                black_box(res);
                            }
                            KvOp::Write(k, v) => {
                                let tx = TxId(iter * (ops.len() as u64) + (op_idx as u64) + 10_000);
                                db.put(tx, k, v).await.expect("Contextra put failed");
                                db.commit(tx).await.expect("Contextra commit failed");
                            }
                        }
                    }
                    total_duration += start.elapsed();
                    black_box(db);
                }
                total_duration
            }
        });
    });

    // redb
    group.bench_function(BenchmarkId::new("redb", MIXED_OP_COUNT), |b| {
        b.iter_custom(|iters| {
            let mut total_duration = Duration::ZERO;
            for _ in 0..iters {
                let tmp = TempDir::new().expect("TempDir failed");
                let db = create_redb(tmp.path());
                populate_redb(&db, &base_dataset);

                let start = Instant::now();
                for op in &ops {
                    match op {
                        KvOp::Read(k) => {
                            let read_txn = db.begin_read().expect("redb begin_read failed");
                            let table = read_txn
                                .open_table(REDB_TABLE)
                                .expect("redb open_table failed");
                            let res = table.get(k.as_slice()).expect("redb get failed");
                            black_box(res);
                        }
                        KvOp::Write(k, v) => {
                            let write_txn = db.begin_write().expect("redb begin_write failed");
                            {
                                let mut table = write_txn
                                    .open_table(REDB_TABLE)
                                    .expect("redb open_table failed");
                                table
                                    .insert(k.as_slice(), v.as_slice())
                                    .expect("redb insert failed");
                            }
                            write_txn.commit().expect("redb commit failed");
                        }
                    }
                }
                total_duration += start.elapsed();
                black_box(db);
            }
            total_duration
        });
    });

    // sled
    group.bench_function(BenchmarkId::new("sled", MIXED_OP_COUNT), |b| {
        b.iter_custom(|iters| {
            let mut total_duration = Duration::ZERO;
            for _ in 0..iters {
                let tmp = TempDir::new().expect("TempDir failed");
                let db = create_sled(tmp.path());
                populate_sled(&db, &base_dataset);

                let start = Instant::now();
                for op in &ops {
                    match op {
                        KvOp::Read(k) => {
                            let res = db.get(k.as_slice()).expect("sled get failed");
                            black_box(res);
                        }
                        KvOp::Write(k, v) => {
                            db.insert(k.as_slice(), v.as_slice())
                                .expect("sled insert failed");
                        }
                    }
                }
                total_duration += start.elapsed();
                black_box(db);
            }
            total_duration
        });
    });

    group.finish();
}

// ---------------------------------------------------------------------------
// 3.d) Scan 10k-Range
// ---------------------------------------------------------------------------
fn bench_scan_10k_range(c: &mut Criterion) {
    let rt = Runtime::new().expect("Tokio runtime creation failed");
    let mut group = c.benchmark_group("d_scan_10k_range");
    group.sample_size(10);
    group.measurement_time(Duration::from_secs(10));
    group.throughput(Throughput::Elements(SCAN_RANGE_LIMIT as u64));

    let dataset = generate_kv_pairs(100_000, 42);
    let start_key = format!("key_{:08}", 10_000).into_bytes();
    let end_key = format!("key_{:08}", 20_000).into_bytes();

    // Setup pre-populated stores
    let tmp_ctx = TempDir::new().expect("TempDir failed");
    let db_ctx = rt.block_on(async {
        let db = create_contextra_db(tmp_ctx.path()).await;
        populate_contextra(&db, &dataset).await;
        db
    });

    let tmp_redb = TempDir::new().expect("TempDir failed");
    let db_redb = create_redb(tmp_redb.path());
    populate_redb(&db_redb, &dataset);

    let tmp_sled = TempDir::new().expect("TempDir failed");
    let db_sled = create_sled(tmp_sled.path());
    populate_sled(&db_sled, &dataset);

    // Contextra-LSM
    group.bench_function(BenchmarkId::new("Contextra-LSM", SCAN_RANGE_LIMIT), |b| {
        b.to_async(&rt).iter(|| async {
            let res = db_ctx
                .scan(
                    Bound::Included(start_key.as_slice()),
                    Bound::Excluded(end_key.as_slice()),
                    Some(SCAN_RANGE_LIMIT),
                )
                .await
                .expect("Contextra scan failed");
            black_box(res);
        });
    });

    // redb
    group.bench_function(BenchmarkId::new("redb", SCAN_RANGE_LIMIT), |b| {
        b.iter(|| {
            let read_txn = db_redb.begin_read().expect("redb begin_read failed");
            let table = read_txn
                .open_table(REDB_TABLE)
                .expect("redb open_table failed");
            let range = table
                .range(start_key.as_slice()..end_key.as_slice())
                .expect("redb range scan failed");
            let mut count = 0;
            for item in range.take(SCAN_RANGE_LIMIT) {
                let pair = item.expect("redb row decode failed");
                black_box(pair);
                count += 1;
            }
            black_box(count);
        });
    });

    // sled
    group.bench_function(BenchmarkId::new("sled", SCAN_RANGE_LIMIT), |b| {
        b.iter(|| {
            let range = db_sled.range(start_key.as_slice()..end_key.as_slice());
            let mut count = 0;
            for item in range.take(SCAN_RANGE_LIMIT) {
                let pair = item.expect("sled row decode failed");
                black_box(pair);
                count += 1;
            }
            black_box(count);
        });
    });

    group.finish();
}

const TARGET_FD_LIMIT: u64 = 65_536;

/// Ensures the file descriptor limit (RLIMIT_NOFILE) is at least `min_fds`.
/// If the soft limit is lower, it attempts to raise it up to `min_fds` or the hard limit.
/// Returns `true` if the soft limit is at least `min_fds`, or `false` otherwise.
#[allow(unsafe_code)]
fn ensure_fd_limit(min_fds: u64) -> bool {
    #[cfg(unix)]
    {
        use std::mem::MaybeUninit;
        unsafe {
            let mut rlim = MaybeUninit::<libc::rlimit>::uninit();
            if libc::getrlimit(libc::RLIMIT_NOFILE, rlim.as_mut_ptr()) == 0 {
                let mut rlim = rlim.assume_init();
                let min_fds_rlim = min_fds as libc::rlim_t;
                if rlim.rlim_cur < min_fds_rlim {
                    let target = std::cmp::min(min_fds_rlim, rlim.rlim_max);
                    rlim.rlim_cur = target;
                    if libc::setrlimit(libc::RLIMIT_NOFILE, &rlim) != 0 {
                        // setrlimit call failed
                    }
                }
                let mut current_rlim = MaybeUninit::<libc::rlimit>::uninit();
                if libc::getrlimit(libc::RLIMIT_NOFILE, current_rlim.as_mut_ptr()) == 0 {
                    let current_rlim = current_rlim.assume_init();
                    if current_rlim.rlim_cur >= min_fds_rlim {
                        return true;
                    } else {
                        eprintln!(
                            "[WARN] FD limit check: current soft limit {} is lower than required {} (hard limit: {}). Unable to raise soft limit.",
                            current_rlim.rlim_cur, min_fds, current_rlim.rlim_max
                        );
                        return false;
                    }
                }
            }
        }
    }
    true
}

// ---------------------------------------------------------------------------
// 3.e) Recovery-Zeit nach 1M Writes: Drop/Close und neu öffnen
// ---------------------------------------------------------------------------
fn bench_recovery_time_1m(c: &mut Criterion) {
    if !ensure_fd_limit(TARGET_FD_LIMIT) {
        eprintln!(
            "[WARN] Skipping bench_recovery_time_1m benchmark group due to insufficient FD limits."
        );
        return;
    }

    let rt = Runtime::new().expect("Tokio runtime creation failed");
    let mut group = c.benchmark_group("e_recovery_time_1m");
    group.sample_size(10);
    group.measurement_time(Duration::from_secs(10));
    group.throughput(Throughput::Elements(WRITE_KEY_COUNT as u64));

    let dataset = generate_kv_pairs(WRITE_KEY_COUNT, 42);

    // Pre-create and populate persisted data directories
    let tmp_ctx = TempDir::new().expect("TempDir failed");
    let ctx_path = tmp_ctx.path().to_path_buf();
    rt.block_on(async {
        let db = create_contextra_db(&ctx_path).await;
        populate_contextra(&db, &dataset).await;
        if let Err(e) = db.close().await {
            eprintln!("[WARN] Contextra close failed during recovery prep: {e}");
        }
    });

    let tmp_redb = TempDir::new().expect("TempDir failed");
    let redb_db_path = tmp_redb.path().join("redb.db");
    {
        let db = RedbDb::create(&redb_db_path).expect("Failed to create redb database");
        populate_redb(&db, &dataset);
    }

    let tmp_sled = TempDir::new().expect("TempDir failed");
    let sled_path = tmp_sled.path().join("sled_db");
    {
        let db = sled::open(&sled_path).expect("Failed to create sled database");
        populate_sled(&db, &dataset);
    }

    // Contextra-LSM Recovery
    group.bench_function(BenchmarkId::new("Contextra-LSM", WRITE_KEY_COUNT), |b| {
        b.to_async(&rt).iter(|| async {
            let config = LsmConfig {
                path: ctx_path.clone(),
                ..Default::default()
            };
            let start = Instant::now();
            match LsmStorage::new(config).await {
                Ok(db) => {
                    let elapsed = start.elapsed();
                    black_box(db);
                    black_box(elapsed);
                }
                Err(err) => {
                    eprintln!("[WARN] Contextra recovery benchmark iteration failed: {err}");
                }
            }
        });
    });

    // redb Recovery
    group.bench_function(BenchmarkId::new("redb", WRITE_KEY_COUNT), |b| {
        b.iter(|| {
            let start = Instant::now();
            match RedbDb::open(&redb_db_path) {
                Ok(db) => {
                    let elapsed = start.elapsed();
                    black_box(db);
                    black_box(elapsed);
                }
                Err(err) => {
                    eprintln!("[WARN] redb recovery benchmark iteration failed: {err}");
                }
            }
        });
    });

    // sled Recovery
    group.bench_function(BenchmarkId::new("sled", WRITE_KEY_COUNT), |b| {
        b.iter(|| {
            let start = Instant::now();
            match sled::open(&sled_path) {
                Ok(db) => {
                    let elapsed = start.elapsed();
                    black_box(db);
                    black_box(elapsed);
                }
                Err(err) => {
                    eprintln!("[WARN] sled recovery benchmark iteration failed: {err}");
                }
            }
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_sequential_write_1m,
    bench_random_read_100k,
    bench_mixed_50_50,
    bench_scan_10k_range,
    bench_recovery_time_1m
);
criterion_main!(benches);
