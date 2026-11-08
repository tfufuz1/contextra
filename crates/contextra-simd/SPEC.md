# Baustein-Spezifikation: `contextra-simd`

**Klasse:** A (Vollständig autarker Baustein)
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
2. **Determinismus (INV-RING0-DETERMINISM-1):** Reduktionen folgen einer festen Summationsordnung über Vektor-Lanes und deterministische Baumreduktion. Numerische Abweichungen zwischen SIMD- und Skalarpfad liegen strikt innerhalb IEEE-754 ULP-Toleranzen (< 1e-4).
3. **Tail-Handling & Validierung:** Beliebige Vektorlängen werden ohne Vorbedingung `len % lane == 0` unterstützt. Ungültige Gleitkommazahlen (NaN, ±Inf) werden via `validate_vector` Fail-Closed zurückgewiesen.
4. **AVX-512 VNNI Vorzeichen- und Wrapping-Semantik:** Bei `dot_product_u8_avx512vnni` (`_mm512_dpbusd_epi32`) und `cosine_similarity_parts_u8_avx512` werden die Operanden auf Byte-Ebene mit 32-Bit Unsigned Wrapping-Akkumulation addiert.
5. **Zero-Panic & Design by Contract:** Öffentliche Funktionen geraten niemals in unkontrollierte Panics; Vorbedingungsverletzungen liefern typisierte `SimdError`-Fehler.
6. **Unsafe-Insel (B10):** Jeder `unsafe`-Block ist durch einen expliziten `// SAFETY:`-Beweis mit Host-Feature- und Slice-Garantien dokumentiert.

---

## 3. Schnittstellen & Typen

### Primäre Typen
- `DistanceMetric`: Lokales Enum (`Cosine`, `Euclidean`, `DotProduct`).
- `SimdError`: Typisierter `thiserror`-Fehlertyp (`InvalidInput`, `DimensionMismatch`, `UnsupportedMetric`), markiert mit `#[non_exhaustive]`.
- `Dispatcher`: Zustandsloser, thread-sicherer Dispatcher mit `SimdTarget` (`Auto`, `Scalar`, `Avx2`, `Avx512`, `Neon`).

### Transitional-Features
- `types-compat`: Bietet `From`-Konvertierungen zu `contextra_types::{DistanceMetric, ContextraError}` für abwärtskompatible Workspace-Kanten bis Welle 6.

---

## 4. Concurrency & Performance
- Alle Funktionen und Dispatcher sind `Send + Sync` und vollständig zustandslos.
- Keine Allokationen auf dem Berechnungspfad.
- Zeitkomplexität: $O(d)$ für Distanzberechnung von Dimension $d$.
- Zusatzspeicher: $O(1)$.
