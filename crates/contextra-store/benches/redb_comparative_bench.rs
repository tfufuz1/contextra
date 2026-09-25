// ============================================================================
// CONTEXTRA TIER-1 COMPARATIVE BENCHMARK: contextra-store vs redb
// ============================================================================
// Zweck:
// Dieser Benchmark misst und vergleicht die Lese-, Schreib- und Scan-Performance
// sowie die physische Datenträgergröße (Disk Amplification) von `contextra-store`
// (`LsmStorage`) direkt gegen den Referenz-Key-Value-Store `redb`.
//
// Wichtiger Hinweis:
// Dies ist eine reine PERFORMANCE-Benchmark zur Laufzeit- und Durchsatzanalyse.
// Die funktionale Korrektheit und Äquivalenz der Operationsergebnisse wird
// separat über Differential-Testing in `tests/differential/redb_operation_sequence.rs`
// sichergestellt.
//
// Durability-Äquivalenz:
// - `contextra-store`: Jeder `commit(tx)`-Aufruf schreibt Änderungen in den WAL
//   und führt ein physikalisches `fsync` aus, um synchrone Haltbarkeit zu garantieren.
// - `redb`: Verwendet die Standard-Durability `Durability::Immediate` für Schreibtransaktionen,
//   welche ebenfalls bei jedem `write_txn.commit()` ein synchrones `fsync` auf den
//   Datenträger durchführt.
//
// Beide Systeme bieten somit äquivalente synchrone ACID-Durability-Garantien pro Batch.
// ============================================================================

use contextra_core::{StorageEngine, TxId};
use contextra_store::lsm::{LsmConfig, LsmStorage};
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use redb::TableDefinition;
use std::ops::Bound;
use std::sync::atomic::{AtomicU64, Ordering};
use tempfile::TempDir;
use tokio::runtime::Runtime;

const TABLE: TableDefinition<&[u8], &[u8]> = TableDefinition::new("bench_table");

fn dir_size(path: &std::path::Path) -> std::io::Result<u64> {
    let mut total = 0;
    if path.is_dir() {
        for entry in std::fs::read_dir(path)? {
            let entry = entry?;
            let p = entry.path();
            if p.is_dir() {
                total += dir_size(&p)?;
            } else {
                total += entry.metadata()?.len();
            }
        }
    } else if path.is_file() {
        total += std::fs::metadata(path)?.len();
    }
    Ok(total)
}

fn print_disk_amplification_summary(rt: &Runtime) {
    let lsm_dir = TempDir::new().unwrap();
    let redb_dir = TempDir::new().unwrap();

    // Populate contextra-store with 10,000 entries in 100 batches of 100
    rt.block_on(async {
        let config = LsmConfig {
            path: lsm_dir.path().to_path_buf(),
            ..Default::default()
        };
        let storage = LsmStorage::new(config).await.unwrap();
        let value = vec![0xABu8; 128];
        let mut tx_counter = 1u64;
        for batch_idx in 0..100 {
            let tx = TxId::new(tx_counter);
            tx_counter += 1;
            for i in 0..100 {
                let key = format!("key_{:012}", batch_idx * 100 + i).into_bytes();
                storage.put(tx, &key, &value).await.unwrap();
            }
            storage.commit(tx).await.unwrap();
        }
    });

    // Populate redb with 10,000 entries in 100 batches of 100
    let redb_path = redb_dir.path().join("redb.db");
    let redb_db = redb::Database::create(&redb_path).unwrap();
    let value = vec![0xABu8; 128];
    for batch_idx in 0..100 {
        let write_txn = redb_db.begin_write().unwrap();
        {
            let mut table = write_txn.open_table(TABLE).unwrap();
            for i in 0..100 {
                let key = format!("key_{:012}", batch_idx * 100 + i).into_bytes();
                table.insert(key.as_slice(), value.as_slice()).unwrap();
            }
        }
        write_txn.commit().unwrap();
    }
    drop(redb_db);

    let lsm_bytes = dir_size(lsm_dir.path()).unwrap_or(0);
    let redb_bytes = dir_size(redb_dir.path()).unwrap_or(0);

    println!("\n============================================================================");
    println!("[DISK AMPLIFICATION] Physical Data Size After 10,000 Sequential Writes (128-byte values):");
    println!("  - contextra-store directory size : {} bytes", lsm_bytes);
    println!("  - redb database directory size    : {} bytes", redb_bytes);
    println!("============================================================================\n");
}

