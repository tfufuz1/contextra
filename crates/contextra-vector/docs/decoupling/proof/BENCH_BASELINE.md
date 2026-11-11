# Messbaseline — `contextra-vector` (Stand `origin/main` f92caabaa)

Stand: 2026-11-11
VM/Host-Umgebung:
- CPU: Intel(R) Xeon(R) Processor @ 2.30GHz (4 Kerne, Haswell-Klasse, AVX2, FMA, BMI2)
- OS: Linux 6.8.0 x86_64
- Toolchain: rustc 1.89.0 (29483883e 2025-08-04), cargo 1.89.0 (c24e10642 2025-06-23)
- Befehle: `cargo bench -p contextra-vector --bench distance_bench`, `cargo bench -p contextra-vector --bench sq8_bench`

*Hinweis:* Zahlen verschiedener virtueller Maschinen sind nicht direkt vergleichbar. Auftrag VEC-A nimmt für spätere Vergleiche eine eigene Vorher/Nachher-Messung auf derselben Maschine vor.

## 1. Messwerte (3 Durchläufe)

### 1.1 Vektordistanzen (`distance_bench`, 128 Dimensionen)

| Benchmark | Lauf 1 (Median) | Lauf 2 (Median) | Lauf 3 (Median) | Gesamt-Median |
|---|---|---|---|---|
| `cosine_distance` (Dispatch/Default) | 4.26 µs | 4.25 µs | 4.28 µs | **4.26 µs** |
| `cosine_scalar` (Skalar) | 2.40 µs | 2.39 µs | 2.40 µs | **2.40 µs** |
| `euclidean_distance` (Dispatch/Default) | 4.13 µs | 4.12 µs | 4.18 µs | **4.13 µs** |
| `euclidean_scalar` (Skalar) | 2.36 µs | 2.35 µs | 2.36 µs | **2.36 µs** |
| `dot_product_distance` (Dispatch/Default) | 4.18 µs | 4.16 µs | 4.29 µs | **4.18 µs** |
| `dot_product_scalar` (Skalar) | 2.35 µs | 2.34 µs | 2.35 µs | **2.35 µs** |

### 1.2 SQ8 Quantisierung (`sq8_bench`, 128 Dimensionen)

| Benchmark | Lauf 1 (Median) | Lauf 2 (Median) | Lauf 3 (Median) | Gesamt-Median |
|---|---|---|---|---|
| `SQ8/quantize` | 14.37 µs | 14.14 µs | 14.43 µs | **14.37 µs** |
| `SQ8/dequantize` | 5.61 µs | 5.58 µs | 5.70 µs | **5.61 µs** |
| `SQ8/asymmetric_dist_cosine` | 4.25 µs | 4.23 µs | 4.31 µs | **4.25 µs** |
| `SQ8/symmetric_dist_cosine` | 5.45 µs | 5.43 µs | 5.52 µs | **5.45 µs** |

## 2. Analyse der Messbaseline
- Skalare Basisfunktionen erreichen auf der vorliegenden VM ca. 2.35–2.40 µs für 128-D Vektoren.
- Quantisierungs-Overhead bei SQ8 liegt bei ~14.4 µs für Vektorkonvertierung und ~4.25 µs für asymmetrische Kosinusberechnung.
