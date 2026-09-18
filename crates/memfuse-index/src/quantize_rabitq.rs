// FILE-CONTEXT
// ZWECK: RaBitQ-Quantisierer (Binär-Quantisierung mit orthogonaler Rotation und Fehlerkorrektur).
// INVARIANTEN: Zero-Panic (keine .unwrap()/.expect() in Produktionscode), NaN-Scan beim Input.
// NICHT-OFFENSICHTLICH: Orthogonale Rotation R R^T = I per Gram-Schmidt erlaubt isometry-erhaltende Binarisierung.
// HOTSPOTS: quantize_rabitq.rs (RaBitQQuantizer::try_train, quantize, asymmetric_distance)

//! RaBitQ Quantizer behind feature flag `experimental-rabitq`.
//!
//! RaBitQ applies a random orthogonal rotation to high-dimensional vectors,
//! binarizes the rotated components into 1-bit codes, and stores scalar metadata
//! (norm and average absolute magnitude) for asymmetric distance estimation.
//!
//! # Mathematical Derivation
//! Given an input vector $x \in \mathbb{R}^D$ and a random orthogonal rotation matrix $R \in \mathbb{R}^{D \times D}$ ($R R^T = I$):
//! 1. Rotated vector: $x' = R x$, with norm $\|x'\| = \|x\|$.
//! 2. Sign binarization: $b_i = \mathbb{I}(x'_i \ge 0) \in \{0, 1\}$.
//! 3. Reconstructed rotated vector estimate: $\hat{x}'_i = \bar{s} \cdot (2 b_i - 1)$, where $\bar{s} = \frac{1}{D} \sum_{i=0}^{D-1} |x'_i|$.
//! 4. Asymmetric inner product estimation for query $q$ (rotated $q' = R q$):
//!    $$\langle q, x \rangle = \langle q', x' \rangle \approx \bar{s} \sum_{i=0}^{D-1} q'_i (2 b_i - 1)$$
//! 5. Asymmetric Euclidean distance estimation:
//!    $$d(q, x) = \sqrt{\max\left(0, \|q\|^2 + \|x\|^2 - 2 \langle q', \hat{x}' \rangle\right)}$$

use memfuse_core::{MemFuseError, Result};
use serde::{Deserialize, Serialize};

/// RaBitQ Quantizer with random orthogonal rotation and scalar metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RaBitQQuantizer {
    dimension: usize,
    /// Row-major orthogonal rotation matrix R of size dimension x dimension.
    rotation_matrix: Vec<f32>,
}

impl RaBitQQuantizer {
    /// Safely trains a `RaBitQQuantizer` for vectors of the specified `dimension`.
    ///
    /// Generates a deterministic orthogonal matrix $R \in \mathbb{R}^{D \times D}$ using
    /// Gram-Schmidt orthogonalization on a seeded Gaussian matrix.
    ///
    /// Returns `MemFuseError::InvalidInput` if `dimension` is 0 or if any vector in `vectors`
    /// contains NaN / infinite values or has a dimension mismatch.
    pub fn try_train(vectors: &[Vec<f32>], dimension: usize) -> Result<Self> {
        if dimension == 0 {
            return Err(MemFuseError::invalid_input(
                "RaBitQQuantizer dimension must be greater than 0",
            ));
        }

        for (idx, vec) in vectors.iter().enumerate() {
            if vec.len() != dimension {
                return Err(MemFuseError::invalid_input(format!(
                    "Vector at index {idx} has dimension {}, expected {dimension}",
                    vec.len()
                )));
            }
            for &val in vec {
                if !val.is_finite() {
                    return Err(MemFuseError::invalid_input(format!(
                        "Vector at index {idx} contains non-finite value: {val}"
                    )));
                }
            }
        }

        let rotation_matrix = generate_orthogonal_matrix(dimension)?;

        Ok(Self {
            dimension,
            rotation_matrix,
        })
    }

    /// Returns the vector dimension supported by this quantizer.
    pub fn dimension(&self) -> usize {
        self.dimension
    }

