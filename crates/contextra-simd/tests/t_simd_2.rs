// SPEC: T-SIMD-2
// ADR-002 / SPEC §27: Test assertions and test dataset casts
#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]
#![allow(clippy::panic)]
#![allow(clippy::indexing_slicing)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_sign_loss)]

//! # Nachweis T-SIMD-2 für `contextra-simd`
//!
//! **Zweck (SPEC D-122 5b, T-SIMD-2):**
//! Kanonische Distanz: 16 Teilsummen (Lane = Index mod 16), kein FMA,
//! feste paarweise Reduktion 16 -> 8 -> 4 -> 2 -> 1.
//! Die Skalar-Referenz implementiert dieselbe Reihenfolge, alle Kernel sind bitgleich
//! (`f32::to_bits`).

use contextra_simd::{
    cosine_distance, cosine_distance_f32_bytes, dot_product_distance,
    dot_product_distance_f32_bytes, euclidean_distance, euclidean_distance_f32_bytes,
    set_dispatch_override, SimdPathOverride,
};

struct SimpleRng {
    state: u64,
}

impl SimpleRng {
    fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    fn next_u32(&mut self) -> u32 {
        self.state = self.state.wrapping_mul(6364136223846793005).wrapping_add(1);
        (self.state >> 32) as u32
    }

    fn next_f32(&mut self) -> f32 {
        let u = self.next_u32();
        // Scale to [-100.0, 100.0]
        ((u as f64 / u32::MAX as f64) * 200.0 - 100.0) as f32
    }

    fn gen_f32_vec(&mut self, len: usize) -> Vec<f32> {
        (0..len).map(|_| self.next_f32()).collect()
    }
}

/// SPEC: T-SIMD-2
#[test]
fn t_simd_2_bit_identical_across_simd_and_scalar() {
    let mut rng = SimpleRng::new(0x1234_5678_9ABC_DEF0);
    let test_dims = [
        0, 1, 2, 3, 4, 7, 8, 15, 16, 17, 31, 32, 33, 63, 64, 65, 127, 128, 129, 255, 256, 257, 512,
        768, 1024, 1536, 2048, 4096,
    ];

    for &dim in &test_dims {
        let a = rng.gen_f32_vec(dim);
        let b = rng.gen_f32_vec(dim);

        let mut b_bytes = Vec::with_capacity(dim * 4);
        for &val in &b {
            b_bytes.extend_from_slice(&val.to_le_bytes());
        }

        // 1. Force Scalar reference
        set_dispatch_override(Some(SimdPathOverride::Scalar));
        let scalar_dot = dot_product_distance(&a, &b).unwrap();
        let scalar_euc = euclidean_distance(&a, &b).unwrap();
        let scalar_cos = cosine_distance(&a, &b).unwrap();
        let scalar_dot_b = dot_product_distance_f32_bytes(&a, &b_bytes).unwrap();
        let scalar_euc_b = euclidean_distance_f32_bytes(&a, &b_bytes).unwrap();
        let scalar_cos_b = cosine_distance_f32_bytes(&a, &b_bytes).unwrap();

        // 2. Clear override (uses active hardware SIMD path if available)
        set_dispatch_override(None);
        let simd_dot = dot_product_distance(&a, &b).unwrap();
        let simd_euc = euclidean_distance(&a, &b).unwrap();
        let simd_cos = cosine_distance(&a, &b).unwrap();
        let simd_dot_b = dot_product_distance_f32_bytes(&a, &b_bytes).unwrap();
        let simd_euc_b = euclidean_distance_f32_bytes(&a, &b_bytes).unwrap();
        let simd_cos_b = cosine_distance_f32_bytes(&a, &b_bytes).unwrap();

        // 3. T-SIMD-2 bit-exact check
        assert_eq!(
            scalar_dot.to_bits(),
            simd_dot.to_bits(),
            "Dot product bit mismatch at dim {dim}: scalar={scalar_dot} ({:#x}), simd={simd_dot} ({:#x})",
            scalar_dot.to_bits(),
            simd_dot.to_bits()
        );
        assert_eq!(
            scalar_euc.to_bits(),
            simd_euc.to_bits(),
            "Euclidean distance bit mismatch at dim {dim}: scalar={scalar_euc} ({:#x}), simd={simd_euc} ({:#x})",
            scalar_euc.to_bits(),
            simd_euc.to_bits()
        );
        assert_eq!(
            scalar_cos.to_bits(),
            simd_cos.to_bits(),
            "Cosine distance bit mismatch at dim {dim}: scalar={scalar_cos} ({:#x}), simd={simd_cos} ({:#x})",
            scalar_cos.to_bits(),
            simd_cos.to_bits()
        );

        assert_eq!(
            scalar_dot_b.to_bits(),
            simd_dot_b.to_bits(),
            "Dot product bytes bit mismatch at dim {dim}"
        );
        assert_eq!(
            scalar_euc_b.to_bits(),
            simd_euc_b.to_bits(),
            "Euclidean bytes bit mismatch at dim {dim}"
        );
        assert_eq!(
            scalar_cos_b.to_bits(),
            simd_cos_b.to_bits(),
            "Cosine bytes bit mismatch at dim {dim}"
        );
    }
}
