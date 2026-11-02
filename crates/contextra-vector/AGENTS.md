# AGENTS – contextra-vector

Abgeleitet aus [`docs/SPEC.md`](../../docs/SPEC.md). Bei Widerspruch gilt die SPEC.

**Zweck:** HNSW-Vektorindex mit Arena, löschbaren Slots und verschlüsseltem Snapshot.

**Ring und Abhängigkeiten:** Ring 2. Erlaubt: `types`, `ports`, `simd`, `crypto`.

**Verboten:** `sys` und `mvcc` (D-88), `std::fs`, `memmap2` (D-94), Nullvektoren als Löschung (D-77), Quantisierung außerhalb der Arena (D-86), Features `experimental-diskann`, `experimental-rabitq`, `partial-index-rebuild` (D-84), regionales HNSW-Rewiring (VETO-F02)

**Entscheidungen:** D-18, D-19, D-77, D-84, D-86, D-88

**Tests:** T-VEC-DEL-1, T-VEC-DEL-2, T-VEC-REBUILD-INDEP, T-VEC-ARENA-1, BM-VEC, BM-VEC-DEL, BM-VEC-REBUILD

**Befunde zu prüfen:** F-23, F-32, F-36
