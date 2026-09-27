use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum KvDeleteMode {
    TombstoneOnly,
    CryptoShred,
}

impl Default for KvDeleteMode {
    fn default() -> Self {
        Self::CryptoShred
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_kv_delete_mode_default() {
        assert_eq!(KvDeleteMode::default(), KvDeleteMode::CryptoShred);
    }

    #[test]
    fn test_kv_delete_mode_serde_roundtrip() {
        let mode = KvDeleteMode::CryptoShred;
        let serialized = serde_json::to_string(&mode).expect("serialize");
        let deserialized: KvDeleteMode = serde_json::from_str(&serialized).expect("deserialize");
        assert_eq!(mode, deserialized);

        let tombstone = KvDeleteMode::TombstoneOnly;
        let serialized_t = serde_json::to_string(&tombstone).expect("serialize");
        let deserialized_t: KvDeleteMode = serde_json::from_str(&serialized_t).expect("deserialize");
        assert_eq!(tombstone, deserialized_t);
    }
}
