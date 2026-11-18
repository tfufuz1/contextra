#![cfg(feature = "workspace")]
use contextra_store::tenant_codec::TenantKeyCodec;
use contextra_types::{CollectionId, DocId, TenantId};
use proptest::prelude::*;

proptest! {
    #[test]
    fn prop_tenant_codec_chunk_key_injective(
        tenant_val in 1u64..10000,
        col_val in 0u64..10000,
        doc_val in 0u64..10000,
    ) {
        let tenant = TenantId::try_new(tenant_val).unwrap();
        let col = CollectionId(col_val);
        let doc = DocId(doc_val);
        let codec = TenantKeyCodec::new(tenant);

        let encoded = codec.encode_chunk_key(&col, doc);
        let decoded = TenantKeyCodec::decode_chunk_key(&encoded);

        prop_assert_eq!(decoded, Some((tenant, col, doc)));
    }
}

#[cfg(feature = "workspace")]
use contextra_store::tenant_codec::{KeyCodecV2, KeyTag};

#[cfg(feature = "workspace")]
proptest! {
    #[test]
    fn prop_v2_codec_injective_and_order_preserving(
        region in any::<u16>(),
        tenant_val in 1u64..100000,
        col_val in any::<u64>(),
        cluster in any::<u32>(),
        doc_val1 in 0u64..50000,
        doc_val2 in 50001u64..100000,
    ) {
        let tenant = TenantId::try_new(tenant_val).unwrap();
        let col = CollectionId(col_val);
        let codec = KeyCodecV2::with_region(region, tenant);

        let k1 = codec.encode_chunk_key_with_cluster(&col, cluster, DocId(doc_val1));
        let k2 = codec.encode_chunk_key_with_cluster(&col, cluster, DocId(doc_val2));

        // 1. Injektivität & Decodierung
        let dec1 = KeyCodecV2::decode_key(&k1).expect("decode k1");
        prop_assert_eq!(dec1.region, region);
        prop_assert_eq!(dec1.tenant_id, tenant);
        prop_assert_eq!(dec1.collection_id, col);
        prop_assert_eq!(dec1.cluster, cluster);
        prop_assert_eq!(dec1.tag, KeyTag::Chunk);
        prop_assert_eq!(dec1.entity_id, doc_val1);

        // 2. Ordnungserhaltung: doc_val1 < doc_val2 -> Byte-Reihenfolge k1 < k2
        prop_assert!(k1 < k2);

        // 3. Präfix-Abgeschlossenheit
        prop_assert!(k1.starts_with(&codec.tenant_prefix()));
        prop_assert!(k1.starts_with(&codec.collection_prefix(&col)));
        prop_assert!(k1.starts_with(&codec.cluster_prefix(&col, cluster)));
    }
}
