# AGENTS.md — `contextra-simd`

## 1. Rolle & Zweck (Baukasten-Prinzip)
`contextra-simd` ist ein isolierter High-Performance-Berechnungsbaustein (Ring 0) für SIMD-vektorisierte Distanz- und Ähnlichkeitsberechnungen (Cosine Similarity, Dot Product, L2/Euclidean Distance, Hamming Distance).
Als genehmigte Unsafe-Insel kapselt es CPU-Intrinsics (AVX2, AVX-512, NEON) und stellt portable, plattformübergreifende Fallbacks bereit, sodass Vektor- und ML-Crates ohne Compiler-Spezifika arbeiten können.

## 2. Ring, Klasse, Reifegrad & `may_use`
- **Ring:** Ring 0 (Hardware-beschleunigte Rechenprimitive, Unsafe-Insel)
- **Klasse:** Kern `[K]`
- **Reifegrad:** `stable`
- **`may_use` (Obergrenze):** `contextra-types` (Spec §6.9, `capabilities.toml`).

## 3. Garantien & Architektur-Invarianten
- **Bit-Identität & Numerische Stabilität:** Plattformübergreifend deterministische und numerisch stabile Distanzberechnungen.
- **Sichere Kernel-Reduktion:** Vermeidung von Sign-Loss-Casts (`u32::from_ne_bytes(hsum.to_ne_bytes())` für SIMD-Reduktionssummen).
- **Zero-Panic-Doktrin:** Vollständige Vorab-Prüfung von Vektordimensionen und Speicher-Alignments; keine Panics bei Dimensionsmismatches (Rückgabe von `Err` oder NaN-Prävention).
- **Explizite Sicherheit:** Jeder interne `unsafe`-Block mit CPU-Intrinsics ist durch `// SAFETY:` dokumentiert und auf Alignment/Bounds geprüft.

## 4. Verbotene Muster
- `unsafe`-Code außerhalb der dedizierten Architektur-Target-Module.
- Direkte I/O- oder Speicherpersistenz-Aufrufe.
- Dynamische Heap-Allokationen innerhalb heißer Distanz-Berechnungsschleifen.
- Abhängigkeiten zu höheren Schichten oder konkreten Indizes.

## 5. Schnittstellen & Datenfluss (Ports & Abstraktionen)
- Stellt optimierte Vektor-Distanz-Routinen für `contextra-vector`, `contextra-rank` und `contextra-infer-*` bereit.

## 6. Tests, Sabotage & Qualitätssicherung
- **Empirische Benchmarks:** Messung von Latenz und Durchsatz je CPU-Architektur (AVX2, AVX-512, NEON vs. Fallback).
- **Numerische Konsistenz-Tests:** Sabotage- und Vergleichstests zwischen SIMD-Pfaden und portablem Skalar-Code.
- **Miri- & Fuzzing-Prüfungen:** Validierung auf Out-of-Bounds Memory-Reads bei ungeraden Vektorlängen.

## 7. Offene Punkte & Technische Schulden
- Keine.