    /// Returns the expected byte length of quantized codes (`dimension.div_ceil(8) + 8` bytes).
    pub fn code_length(&self) -> usize {
        self.dimension.div_ceil(8) + 8
    }

    /// Quantizes a single $f32$ vector into a RaBitQ packed bitcode buffer.
    ///
    /// The returned code consists of:
    /// - $\lceil D / 8 \rceil$ bytes of packed sign bits.
    /// - 4 bytes: $\|x\|^2$ ($f32$ little-endian).
    /// - 4 bytes: $\bar{s}$ ($f32$ little-endian).
    pub fn quantize(&self, vector: &[f32]) -> Result<Vec<u8>> {
        if vector.len() != self.dimension {
            return Err(MemFuseError::invalid_input(format!(
                "Vector dimension mismatch: expected {}, got {}",
                self.dimension,
                vector.len()
            )));
        }

        for &val in vector {
            if !val.is_finite() {
                return Err(MemFuseError::invalid_input(format!(
                    "Cannot quantize vector containing non-finite value: {val}"
                )));
            }
        }

        let bit_bytes = self.dimension.div_ceil(8);
        let mut code = vec![0u8; bit_bytes + 8];

        // 1. Compute squared norm of original vector
        let norm_sq: f32 = vector.iter().map(|&x| x * x).sum();

        // 2. Rotate vector x' = R * x
        let mut sum_abs = 0.0f32;

        for i in 0..self.dimension {
            let row_offset = i * self.dimension;
            let row = &self.rotation_matrix[row_offset..row_offset + self.dimension];
            let mut val = 0.0f32;
            for (j, &v) in vector.iter().enumerate().take(self.dimension) {
                val += row[j] * v;
            }
            sum_abs += val.abs();

            if val >= 0.0 {
                let byte_idx = i / 8;
                let bit_idx = i % 8;
                code[byte_idx] |= 1 << bit_idx;
            }
        }

        let bar_s = sum_abs / (self.dimension as f32);

        // Append metadata (norm_sq and bar_s) as little-endian bytes
        let norm_bytes = norm_sq.to_le_bytes();
        let bar_s_bytes = bar_s.to_le_bytes();

        code[bit_bytes..bit_bytes + 4].copy_from_slice(&norm_bytes);
        code[bit_bytes + 4..bit_bytes + 8].copy_from_slice(&bar_s_bytes);

        Ok(code)
    }

    /// Computes the asymmetric Euclidean distance between an unquantized query vector and a quantized bitcode.
    pub fn asymmetric_distance(&self, query: &[f32], code: &[u8]) -> Result<f32> {
        if query.len() != self.dimension {
            return Err(MemFuseError::invalid_input(format!(
                "Query dimension mismatch: expected {}, got {}",
                self.dimension,
                query.len()
            )));
        }

        for &val in query {
            if !val.is_finite() {
                return Err(MemFuseError::invalid_input(format!(
                    "Query vector contains non-finite value: {val}"
                )));
            }
        }

        let expected_code_len = self.code_length();
        if code.len() != expected_code_len {
            return Err(MemFuseError::invalid_input(format!(
                "Quantized code length mismatch: expected {expected_code_len} bytes, got {}",
                code.len()
            )));
        }

        let bit_bytes = self.dimension.div_ceil(8);

        let norm_x_sq = f32::from_le_bytes([
            code[bit_bytes],
            code[bit_bytes + 1],
            code[bit_bytes + 2],
            code[bit_bytes + 3],
        ]);

        let bar_s = f32::from_le_bytes([
            code[bit_bytes + 4],
            code[bit_bytes + 5],
            code[bit_bytes + 6],
            code[bit_bytes + 7],
        ]);

        let norm_q_sq: f32 = query.iter().map(|&q| q * q).sum();

        // Rotate query q' = R * q and estimate inner product <q', x'>
        let mut sum_q_rot_sign = 0.0f32;

        for i in 0..self.dimension {
            let row_offset = i * self.dimension;
            let row = &self.rotation_matrix[row_offset..row_offset + self.dimension];
            let mut q_rot_i = 0.0f32;
            for j in 0..self.dimension {
                q_rot_i += row[j] * query[j];
            }

            let byte_idx = i / 8;
            let bit_idx = i % 8;
            let bit = (code[byte_idx] >> bit_idx) & 1;
            let sign = if bit == 1 { 1.0f32 } else { -1.0f32 };

            sum_q_rot_sign += q_rot_i * sign;
        }

        let estimated_dot = bar_s * sum_q_rot_sign;

        let dist_sq = norm_q_sq + norm_x_sq - 2.0 * estimated_dot;
        Ok(dist_sq.max(0.0).sqrt())
    }
}

