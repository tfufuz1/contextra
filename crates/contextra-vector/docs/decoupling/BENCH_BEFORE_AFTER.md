# Benchmark Vorher/Nachher: `contextra-vector` (S-36)

## Baseline Messungen (Vorher)
Gemessen am Klon-Stand `f92caabaa` mit Toolchain `rustc 1.89.0` (Target `/tmp/target`).

### 1. `distance_bench` (Dim = 1536)
- `cosine_distance`: Median **4.1526 µs** [4.1356 µs, 4.1796 µs]
- `cosine_scalar`: Median **2.3873 µs** [2.3847 µs, 2.3899 µs]
- `euclidean_distance`: Median **4.1811 µs** [4.1738 µs, 4.1891 µs]
- `euclidean_scalar`: Median **2.3868 µs** [2.3768 µs, 2.3995 µs]
- `dot_product_distance`: Median **4.2670 µs** [4.2042 µs, 4.3433 µs]
- `dot_product_scalar`: Median **2.4310 µs** [2.3957 µs, 2.4692 µs]

### 2. `sq8_bench`
- `SQ8/quantize`: Median **13.958 µs** [13.932 µs, 13.991 µs]
- `SQ8/dequantize`: Median **6.0462 µs** [5.8741 µs, 6.2607 µs]
- `SQ8/asymmetric_dist_cosine`: Median **4.2364 µs** [4.2318 µs, 4.2418 µs]
- `SQ8/symmetric_dist_cosine`: Median **5.4454 µs** [5.4325 µs, 5.4608 µs]

---

## Nachher-Messungen (nach S-36-C und S-36-E)
*Wird nach Umsetzung von Schritt 36-C und 36-E ergänzt.*
