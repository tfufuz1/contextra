use contextra_core::{ContextraError, DistanceMetric, Result};

use super::types::HNSW_REBUILD_DELETION_RATIO;

#[derive(Debug, Clone)]
pub struct HnswConfig {
    /// Vector dimensionality.
    pub dimension: usize,
    /// Maximum number of elements.
    pub max_elements: usize,
    /// Number of connections per element (M parameter).
    pub m: usize,
    /// Dynamic candidate list size during graph construction (`ef_construction`).
    pub ef_construction: usize,
    /// Dynamic candidate list size during search.
    pub ef_search: usize,
    /// Distance metric.
    pub distance_metric: DistanceMetric,
    /// Rebuild threshold.
    pub rebuild_threshold: f64,
    /// Whether to apply SQ8 Scalar Quantization to the index vectors to reduce RAM.
    pub quantize: bool,
    /// Sample size used for ScalarQuantizer recalibration during rebuilds.
    pub quantizer_recalibration_sample_size: usize,
    /// Quantizer drift ratio threshold above which an index rebuild is recommended.
    pub quantizer_drift_threshold: f32,
    /// Optional compute pool for background index rebuilds.
    pub compute_pool: Option<crate::compute_pool::ComputePool>,
    /// Partial rebuild configuration for hot-path local rebuilds (F-02).
    #[cfg(feature = "partial-index-rebuild")]
    pub partial_rebuild_config: crate::partial_rebuild::PartialRebuildConfig,
}

impl Default for HnswConfig {
    fn default() -> Self {
        Self {
            dimension: 1536,
            max_elements: 1_000_000,
            m: 16,
            ef_construction: 200,
            ef_search: 64,
            distance_metric: DistanceMetric::Cosine,
            rebuild_threshold: 1.0 - HNSW_REBUILD_DELETION_RATIO,
            quantize: false,
            quantizer_recalibration_sample_size: 10_000,
            quantizer_drift_threshold: 0.10,
            compute_pool: None,
            #[cfg(feature = "partial-index-rebuild")]
            partial_rebuild_config: crate::partial_rebuild::PartialRebuildConfig::default(),
        }
    }
}

/// Validates that a vector is non-empty and contains no NaN or Infinite values.
pub(super) fn validate_vector(vec: &[f32]) -> Result<()> {
    contextra_simd::validate_vector(vec)
}

impl HnswConfig {
    /// Validates that the configuration parameters are within acceptable bounds.
    pub fn validate(&self) -> Result<()> {
        if self.dimension == 0 {
            return Err(ContextraError::invalid_input(
                "dimension must be greater than 0",
            ));
        }
        if self.m == 0 {
            return Err(ContextraError::invalid_input("m must be greater than 0"));
        }
        if self.ef_search == 0 {
            return Err(ContextraError::invalid_input(
                "ef_search must be greater than 0",
            ));
        }
        if self.ef_construction < self.m {
            return Err(ContextraError::invalid_input(format!(
                "ef_construction ({}) must be >= m ({})",
                self.ef_construction, self.m
            )));
        }
        if !(0.0..=1.0).contains(&self.rebuild_threshold) {
            return Err(ContextraError::invalid_input(format!(
                "rebuild_threshold ({}) must be between 0.0 and 1.0",
                self.rebuild_threshold
            )));
        }
        if !(0.0..=1.0).contains(&self.quantizer_drift_threshold) {
            return Err(ContextraError::invalid_input(format!(
                "quantizer_drift_threshold ({}) must be between 0.0 and 1.0",
                self.quantizer_drift_threshold
            )));
        }
        Ok(())
    }
}

/// Builder for HnswConfig with resource limit enforcements to prevent OOM.
#[derive(Debug, Clone)]
pub struct HnswConfigBuilder {
    config: HnswConfig,
}

impl HnswConfigBuilder {
    /// Creates a new builder with the chosen dimensionality.
    pub fn new(dimension: usize) -> Self {
        Self {
            config: HnswConfig {
                dimension,
                ..Default::default()
            },
        }
    }

    /// Set max elements with a hardcap limit to avoid OOM.
    pub fn max_elements(mut self, max: usize) -> Self {
        self.config.max_elements = max.min(50_000_000);
        self
    }

    /// Set the number of connections per element (M).
    pub fn m(mut self, m: usize) -> Self {
        self.config.m = m.clamp(4, 256);
        self
    }

    /// Set dynamic candidate list size for construction.
    pub fn ef_construction(mut self, ef: usize) -> Self {
        self.config.ef_construction = ef.min(4000);
        self
    }

    /// Set dynamic candidate list size for search.
    pub fn ef_search(mut self, ef: usize) -> Self {
        self.config.ef_search = ef.min(4000);
        self
    }

    /// Use a specific distance metric.
    pub fn distance_metric(mut self, metric: DistanceMetric) -> Self {
        self.config.distance_metric = metric;
        self
    }

    /// Enable or disable scalar quantization (SQ8) to reduce footprint.
    pub fn quantize(mut self, quantize: bool) -> Self {
        self.config.quantize = quantize;
        self
    }

    /// Sets the sample size used for ScalarQuantizer recalibration during rebuilds.
    pub fn quantizer_recalibration_sample_size(mut self, size: usize) -> Self {
        self.config.quantizer_recalibration_sample_size = size;
        self
    }

    /// Sets the drift ratio threshold for ScalarQuantizer recalibration and rebuilds.
    pub fn quantizer_drift_threshold(mut self, threshold: f32) -> Self {
        self.config.quantizer_drift_threshold = threshold.clamp(0.0, 1.0);
        self
    }

    /// Sets the rebuild threshold.
    pub fn rebuild_threshold(mut self, threshold: f64) -> Self {
        self.config.rebuild_threshold = threshold.clamp(0.0, 1.0);
        self
    }

    /// Sets the partial rebuild configuration for local hot-path rebuilds (F-02).
    #[cfg(feature = "partial-index-rebuild")]
    pub fn partial_rebuild_config(
        mut self,
        config: crate::partial_rebuild::PartialRebuildConfig,
    ) -> Self {
        self.config.partial_rebuild_config = config;
        self
    }

    /// Build the configuration after validating bounds.
    pub fn build(self) -> Result<HnswConfig> {
        self.config.validate()?;
        Ok(self.config)
    }
}
