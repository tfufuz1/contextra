//! Legacy-Referenz und Differenztest für DocId in contextra-rank.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct LegacyDocId(pub u64);

impl LegacyDocId {
    pub fn from_key(key: &str) -> Result<Self, String> {
        if let Ok(val) = key.parse::<u64>() {
            return Ok(LegacyDocId(val));
        }
        if let Some(stripped) = key
            .strip_prefix("doc_")
            .or_else(|| key.strip_prefix("doc-"))
        {
            if let Ok(val) = stripped.parse::<u64>() {
                return Ok(LegacyDocId(val));
            }
        }
        let mut hasher = ahash::AHasher::default();
        std::hash::Hash::hash(key, &mut hasher);
        use std::hash::Hasher;
        Ok(LegacyDocId(hasher.finish()))
    }
}

#[test]
fn test_legacy_doc_id_semantics() {
    let d1 = LegacyDocId::from_key("12345").unwrap();
    assert_eq!(d1.0, 12345);

    let d2 = LegacyDocId::from_key("doc_6789").unwrap();
    assert_eq!(d2.0, 6789);

    let d3 = LegacyDocId::from_key("doc-999").unwrap();
    assert_eq!(d3.0, 999);

    let d4 = LegacyDocId::from_key("arbitrary_string").unwrap();
    assert!(d4.0 > 0);
}
