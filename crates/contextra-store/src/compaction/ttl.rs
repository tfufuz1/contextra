//! TTL (Time-To-Live) Metadata and deterministic expiry evaluation for compaction (§4.12, P28).

/// Magic prefix bytes identifying a TTL-annotated payload value in storage (`"TTL\0"`).
pub const TTL_MAGIC_PREFIX: &[u8; 4] = b"TTL\0";

/// Total header size in bytes for a TTL-annotated payload (4 bytes magic + 8 bytes u64 LE timestamp).
pub const TTL_HEADER_SIZE: usize = 12;

/// Represents an absolute TTL expiration timestamp in nanoseconds since UNIX epoch.
///
/// # Determinism Guarantee (P28 / INV-TTL-1)
/// Expiration timestamps are computed strictly prior to compaction at write time (`clock.now_unix_nanos() + ttl_duration`).
/// Compaction performs purely a comparison operation (`entry.expires_at_unix_nanos < clock.now_unix_nanos()`)
/// without calculating new expiration times.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize,
)]
pub struct TtlMetadata {
    /// Absolute expiration timestamp in nanoseconds since UNIX epoch.
    pub expires_at_unix_nanos: u64,
}

impl TtlMetadata {
    /// Creates a new `TtlMetadata` with an explicit absolute expiration timestamp in nanoseconds since UNIX epoch.
    #[inline]
    pub const fn new(expires_at_unix_nanos: u64) -> Self {
        Self {
            expires_at_unix_nanos,
        }
    }

    /// Evaluates whether an entry with this expiration timestamp is expired at `now_unix_nanos`.
    ///
    /// # Invariant INV-TTL-1
    /// Pure comparison operation: returns `self.expires_at_unix_nanos < now_unix_nanos`.
    #[inline]
    pub fn is_expired(&self, now_unix_nanos: u64) -> bool {
        self.expires_at_unix_nanos < now_unix_nanos
    }

    /// Prepends a 12-byte TTL header (`"TTL\0"` + 8-byte LE u64) to `raw_value`.
    pub fn encode_value(&self, raw_value: &[u8]) -> Vec<u8> {
        let mut buf = Vec::with_capacity(TTL_HEADER_SIZE + raw_value.len());
        buf.extend_from_slice(TTL_MAGIC_PREFIX);
        buf.extend_from_slice(&self.expires_at_unix_nanos.to_le_bytes());
        buf.extend_from_slice(raw_value);
        buf
    }

    /// Parses `TtlMetadata` from a value slice if it contains the TTL magic prefix.
    pub fn parse_from_value(value: &[u8]) -> Option<Self> {
        if value.len() >= TTL_HEADER_SIZE && &value[0..4] == TTL_MAGIC_PREFIX {
            let ts_bytes: [u8; 8] = value[4..12].try_into().ok()?;
            let expires_at_unix_nanos = u64::from_le_bytes(ts_bytes);
            Some(Self {
                expires_at_unix_nanos,
            })
        } else {
            None
        }
    }

    /// Strips the TTL header if present, returning the underlying raw value slice.
    pub fn unwrap_value(value: &[u8]) -> &[u8] {
        if value.len() >= TTL_HEADER_SIZE && &value[0..4] == TTL_MAGIC_PREFIX {
            &value[TTL_HEADER_SIZE..]
        } else {
            value
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ttl_metadata_expiry_comparison() {
        let meta = TtlMetadata::new(1000);
        assert!(meta.is_expired(1001));
        assert!(!meta.is_expired(1000));
        assert!(!meta.is_expired(999));
    }

    #[test]
    fn test_ttl_metadata_encoding_roundtrip() {
        let raw_payload = b"hello world";
        let meta = TtlMetadata::new(5_000_000_000);
        let encoded = meta.encode_value(raw_payload);

        let parsed = TtlMetadata::parse_from_value(&encoded);
        assert_eq!(parsed, Some(meta));

        let unwrapped = TtlMetadata::unwrap_value(&encoded);
        assert_eq!(unwrapped, raw_payload);
    }

    #[test]
    fn test_plain_value_without_ttl_prefix() {
        let raw_payload = b"plain payload without header";
        assert_eq!(TtlMetadata::parse_from_value(raw_payload), None);
        assert_eq!(TtlMetadata::unwrap_value(raw_payload), raw_payload);
    }
}
