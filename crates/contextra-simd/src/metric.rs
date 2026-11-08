//! Lokales Metrik-Enum für `contextra-simd` (B1/B3 Autarkie).

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum DistanceMetric {
    Cosine,
    Euclidean,
    DotProduct,
}

#[cfg(feature = "types-compat")]
impl From<DistanceMetric> for contextra_types::DistanceMetric {
    fn from(m: DistanceMetric) -> Self {
        match m {
            DistanceMetric::Cosine => contextra_types::DistanceMetric::Cosine,
            DistanceMetric::Euclidean => contextra_types::DistanceMetric::Euclidean,
            DistanceMetric::DotProduct => contextra_types::DistanceMetric::DotProduct,
        }
    }
}

#[cfg(feature = "types-compat")]
impl From<contextra_types::DistanceMetric> for DistanceMetric {
    fn from(m: contextra_types::DistanceMetric) -> Self {
        match m {
            contextra_types::DistanceMetric::Cosine => DistanceMetric::Cosine,
            contextra_types::DistanceMetric::Euclidean => DistanceMetric::Euclidean,
            contextra_types::DistanceMetric::DotProduct => DistanceMetric::DotProduct,
            _ => DistanceMetric::Cosine,
        }
    }
}
