# contextra-simd — Crate-Spezifikation (Hardware-Vektorkerne & SIMD-Dispatch)

**Ebene:** E0 (Autarkes Fundament / Ring 0 — Unsafe-Insel)  
**Kern-Closure:** Leer (0 interne Workspace-Abhängigkeiten)  
**Status:** Normativ  
**Version:** 1.0  
**Geltungsbereich:** `crates/contextra-simd`  
**Bindung:** `docs/SPEC.md` §2.3 (R5 Unsafe-Inseln), §13 (INV-3, INV-5 Determinismus)

---

## 1. Zweck & Systemrolle

`contextra-simd` ist die hardware- und architekturnahe **Unsafe-Insel** für datenparallele Vektoroperationen in Contextra. Es implementiert hochoptimierte Distanz- und Ähnlichkeitskerne für Gleitkomma- (`f32`) und quantisierte Vektoren (`u8`), die von `contextra-vector` für die HNSW-Suche genutzt werden.

### Die Unsafe-Insel-Regel (SPEC §2.3 R5 / INV-3)
Neben `contextra-sys` ist `contextra-simd` das einzige Crate im gesamten Workspace mit erlaubtem `unsafe`. Die Unsafe-Blöcke dienen **ausschließlich** dem Aufruf architekturgebundener SIMD-Intrinsics (AVX2, AVX-512, NEON). Die öffentliche Schnittstelle ist zu 100 % speichersicher und validiert alle Pufferlängen und Wertebereiche vor dem Hardware-Aufruf.

---

## 2. Architektur & Hardware-Dispatch

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                                contextra-simd                               │
│                                                                             │
│   ┌─────────────────────────────────────────────────────────────────────┐   │
│   │                         Dispatcher (lib.rs)                         │   │
│   │   (Dynamische CPU-Feature-Erkennung beim Start via cpuid/getauxval)  │   │
│   └──────────────────────────────────┬──────────────────────────────────┘   │
│                                      │                                      │
│         ┌────────────────────────────┼────────────────────────────┐         │
│         ▼                            ▼                            ▼         │
│   ┌───────────────┐            ┌───────────────┐            ┌───────────┐   │
│   │    AVX-512    │            │     AVX2      │            │ ARM NEON  │   │
│   │ (x86_64 512b) │            │ (x86_64 256b) │            │(AArch64)  │   │
│   └───────────────┘            └───────────────┘            └───────────┘   │
│                                      │                                      │
│                                      ▼ Fallback                             │
│                        ┌───────────────────────────┐                        │
│                        │     Scalar Reference      │                        │
│                        │  (IEEE 754 Referenzpfad)  │                        │
│                        └───────────────────────────┘                        │
└─────────────────────────────────────────────────────────────────────────────┘
```

### 2.1 Dynamische Erkennung (`dispatch`)

- Ermittelt beim Start der Anwendung die Instruktionssatz-Erweiterungen der CPU:
  - **x86_64:** AVX-512F, AVX2 + FMA, SSE4.2.
  - **AArch64:** ARM NEON (Advanced SIMD).
- Setzt atomare Funktionszeiger auf den schnellsten verfügbaren, sicheren Pfad.
- **Keine Compile-Time-Einschränkung:** Das erzeugte Binärprogramm läuft auf älteren CPUs ohne AVX2 ebenso wie auf modernen Server-CPUs mit AVX-512.

### 2.2 Unterstützte Metriken & Kerne

| Metrik | Datentypen | Implementierte Funktionen |
|--------|------------|---------------------------|
| **Cosine-Distanz** | `f32`, Rohbytes, SQ8 (`u8`) | `cosine_distance`, `cosine_distance_f32_bytes`, `cosine_similarity_parts_u8` |
| **Euklidische Distanz (L2)** | `f32`, Rohbytes, SQ8 (`u8`) | `euclidean_distance`, `euclidean_distance_f32_bytes`, `euclidean_distance_sq_u8` |
| **Skalarprodukt (Dot)** | `f32`, Rohbytes, SQ8 (`u8`) | `dot_product_distance`, `dot_product_distance_f32_bytes`, `dot_product_u8` |

---

## 3. Eingabe-Validierung & NaN-Schutz

SIMD-Operationen dürfen niemals unbemerkt ungültige Gleitkommazustände propagieren:
1. **NaN- und Unendlich-Prüfung:** Jeder Vektor wird vor der Distanzberechnung geprüft. Bei Vorkommen von `f32::NAN` oder `f32::INFINITY` bricht die Funktion mit `Err(SimdError::InvalidFloat)` ab.
2. **Dimensions-Konsistenz:** Vektoren müssen exakt dieselbe Dimension aufweisen. Ein Mismatch führt zu `Err(SimdError::DimensionMismatch { expected, actual })`.
3. **Alignment-Resilienz:** Byte-Puffer werden speichersicher ausgerichtet; unaligned Slices werden ohne SIGBUS oder Panic verarbeitet.

---

## 4. Determinismus-Invariante (INV-5)

Für jeden SIMD-Instruktionssatz existiert im Modul `kernels/scalar.rs` eine mathematische Referenzimplementierung.

**Normative Regel:**
Jeder SIMD-Pfad MUSS auf derselben Prozessorarchitektur bitidentische Ergebnisse zur skalaren Referenz liefern (innerhalb der IEEE-754 Rundungsvorschrift). Dies wird durch kontinuierliche differenzielle Proptest-Suiten sichergestellt.

---

## 5. Abhängigkeiten

| Art | Bibliothek |
|-----|-----------|
| **Intern** | Keine (0 interne Workspace-Abhängigkeiten) |
| **Extern** | `bytemuck`, `thiserror`, `tracing` (optional `serde`) |

---

## 6. Invarianten

1. **Unsafe-Begrenzung:** `unsafe` ist ausschließlich für Intrinsics in `kernels/avx2.rs`, `kernels/avx512.rs` und `kernels/neon.rs` zulässig.
2. **Sichere Signatur:** Alle öffentlichen Funktionen sind vollständig sichere Rust-Funktionen (`fn`, nicht `unsafe fn`).
3. **Zero-Panic-Policy:** Keine `unwrap()`, keine OOB-Indexierungen, keine Panics bei fehlerhaften Slices.

---

## 7. Test-Pflichten

| Test-ID | Beschreibung |
|---------|-------------|
| T-SIMD1 | Differenzialtest: AVX2-, AVX-512- und NEON-Pfade liefern bitidentische Distanzen zur skalaren Referenz |
| T-SIMD2 | NaN-Schutz: Vektor mit `f32::NAN` wird sofort mit `InvalidFloat` abgewiesen |
| T-SIMD3 | Dimensions-Mismatch: Vektoren ungleicher Länge führen zu `DimensionMismatch` |
| T-SIMD4 | Alignment-Test: Nicht-16/32-Byte ausgerichtete Byte-Slices rechnen fehlerfrei |
| T-SIMD5 | SQ8-Quantisierung: `cosine_similarity_parts_u8` liefert korrekte Integer-Skalierung |
