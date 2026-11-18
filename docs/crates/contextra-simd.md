# contextra-simd — Implementierungsspezifikation (Soll-Zustand)

**Ebene:** E0 (Autarkes Fundament / Ring 0 — Unsafe-Insel)  
**Kern-Closure:** Leer (0 interne Workspace-Abhängigkeiten)  
**Status:** Normativ (Soll-Zustand für Entwickler)  
**Version:** 2.0  
**Geltungsbereich:** `crates/contextra-simd`  
**Referenzdokumente:** `docs/SPEC.md` §2.3 (R1, R5), §13 (INV-3, INV-5); `ANALYSIS-E0-CONTRADICTIONS.md` (§2.2, §10); `ALTLASTEN-AUDIT.md` (AP-15, AP-21)  

---

## 1. Zweck & Architekturgarantien

`contextra-simd` ist die hardwarenahe **Unsafe-Insel** für datenparallele Vektoroperationen in Contextra. Es implementiert optimierte Distanz- und Ähnlichkeitskerne für Gleitkomma- (`f32`) und quantisierte Vektoren (`u8`), die von `contextra-vector` (HNSW) und `contextra-rank` genutzt werden.

### Beseitigung des Ebenenverstoßes (R1)
Im Altzustand existierte in `crates/contextra-simd/Cargo.toml` eine optionale Abhängigkeit auf `contextra-types` (Ebene 2) mit den Features `workspace` und `types-compat`. **Dieser Ebenenverstoß wird ersatzlos gestrichen.** `contextra-simd` operiert als reiner E0-Kernel ausschließlich auf primitiven Slices (`&[f32]`, `&[u8]`).

### Unsafe-Insel-Garantien (R5, INV-3)
1. **Unsafe-Begrenzung:** `unsafe` ist **ausschließlich** für SIMD-Intrinsics in `kernels/avx2.rs`, `kernels/avx512.rs` und `kernels/neon.rs` erlaubt.
2. **Dokumentation:** Jeder `unsafe`-Block benötigt zwingend einen `// SAFETY:`-Kommentar.
3. **100 % sichere API:** Alle öffentlichen Einstiegspunkte sind reguläre Safe-Rust-Funktionen (`fn`, nicht `unsafe fn`).
4. **NaN- und Unendlich-Schutz:** Vektoren mit `f32::NAN` oder `f32::INFINITY` werden vor der Berechnung abgewiesen (`SimdError::InvalidFloat`).
5. **Determinismus (INV-5):** Jeder SIMD-Pfad liefert bitidentische Distanzergebnisse zur skalaren IEEE-754 Referenz (`kernels/scalar.rs`).
6. **Leere Default-Features (R3):** `default = []`.

---

## 2. Hardware-Dispatch & Architektur

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

---

### 2.1 Dynamischer Funktions-Dispatch

```rust
use std::sync::atomic::{AtomicPtr, Ordering};

pub type DistanceFnF32 = fn(&[f32], &[f32]) -> Result<f32, SimdError>;
pub type DistanceFnU8 = fn(&[u8], &[u8]) -> Result<u32, SimdError>;

/// Unterstützte Distanzmetriken.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum VectorMetric {
    Cosine,
    EuclideanL2,
    DotProduct,
}

pub struct SimdDispatcher {
    cosine_f32: AtomicPtr<()>,
    l2_f32: AtomicPtr<()>,
    dot_f32: AtomicPtr<()>,
}

impl SimdDispatcher {
    /// Initialisiert den Dispatcher basierend auf CPU-Instruktionssatz-Erkennung.
    pub fn detect() -> Self {
        #[cfg(target_arch = "x86_64")]
        {
            if is_x86_feature_detected!("avx512f") {
                return Self::avx512();
            }
            if is_x86_feature_detected!("avx2") && is_x86_feature_detected!("fma") {
                return Self::avx2();
            }
        }

        #[cfg(target_arch = "aarch64")]
        {
            if std::arch::is_aarch64_feature_detected!("neon") {
                return Self::neon();
            }
        }

        Self::scalar()
    }
}
```

---

### 2.2 Distanzfunktionen & NaN-Validierung

