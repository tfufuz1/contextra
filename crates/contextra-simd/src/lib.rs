// FILE-CONTEXT
// ZWECK: Layer-0 Ring-0 Unsafe-Insel für SIMD-Distanzkernel und Laufzeit-Dispatch.
// INVARIANTEN: Safe Public API, zero panic, bytemuck/slice alignments.
// HOTSPOTS: lib.rs, kernels/, dispatch.rs

//! # Crate `contextra-simd`
//!
//! **Ring-Zugehörigkeit:** Ring 0 (Kern, unsafe-Insel).
//!
//! **Zweck:**
//! Diese Crate stellt hochoptimierte Vektor-Distanzberechnungen bereit.
//! SIMD (Single Instruction, Multiple Data) nutzt spezielle Prozessorregister (z. B. 128-Bit bei ARM NEON,
//! 256-Bit bei x86 AVX2, 512-Bit bei x86 AVX-512), um mehrere Gleitkomma- oder Ganzzahl-Operationen parallel
//! mit einem einzigen CPU-Befehl auszuführen.
//!
//! **Verbindung im System:**
//! - **Abhängigkeiten (normal):** `contextra-types` (für `ContextraError` und `DistanceMetric`), `bytemuck`, `thiserror`, `tracing`.
//! - **Wird eingebunden von:** `contextra-vector` (Ring 1) zur Beschleunigung von Ähnlichkeitssuchen in Vektorindizes
//!   (z. B. HNSW = Hierarchical Navigable Small World).
//!
//! **Architektur:**
//! 1. `kernels/`: Enthält isolierte Implementierungen für Skalar (Referenz), AVX2, AVX-512 und ARM NEON.
//! 2. `dispatch.rs`: Ermittelt beim Start dynamisch die CPU-Fähigkeiten und wählt den schnellsten sicheren Pfad.
//! 3. `lib.rs`: Bietet die sichere öffentliche Schnittstelle mit Vorab-Validierung gegen ungültige Werte (NaN/Unendlich).

#![allow(unsafe_code)]
#![allow(clippy::missing_safety_doc)]
#![allow(unused_unsafe)]

// ANCHOR[contextra-simd:CORE-MOD-DISPATCH]
// Dynamische CPU-Erkennung und Funktionszeiger-Auswahl.
pub mod dispatch;
// ANCHOR[contextra-simd:CORE-MOD-KERNELS]
// Hardwarespezifische Rechenkerne (Scalar, AVX2, AVX-512, NEON).
pub mod kernels;

pub use dispatch::*;
pub use kernels::scalar::{
    cosine_distance_f32_bytes_scalar, cosine_distance_scalar, cosine_similarity_parts_f32_u8,
    cosine_similarity_parts_u8_scalar, dot_product_f32_bytes_scalar, dot_product_f32_u8,
    dot_product_scalar, dot_product_u8_scalar, euclidean_distance_f32_bytes_scalar,
    euclidean_distance_scalar, euclidean_distance_sq_f32_u8, euclidean_distance_sq_u8_scalar,
    normalize_inplace, CosineSimilarityPartsF32U8, CosineSimilarityPartsU8,
};

use contextra_types::{ContextraError, DistanceMetric};

// ANCHOR[contextra-simd:CORE-VALIDATE-VEC]
/// Prüft, ob ein Vektor gültige endliche Zahlen enthält (kein NaN und kein Unendlich).
///
/// NaN (Not a Number) und Unendlich führen in SIMD-Registern und Metriken zu undefinierten
/// oder korrupten Ergebnissen. Diese Prüfung verhindert solche Fehler im Vorfeld (Fail-Closed).
///
/// # Ablauf von `validate_vector`:
/// 1. Iteration über alle Elemente der `f32`-Slice.
/// 2. Prüfung mittels `v.is_finite()`.
/// 3. Abbruch mit Fehler (`ContextraError::invalid_input`), falls mindestens ein Wert ungültig ist.
/// 4. Rückgabe von `Ok(())`, wenn alle Werte endlich sind.
///
/// # Fehlerfälle:
/// - Gibt `Err(ContextraError::InvalidInput)` zurück, wenn ein Wert `f32::NAN`, `f32::INFINITY` oder `f32::NEG_INFINITY` ist.
///
/// # Nebenwirkungen:
/// Keine (reine Speicherprüfung im RAM).
///
/// # Beispiel:
/// ```
/// use contextra_simd::validate_vector;
///
/// let valid = [1.0, 2.0, 3.0];
/// assert!(validate_vector(&valid).is_ok());
///
/// let invalid = [1.0, f32::NAN, 3.0];
/// assert!(validate_vector(&invalid).is_err());
/// ```
#[inline]
pub fn validate_vector(vec: &[f32]) -> contextra_types::Result<()> {
    if vec.iter().any(|v| !v.is_finite()) {
        return Err(ContextraError::invalid_input(
            "NaN or Infinity detected in vector",
        ));
    }
    Ok(())
}

