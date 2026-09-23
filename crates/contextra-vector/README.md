# contextra-index

`contextra-index` stellt den HNSW- und DiskANN-Vektorindex sowie Skalar-/RaBitQ-Quantisierung bereit (Ring 0 / In Migration nach `contextra-vector`).

## Zweck

Core Vector Search Engine für k-NN Abfragen, HNSW-Graphaufbau, DiskANN Disk-Backed Indexierung, SQ8 Skalarquantisierung und Mmap-Persistenz.

## Ring-Zugehörigkeit & Status

- **Ring:** Ring 0 (Synchroner Vektorindex-Kern)
- **Status:** 🟡 In Migration (Zielzustand: `contextra-vector`)
- **Sicherheits-Invariante:** `#![forbid(unsafe_code)]` (SIMD-Kernels wurden nach `contextra-simd` ausgelagert)

## Öffentliche API-Übersicht

- **HNSW Index:** `HnswIndex`, `HnswConfig`, `RebuildStatus`
- **DiskANN Index:** `DiskAnnIndex`, `DiskAnnConfig`, `DiskAnnFallbackPolicy`
- **Quantisierung:** `ScalarQuantizer`, `RaBitQQuantizer`
- **Persistenz:** `HnswHeader`, `MmapIndex`

## Safety & SIMD Isolation

Im Zuge der Zielarchitektur v2 wurden alle hardwarenahen SIMD-Assembly/Intrinsics-Codeblöcke aus `contextra-index` in das dedizierte Unsafe-Insel-Crate `contextra-simd` ausgelagert. `contextra-index` selbst erzwingt `#![forbid(unsafe_code)]`.

## Architektur & Verweise

Details zum HNSW- und DiskANN-Index finden sich in [`ARCHITECTURE.md`](../../ARCHITECTURE.md) (folgt in Kürze) sowie `README.md` §7.4.
