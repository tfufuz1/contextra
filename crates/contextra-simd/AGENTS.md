# AGENTS.md — `contextra-simd`

> **Normative Vorgabe gemäß [docs/SPEC.md §8.4](docs/SPEC.md) und SPEC §15.4**

## 1. Rolle & Zweck
`contextra-simd` ist die zweite zugelassene `unsafe`-Insel (D-29) im Gesamtsystem. Es stellt hochoptimierte Hardware-Distanzberechnungen für f32-Vektoren (`l2`, `dot`, `cosine`) bereit, die in `contextra-vector` genutzt werden.

## 2. Ring, Abhängigkeiten & Insel-Regeln
- **Ring:** Ring 1 (Distanzkernel, `unsafe`-Insel 2 von 2)
- **Erlaubte Abhängigkeiten:** `contextra-types`
- **Insel-Doktrin (P2, SPEC §8.4):**
  - Jeder `unsafe`-Block erfordert ein präzises `// SAFETY:`-Kommentar mit Nachweis über Alignment, Bounds und SIMD-Registergrenzen.
  - Sichere Skalar-Referenzimplementierungen dienen als unveränderliche Ground-Truth.
  - Exponiert ausschließlich **sichere** öffentliche Funktionen nach außen.

## 3. Funktionsumfang & Kernel-Matrix (Soll-Zustand laut SPEC §8.4)

| Metrik | Datentyp | Implementierungen & Dispatch |
|---|---|---|
| L2-Distanz (Euklidisch) | `f32` | Skalar-Referenz, AVX2 + FMA, AVX-512 (f32), ARM NEON |
| Skalarprodukt (`dot`) | `f32` | Skalar-Referenz, AVX2 + FMA, AVX-512 (f32), ARM NEON |
| Kosinus-Ähnlichkeit | `f32` | Skalar-Referenz, AVX2 + FMA, AVX-512 (f32), ARM NEON |

> [!CAUTION]
> **Gestrichen gemäß SPEC D-28:** Der historische u8-VNNI-Pfad (`_mm512_dpbusd_epi32`) war fehlerhaft und ist **vollständig zu entfernen**. In v1 gibt es keine quantisierten u8-VNNI-Kernel.

## 4. Kritische Invarianten
1. **Genauigkeitskorridor:** Jeder SIMD-Kernel liefert für identische Eingaben das Ergebnis der Skalar-Referenz innerhalb einer relativen Fehlertoleranz von $\epsilon = 10^{-4}$ bei Vektoren mit Norm $\le 10^3$.
2. **Sicheres Tail-Handling:** Vektorlängen, die kein Vielfaches der Registerbreite (8 bzw. 16 Floats) sind, werden deterministisch über den Skalar-Rest berechnet. Keinerlei Out-of-Bounds Lesezugriffe über Slice-Grenzen hinaus!
3. **Deterministischer Laufzeit-Dispatch:** CPU-Feature-Prüfung erfolgt dynamisch über Standard-Makros (`is_x86_feature_detected!`); bei fehlenden Befehlssatzerweiterungen wird verzweigungsfrei auf die Skalar-Referenz zurückgefallen.

## 5. Crate-Arbeitspakete zum SOLL-Zustand (SPEC §15.4)

- **[ ] WP-1.4: SIMD-Reduktion (D-28)**
  - Vollständiges Entfernen des u8-VNNI-Codes und Bereinigung der Kernel-Schnittstellen auf f32.
- **[ ] WP-6.4: Unsafe-Audit & Miri-Validierung**
  - Überprüfung aller Speicherzugriffe und Pointer-Arithmetik mit Miri.
  - Vollständige `// SAFETY:`-Dokumentation für jeden SIMD-Intrinsic-Aufruf.

## 6. Verifikation & Benchmarks
- **T-SIMD-1:** Property-Tests über $10^5$ Zufallsvektoren (Längen 1 bis 4096 Floats) über alle Dispatch-Pfade gegen die Skalar-Referenz.
- **B-SIMD:** Durchsatz-Benchmarks für alle Kernel; Regressionsgrenze $\le 5\,\%$.