// ANCHOR[contextra-simd:CORE-COMPUTE-DIST]
/// Berechnet die Distanz zwischen zwei Vektoren unter Verwendung der angegebenen Metrik inklusive Validierung.
///
/// Diese Funktion ist der primäre Einstiegspunkt für sichere Berechnungen von ungesicherten Eingaben.
///
/// # Ablauf von `compute_distance`:
/// 1. Vektor `a` auf Nicht-Endlichkeit (NaN / Unendlich) validieren.
/// 2. Vektor `b` auf Nicht-Endlichkeit (NaN / Unendlich) validieren.
/// 3. An [`compute_distance_trusted`] delegieren.
///
/// # Fehlerfälle:
/// - Gibt `Err(ContextraError::InvalidInput)` zurück, wenn Vektor `a` oder `b` NaN/Unendlich enthält.
/// - Gibt `Err(ContextraError::InvalidInput)` zurück, wenn die Vektorlängen ungleich sind.
/// - Gibt `Err(ContextraError::InvalidInput)` zurück, wenn eine unbekannte oder nicht unterstützte Metrik übergeben wurde.
///
/// # Nebenwirkungen:
/// Keine.
///
/// # Beispiel:
/// ```
/// use contextra_simd::compute_distance;
/// use contextra_types::DistanceMetric;
///
/// let a = [1.0, 0.0, 0.0];
/// let b = [0.0, 1.0, 0.0];
/// let dist = compute_distance(&a, &b, DistanceMetric::Cosine)?;
/// # Ok::<(), contextra_types::ContextraError>(())
/// ```
#[inline]
pub fn compute_distance(
    a: &[f32],
    b: &[f32],
    metric: DistanceMetric,
) -> contextra_types::Result<f32> {
    validate_vector(a)?;
    validate_vector(b)?;
    compute_distance_trusted(a, b, metric)
}

// ANCHOR[contextra-simd:CORE-COMPUTE-DIST-TRUSTED]
/// Berechnet die Distanz zwischen zwei bereits validierten `f32`-Vektoren ohne erneute NaN-Prüfung.
///
/// Wird intern verwendet, wenn Vektoren bereits bei der Aufnahme in den Speicher validiert wurden,
/// um bei tausenden Vergleichen pro Sekunde CPU-Zyklen einzusparen.
///
/// # Ablauf von `compute_distance_trusted`:
/// 1. Fallunterscheidung anhand der `DistanceMetric`.
/// 2. Aufruf der passenden SIMD-Dispatch-Funktion (`cosine_distance`, `euclidean_distance`, `dot_product_distance`).
/// 3. Längenabgleich und Ausführung im gewählten Hardware-Pfad.
///
/// # Fehlerfälle:
/// - Gibt `Err(ContextraError::InvalidInput)` zurück, falls Längen abweichen oder eine nicht unterstützte Metrik gefordert wird.
///
/// # Nebenwirkungen:
/// Keine.
#[inline]
pub fn compute_distance_trusted(
    a: &[f32],
    b: &[f32],
    metric: DistanceMetric,
) -> contextra_types::Result<f32> {
    match metric {
        DistanceMetric::Cosine => cosine_distance(a, b),
        DistanceMetric::Euclidean => euclidean_distance(a, b),
        DistanceMetric::DotProduct => dot_product_distance(a, b),
        _ => Err(ContextraError::invalid_input("Unsupported distance metric")),
    }
}

// ANCHOR[contextra-simd:CORE-COMPUTE-DIST-BYTES-TRUSTED]
/// Berechnet die Distanz zwischen einem `f32`-Vektor und einem rohen Byte-Puffer (`&[u8]`), der `f32`-Werte darstellt.
///
/// Dies ist besonders nützlich, wenn Vektoren aus dem Speicher oder Disk-Cache (z. B. LSM-Tree oder Memory-Mapped Files)
/// als Byte-Arrays vorliegen, um Kopier- und Allokationsaufwand zu vermeiden (Zero-Copy).
///
/// # Ablauf von `compute_distance_f32_bytes_trusted`:
/// 1. Fallunterscheidung nach `DistanceMetric`.
/// 2. Aufruf der byte-kompatiblen Dispatch-Funktion (`cosine_distance_f32_bytes`, etc.).
/// 3. Konvertierung/Reinterpretation der Bytes als `f32` im Ziel-Kernel.
///
/// # Fehlerfälle:
/// - Gibt `Err(ContextraError::InvalidInput)` zurück, wenn die Byte-Länge kein Vielfaches von 4 (`sizeof(f32)`) ist
///   oder die Dimensionen nicht übereinstimmen.
/// - Gibt `Err(ContextraError::InvalidInput)` bei unbekannter Metrik zurück.
///
/// # Nebenwirkungen:
/// Keine.
#[inline]
pub fn compute_distance_f32_bytes_trusted(
    a: &[f32],
    b_bytes: &[u8],
    metric: DistanceMetric,
) -> contextra_types::Result<f32> {
    match metric {
        DistanceMetric::Cosine => cosine_distance_f32_bytes(a, b_bytes),
        DistanceMetric::Euclidean => euclidean_distance_f32_bytes(a, b_bytes),
        DistanceMetric::DotProduct => dot_product_distance_f32_bytes(a, b_bytes),
        _ => Err(ContextraError::invalid_input("Unsupported distance metric")),
    }
}
