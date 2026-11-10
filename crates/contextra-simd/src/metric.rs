//! Lokales Metrik-Enum für `contextra-simd` (B1/B3 Autarkie).

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum DistanceMetric {
    Cosine,
    Euclidean,
    DotProduct,
}
