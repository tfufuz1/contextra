//! Lokales Metrik-Enum für `contextra-simd` (B1/B3 Autarkie).
//!
//! Dieses Modul definiert die mathematischen Distanzmetriken, die von den SIMD-Kernen
//! unterstützt werden. Es ist vollständig frei von internen Abhängigkeiten.

// SECTION METRIC-ENUM
// ANCHOR[contextra-simd:METRIC-ENUM]
/// Unterstützte Distanzmetriken für Vektorberechnungen.
///
/// Definiert als geschlossenes Wert-Enum (B09-Entscheidung).
/// Besitzt bitidentische Serialisierung zum Workspace-Typ `DistanceMetric`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum DistanceMetric {
    /// Kosinus-Distanz ($1.0 - \text{Kosinus-Ähnlichkeit}$). Wertebereich: $[0.0, 2.0]$.
    Cosine,
    /// Euklidische Distanz ($\|a - b\|_2 = \sqrt{\sum (a_i - b_i)^2}$). Wertebereich: $[0.0, \infty)$.
    Euclidean,
    /// Skalarprodukt-Distanz (negierter Dot-Product $-a \cdot b$). Höhere Ähnlichkeit = kleinere Distanz.
    DotProduct,
}
// !SECTION METRIC-ENUM
