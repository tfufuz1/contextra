# ADR-simsimd: simsimd als Benchmark-Referenz (dev-dependencies)

## Kontext
Im Rahmen von WP-91 soll die Leistungsfähigkeit der handgeschriebenen SIMD-Kernel in `contextra-simd` gegen eine etablierte externe SIMD-Bibliothek (`simsimd`) und die Skalar-Referenz (Autovektorisierung durch LLVM) verglichen werden.

## Entscheidung
`simsimd` wird ausschließlich crate-lokal in `crates/contextra-simd/Cargo.toml` unter `[dev-dependencies]` hinzugefügt. Es wird keine Produktionsabhängigkeit eingeführt.

## Alternativen
- Eigene Messung ohne externe Bibliothek: Erfüllt nicht die WP-91-Vorgabe des Vergleichs mit simsimd.
- Externe Abhängigkeit in `[dependencies]`: Verletzt die Autarkie von `contextra-simd` als Ring-0-Insel mit minimalem Fußabdruck.
