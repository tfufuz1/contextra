// FILE-CONTEXT
// STAND: 2026-11-08T00:00:00Z
// ZWECK: Criterion benchmarks for contextra-sys low-level OS primitives (SPEC v4.0 §8 / B8)
// INVARIANTEN: Zero panic; reliable benchmarking of lock, flush, and mmap operations.

//! # Benchmark-Suite: `sys_bench`
//!
//! Misst Durchsatz und Latenz zentraler Betriebssystem-Primitive aus `contextra-sys`:
//! - **Durability Flushes:** `durable_flush_file` und `write_all_durable`.
//! - **Memory-Locking & Vault:** `mem_lock` / `mem_unlock`, `LockedRegions` Allokation, Sperrung und Zeroization.
//! - **Memory-Mapping:** `mmap_readonly` und `SealedMap` Zugriffe.

#![allow(unsafe_code)]
#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]
#![allow(clippy::indexing_slicing)]

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use std::fs::{File, OpenOptions};
use std::io::Write;

use contextra_sys::{
    durable_flush_file, mem_lock, mem_unlock, mmap_readonly, write_all_durable, LockedRegions,
    MemLockPolicy, SealedFile,
};

fn bench_durability_primitives(c: &mut Criterion) {
    let mut group = c.benchmark_group("durability");

    let temp_dir = tempfile::tempdir().expect("tempdir");
    let file_path = temp_dir.path().join("bench_durability.bin");

    // Pre-create file
    {
        let mut f = File::create(&file_path).expect("create file");
        f.write_all(&vec![0xAAu8; 64 * 1024]).expect("write dummy");
        f.sync_all().expect("sync");
    }

    let file = OpenOptions::new()
        .read(true)
        .write(true)
        .open(&file_path)
        .expect("open file");

    group.bench_function("durable_flush_file_no_meta", |b| {
        b.iter(|| {
            durable_flush_file(black_box(&file), false).expect("durable flush");
        })
    });

    group.bench_function("durable_flush_file_with_meta", |b| {
        b.iter(|| {
            durable_flush_file(black_box(&file), true).expect("durable flush meta");
        })
    });

    let mut write_file = OpenOptions::new()
        .write(true)
        .open(&file_path)
        .expect("open write file");

    let chunk = vec![0xBBu8; 4096];
    group.bench_function("write_all_durable_4k", |b| {
        b.iter(|| {
            write_all_durable(black_box(&mut write_file), black_box(&chunk))
                .expect("write all durable");
        })
    });

    group.finish();
}

fn bench_memory_locking_primitives(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_locking");

    let buffer = vec![0x42u8; 4096];

    group.bench_function("mem_lock_and_unlock_4k", |b| {
        b.iter(|| {
            let ptr = buffer.as_ptr();
            let len = buffer.len();
            let _ = mem_lock(black_box(ptr), black_box(len));
            mem_unlock(black_box(ptr as usize), black_box(len));
        })
    });

    group.bench_function("locked_regions_lock_and_zeroize", |b| {
        b.iter(|| {
            let mut regions = LockedRegions::new();
            let mut secret = vec![0x99u8; 1024];
            let _ = regions.lock_slice_with_policy(
                &secret,
                false, // attempt_mlock = false for pure zeroize/region overhead
                MemLockPolicy::Tolerant,
            );
            LockedRegions::zeroize_slice(black_box(&mut secret));
            regions.unlock_all();
        })
    });

    group.finish();
}

fn bench_mmap_primitives(c: &mut Criterion) {
    let mut group = c.benchmark_group("mmap");

    let temp_dir = tempfile::tempdir().expect("tempdir");
    let file_path = temp_dir.path().join("bench_mmap.bin");

    let data = vec![0x77u8; 64 * 1024];
    {
        let mut f = File::create(&file_path).expect("create file");
        f.write_all(&data).expect("write data");
        f.sync_all().expect("sync");
    }

    let file = File::open(&file_path).expect("open file");

    group.bench_function("mmap_readonly_map", |b| {
        b.iter(|| {
            let map = mmap_readonly(black_box(&file)).expect("mmap readonly");
            black_box(map.first().copied());
        })
    });

    let sealed_file = SealedFile::new_verified(file, data.len() as u64).expect("sealed file");
    group.bench_function("sealed_file_map_readonly", |b| {
        b.iter(|| {
            let sealed_map = sealed_file.map_readonly().expect("map readonly");
            black_box(sealed_map.first().copied());
        })
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_durability_primitives,
    bench_memory_locking_primitives,
    bench_mmap_primitives
);
criterion_main!(benches);
