# Pending API Changes P07

Format: Symbol | betroffene Crates | gewünschte Änderung

`simd-avx512` (Feature) | `contextra-simd` (Konsument: ggf. künftige AVX-512 Vektor-Index-Integration in `contextra-vector`) | Feature `simd-avx512` ist als optionales, nicht im Default enthaltenes Cargo-Feature in `contextra-simd` definiert. Konsumenten, die AVX-512 explizit anfordern wollen, müssen `features = ["simd-avx512"]` deklarieren.
