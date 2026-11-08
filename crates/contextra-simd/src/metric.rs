//! Distanzmetrik der SIMD-Kerne (lokal, SPEC v4.0 §3 Prinzip 1).

/// Unterstützte Distanzmetriken.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DistanceMetric {
    /// Kosinus-Distanz (`1.0 - cos(angle)`).
    #[default]
    Cosine,
    /// Euklidische (L2) Distanz.
    Euclidean,
    /// Negiertes Skalarprodukt.
    DotProduct,
}
