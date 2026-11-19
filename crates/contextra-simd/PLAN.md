# contextra-simd — Implementierungsplan zum Soll-Zustand

**Normative Basis:** `docs/crates/contextra-simd.md`. **Ausgangslage:** Der SIMD-Kern hat historische `types`-Kompatibilität, Default-Feature `std` und Nicht-Test-Panikmuster; er darf nicht von `contextra-types` abhängen.

1. Entferne E2-Abhängigkeit und Kompatibilitäts-Feature; der Kern akzeptiert nur primitive Slices/skalare Konfiguration und besitzt `SimdError` selbst.
2. Setze `default = []` und einen sicheren Laufzeit-Dispatch Scalar → AVX2/AVX-512/NEON, der CPU-Merkmale vor jedem Intrinsic prüft.
3. Implementiere L2, Dot, Cosine und SQ8 als identische sichere API; prüfe gleiche Länge, leere Eingabe und NaN/Inf vor Dispatch.
4. Kapsele Intrinsics in kleine, dokumentierte Unsafe-Funktionen mit `SAFETY`-Begründung; ersetze alle Nicht-Test-Paniken durch Ergebnisse.
5. Differentialtests gegen die Skalarreferenz (randomisierte Dimensionen, NaN, Ränder) und Benchmarks ohne semantischen Einfluss.

**Abnahme:** Scalar/SIMD-Bit- bzw. toleranzdefinierte Gleichheit, Feature-/Layering-Gate und Tests auf unterstützten Architekturen.
