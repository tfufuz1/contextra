# AGENTS – contextra-simd

Abgeleitet aus [`docs/SPEC.md`](../../docs/SPEC.md) (8.4, D-28, D-29, 15.1). Bei Widerspruch gilt die SPEC (D-98). Gemeinsame Regeln: [`AGENTS.md`](../../AGENTS.md).

**Zweck:** Vektor-Distanzfunktionen (L2, Dot, Cosine) mit SIMD-Kernen und Skalar-Referenz.

**Ring und Abhängigkeiten:** Ring 1. Erlaubt: `types`. `unsafe` ist hier erlaubt (AR-8), jede Zeile mit `// SAFETY:`.

## Fachprofil (Arbeitsrolle)

HPC-Ingenieur für Vektorkerne (AVX2, AVX-512, NEON) mit Schwerpunkt Reproduzierbarkeit. Dieses Profil beschreibt Arbeitsweise und Fachwissen und begründet keine eigene Anforderung.

- Distanzkerne (L2, Dot, Cosine) mit Laufzeit-Dispatch, Ausrichtung und Tail-Behandlung.
- Gleitkomma-Determinismus: feste Reduktionsreihenfolge, FMA-Verhalten, Denormale; Ergebnisgleichheit zwischen Pfaden oder dokumentierte Toleranz mit Kreuzvalidierung gegen den Skalarpfad.
- `unsafe`-Disziplin (Intrinsics, `target_feature`, Alignment), Fuzz- und Differentialtests.

## Normative Anker

- Nur **f32-Kerne** (Dot, L2, Cosine) mit Laufzeit-Dispatch und Skalar-Referenz. Der u8-VNNI-Kernel entfällt (D-28: falsche Operandentypen bei `_mm512_dpbusd_epi32`).
- **Jeder Kernel hat einen Cross-Check gegen die Skalar-Referenz** (T-SIMD-1). Ohne ihn ist ein Kernel nicht fertig.
- Der deterministische Rebuild des Vektorindex hängt nur von der Menge der lebenden Slots ab (D-77, T-VEC-REBUILD-INDEP). **Folgerung:** Die Kernel-Ergebnisse müssen für gleiche Eingaben reproduzierbar sein; Determinismus der Graphkonstruktion hat Vorrang vor Spitzengeschwindigkeit.
- Keine Quantisierung außerhalb der Arena-Repräsentationen (D-86); SQ8/PQ/Binär sind spätere Erweiterungen im selben Slot-Layout.
- `unsafe` nur in dieser Insel und in `sys` (D-29). Kein `mmap`.

## Verboten

u8-VNNI-Kernel (D-28), Kernel ohne Skalar-Referenz und Cross-Check, `unsafe` außerhalb der Insel, Panik bei beliebigen Eingabelängen (P6), Features, die Pfade schalten (AR-9).

## Abnahme

`T-SIMD-1` (Property-Tests Skalar gegen SIMD), `T-UNSAFE-1` (Miri auf der Skalar-Referenz), CI-Matrix x86_64 und aarch64 (`.github/workflows/simd-cross-validation.yml`), Benchmark mit Regressionsgrenze (P9, 15.2).

## Arbeitspakete

Rückbau VNNI nach D-28 (WP-1.2); Kerne dienen WP-8.1. Statusquelle: `.agents/jules/tasks/BACKLOG.md`.

## Prüfbefehle

```bash
cargo test -p contextra-simd --locked
cargo +nightly miri test -p contextra-simd
cargo test -p contextra-arch-tests --locked
```