fn bench_sequential_write(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let mut group = c.benchmark_group("sequential_write");
    group.sample_size(10);

    group.bench_function("contextra_store_sequential_write", |b| {
        b.to_async(&rt).iter(|| async {
            let tmp = TempDir::new().unwrap();
            let config = LsmConfig {
                path: tmp.path().to_path_buf(),
                ..Default::default()
            };
            let storage = LsmStorage::new(config).await.unwrap();
            let val = vec![0xABu8; 128];
            let mut tx_counter = 1u64;
            for batch in 0..100 {
                let tx = TxId::new(tx_counter);
                tx_counter += 1;
                for i in 0..100 {
                    let k = format!("key_{:012}", batch * 100 + i).into_bytes();
                    storage.put(tx, &k, &val).await.unwrap();
                }
                storage.commit(tx).await.unwrap();
            }
            black_box(storage);
        });
    });

    group.bench_function("redb_sequential_write", |b| {
        b.iter(|| {
            let tmp = TempDir::new().unwrap();
            let db_path = tmp.path().join("redb.db");
            let db = redb::Database::create(&db_path).unwrap();
            let val = vec![0xABu8; 128];
            for batch in 0..100 {
                let write_txn = db.begin_write().unwrap();
                {
                    let mut table = write_txn.open_table(TABLE).unwrap();
                    for i in 0..100 {
                        let k = format!("key_{:012}", batch * 100 + i).into_bytes();
                        table.insert(k.as_slice(), val.as_slice()).unwrap();
                    }
                }
                write_txn.commit().unwrap();
            }
            black_box(db);
        });
    });

    group.finish();

    print_disk_amplification_summary(&rt);
}

fn bench_random_read(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    let lsm_dir = TempDir::new().unwrap();
    let lsm_storage = rt.block_on(async {
        let config = LsmConfig {
            path: lsm_dir.path().to_path_buf(),
            ..Default::default()
        };
        let storage = LsmStorage::new(config).await.unwrap();
        let val = vec![0xABu8; 128];
        let tx = TxId::new(1);
        for i in 0..10_000 {
            let k = format!("key_{:012}", i).into_bytes();
            storage.put(tx, &k, &val).await.unwrap();
        }
        storage.commit(tx).await.unwrap();
        storage
    });

    let redb_dir = TempDir::new().unwrap();
    let redb_path = redb_dir.path().join("redb.db");
    let redb_db = {
        let db = redb::Database::create(&redb_path).unwrap();
        let val = vec![0xABu8; 128];
        let write_txn = db.begin_write().unwrap();
        {
            let mut table = write_txn.open_table(TABLE).unwrap();
            for i in 0..10_000 {
                let k = format!("key_{:012}", i).into_bytes();
                table.insert(k.as_slice(), val.as_slice()).unwrap();
            }
        }
        write_txn.commit().unwrap();
        db
    };

    let sample_keys: Vec<Vec<u8>> = (0..1000)
        .map(|i| {
            let idx = (i * 37 + 13) % 10_000;
            format!("key_{:012}", idx).into_bytes()
        })
        .collect();

    let mut group = c.benchmark_group("random_read");

    group.bench_function("contextra_store_random_read", |b| {
        let storage = &lsm_storage;
        let keys = &sample_keys;
        b.to_async(&rt).iter(|| async move {
            for k in keys {
                let val = storage.get(k).await.unwrap();
                black_box(val);
            }
        });
    });

    group.bench_function("redb_random_read", |b| {
        let db = &redb_db;
        let keys = &sample_keys;
        b.iter(|| {
            let read_txn = db.begin_read().unwrap();
            let table = read_txn.open_table(TABLE).unwrap();
            for k in keys {
                let val = table.get(k.as_slice()).unwrap().map(|v| v.value().to_vec());
                black_box(val);
            }
        });
    });

    group.finish();
}

fn bench_range_scan(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    let lsm_dir = TempDir::new().unwrap();
    let lsm_storage = rt.block_on(async {
        let config = LsmConfig {
            path: lsm_dir.path().to_path_buf(),
            ..Default::default()
        };
        let storage = LsmStorage::new(config).await.unwrap();
        let val = vec![0xABu8; 128];
        let tx = TxId::new(1);
        for i in 0..10_000 {
            let k = format!("key_{:012}", i).into_bytes();
            storage.put(tx, &k, &val).await.unwrap();
        }
        storage.commit(tx).await.unwrap();
        storage
    });

    let redb_dir = TempDir::new().unwrap();
    let redb_path = redb_dir.path().join("redb.db");
    let redb_db = {
        let db = redb::Database::create(&redb_path).unwrap();
        let val = vec![0xABu8; 128];
        let write_txn = db.begin_write().unwrap();
        {
            let mut table = write_txn.open_table(TABLE).unwrap();
            for i in 0..10_000 {
                let k = format!("key_{:012}", i).into_bytes();
                table.insert(k.as_slice(), val.as_slice()).unwrap();
            }
        }
        write_txn.commit().unwrap();
        db
    };

    let scan_ranges: Vec<(Vec<u8>, Vec<u8>)> = (0..10)
        .map(|i| {
            let start_idx = i * 1000;
            let end_idx = start_idx + 100;
            (
                format!("key_{:012}", start_idx).into_bytes(),
                format!("key_{:012}", end_idx).into_bytes(),
            )
        })
        .collect();

    let mut group = c.benchmark_group("range_scan");

    group.bench_function("contextra_store_range_scan", |b| {
        let storage = &lsm_storage;
        let ranges = &scan_ranges;
        b.to_async(&rt).iter(|| async move {
            for (start, end) in ranges {
                let items = storage
                    .scan(
                        Bound::Included(start.as_slice()),
                        Bound::Excluded(end.as_slice()),
                        None,
                    )
                    .await
                    .unwrap();
                black_box(items);
            }
        });
    });

    group.bench_function("redb_range_scan", |b| {
        let db = &redb_db;
        let ranges = &scan_ranges;
        b.iter(|| {
            let read_txn = db.begin_read().unwrap();
            let table = read_txn.open_table(TABLE).unwrap();
            for (start, end) in ranges {
                let range = table
                    .range::<&[u8]>((
                        Bound::Included(start.as_slice()),
                        Bound::Excluded(end.as_slice()),
                    ))
                    .unwrap();
                let mut items = Vec::new();
                for item in range {
                    let (k, v) = item.unwrap();
                    items.push((k.value().to_vec(), v.value().to_vec()));
                }
                black_box(items);
            }
        });
    });

    group.finish();
}

