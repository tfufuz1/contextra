//! KV prefix store port trait definitions and types (Spec §9.2).

use bytes::Bytes;
use memfuse_types::model_fingerprint::ModelFingerprint;
use memfuse_types::types::domain::TenantId;
use memfuse_types::MemFuseError;
use serde::{Deserialize, Serialize};

/// KV cache layer and dimensions layout configuration.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct KvLayout {
    /// Number of transformer layers.
    pub n_layer: u32,
    /// Number of KV heads.
    pub n_kv_head: u32,
    /// Dimension per head.
    pub head_dim: u32,
    /// Data type representation (e.g. "f16", "f32", "q4_0").
    pub dtype: String,
}

/// Rotary position embedding (RoPE) configuration.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RopeConfig {
    /// Base frequency.
    pub base: f32,
    /// Optional frequency scaling factor.
    pub scaling: Option<f32>,
}

impl Eq for RopeConfig {}

impl std::hash::Hash for RopeConfig {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.base.to_bits().hash(state);
        if let Some(sc) = self.scaling {
            sc.to_bits().hash(state);
        }
    }
}

/// Key identifying a model KV cache prefix structure.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PrefixKey {
    /// Target model fingerprint.
    pub model: ModelFingerprint,
    /// SHA256/BLAKE3 hash of the tokenizer vocabulary.
    pub tokenizer_hash: [u8; 32],
    /// KV tensor memory layout.
    pub layout: KvLayout,
    /// Rotary embedding settings.
    pub rope: RopeConfig,
}

impl std::hash::Hash for PrefixKey {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.model.hash.hash(state);
        self.model.model_id.hash(state);
        self.model.quantization.hash(state);
        self.tokenizer_hash.hash(state);
        self.layout.hash(state);
        self.rope.hash(state);
    }
}

fn serialize_bytes<S>(bytes: &Bytes, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    serializer.serialize_bytes(bytes)
}

fn deserialize_bytes<'de, D>(deserializer: D) -> Result<Bytes, D::Error>
where
    D: serde::Deserializer<'de>,
{
    struct BytesVisitor;
    impl<'de> serde::de::Visitor<'de> for BytesVisitor {
        type Value = Bytes;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("byte array or vec")
        }

        fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            Ok(Bytes::copy_from_slice(v))
        }

        fn visit_byte_buf<E>(self, v: Vec<u8>) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            Ok(Bytes::from(v))
        }

        fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
        where
            A: serde::de::SeqAccess<'de>,
        {
            let mut vec = Vec::new();
            while let Some(byte) = seq.next_element()? {
                vec.push(byte);
            }
            Ok(Bytes::from(vec))
        }
    }
    deserializer.deserialize_byte_buf(BytesVisitor)
}

/// Represents an exported KV block segment.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KvBlock {
    /// Block sequence identifier.
    pub block_id: u64,
    /// Encrypted or raw block tensor bytes.
    #[serde(
        serialize_with = "serialize_bytes",
        deserialize_with = "deserialize_bytes"
    )]
    pub data: Bytes,
}

/// Represents a prefix match hit in the KV store.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KvPrefixHit {
    /// Number of tokens matched by the prefix.
    pub matched_tokens: usize,
    /// Extracted KV blocks for the matched prefix.
    pub blocks: Vec<KvBlock>,
}

/// Trait defining the contract for KV prefix caching and block reuse (Spec §9.2).
///
/// # Dyn-Kompatibilität
/// Dieser Trait ist durch synchrone Schnittstellen vtable-kompatibel (dyn-safe).
pub trait KvPrefixStore: Send + Sync + 'static {
    /// Returns the longest exact matching prefix in blocks for the prompt tokens, if any.
    fn lookup(&self, tenant: TenantId, key: &PrefixKey, tokens: &[u32]) -> Option<KvPrefixHit>;

    /// Inserts KV blocks for a given token prefix into the store.
    fn insert(
        &self,
        tenant: TenantId,
        key: &PrefixKey,
        tokens: &[u32],
        blocks: Vec<KvBlock>,
    ) -> Result<(), MemFuseError>;

    /// Evicts KV prefix entries for the given tenant and prefix key.
    /// Returns the number of blocks evicted.
    fn evict(&self, tenant: TenantId, key: &PrefixKey) -> Result<u64, MemFuseError>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::sync::Mutex;

    #[derive(Default)]
    struct MockKvStore {
        entries: Mutex<HashMap<(TenantId, PrefixKey), (Vec<u32>, Vec<KvBlock>)>>,
    }

    impl KvPrefixStore for MockKvStore {
        fn lookup(&self, tenant: TenantId, key: &PrefixKey, tokens: &[u32]) -> Option<KvPrefixHit> {
            let guard = self.entries.lock().unwrap();
            let (cached_tokens, blocks) = guard.get(&(tenant, key.clone()))?;

            let match_len = tokens
                .iter()
                .zip(cached_tokens.iter())
                .take_while(|(a, b)| a == b)
                .count();

            if match_len == 0 {
                None
            } else {
                Some(KvPrefixHit {
                    matched_tokens: match_len,
                    blocks: blocks.clone(),
                })
            }
        }

        fn insert(
            &self,
            tenant: TenantId,
            key: &PrefixKey,
            tokens: &[u32],
            blocks: Vec<KvBlock>,
        ) -> Result<(), MemFuseError> {
            let mut guard = self.entries.lock().unwrap();
            guard.insert((tenant, key.clone()), (tokens.to_vec(), blocks));
            Ok(())
        }

        fn evict(&self, tenant: TenantId, key: &PrefixKey) -> Result<u64, MemFuseError> {
            let mut guard = self.entries.lock().unwrap();
            if let Some((_, blocks)) = guard.remove(&(tenant, key.clone())) {
                Ok(blocks.len() as u64)
            } else {
                Ok(0)
            }
        }
    }

    #[test]
    fn test_kv_prefix_store_mock_roundtrip() {
        let store = MockKvStore::default();
        let tenant = TenantId::try_new(1).unwrap();
        let key = PrefixKey {
            model: ModelFingerprint::new([0xab; 32], "test-model", "F16"),
            tokenizer_hash: [0u8; 32],
            layout: KvLayout {
                n_layer: 32,
                n_kv_head: 8,
                head_dim: 128,
                dtype: "f16".into(),
            },
            rope: RopeConfig {
                base: 10000.0,
                scaling: None,
            },
        };

        let tokens = vec![101, 2004, 2005, 1000];
        let blocks = vec![KvBlock {
            block_id: 1,
            data: Bytes::from_static(b"block_data_0"),
        }];

        store.insert(tenant, &key, &tokens, blocks.clone()).unwrap();

        let hit = store.lookup(tenant, &key, &tokens).unwrap();
        assert_eq!(hit.matched_tokens, 4);
        assert_eq!(hit.blocks, blocks);

        let json = serde_json::to_string(&hit.blocks[0]).expect("serialization");
        let deser: KvBlock = serde_json::from_str(&json).expect("deserialization");
        assert_eq!(deser, hit.blocks[0]);

        let evicted = store.evict(tenant, &key).unwrap();
        assert_eq!(evicted, 1);
        assert!(store.lookup(tenant, &key, &tokens).is_none());
    }
}
