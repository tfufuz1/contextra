# Unsafe-Inseln und SIMD

Abgeleitet aus SPEC 8.3, 8.4, D-28, D-29, D-94, T-SIMD-1, T-UNSAFE-1, AR-8.

- `unsafe` ist nur in `contextra-sys` und `contextra-simd` erlaubt. Alle anderen Crates setzen `#![forbid(unsafe_code)]`.
- Jeder `unsafe`-Block trägt einen `// SAFETY:`-Kommentar mit den Vorbedingungen (Grenzen, Ausrichtung, Gültigkeit).
- Eine Funktion mit offener Vorbedingung ist `unsafe fn`. Sie wird außerhalb der Inseln nicht angeboten (D-29).
- SIMD: Es gibt f32-Kerne für Dot, L2 und Cosine mit Laufzeit-Dispatch und Skalar-Referenz. Jeder Kern wird gegen die Referenz geprüft (T-SIMD-1).
- Der u8-VNNI-Pfad entfällt (D-28). Grund: falsche Operandentypen bei `_mm512_dpbusd_epi32`.
- Keine Speicherabbildung (`mmap`, `memmap2`) in Produktcrates (D-94).
- Prüfung: Miri auf der Skalar-Referenz von `simd`; AddressSanitizer auf `sys` (`docs/COMPILATION.md`).
