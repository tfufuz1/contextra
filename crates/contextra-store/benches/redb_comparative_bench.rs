//! Performance Benchmark: `contextra-store` (`LsmStorage`) vs `redb::Database`
//!
//! # Purpose
//! This benchmark suite evaluates the comparative execution performance (latency,
//! throughput) and physical disk usage (write/space amplification) of `contextra-store`
//! against `redb` (v2) across identical workload profiles as specified in Spec §22.2.
//!
//! # Durability Equivalence
//! - `contextra-store` (`LsmStorage`): Every transaction commit executes a synchronous
//!   write and fsync flush to the Write-Ahead Log (WAL).
//! - `redb`: Operates with `Durability::Immediate` by default, which executes a synchronous
//!   sync to disk per write transaction commit.
//! Both engines operate under strictly equivalent `Immediate` durability guarantees.
//!
//! # Scope
//! This is a PERFORMANCE benchmark. Functional correctness and differential state
//! verification are covered separately in `tests/differential/redb_operation_sequence.rs`.

#![allow(clippy::unwrap_used)]

use contextra_core::{StorageEngine, TxId};
use contextra_store::lsm::{LsmConfig, LsmStorage};
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use redb::{Database, Durability, TableDefinition};
use std::fs;
use std::ops::Bound;
use std::path::Path;
use tempfile::TempDir;
use tokio::runtime::Runtime;

const REDB_TABLE: TableDefinition<&[u8], &[u8]> = TableDefinition::new("bench_table");

const NUM_ITEMS: usize = 10_000;
const BATCH_SIZE: usize = 100;
const VALUE_SIZE: usize = 128;
const PREPOPULATE_COUNT: usize = 5_000;
const MIXED_OPS_COUNT: usize = 100;

fn calculate_dir_size(path: &Path) -> std::io::Result<u64> {
    let mut total = 0;
    if path.is_dir() {
        for entry in fs::read_dir(path)? {
            let entry = entry?;
            let p = entry.path();
            if p.is_dir() {
                total += calculate_dir_size(&p)?;
            } else {
                total += entry.metadata()?.len();
            }
        }
    } else if path.is_file() {
        total += fs::metadata(path)?.len();
    }
    Ok(total)
}

fn generate_key(i: usize) -> Vec<u8> {
    format!("key_{:08}", i).into_bytes()
}

fn generate_value() -> Vec<u8> {
    vec![0xA5; VALUE_SIZE]
}

fn setup_redb_table(db: &Database) {
    let mut write_txn = db.begin_write().unwrap();
    write_txn.set_durability(Durability::Immediate);
    {
        let _ = write_txn.open_table(REDB_TABLE).unwrap();
    }
    write_txn.commit().unwrap();
}

