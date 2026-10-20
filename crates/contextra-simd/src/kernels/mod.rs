// FILE-CONTEXT
// ZWECK: Kernel-Modul-Deklarationen für SIMD-Distanzfunktionen.

pub(crate) mod avx2;
pub(crate) mod avx512;
pub(crate) mod neon;
pub mod scalar;

pub use scalar::*;

#[cfg(test)]
#[allow(unsafe_code)]
mod direct_kernel_differential_tests {
    use super::*;

    #[test]
    fn test_direct_kernel_differential_against_scalar_all_lengths() {
        let lengths = [
            0, 1, 2, 3, 4, 5, 7, 8, 9, 15, 16, 17, 31, 32, 33, 63, 64, 65, 127, 255, 512,
        ];

        for &len in &lengths {
            let a: Vec<f32> = (0..len).map(|i| ((i as f32 * 0.37).sin() * 10.0)).collect();
            let b: Vec<f32> = (0..len).map(|i| ((i as f32 * 0.53).cos() * 10.0)).collect();
            let b_bytes: Vec<u8> = b.iter().flat_map(|x| x.to_le_bytes()).collect();

            let a_u8: Vec<u8> = (0..len).map(|i| ((i * 37) % 256) as u8).collect();
            let b_u8: Vec<u8> = (0..len).map(|i| ((i * 53) % 256) as u8).collect();

            // Scalar Reference Values
            let sc_cos = scalar::cosine_distance_scalar(&a, &b);
            let sc_euc = scalar::euclidean_distance_scalar(&a, &b);
            let sc_dot = scalar::dot_product_scalar(&a, &b);
            let sc_cos_b = scalar::cosine_distance_f32_bytes_scalar(&a, &b_bytes);
            let sc_euc_b = scalar::euclidean_distance_f32_bytes_scalar(&a, &b_bytes);
            let sc_dot_b = scalar::dot_product_f32_bytes_scalar(&a, &b_bytes);
            let sc_dot_u8 = scalar::dot_product_u8_scalar(&a_u8, &b_u8);
            let sc_euc_u8 = scalar::euclidean_distance_sq_u8_scalar(&a_u8, &b_u8);
            let sc_parts_u8 = scalar::cosine_similarity_parts_u8_scalar(&a_u8, &b_u8);

            // 1. Direct AVX2 Kernel Differential
            #[cfg(target_arch = "x86_64")]
            {
                if is_x86_feature_detected!("avx2") && is_x86_feature_detected!("fma") {
                    let k_cos = unsafe { avx2::cosine_distance_avx2(&a, &b) };
                    let diff_cos = (sc_cos - k_cos).abs();
                    assert!(
                        diff_cos <= 1e-4 * sc_cos.abs().max(k_cos.abs()).max(1.0),
                        "AVX2 Cosine len {len}"
                    );

                    let k_euc = unsafe { avx2::euclidean_distance_avx2(&a, &b) };
                    let diff_euc = (sc_euc - k_euc).abs();
                    assert!(
                        diff_euc <= 1e-3 * sc_euc.abs().max(k_euc.abs()).max(1.0),
                        "AVX2 Euclidean len {len}"
                    );

                    let k_dot = unsafe { avx2::dot_product_avx2(&a, &b) };
                    let diff_dot = (sc_dot - k_dot).abs();
                    assert!(
                        diff_dot <= 1e-3 * sc_dot.abs().max(k_dot.abs()).max(1.0),
                        "AVX2 Dot len {len}"
                    );

                    let k_cos_b = unsafe { avx2::cosine_distance_f32_bytes_avx2(&a, &b_bytes) };
                    let diff_cos_b = (sc_cos_b - k_cos_b).abs();
                    assert!(
                        diff_cos_b <= 1e-4 * sc_cos_b.abs().max(k_cos_b.abs()).max(1.0),
                        "AVX2 Cosine f32_bytes len {len}"
                    );

                    let k_euc_b = unsafe { avx2::euclidean_distance_f32_bytes_avx2(&a, &b_bytes) };
                    let diff_euc_b = (sc_euc_b - k_euc_b).abs();
                    assert!(
                        diff_euc_b <= 1e-3 * sc_euc_b.abs().max(k_euc_b.abs()).max(1.0),
                        "AVX2 Euclidean f32_bytes len {len}"
                    );

                    let k_dot_b = unsafe { avx2::dot_product_f32_bytes_avx2(&a, &b_bytes) };
                    let diff_dot_b = (sc_dot_b - k_dot_b).abs();
                    assert!(
                        diff_dot_b <= 1e-3 * sc_dot_b.abs().max(k_dot_b.abs()).max(1.0),
                        "AVX2 Dot f32_bytes len {len}"
                    );

                    let k_dot_u8 = unsafe { avx2::dot_product_u8_avx2(&a_u8, &b_u8) };
                    assert_eq!(sc_dot_u8, k_dot_u8, "AVX2 Dot u8 len {len}");

                    let k_euc_u8 = unsafe { avx2::euclidean_distance_sq_u8_avx2(&a_u8, &b_u8) };
                    assert_eq!(sc_euc_u8, k_euc_u8, "AVX2 Euc sq u8 len {len}");

                    let k_parts_u8 = unsafe { avx2::cosine_similarity_parts_u8_avx2(&a_u8, &b_u8) };
                    assert_eq!(sc_parts_u8, k_parts_u8, "AVX2 Cosine parts u8 len {len}");
                }

                // 2. Direct AVX-512 Kernel Differential
                if is_x86_feature_detected!("avx512f") {
                    let k_cos = unsafe { avx512::cosine_distance_avx512(&a, &b) };
                    let diff_cos = (sc_cos - k_cos).abs();
                    assert!(
                        diff_cos <= 1e-4 * sc_cos.abs().max(k_cos.abs()).max(1.0),
                        "AVX512 Cosine len {len}"
                    );

                    let k_euc = unsafe { avx512::euclidean_distance_avx512(&a, &b) };
                    let diff_euc = (sc_euc - k_euc).abs();
                    assert!(
                        diff_euc <= 1e-3 * sc_euc.abs().max(k_euc.abs()).max(1.0),
                        "AVX512 Euclidean len {len}"
                    );

                    let k_dot = unsafe { avx512::dot_product_avx512(&a, &b) };
                    let diff_dot = (sc_dot - k_dot).abs();
                    assert!(
                        diff_dot <= 1e-3 * sc_dot.abs().max(k_dot.abs()).max(1.0),
                        "AVX512 Dot len {len}"
                    );

                    let k_cos_b = unsafe { avx512::cosine_distance_f32_bytes_avx512(&a, &b_bytes) };
                    let diff_cos_b = (sc_cos_b - k_cos_b).abs();
                    assert!(
                        diff_cos_b <= 1e-4 * sc_cos_b.abs().max(k_cos_b.abs()).max(1.0),
                        "AVX512 Cosine f32_bytes len {len}"
                    );

                    let k_euc_b =
                        unsafe { avx512::euclidean_distance_f32_bytes_avx512(&a, &b_bytes) };
                    let diff_euc_b = (sc_euc_b - k_euc_b).abs();
                    assert!(
                        diff_euc_b <= 1e-3 * sc_euc_b.abs().max(k_euc_b.abs()).max(1.0),
                        "AVX512 Euclidean f32_bytes len {len}"
                    );

                    let k_dot_b = unsafe { avx512::dot_product_f32_bytes_avx512(&a, &b_bytes) };
                    let diff_dot_b = (sc_dot_b - k_dot_b).abs();
                    assert!(
                        diff_dot_b <= 1e-3 * sc_dot_b.abs().max(k_dot_b.abs()).max(1.0),
                        "AVX512 Dot f32_bytes len {len}"
                    );

                    if is_x86_feature_detected!("avx512bw")
                        && is_x86_feature_detected!("avx512vnni")
                    {
                        let k_dot_u8 = unsafe { avx512::dot_product_u8_avx512vnni(&a_u8, &b_u8) };
                        assert_eq!(sc_dot_u8, k_dot_u8, "AVX512 Dot u8 len {len}");

                        let k_parts_u8 =
                            unsafe { avx512::cosine_similarity_parts_u8_avx512(&a_u8, &b_u8) };
                        assert_eq!(sc_parts_u8, k_parts_u8, "AVX512 Cosine parts u8 len {len}");
                    }

                    if is_x86_feature_detected!("avx512bw") {
                        let k_euc_u8 =
                            unsafe { avx512::euclidean_distance_sq_u8_avx512(&a_u8, &b_u8) };
                        assert_eq!(sc_euc_u8, k_euc_u8, "AVX512 Euc sq u8 len {len}");
                    }
                }
            }

            // 3. Direct NEON Kernel Differential
            #[cfg(target_arch = "aarch64")]
            {
                if std::arch::is_aarch64_feature_detected!("neon") {
                    let k_cos = unsafe { neon::cosine_distance_neon(&a, &b) };
                    let diff_cos = (sc_cos - k_cos).abs();
                    assert!(
                        diff_cos <= 1e-4 * sc_cos.abs().max(k_cos.abs()).max(1.0),
                        "NEON Cosine len {len}"
                    );

                    let k_euc = unsafe { neon::euclidean_distance_neon(&a, &b) };
                    let diff_euc = (sc_euc - k_euc).abs();
                    assert!(
                        diff_euc <= 1e-3 * sc_euc.abs().max(k_euc.abs()).max(1.0),
                        "NEON Euclidean len {len}"
                    );

                    let k_dot = unsafe { neon::dot_product_neon(&a, &b) };
                    let diff_dot = (sc_dot - k_dot).abs();
                    assert!(
                        diff_dot <= 1e-3 * sc_dot.abs().max(k_dot.abs()).max(1.0),
                        "NEON Dot len {len}"
                    );
                }
            }
        }
    }
}
