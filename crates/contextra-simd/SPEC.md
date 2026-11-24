# Baustein-Spezifikation: `contextra-simd`

> **Revision R12** (Repo-Abgleich, HEAD `60ea034`): Änderungen gemäß `SPEC.md` K-58…K-63 und Anhang B.5. Abschnitte mit Kennzeichnung „R12“ sind normativ.

**Klasse:** L0 Unsafe-Insel (R12, K-58; zuvor A)
**Normativität:** Ring 0 (Unsafe-Insel)
**Status:** Stabil
**MSRV:** 1.89.0

---

## 1. Zweck & Abgrenzung
`contextra-simd` stellt hardwarebeschleunigte mathematische Distanzkerne für Vektoren bereit (Kosinus-Distanz, Euklidische Distanz, Skalarprodukt-Distanz) sowohl für 32-Bit Gleitkommazahlen (`f32`) als auch für quantisierte 8-Bit Ganzzahlen (`u8`).

### Nicht-Ziele
- Keine Vektor-Indexierungsstrukturen (HNSW, IVFPQ leben in `contextra-vector`).
- Keine Einbettungsmodelle oder Inferenz (gehört in `contextra-infer-*`).
- Keine Speicherpersistenz, WAL oder Transaktionslogik.
- Keine Mandanten- oder Löschverwaltung (Löschung lebt in `contextra-erasure`).

---

## 2. Architektur & Invarianten

### Invarianten
1. **Skalares Orakel:** Die skalare Implementierung in `kernels/scalar.rs` ist die deterministische Referenz für alle ISA-Kerne.
2. **INV-SIMD-DETERMINISM-1 (Determinismus):** Reduktionen folgen einer festen Summationsordnung über Vektor-Lanes und deterministische Baumreduktion. Numerische Abweichungen zwischen SIMD- und Skalarpfad liegen strikt innerhalb der dokumentierten Toleranz (< 1e-5 relativ, < 1e-4 absolut für hohe Dimensionen). Test: `test_inv_simd_determinism_1`.
3. **INV-SIMD-FINITE-1 (Endlichkeit):** Ungültige Gleitkommazahlen (NaN, ±Inf) werden via `validate_vector` Fail-Closed zurückgewiesen. Es wird nie mit ungültigen Werten gerechnet. Test: `test_inv_simd_finite_1`.
4. **INV-SIMD-ALIGN-1 (Byte-Ausrichtung):** Byte-Puffer müssen eine durch 4 teilbare Länge aufweisen. Es werden keine Alignment-Annahmen an Speicheradressen getroffen (unaligned Zero-Copy Reads). Test: `test_inv_simd_align_1`.
5. **VNNI-Vorzeichen und Akkumulation (korrigiert R12):** `_mm512_dpbusd_epi32` multipliziert **vorzeichenlos × vorzeichenbehaftet** (u8 × i8). Ein u8×u8-Skalarprodukt wird deshalb über Offset-Transformation berechnet (`a' = a − 128` als i8, Korrekturterm `128·Σb` bzw. analog für `a`), exakt in i32. Die Akkumulation läuft in i32 mit Flush nach i64 nach höchstens `VNNI_FLUSH_EVERY` Blöcken. **Kein stilles Wrapping** im Ergebnis. Test: `test_inv_simd_vnni_sign_1` (Differenztest gegen Skalar bis d = 2^20).
6. **Zero-Panic & Design by Contract:** Öffentliche Funktionen geraten niemals in unkontrollierte Panics; Vorbedingungsverletzungen liefern typisierte `SimdError`/`Error`-Fehler.
7. **Unsafe-Insel (B10):** Jeder `unsafe`-Block ist durch einen expliziten `// SAFETY:`-Beweis mit Host-Feature- und Slice-Garantien dokumentiert.

---

## 3. Schnittstellen & Typen

### Primäre Typen
- `DistanceMetric`: Lokales Enum (`Cosine`, `Euclidean`, `DotProduct`).
- `SimdError`: Typisierter `thiserror`-Fehlertyp (`InvalidInput`, `DimensionMismatch`, `UnsupportedMetric`), markiert mit `#[non_exhaustive]`.
- `Dispatcher`: Zustandsloser, thread-sicherer Dispatcher mit `SimdTarget` (`Auto`, `Scalar`, `Avx2`, `Avx512`, `Neon`).

### Transitional-Features
- `types-compat`: Bietet `From`-Konvertierungen zu `contextra_types::{DistanceMetric, ContextraError}`. Die Kante `simd → types` ist L0→L0 und zulässig, solange `types` `simd` nicht kennt (K-58). Entfällt mit G-CUTOVER.

---

## 4. Concurrency & Performance
- Alle Funktionen und Dispatcher sind `Send + Sync` und vollständig zustandslos.
- Keine Allokationen auf dem Berechnungspfad.
- Zeitkomplexität: $O(d)$ für Distanzberechnung von Dimension $d$.
- Zusatzspeicher: $O(1)$.