fn bench_sequential_write(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let mut group = c.benchmark_group("sequential_write");

    let val = generate_value();

    {
        let val = val.clone();
        group.bench_function("contextra_store_sequential_write", |b| {
            let val = val.clone();
            b.to_async(&rt).iter_custom(|iters| {
                let val = val.clone();
                async move {
                    let mut total_duration = std::time::Duration::ZERO;

                    for _ in 0..iters {
                        let tmp_dir = TempDir::new().unwrap();
                        let config = LsmConfig {
                            path: tmp_dir.path().to_path_buf(),
                            memtable_size_limit: 1024 * 1024,
                            ..Default::default()
                        };
                        let storage = LsmStorage::new(config).await.unwrap();

                        let start = std::time::Instant::now();
                        let mut tx_counter = 1u64;

                        for chunk in (0..NUM_ITEMS).collect::<Vec<_>>().chunks(BATCH_SIZE) {
                            let tx = TxId::new(tx_counter);
                            tx_counter += 1;
                            for &i in chunk {
                                let key = generate_key(i);
                                storage.put(tx, &key, &val).await.unwrap();
                            }
                            storage.commit(tx).await.unwrap();
                        }

                        total_duration += start.elapsed();
                    }

                    total_duration
                }
            });
        });
    }

    {
        let val = val.clone();
        group.bench_function("redb_sequential_write", |b| {
            let val = val.clone();
            b.iter_custom(|iters| {
                let mut total_duration = std::time::Duration::ZERO;

                for _ in 0..iters {
                    let tmp_dir = TempDir::new().unwrap();
                    let db_path = tmp_dir.path().join("redb_bench.db");
                    let db = Database::create(&db_path).unwrap();
                    setup_redb_table(&db);

                    let start = std::time::Instant::now();

                    for chunk in (0..NUM_ITEMS).collect::<Vec<_>>().chunks(BATCH_SIZE) {
                        let mut write_txn = db.begin_write().unwrap();
                        write_txn.set_durability(Durability::Immediate);
                        {
                            let mut table = write_txn.open_table(REDB_TABLE).unwrap();
                            for &i in chunk {
                                let key = generate_key(i);
                                table.insert(key.as_slice(), val.as_slice()).unwrap();
                            }
                        }
                        write_txn.commit().unwrap();
                    }

                    total_duration += start.elapsed();
                }

                total_duration
            });
        });
    }

    group.finish();

    // Perform disk usage measurement after sequential write workload
    rt.block_on(async {
        let lsm_tmp = TempDir::new().unwrap();
        let lsm_config = LsmConfig {
            path: lsm_tmp.path().to_path_buf(),
            memtable_size_limit: 1024 * 1024,
            ..Default::default()
        };
        let lsm_storage = LsmStorage::new(lsm_config).await.unwrap();
        let mut tx_counter = 1u64;
        for chunk in (0..NUM_ITEMS).collect::<Vec<_>>().chunks(BATCH_SIZE) {
            let tx = TxId::new(tx_counter);
            tx_counter += 1;
            for &i in chunk {
                lsm_storage.put(tx, &generate_key(i), &val).await.unwrap();
            }
            lsm_storage.commit(tx).await.unwrap();
        }
        lsm_storage.force_flush().await.unwrap();
        let lsm_bytes = calculate_dir_size(lsm_tmp.path()).unwrap_or(0);

        let redb_tmp = TempDir::new().unwrap();
        let redb_path = redb_tmp.path().join("redb_bench.db");
        let redb_db = Database::create(&redb_path).unwrap();
        setup_redb_table(&redb_db);
        for chunk in (0..NUM_ITEMS).collect::<Vec<_>>().chunks(BATCH_SIZE) {
            let mut write_txn = redb_db.begin_write().unwrap();
            write_txn.set_durability(Durability::Immediate);
            {
                let mut table = write_txn.open_table(REDB_TABLE).unwrap();
                for &i in chunk {
                    table
                        .insert(generate_key(i).as_slice(), val.as_slice())
                        .unwrap();
                }
            }
            write_txn.commit().unwrap();
        }
        let redb_bytes = calculate_dir_size(redb_tmp.path()).unwrap_or(0);

        println!(
            "\n--- PHYSICAL DISK AMPLIFICATION ({NUM_ITEMS} items, {VALUE_SIZE}B values) ---"
        );
        println!(
            "  contextra-store physical dir size: {} bytes ({:.2} MB)",
            lsm_bytes,
            lsm_bytes as f64 / 1_048_576.0
        );
        println!(
            "  redb physical dir size:           {} bytes ({:.2} MB)",
            redb_bytes,
            redb_bytes as f64 / 1_048_576.0
        );
        println!("------------------------------------------------------------\n");
    });
}

fn bench_random_read(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let mut group = c.benchmark_group("random_read");

    let val = generate_value();

    // Contextra store setup
    let lsm_tmp = TempDir::new().unwrap();
    let lsm_config = LsmConfig {
        path: lsm_tmp.path().to_path_buf(),
        memtable_size_limit: 1024 * 1024,
        ..Default::default()
    };
    let lsm_storage = rt.block_on(async {
        let storage = LsmStorage::new(lsm_config).await.unwrap();
        let mut tx_counter = 1u64;
        for chunk in (0..PREPOPULATE_COUNT).collect::<Vec<_>>().chunks(BATCH_SIZE) {
            let tx = TxId::new(tx_counter);
            tx_counter += 1;
            for &i in chunk {
                storage.put(tx, &generate_key(i), &val).await.unwrap();
            }
            storage.commit(tx).await.unwrap();
        }
        storage.force_flush().await.unwrap();
        storage
    });

    // redb setup
    let redb_tmp = TempDir::new().unwrap();
    let redb_path = redb_tmp.path().join("redb_bench.db");
    let redb_db = {
        let db = Database::create(&redb_path).unwrap();
        setup_redb_table(&db);
        for chunk in (0..PREPOPULATE_COUNT).collect::<Vec<_>>().chunks(BATCH_SIZE) {
            let mut write_txn = db.begin_write().unwrap();
            write_txn.set_durability(Durability::Immediate);
            {
                let mut table = write_txn.open_table(REDB_TABLE).unwrap();
                for &i in chunk {
                    table
                        .insert(generate_key(i).as_slice(), val.as_slice())
                        .unwrap();
                }
            }
            write_txn.commit().unwrap();
        }
        db
    };

    let sample_keys: Vec<Vec<u8>> = {
        let mut rng = StdRng::seed_from_u64(42);
        (0..1_000)
            .map(|_| generate_key(rng.gen_range(0..PREPOPULATE_COUNT)))
            .collect()
    };

    group.bench_function("contextra_store_random_read", |b| {
        let mut idx = 0;
        b.to_async(&rt).iter(|| {
            let key = &sample_keys[idx % sample_keys.len()];
            idx += 1;
            let storage = &lsm_storage;
            async move {
                let res = storage.get(black_box(key)).await.unwrap();
                black_box(res);
            }
        });
    });

    group.bench_function("redb_random_read", |b| {
        let mut idx = 0;
        b.iter(|| {
            let key = &sample_keys[idx % sample_keys.len()];
            idx += 1;
            let read_txn = redb_db.begin_read().unwrap();
            let table = read_txn.open_table(REDB_TABLE).unwrap();
            let res = table.get(black_box(key.as_slice())).unwrap();
            black_box(res.map(|v| v.value().to_vec()));
        });
    });

    group.finish();
}

