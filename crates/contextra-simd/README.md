# contextra-simd

Hardwarenahe f32-Distanzkernel für Vektorsuchen (`l2`, `dot`, `cosine`).

- **Ring:** Ring 1 (Unsafe-Insel 2 von 2, SPEC §8.4, D-29)
- **Normativ:** [docs/SPEC.md §8.4](../../docs/SPEC.md)
- **Arbeitsanweisungen:** [AGENTS.md](AGENTS.md)

## Rolle & Zweck
`contextra-simd` stellt optimierte Distanzberechnungen für f32-Vektoren bereit:
- Euklidische Distanz (`l2`), Skalarprodukt (`dot`), Kosinus-Ähnlichkeit (`cosine`).
- Implementierungen: Skalar-Referenz, AVX2 + FMA, AVX-512 (f32), ARM NEON.
- Dynamischer Laufzeit-Dispatch mit Fallback auf die Skalar-Referenz.

## Invarianten
- Genauigkeitskorridor: Relativer Fehler gegenüber Skalar-Referenz $\le 10^{-4}$ bei Vektoren mit Norm $\le 10^3$.
- Sicheres Tail-Handling: Reste werden deterministisch skalar berechnet; keine Out-of-Bounds-Lesezugriffe.
- Der historische u8-VNNI-Pfad ist gestrichen (`D-28`).
- Jeder `unsafe`-Block erfordert ein `// SAFETY:`-Kommentar.
