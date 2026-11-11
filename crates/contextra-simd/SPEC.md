# contextra-simd — Spezifikation

> **Geltung:** Crate-lokale Spezifikation. Normativ ist `docs/SPEC.md` (v3/v4.0).
> Dieses Dokument definiert die Anforderungen, Invarianten und Schnittstellen für `contextra-simd` als autarken Baustein.

**Pfad:** `crates/contextra-simd/` · **Umsetzungsplan:** `PLAN.md` (im selben Verzeichnis)

## 1. Zweck

Hardwarenahe SIMD-Distanzkerne (AVX2, AVX-512, ARM NEON) mit skalarem Referenzpfad und typsicherem, injizierbarem Dispatcher. Eine der zwei Unsafe-Inseln des Workspace (Ring 0, Klasse A).

## 2. Zielzustand (docs/SPEC.md §3.1)

| Merkmal | Ziel |
|---|---|
| Ring | Ring 0 (Unsafe-Insel) |
| Gruppe | foundation |
| Ebene (Kern-Closure) | E0 (Kern-Closure leer) |
| Klasse | Klasse A (Autarker Baustein) |
| Default-Features | `std` |
| Fehlertyp (R3) | `SimdError` (bzw. Alias `Error`) |
| Brücke (hinter Feature `workspace`) | `From<DistanceMetric>` ↔ `contextra_types::DistanceMetric`, `From<SimdError>` ↔ `contextra_types::ContextraError` in `src/compat/` |
| Autarkie | 0 interne Pflicht-Dependencies. Kompiliert standalone ohne Workspace. |

## 3. Ist-Zustand (Audit-Metriken)

| Messgröße | Wert |
|---|---|
| Modularitäts-Score | 94/100 (Exzellenter Baustein) |
| Lines of Code (Produktion) | ~1900 LOC in 10 Dateien |
| Ungegatete Brücken-Referenzen | 0 (vollständig in `src/compat/` isoliert) |
| Interne Abhängigkeiten (Pflicht) | 0 |
| Interne Abhängigkeiten (Optional) | `contextra-types` (hinter Feature `workspace` / `types-compat`) |
| Schnittstellen | 0 Traits, 4 Structs, 3 Enums |
| Funktionen | 64 pub / 85 gesamt |
| Unsafe-Status | Ring 0 Unsafe-Insel; alle `unsafe`-Blöcke mit lückenloser `// SAFETY:`-Dokumentation |
| Zero-Panic | 0 unkontrollierte Panics in Produktionspfaden |

## 4. Invarianten & Garantien

1. **INV-SIMD-DETERMINISM-1 (Determinismus):** Float-Reduktionen folgen einer festen, unveränderlichen Summationsreihenfolge über Lane-Akkumulatoren und einer deterministischen Baumreduktion (`hsum`). Numerische Abweichungen zwischen SIMD- und Skalarpfaden liegen strikt innerhalb IEEE-754 ULP-Toleranzen (relativ < 1e-5).
2. **INV-SIMD-FINITE-1 (Endlichkeit):** `validate_vector` weist `NaN` und `±Inf` deterministisch zurück (Fail-Closed mit `SimdError::InvalidInput`).
3. **INV-SIMD-ALIGN-1 (Speicherausrichtung):** Byte-Puffer (`&[u8]`) werden über unaligned Loads (`_mm256_loadu_ps`, `vld1q_f32`, `from_le_bytes`) ohne Annahmen über Zeiger-Alignment sicher verarbeitet.
4. **Skalares Orakel:** Die plattformunabhängige Skalarimplementierung in `kernels/scalar.rs` dient als formales Orakel und Referenz für alle SIMD-Kernel in Differenztests.
5. **Injizierbarkeit:** Der `Dispatcher` ist zustandslos und instanziierbar (`Dispatcher::auto()`, `Dispatcher::scalar()`, `Dispatcher::new(SimdTarget)`), was deterministische Tests und CPU-spezifische Isolierung erlaubt.
6. **B09-Entscheidung (DistanceMetric):** `DistanceMetric` existiert als geschlossenes, autarkes Enum lokal in `metric.rs`. Die Konformität zur Workspace-Variante wird in `compat/mod.rs` bidirektional geprüft.

## 5. Modulstruktur

- `lib.rs`: Sichere Fassade (`compute_distance`, `validate_vector`, Re-Exports).
- `dispatch.rs`: Hardware-Feature-Erkennung, `Dispatcher`-Typ und Funktionszeiger.
- `error.rs`: `SimdError` mit `InvalidInput`, `DimensionMismatch`, `UnsupportedMetric`.
- `metric.rs`: Autarkes `DistanceMetric`-Enum.
- `kernels/scalar.rs`: Sichere Referenzimplementierung (f32, u8, mixed, bytes).
- `kernels/avx2.rs`: AVX2 + FMA 256-Bit Vektorkerne.
- `kernels/avx512.rs`: AVX-512 F/BW/VNNI 512-Bit Vektorkerne.
- `kernels/neon.rs`: ARM NEON 128-Bit Vektorkerne.
- `compat/mod.rs`: Optionale Kompatibilitätsbrücke zu `contextra-types` hinter Feature `workspace`.