fn bench_range_scan(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let mut group = c.benchmark_group("range_scan");

    let val = generate_value();

    let lsm_tmp = TempDir::new().unwrap();
    let lsm_config = LsmConfig {
        path: lsm_tmp.path().to_path_buf(),
        memtable_size_limit: 1024 * 1024,
        ..Default::default()
    };
    let lsm_storage = rt.block_on(async {
        let storage = LsmStorage::new(lsm_config).await.unwrap();
        let mut tx_counter = 1u64;
        for chunk in (0..PREPOPULATE_COUNT).collect::<Vec<_>>().chunks(BATCH_SIZE) {
            let tx = TxId::new(tx_counter);
            tx_counter += 1;
            for &i in chunk {
                storage.put(tx, &generate_key(i), &val).await.unwrap();
            }
            storage.commit(tx).await.unwrap();
        }
        storage.force_flush().await.unwrap();
        storage
    });

    let redb_tmp = TempDir::new().unwrap();
    let redb_path = redb_tmp.path().join("redb_bench.db");
    let redb_db = {
        let db = Database::create(&redb_path).unwrap();
        setup_redb_table(&db);
        for chunk in (0..PREPOPULATE_COUNT).collect::<Vec<_>>().chunks(BATCH_SIZE) {
            let mut write_txn = db.begin_write().unwrap();
            write_txn.set_durability(Durability::Immediate);
            {
                let mut table = write_txn.open_table(REDB_TABLE).unwrap();
                for &i in chunk {
                    table
                        .insert(generate_key(i).as_slice(), val.as_slice())
                        .unwrap();
                }
            }
            write_txn.commit().unwrap();
        }
        db
    };

    let start_key = generate_key(1_000);
    let end_key = generate_key(1_500);

    group.bench_function("contextra_store_range_scan", |b| {
        b.to_async(&rt).iter(|| {
            let storage = &lsm_storage;
            let sk = &start_key;
            let ek = &end_key;
            async move {
                let items = storage
                    .scan(
                        Bound::Included(black_box(sk.as_slice())),
                        Bound::Excluded(black_box(ek.as_slice())),
                        None,
                    )
                    .await
                    .unwrap();
                black_box(items);
            }
        });
    });

    group.bench_function("redb_range_scan", |b| {
        b.iter(|| {
            let read_txn = redb_db.begin_read().unwrap();
            let table = read_txn.open_table(REDB_TABLE).unwrap();
            let range = table
                .range::<&[u8]>((
                    Bound::Included(black_box(start_key.as_slice())),
                    Bound::Excluded(black_box(end_key.as_slice())),
                ))
                .unwrap();

            let mut count = 0;
            for item in range {
                let (k, v) = item.unwrap();
                black_box((k.value(), v.value()));
                count += 1;
            }
            black_box(count);
        });
    });

    group.finish();
}

#[derive(Clone)]
enum MixedOp {
    Read(Vec<u8>),
    Write(Vec<u8>, Vec<u8>),
}

