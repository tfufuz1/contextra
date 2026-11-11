# Dokumentation der Sonderwerte-Semantik für contextra-simd

Dieses Dokument beschreibt die exakten mathematischen und deterministischen Ergebnisse
aller Kernel (Scalar, AVX2, AVX-512, NEON) bei extremen Eingabewerten und Grenzfällen (WP-3).

## 1. Übersicht der Sonderwerte-Semantik

| Fall | Eingabe $a$ / $b$ | Metrik | Skalar-Ergebnis | SIMD-Ergebnis | Garantie / Verhalten |
|---|---|---|---|---|---|
| **Null-Vektoren** | Alle Elemente `+0.0` | Euclidean | `0.0` | `0.0` | Exakt 0 (Bitgleich) |
| **Null-Vektoren** | Alle Elemente `+0.0` | DotProduct | `0.0` | `0.0` | Exakt 0 (Bitgleich) |
| **Null-Vektoren** | Alle Elemente `+0.0` | Cosine | `1.0` | `1.0` | Norm ist 0 $\to$ Fallback $1.0$ (Orthogonalitätsäquivalent) |
| **Vorzeichen-Nullen** | Mix aus `+0.0` und `-0.0` | Euclidean / Dot | `0.0` | `0.0` | $(-0.0)^2 = +0.0$, Summe deterministisch |
| **Subnormale Floats** | $0 < \|x\| < 1.175494 \times 10^{-38}$ | Alle Metriken | Berechnet | Berechnet | Relativer Fehler $\le 1 \times 10^{-5}$ oder Flush-to-Zero im Noise-Bereich |
| **Sehr große Beträge** | z. B. $10^{16}$ | Cosine | Normalisiert | Normalisiert | Skalierungsinvariant, relativer Fehler $\le 1 \times 10^{-5}$ |
| **Sehr kleine Beträge** | z. B. $10^{-18}$ | Cosine | Normalisiert | Normalisiert | Skalierungsinvariant, relativer Fehler $\le 1 \times 10^{-5}$ |
| **NaN / Not-a-Number** | Mind. ein Element `f32::NAN` | Alle | `Err(InvalidInput)` | `Err(InvalidInput)` | Fail-Closed via `validate_vector` bzw. `compute_distance` |
| **Infinity / $\pm\infty$** | Mind. ein Element $\pm\infty$ | Alle | `Err(InvalidInput)` | `Err(InvalidInput)` | Fail-Closed via `validate_vector` bzw. `compute_distance` |
| **Identische Vektoren** | $a = b \ne 0$ | Cosine | $0.0$ | $0.0$ | Exakt 0 (innerhalb $1 \times 10^{-6}$) |
| **Entgegengesetzte Vektoren** | $a = -b$ | Cosine | $2.0$ | $2.0$ | Exakt 2.0 (innerhalb $1 \times 10^{-5}$) |
| **Orthogonale Vektoren** | $a \cdot b = 0$ | Cosine | $1.0$ | $1.0$ | Exakt 1.0 (innerhalb $1 \times 10^{-5}$) |

## 2. R3-Dimensionen
Die Differenz- und Orakeltests prüfen alle Metriken über die kanonischen Dimensionen:
- $d = 1$ (Minimalfall, Lane-Rest)
- $d = 4$ (NEON Lane-Größe)
- $d = 17$ (Ungerade Dimension, Lane-Rest $17 \pmod 8 = 1$, $17 \pmod{16} = 1$)
- $d = 128$ (Kompakte Embeddings / Feature-Vektoren)
- $d = 768$ (Standard LLM/Transformer-Embedding, z. B. BERT, MiniLM, Nomic)
- $d = 1536$ (OpenAI text-embedding-3-small, Ada-002)
