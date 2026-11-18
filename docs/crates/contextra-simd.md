# contextra-simd — Crate-Spezifikation

**Ebene:** E0 (Kern-Closure leer)  
**Bindung:** `docs/SPEC.md` §2

---

## 1 Zweck

Unsafe-Insel für SIMD-beschleunigte Vektoroperationen: Distanzberechnung (Cosine, L2, Dot), Quantisierung (SQ4, SQ8). Automatische Runtime-Erkennung der verfügbaren Instruktionssätze (SSE4.2, AVX2, AVX-512, NEON).

## 2 Abhängigkeiten

Keine internen Abhängigkeiten.

## 3 Regeln

- Einziges Crate (neben `sys`) mit erlaubtem `unsafe`.
- Jeder SIMD-Pfad MUSS gegen eine skalare Referenzimplementierung getestet sein (Determinismus-Guard).
- Alle Pfade liefern bitidentische Ergebnisse auf derselben Architektur.