fn generate_mixed_ops(count: usize) -> Vec<MixedOp> {
    let mut rng = StdRng::seed_from_u64(1337);
    let mut ops = Vec::with_capacity(count);
    let val = generate_value();

    for _ in 0..count {
        let key_idx = rng.gen_range(0..PREPOPULATE_COUNT);
        let key = generate_key(key_idx);
        let roll: f64 = rng.gen();
        if roll < 0.8 {
            ops.push(MixedOp::Read(key));
        } else {
            ops.push(MixedOp::Write(key, val.clone()));
        }
    }
    ops
}

fn bench_mixed_workload(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let mut group = c.benchmark_group("mixed_workload");

    let val = generate_value();
    let mixed_ops = generate_mixed_ops(MIXED_OPS_COUNT);

    {
        let val = val.clone();
        let mixed_ops = mixed_ops.clone();
        group.bench_function("contextra_store_mixed_workload", |b| {
            let val = val.clone();
            let mixed_ops = mixed_ops.clone();
            b.to_async(&rt).iter_custom(|iters| {
                let val = val.clone();
                let mixed_ops = mixed_ops.clone();
                async move {
                    let mut total_duration = std::time::Duration::ZERO;

                    for _ in 0..iters {
                        let tmp_dir = TempDir::new().unwrap();
                        let config = LsmConfig {
                            path: tmp_dir.path().to_path_buf(),
                            memtable_size_limit: 1024 * 1024,
                            ..Default::default()
                        };
                        let storage = LsmStorage::new(config).await.unwrap();
                        let mut tx_counter = 1u64;

                        // Pre-populate
                        for chunk in (0..PREPOPULATE_COUNT).collect::<Vec<_>>().chunks(BATCH_SIZE) {
                            let tx = TxId::new(tx_counter);
                            tx_counter += 1;
                            for &i in chunk {
                                storage.put(tx, &generate_key(i), &val).await.unwrap();
                            }
                            storage.commit(tx).await.unwrap();
                        }

                        let start = std::time::Instant::now();

                        for op in &mixed_ops {
                            match op {
                                MixedOp::Read(k) => {
                                    let res = storage.get(black_box(k)).await.unwrap();
                                    black_box(res);
                                }
                                MixedOp::Write(k, v) => {
                                    let tx = TxId::new(tx_counter);
                                    tx_counter += 1;
                                    storage.put(tx, k, v).await.unwrap();
                                    storage.commit(tx).await.unwrap();
                                }
                            }
                        }

                        total_duration += start.elapsed();
                    }

                    total_duration
                }
            });
        });
    }

    {
        let val = val.clone();
        let mixed_ops = mixed_ops.clone();
        group.bench_function("redb_mixed_workload", |b| {
            let val = val.clone();
            let mixed_ops = mixed_ops.clone();
            b.iter_custom(|iters| {
                let mut total_duration = std::time::Duration::ZERO;

                for _ in 0..iters {
                    let tmp_dir = TempDir::new().unwrap();
                    let db_path = tmp_dir.path().join("redb_bench.db");
                    let db = Database::create(&db_path).unwrap();
                    setup_redb_table(&db);

                    // Pre-populate
                    for chunk in (0..PREPOPULATE_COUNT).collect::<Vec<_>>().chunks(BATCH_SIZE) {
                        let mut write_txn = db.begin_write().unwrap();
                        write_txn.set_durability(Durability::Immediate);
                        {
                            let mut table = write_txn.open_table(REDB_TABLE).unwrap();
                            for &i in chunk {
                                table
                                    .insert(generate_key(i).as_slice(), val.as_slice())
                                    .unwrap();
                            }
                        }
                        write_txn.commit().unwrap();
                    }

                    let start = std::time::Instant::now();

                    for op in &mixed_ops {
                        match op {
                            MixedOp::Read(k) => {
                                let read_txn = db.begin_read().unwrap();
                                let table = read_txn.open_table(REDB_TABLE).unwrap();
                                let res = table.get(black_box(k.as_slice())).unwrap();
                                black_box(res.map(|v| v.value().to_vec()));
                            }
                            MixedOp::Write(k, v) => {
                                let mut write_txn = db.begin_write().unwrap();
                                write_txn.set_durability(Durability::Immediate);
                                {
                                    let mut table = write_txn.open_table(REDB_TABLE).unwrap();
                                    table.insert(k.as_slice(), v.as_slice()).unwrap();
                                }
                                write_txn.commit().unwrap();
                            }
                        }
                    }

                    total_duration += start.elapsed();
                }

                total_duration
            });
        });
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_sequential_write,
    bench_random_read,
    bench_range_scan,
    bench_mixed_workload
);
criterion_main!(benches);
