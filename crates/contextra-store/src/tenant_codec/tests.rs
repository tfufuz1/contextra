#![allow(clippy::unwrap_used)]
use super::*;

    #[test]
    fn test_encode_decode_roundtrip() {
        let tenant = TenantId::try_new(7).unwrap();
        let codec = TenantKeyCodec::new(tenant);
        let col = CollectionId(42);
        let doc = DocId(99);
        let key = codec.encode_chunk_key(&col, doc);
        assert_eq!(TenantKeyCodec::decode_tenant_id(&key), Some(tenant));
    }

    #[test]
    fn test_cross_tenant_isolation() {
        let codec_a = TenantKeyCodec::new(TenantId::try_new(1).unwrap());
        let codec_b = TenantKeyCodec::new(TenantId::try_new(2).unwrap());
        let col = CollectionId(1);
        let doc = DocId(1);
        let key_a = codec_a.encode_chunk_key(&col, doc);
        // Key von Tenant A darf NICHT mit Präfix von Tenant B matchen
        assert!(!key_a.starts_with(codec_b.scan_prefix()));
    }

    #[test]
    fn test_scan_prefix_cached_no_alloc() {
        let codec = TenantKeyCodec::new(TenantId::try_new(42).unwrap());
        assert!(codec.scan_prefix().starts_with(b"t:42:"));
    }

    #[test]
    fn test_decode_invalid_key_returns_none() {
        assert!(TenantKeyCodec::decode_tenant_id(b"invalid").is_none());
        assert!(TenantKeyCodec::decode_tenant_id(b"t:0:col:chunk:1").is_none());
        // SYSTEM
    }

    #[test]
    fn test_upper_bound_calculation() {
        let prefix = b"t:101:";
        let ub = upper_bound_for_prefix(prefix).unwrap();
        assert_eq!(ub, b"t:101;");
    }