/// Generates a $D \times D$ orthogonal rotation matrix using a Gram-Schmidt process.
fn generate_orthogonal_matrix(dim: usize) -> Result<Vec<f32>> {
    let mut mat = vec![0.0f32; dim * dim];

    // Simple deterministic pseudo-random Gaussian initialization (Box-Muller transform)
    let mut seed = 0x0A_B1_0C_u64.wrapping_add(dim as u64);
    let mut next_u32 = || {
        seed = seed
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        (seed >> 32) as u32
    };

    let mut next_f32 = || -> f32 {
        let u = next_u32();
        ((u >> 8) as f32 + 1.0) / 16777216.0
    };

    for i in 0..dim {
        for j in (0..dim).step_by(2) {
            let u1: f32 = next_f32().clamp(1e-7, 1.0);
            let u2: f32 = next_f32();
            let log_u1: f32 = u1.ln();
            let r: f32 = (-2.0 * log_u1).sqrt();
            let z0 = r * (2.0 * std::f32::consts::PI * u2).cos();
            let z1 = r * (2.0 * std::f32::consts::PI * u2).sin();

            mat[i * dim + j] = z0;
            if j + 1 < dim {
                mat[i * dim + j + 1] = z1;
            }
        }
    }

    // Gram-Schmidt orthogonalization
    let mut ortho = vec![0.0f32; dim * dim];

    for i in 0..dim {
        let mut v: Vec<f32> = mat[i * dim..(i + 1) * dim].to_vec();

        for j in 0..i {
            let u_j = &ortho[j * dim..(j + 1) * dim];
            let dot: f32 = v.iter().zip(u_j.iter()).map(|(&a, &b)| a * b).sum();
            for k in 0..dim {
                v[k] -= dot * u_j[k];
            }
        }

        let mut norm: f32 = v.iter().map(|&x| x * x).sum::<f32>().sqrt();

        if norm < 1e-12 {
            // Fallback: use canonical basis vector if degenerate
            v.fill(0.0);
            v[i] = 1.0;
            for j in 0..i {
                let u_j = &ortho[j * dim..(j + 1) * dim];
                let dot: f32 = v.iter().zip(u_j.iter()).map(|(&a, &b)| a * b).sum();
                for k in 0..dim {
                    v[k] -= dot * u_j[k];
                }
            }
            norm = v.iter().map(|&x| x * x).sum::<f32>().sqrt();
            if norm < 1e-12 {
                return Err(MemFuseError::Internal(
                    "Gram-Schmidt matrix orthogonalization failed to normalize row".to_string(),
                ));
            }
        }

        for k in 0..dim {
            ortho[i * dim + k] = v[k] / norm;
        }
    }

    Ok(ortho)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rabitq_recall_synthetic() -> Result<()> {
        let dim = 128;
        let num_vecs = 1000;
        let top_k = 10;

        let mut seed = 12345u64;
        let mut next_f32 = || -> f32 {
            seed = seed
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            ((seed >> 32) as f32 / 4294967296.0) * 2.0 - 1.0
        };

        // Generate 100 clusters of 10 vectors each (total N=1000)
        let num_clusters = 100;
        let cluster_size = 10;
        let mut cluster_centers = Vec::with_capacity(num_clusters);
        for _ in 0..num_clusters {
            let center: Vec<f32> = (0..dim).map(|_| next_f32() * 10.0).collect();
            cluster_centers.push(center);
        }

        let mut vectors = Vec::with_capacity(num_vecs);
        for i in 0..num_vecs {
            let center = &cluster_centers[i / cluster_size];
            let vec: Vec<f32> = center.iter().map(|&c| c + next_f32() * 0.05).collect();
            vectors.push(vec);
        }

        let quantizer = RaBitQQuantizer::try_train(&vectors, dim)?;
        let codes: Vec<Vec<u8>> = vectors
            .iter()
            .map(|v| quantizer.quantize(v))
            .collect::<Result<Vec<_>>>()?;

        let num_queries = 20;
        let mut total_hits = 0;

        for q_idx in 0..num_queries {
            let center = &cluster_centers[q_idx % num_clusters];
            let query: Vec<f32> = center.iter().map(|&c| c + next_f32() * 0.05).collect();

            // Ground truth brute-force Euclidean distance
            let mut gt_distances: Vec<(usize, f32)> = vectors
                .iter()
                .enumerate()
                .map(|(idx, v)| {
                    let dist = query
                        .iter()
                        .zip(v.iter())
                        .map(|(&a, &b)| (a - b) * (a - b))
                        .sum::<f32>()
                        .sqrt();
                    (idx, dist)
                })
                .collect();

            gt_distances.select_nth_unstable_by(top_k - 1, |a, b| {
                a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal)
            });
            let gt_top_k: std::collections::HashSet<usize> =
                gt_distances[..top_k].iter().map(|&(idx, _)| idx).collect();

            // RaBitQ asymmetric distance estimation
            let mut rabitq_distances: Vec<(usize, f32)> = codes
                .iter()
                .enumerate()
                .map(|(idx, code)| {
                    let dist = quantizer.asymmetric_distance(&query, code)?;
                    Ok((idx, dist))
                })
                .collect::<Result<Vec<_>>>()?;

            rabitq_distances.select_nth_unstable_by(top_k - 1, |a, b| {
                a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal)
            });
            let rabitq_top_k: std::collections::HashSet<usize> = rabitq_distances[..top_k]
                .iter()
                .map(|&(idx, _)| idx)
                .collect();

            let hits = gt_top_k.intersection(&rabitq_top_k).count();
            total_hits += hits;
        }

        let recall = (total_hits as f32) / ((num_queries * top_k) as f32);
        assert!(
            recall >= 0.85,
            "RaBitQ Recall@10 must be >= 0.85, got: {recall:.4}"
        );

        Ok(())
    }

    #[test]
    fn test_nan_safety_guard() {
        let dim = 4;
        let nan_vec = vec![1.0, f32::NAN, 0.0, 0.5];
        let valid_vec = vec![1.0, 0.0, 0.0, 0.5];

        // NaN during training
        let res_train = RaBitQQuantizer::try_train(&[nan_vec.clone()], dim);
        assert!(matches!(res_train, Err(MemFuseError::InvalidInput(_))));

        let quantizer = RaBitQQuantizer::try_train(&[valid_vec.clone()], dim).expect("valid train");

        // NaN during quantization
        let res_quant = quantizer.quantize(&nan_vec);
        assert!(matches!(res_quant, Err(MemFuseError::InvalidInput(_))));

        let valid_code = quantizer.quantize(&valid_vec).expect("valid code");

        // NaN during asymmetric distance computation
        let res_asym = quantizer.asymmetric_distance(&nan_vec, &valid_code);
        assert!(matches!(res_asym, Err(MemFuseError::InvalidInput(_))));
    }

    #[test]
    fn test_dimension_mismatch_and_code_length_guard() -> Result<()> {
        let dim = 8;
        let vec8 = vec![1.0; 8];
        let vec4 = vec![1.0; 4];

        let quantizer = RaBitQQuantizer::try_train(&[vec8.clone()], dim)?;

        // Training mismatch
        assert!(RaBitQQuantizer::try_train(&[vec4.clone()], dim).is_err());

        // Quantize mismatch
        assert!(quantizer.quantize(&vec4).is_err());

        let code = quantizer.quantize(&vec8)?;

        // Query dimension mismatch
        assert!(quantizer.asymmetric_distance(&vec4, &code).is_err());

        // Corrupted code length
        let mut short_code = code.clone();
        short_code.pop();
        assert!(quantizer.asymmetric_distance(&vec8, &short_code).is_err());

        Ok(())
    }
}
