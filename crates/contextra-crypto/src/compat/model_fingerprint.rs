//! # ModelFingerprint-Kompatibilität (`compat/model_fingerprint.rs`)
//!
//! Stellt wechselseitige Konvertierungen zwischen [`crate::kv_cipher::ModelFingerprint`]
//! und [`contextra_types::ModelFingerprint`] bereit.

use crate::kv_cipher::ModelFingerprint as LocalFingerprint;
use contextra_types::ModelFingerprint as TypesFingerprint;

impl From<TypesFingerprint> for LocalFingerprint {
    fn from(tf: TypesFingerprint) -> Self {
        Self {
            hash: tf.hash,
            model_id: tf.model_id,
            quantization: tf.quantization,
        }
    }
}

impl From<LocalFingerprint> for TypesFingerprint {
    fn from(lf: LocalFingerprint) -> Self {
        Self {
            hash: lf.hash,
            model_id: lf.model_id,
            quantization: lf.quantization,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_model_fingerprint_conversion_roundtrip() {
        let types_fp = TypesFingerprint::new([0x42; 32], "llama-3", "Q4_K_M");
        let local_fp: LocalFingerprint = types_fp.clone().into();
        assert_eq!(local_fp.hash, types_fp.hash);
        assert_eq!(local_fp.model_id, types_fp.model_id);
        assert_eq!(local_fp.quantization, types_fp.quantization);

        let roundtrip: TypesFingerprint = local_fp.into();
        assert_eq!(roundtrip, types_fp);
    }
}
