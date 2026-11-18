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

    #[test]
    fn test_v2_encode_decode_roundtrip() {
        let tenant = TenantId::try_new(42).unwrap();
        let codec = KeyCodecV2::with_region(1, tenant);
        let col = CollectionId(100);
        let cluster = 7;
        let doc = DocId(12345);

        let key = codec.encode_chunk_key_with_cluster(&col, cluster, doc);
        assert_eq!(key.len(), 32);
        assert_eq!(key[0], KEY_VERSION_V2);

        let decoded = KeyCodecV2::decode_key(&key).expect("decode v2");
        assert_eq!(decoded.region, 1);
        assert_eq!(decoded.tenant_id, tenant);
        assert_eq!(decoded.collection_id, col);
        assert_eq!(decoded.cluster, cluster);
        assert_eq!(decoded.tag, KeyTag::Chunk);
        assert_eq!(decoded.entity_id, 12345);

        // Universal decoder
        assert_eq!(TenantKeyCodec::decode_any_tenant_id(&key), Some(tenant));
    }

    #[test]
    fn test_v2_order_preservation() {
        let tenant = TenantId::try_new(10).unwrap();
        let codec = KeyCodecV2::new(tenant);
        let col = CollectionId(1);

        // Integer-Ordnung muss exakt der Byte-Ordnung entsprechen
        let k1 = codec.encode_chunk_key_with_cluster(&col, 1, DocId(9));
        let k2 = codec.encode_chunk_key_with_cluster(&col, 1, DocId(10));
        let k3 = codec.encode_chunk_key_with_cluster(&col, 2, DocId(1));

        assert!(k1 < k2, "Doc 9 must be smaller than Doc 10");
        assert!(k2 < k3, "Cluster 1 must be smaller than Cluster 2");
    }

    #[test]
    fn test_v2_prefix_matching() {
        let tenant = TenantId::try_new(5).unwrap();
        let codec = KeyCodecV2::new(tenant);
        let col = CollectionId(12);

        let t_prefix = codec.tenant_prefix();
        let c_prefix = codec.collection_prefix(&col);
        let cl_prefix = codec.cluster_prefix(&col, 99);

        let key = codec.encode_chunk_key_with_cluster(&col, 99, DocId(42));

        assert!(key.starts_with(&t_prefix));
        assert!(key.starts_with(&c_prefix));
        assert!(key.starts_with(&cl_prefix));

        let other_tenant = TenantId::try_new(6).unwrap();
        let other_codec = KeyCodecV2::new(other_tenant);
        assert!(!key.starts_with(&other_codec.tenant_prefix()));
    }