```rust
/// Berechnet die Cosine-Distanz zweier f32-Vektoren (1.0 - Cosine-Similarity).
pub fn cosine_distance(a: &[f32], b: &[f32]) -> Result<f32, SimdError> {
    validate_slices_f32(a, b)?;
    let sim = get_dispatcher().cosine_f32(a, b)?;
    Ok(1.0 - sim)
}

/// Berechnet die quadratische Euklidische Distanz (L2-Squared) zweier f32-Vektoren.
pub fn euclidean_distance_sq(a: &[f32], b: &[f32]) -> Result<f32, SimdError> {
    validate_slices_f32(a, b)?;
    get_dispatcher().l2_f32(a, b)
}

/// Berechnet das Skalarprodukt zweier f32-Vektoren.
pub fn dot_product(a: &[f32], b: &[f32]) -> Result<f32, SimdError> {
    validate_slices_f32(a, b)?;
    get_dispatcher().dot_f32(a, b)
}

/// Schnelle Validierung auf Dimensionsübereinstimmung und ungültige Floats.
#[inline]
pub fn validate_slices_f32(a: &[f32], b: &[f32]) -> Result<(), SimdError> {
    if a.len() != b.len() {
        return Err(SimdError::DimensionMismatch {
            expected: a.len(),
            actual: b.len(),
        });
    }
    if a.is_empty() {
        return Err(SimdError::EmptyVector);
    }
    for &x in a.iter().chain(b.iter()) {
        if x.is_nan() || x.is_infinite() {
            return Err(SimdError::InvalidFloat);
        }
    }
    Ok(())
}
```

---

### 2.3 SQ8 Quantisierungs-Kerne (`u8`)

Für Quantized HNSW-Suche in `contextra-vector`:

```rust
/// Berechnet das Skalarprodukt zweier 8-Bit quantisierter Vektoren.
pub fn dot_product_u8(a: &[u8], b: &[u8]) -> Result<u32, SimdError> {
    if a.len() != b.len() {
        return Err(SimdError::DimensionMismatch {
            expected: a.len(),
            actual: b.len(),
        });
    }
    if a.is_empty() {
        return Err(SimdError::EmptyVector);
    }
    get_dispatcher().dot_u8(a, b)
}
```

---

## 3. Fehlerbehandlung (`SimdError`)

```rust
#[non_exhaustive]
#[derive(Debug, thiserror::Error, PartialEq, Eq, Clone)]
pub enum SimdError {
    #[error("Vector dimensions do not match: expected {expected}, got {actual}")]
    DimensionMismatch { expected: usize, actual: usize },

    #[error("Vector cannot be empty")]
    EmptyVector,

    #[error("Vector contains invalid floating point values (NaN or Infinity)")]
    InvalidFloat,

    #[error("Vector norm is zero, cannot compute cosine similarity")]
    ZeroNorm,

    #[error("Hardware execution failed: {0}")]
    HardwareError(String),
}
```

---

## 4. `Cargo.toml` Spezifikation

```toml
[package]
name = "contextra-simd"
version.workspace = true
edition.workspace = true
license.workspace = true
rust-version.workspace = true
description = "Hardware-accelerated vector distance kernels with SIMD dynamic dispatch and scalar IEEE-754 reference"

[lints]
workspace = true

[features]
default = []
std = []
simd-avx512 = []

[dependencies]
bytemuck = "1.25"
thiserror = { workspace = true }
tracing = { workspace = true, optional = true }

[dev-dependencies]
proptest = "1.4"
criterion = "0.5"
```

---

## 5. Implementierungs-Arbeitspakete für Entwickler

1. **`Cargo.toml`:**
   - Entfernen der Abhängigkeit `contextra-types`.
   - Entfernen der Features `workspace` und `types-compat`.
   - `default = []` setzen (R3).
2. **Kerne-Bereinigung:**
   - In `kernels/avx2.rs`, `kernels/avx512.rs`, `kernels/neon.rs` sicherstellen, dass jeder `unsafe`-Block einen `// SAFETY:`-Kommentar trägt.
3. **Differenzialtests (INV-5):**
   - Proptest-Suite ausführen: Jeder AVX/NEON-Kernel liefert exakt dieselben Werte wie `scalar.rs`.
4. **Verifikation:**
   - `cargo test -p contextra-simd`