enum MixedOp {
    Read(Vec<u8>),
    Write(Vec<u8>, Vec<u8>),
}

fn generate_mixed_ops(seed: u64, count: usize) -> Vec<MixedOp> {
    let mut rng = StdRng::seed_from_u64(seed);
    let mut ops = Vec::with_capacity(count);
    let val = vec![0xCDu8; 128];

    for _ in 0..count {
        let is_write = rng.gen_bool(0.20);
        let idx = rng.gen_range(0..10_000);
        let key = format!("key_{:012}", idx).into_bytes();

        if is_write {
            ops.push(MixedOp::Write(key, val.clone()));
        } else {
            ops.push(MixedOp::Read(key));
        }
    }

    ops
}

fn bench_mixed_workload(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    let ops = generate_mixed_ops(42, 1000);

    let lsm_dir = TempDir::new().unwrap();
    let lsm_storage = rt.block_on(async {
        let config = LsmConfig {
            path: lsm_dir.path().to_path_buf(),
            ..Default::default()
        };
        let storage = LsmStorage::new(config).await.unwrap();
        let val = vec![0xABu8; 128];
        let tx = TxId::new(1);
        for i in 0..10_000 {
            let k = format!("key_{:012}", i).into_bytes();
            storage.put(tx, &k, &val).await.unwrap();
        }
        storage.commit(tx).await.unwrap();
        storage
    });

    let redb_dir = TempDir::new().unwrap();
    let redb_path = redb_dir.path().join("redb.db");
    let redb_db = {
        let db = redb::Database::create(&redb_path).unwrap();
        let val = vec![0xABu8; 128];
        let write_txn = db.begin_write().unwrap();
        {
            let mut table = write_txn.open_table(TABLE).unwrap();
            for i in 0..10_000 {
                let k = format!("key_{:012}", i).into_bytes();
                table.insert(k.as_slice(), val.as_slice()).unwrap();
            }
        }
        write_txn.commit().unwrap();
        db
    };

    let lsm_tx_counter = AtomicU64::new(2);

    let mut group = c.benchmark_group("mixed_workload");

    group.bench_function("contextra_store_mixed_workload", |b| {
        let storage = &lsm_storage;
        let ops = &ops;
        let counter = &lsm_tx_counter;
        b.to_async(&rt).iter(|| async move {
            for op in ops {
                match op {
                    MixedOp::Read(k) => {
                        let res = storage.get(k).await.unwrap();
                        black_box(res);
                    }
                    MixedOp::Write(k, v) => {
                        let tx = TxId::new(counter.fetch_add(1, Ordering::SeqCst));
                        storage.put(tx, k, v).await.unwrap();
                        storage.commit(tx).await.unwrap();
                    }
                }
            }
        });
    });

    group.bench_function("redb_mixed_workload", |b| {
        let db = &redb_db;
        let ops = &ops;
        b.iter(|| {
            for op in ops {
                match op {
                    MixedOp::Read(k) => {
                        let read_txn = db.begin_read().unwrap();
                        let table = read_txn.open_table(TABLE).unwrap();
                        let res = table.get(k.as_slice()).unwrap().map(|v| v.value().to_vec());
                        black_box(res);
                    }
                    MixedOp::Write(k, v) => {
                        let write_txn = db.begin_write().unwrap();
                        {
                            let mut table = write_txn.open_table(TABLE).unwrap();
                            table.insert(k.as_slice(), v.as_slice()).unwrap();
                        }
                        write_txn.commit().unwrap();
                    }
                }
            }
        });
    });

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
