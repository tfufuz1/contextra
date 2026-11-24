# Fuzzing Suite — `contextra-vector`

Diese Fuzzing-Ziele sichern die Persistenz- und Parsing-Pfade gegen Crashes, Endlosschleifen und Panics ab.

## Fuzz Targets

1. `fuzz_hnsw_persistence`: Fuzzt die HNSW Serialisierung und Deserialisierung via MmapIndex.
2. `fuzz_mmap_header`: Fuzzt die Header-Validierung über rohe Bytes.
3. `fuzz_hnsw_header_versions`: Fuzzt alle Versionen, Roundtrip-Serialisierung und Checksummen.
4. `fuzz_diskann_formats`: Fuzzt DiskANN DANN, DANF, PWAL und TWAL Formate sowie Recovery.
5. `fuzz_sq8_quantizer`: Fuzzt Training, Quantisierung und Dequantisierung beliebiger Byte-Streams.
6. `hnsw_insert_search`: Fuzzt Index-Operationen (Insert, Search).

## Aufruf mit `cargo-fuzz` (sofern Nightly & cargo-fuzz installiert)

```bash
cargo +nightly fuzz run fuzz_hnsw_header_versions -- -max_total_time=30
cargo +nightly fuzz run fuzz_diskann_formats -- -max_total_time=30
cargo +nightly fuzz run fuzz_sq8_quantizer -- -max_total_time=30
```

## Smoke-Test ohne `cargo-fuzz`

Wenn `cargo-fuzz` nicht verfügbar ist, kann der Smoke-Runner `./run_smoke.sh` ausgeführt werden.
Er führt die Targets gegen die hinterlegten Seed-Korpora aus.
