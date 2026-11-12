# Baseline-Bericht STORE-MOD-B

- Datum: 2026-11-11
- Crate: `contextra-store`
- Komponente: SSTable (Format V4/Legacy, Builder, Reader, Bloom, Block-Cache)
- Toolchain: 1.89.0 (laut `rust-toolchain.toml`)

## 1. Ausgangszustand & Vorbefunde
- Baseline-Defekt 1: Fehlende Modul-Deklarationen `pub mod budget;` und `pub mod stats;` in `crates/contextra-store/src/lib.rs` führt zu Kompilierfehlern in abhängigen Modulen (`compaction`, `lsm`).
- Baseline-Defekt 2: `crates/contextra-store/Cargo.toml` enthielt in `[features]` die Definition `workspace = ["dep:contextra-types", "dep:contextra-ports"]`, während `contextra-types` und `contextra-ports` unter `[dependencies]` nicht mit `optional = true` deklariert waren.
- Golden-Corpus-Befund: `tests/golden/.wal_integrity_key` hat eine Länge von 59 Bytes (erwartet 32 Bytes für WAL HMAC Key), weshalb `test_golden_backward_compatibility_read` im WAL-Teil fehlschlägt. Der SSTable-Teil (Single-Block V4, Multiblock Bloom, Legacy V3 MFST) liest erfolgreich.
- Feature-Befund `sieve-cache`: Keine `cfg`-Referenzen im Crate gefunden; Feature dient als Alias/Platzhalter.

## 2. SSTable-Modulmetriken (Ist-Zustand HEAD vor Modularisierung)
- `src/sstable.rs`: 20 Zeilen (Glob-Reexporte)
- `src/sstable/reader.rs`: 1110 Zeilen
- `src/sstable/tests.rs`: 918 Zeilen
- `src/sstable/builder.rs`: 622 Zeilen
- `src/sstable/block_cache.rs`: 472 Zeilen
- `src/sstable/reader_ext.rs`: 303 Zeilen
- `src/sstable/block_search.rs`: 228 Zeilen
- `src/sstable/bloom.rs`: 177 Zeilen
- `src/sstable/stream.rs`: 138 Zeilen
- `src/sstable/io.rs`: 25 Zeilen
Gesamt: 4013 Zeilen.

## 3. Kompilierungsstatus
- `cargo check -p contextra-store --features workspace,test-hooks`: GRÜN (mit Warnings bzgl. ungenutzter Imports / Doku außerhalb unseres Eigentums).
