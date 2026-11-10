//! Konvertierung zwischen lokalem ModelFingerprint und contextra-types ModelFingerprint.

use crate::model_registry::ModelFingerprint as LocalModelFingerprint;

impl From<LocalModelFingerprint> for contextra_types::ModelFingerprint {
    fn from(fp: LocalModelFingerprint) -> Self {
        contextra_types::ModelFingerprint {
            hash: fp.hash,
            model_id: fp.model_id,
            quantization: fp.quantization,
        }
    }
}

impl From<contextra_types::ModelFingerprint> for LocalModelFingerprint {
    fn from(fp: contextra_types::ModelFingerprint) -> Self {
        LocalModelFingerprint {
            hash: fp.hash,
            model_id: fp.model_id,
            quantization: fp.quantization,
        }
    }
}
