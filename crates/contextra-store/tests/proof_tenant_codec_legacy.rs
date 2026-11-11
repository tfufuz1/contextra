//! Differenztests Tenant-Codec: Aktuelle Implementierung vs. Legacy-Referenz (PROOF-3)
//!
//! Verifiziert mindestens 10.000 generierte Werte über proptest sowie Randfälle.

mod legacy_reference;

use contextra_store::tenant_codec::TenantKeyCodec;
use contextra_types::{CollectionId, DocId, TenantId};
use legacy_reference::tenant_codec_legacy::TenantKeyCodecLegacy;
use proptest::prelude::*;

#[test]
fn test_edge_cases_tenant_codec_differential() {
    let edge_tenants = [1u64, 2, 42, 1000, u32::MAX as u64, u64::MAX - 1];
    let edge_cols = [0u64, 1, 42, u64::MAX];
    let edge_docs = [0u64, 1, 42, 100_000, u64::MAX];

    for &t in &edge_tenants {
        let tid = TenantId::new(t);
        let current = TenantKeyCodec::new(tid);
        let legacy = TenantKeyCodecLegacy::new(tid);

        assert_eq!(current.scan_prefix(), legacy.scan_prefix());

        for &c in &edge_cols {
            let cid = CollectionId(c);
            assert_eq!(current.collection_prefix(&cid), legacy.collection_prefix(&cid));

            for &d in &edge_docs {
                let did = DocId(d);
                let cur_chunk = current.encode_chunk_key(&cid, did);
                let leg_chunk = legacy.encode_chunk_key(&cid, did);
                assert_eq!(cur_chunk, leg_chunk, "Chunk key mismatch for t={t}, c={c}, d={d}");

                let cur_dec = TenantKeyCodec::decode_chunk_key(&cur_chunk);
                let leg_dec = TenantKeyCodecLegacy::decode_chunk_key(&leg_chunk);
                assert_eq!(cur_dec, leg_dec);

                let cur_tid = TenantKeyCodec::decode_tenant_id(&cur_chunk);
                let leg_tid = TenantKeyCodecLegacy::decode_tenant_id(&leg_chunk);
                assert_eq!(cur_tid, leg_tid);

                let cur_graph = current.encode_graph_key(&cid, d);
                let leg_graph = legacy.encode_graph_key(&cid, d);
                assert_eq!(cur_graph, leg_graph);
            }
        }
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(10_000))]

    #[test]
    fn test_proptest_10k_differential_tenant_codec(
        tenant_val in 1u64..=u64::MAX,
        col_val in any::<u64>(),
        doc_val in any::<u64>(),
        entity_val in any::<u64>(),
    ) {
        let tid = TenantId::new(tenant_val);
        let cid = CollectionId(col_val);
        let did = DocId(doc_val);

        let current = TenantKeyCodec::new(tid);
        let legacy = TenantKeyCodecLegacy::new(tid);

        prop_assert_eq!(current.scan_prefix(), legacy.scan_prefix());
        prop_assert_eq!(current.collection_prefix(&cid), legacy.collection_prefix(&cid));

        let cur_encoded = current.encode_chunk_key(&cid, did);
        let leg_encoded = legacy.encode_chunk_key(&cid, did);
        prop_assert_eq!(&cur_encoded, &leg_encoded);

        let cur_decoded = TenantKeyCodec::decode_chunk_key(&cur_encoded);
        let leg_decoded = TenantKeyCodecLegacy::decode_chunk_key(&leg_encoded);
        prop_assert_eq!(cur_decoded, leg_decoded);

        let cur_graph = current.encode_graph_key(&cid, entity_val);
        let leg_graph = legacy.encode_graph_key(&cid, entity_val);
        prop_assert_eq!(cur_graph, leg_graph);
    }
}
