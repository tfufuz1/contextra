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
