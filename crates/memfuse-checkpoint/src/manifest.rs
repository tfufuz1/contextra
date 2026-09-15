use crate::meta::{validate_identifier, CheckpointMeta};
use memfuse_core::{MemFuseError, Result};
use serde::{Deserialize, Serialize};

/// AI-TAG\[PANIC-SAFETY\]\[CRITICAL\] RESOLVED: AGT-CKPT-f3a1b2c4 (TS:2026-08-29T08:06:29Z) (SESSION:14348074)
/// Fault-Injection-Tests in
/// tests/manifest_fault_injection.rs beweisen atomare Schreib-Semantik
/// und Tamper-Erkennung via Blake3-Checksum.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CheckpointManifest {
    pub meta: CheckpointMeta,
    pub components: Vec<String>,
    pub checksum: String,
}

impl CheckpointManifest {
    pub fn new(meta: CheckpointMeta, components: Vec<String>) -> Result<Self> {
        validate_identifier("Checkpoint name", &meta.name)?;
        validate_identifier("Collection ID", &meta.collection_id)?;
        for comp in &components {
            if comp.trim().is_empty() {
                return Err(MemFuseError::InvalidInput(
                    "Checkpoint component name cannot be empty".to_string(),
                ));
            }
        }
        let payload = serde_json::to_vec(&(&meta, &components))
            .map_err(|e| MemFuseError::Serialization(e.to_string()))?;
        let checksum = blake3::hash(&payload).to_hex().to_string();
        Ok(Self {
            meta,
            components,
            checksum,
        })
    }

    pub fn verify(&self) -> Result<()> {
        let payload = serde_json::to_vec(&(&self.meta, &self.components))
            .map_err(|e| MemFuseError::Serialization(e.to_string()))?;
        let expected = blake3::hash(&payload).to_hex().to_string();
        if self.checksum != expected {
            return Err(MemFuseError::Serialization(format!(
                "Checkpoint manifest checksum mismatch for '{}': expected {}, got {}",
                self.meta.name, expected, self.checksum
            )));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use memfuse_core::TxId;

    #[test]
    fn test_manifest_validation_blank_component() {
        let meta = CheckpointMeta {
            name: "cp1".to_string(),
            collection_id: "col1".to_string(),
            seq_no: 1,
            tx_id: TxId::new(1),
            metadata: serde_json::json!({}),
            created_at: 100,
        };

        let res = CheckpointManifest::new(meta, vec!["   ".to_string()]);
        assert!(matches!(res, Err(MemFuseError::InvalidInput(_))));
    }

    #[allow(non_snake_case)]
    #[test]
    fn checkpoint_manifest_CASE_whitespace_component_rejected() {
        let meta = CheckpointMeta {
            name: "cp_ws".to_string(),
            collection_id: "col_ws".to_string(),
            seq_no: 1,
            tx_id: TxId::new(10),
            metadata: serde_json::json!({}),
            created_at: 100,
        };
        let res = CheckpointManifest::new(meta, vec!["   ".to_string()]);
        assert!(matches!(res, Err(MemFuseError::InvalidInput(_))));
    }

    #[allow(non_snake_case)]
    #[test]
    fn checkpoint_manifest_CASE_tampered_manifest_fails_verify() {
        let meta = CheckpointMeta {
            name: "cp_tamper".to_string(),
            collection_id: "col_tamper".to_string(),
            seq_no: 5,
            tx_id: TxId::new(50),
            metadata: serde_json::json!({"version": 1}),
            created_at: 500,
        };
        let mut manifest = CheckpointManifest::new(meta, vec!["comp1".to_string()])
            .expect("// expect #[cfg(test)]");
        manifest.components.push("tampered_comp".to_string());
        let res = manifest.verify();
        assert!(matches!(res, Err(MemFuseError::Serialization(_))));
    }

    #[test]
    fn test_manifest_creation_invalid_meta_name() {
        let meta = CheckpointMeta {
            name: "   ".to_string(),
            collection_id: "col_valid".to_string(),
            seq_no: 1,
            tx_id: TxId::new(10),
            metadata: serde_json::json!({}),
            created_at: 100,
        };
        let res = CheckpointManifest::new(meta, vec!["comp1".to_string()]);
        assert!(matches!(res, Err(MemFuseError::InvalidInput(_))));
    }

    proptest::proptest! {
        #[test]
        fn prop_manifest_roundtrip(
            name: String,
            col: String,
            seq_no: u64,
            tx: u64,
            created_at: u64,
        ) {
            if name.trim().is_empty()
                || col.trim().is_empty()
                || name.len() > 256
                || col.len() > 256
            {
                // Expected validation failure for invalid boundaries
                let meta = CheckpointMeta {
                    name,
                    collection_id: col,
                    seq_no,
                    tx_id: TxId::new(tx),
                    metadata: serde_json::json!({}),
                    created_at,
                };
                proptest::prop_assert!(CheckpointManifest::new(meta, vec!["valid_comp".to_string()]).is_err());
            } else {
                let meta = CheckpointMeta {
                    name: name.clone(),
                    collection_id: col.clone(),
                    seq_no,
                    tx_id: TxId::new(tx),
                    metadata: serde_json::json!({}),
                    created_at,
                };
                let manifest = CheckpointManifest::new(meta, vec!["comp_a".to_string(), "comp_b".to_string()]);
                proptest::prop_assert!(manifest.is_ok());
                let manifest = manifest.expect("// expect #[cfg(test)]");
                proptest::prop_assert!(manifest.verify().is_ok());
                proptest::prop_assert_eq!(manifest.meta.name, name);
                proptest::prop_assert_eq!(manifest.meta.collection_id, col);
                proptest::prop_assert_eq!(manifest.meta.seq_no, seq_no);
                proptest::prop_assert_eq!(manifest.meta.tx_id, TxId::new(tx));
                proptest::prop_assert_eq!(manifest.meta.created_at, created_at);
            }
        }
    }
}
