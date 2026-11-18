//! Property-Tests Key-Codec v2 (P2): Roundtrip, Ordnungserhaltung, Praefix-Abgeschlossenheit,
//! v1/v2-Koexistenz.
#![cfg(not(feature = "docid-128"))]
#![allow(clippy::unwrap_used, clippy::expect_used)]

use contextra_ids::{CollectionId, DocId, TenantId};
use contextra_store::tenant_codec::{KeyCodecV2, KeyTag, TenantKeyCodec};
use proptest::prelude::*;

fn tenant() -> impl Strategy<Value = u64> {
    1u64..=u64::MAX
}

fn tag() -> impl Strategy<Value = u8> {
    any::<u8>()
}

type Tuple = (u16, u64, u64, u32, u8, u64);

fn tuple() -> impl Strategy<Value = Tuple> {
    (any::<u16>(), tenant(), any::<u64>(), any::<u32>(), tag(), any::<u64>())
}

fn enc(t: &Tuple) -> [u8; 32] {
    KeyCodecV2::with_region(t.0, TenantId(t.1)).encode_key(
        &CollectionId(t.2),
        t.3,
        KeyTag::from(t.4),
        t.5,
    )
}

proptest! {
    #[test]
    fn roundtrip(t in tuple()) {
        let key = enc(&t);
        let d = KeyCodecV2::decode_key(&key).expect("decode");
        prop_assert_eq!(d.region, t.0);
        prop_assert_eq!(d.tenant_id.inner(), t.1);
        prop_assert_eq!(d.collection_id.0, t.2);
        prop_assert_eq!(d.cluster, t.3);
        prop_assert_eq!(d.tag.to_u8(), t.4);
        prop_assert_eq!(d.entity_id, t.5);
    }

    #[test]
    fn order_preserved(a in tuple(), b in tuple()) {
        prop_assert_eq!(a.cmp(&b), enc(&a).cmp(&enc(&b)));
    }

    #[test]
    fn tag_order_matches_byte_order(a in tag(), b in tag()) {
        prop_assert_eq!(KeyTag::from(a).cmp(&KeyTag::from(b)), a.cmp(&b));
        prop_assert_eq!(KeyTag::Custom(a) == KeyTag::from(a), true);
    }

    #[test]
    fn prefix_closure(t in tuple(), other in tuple()) {
        let codec = KeyCodecV2::with_region(t.0, TenantId(t.1));
        let col = CollectionId(t.2);
        let key = enc(&t);
        prop_assert!(key.starts_with(&codec.tenant_prefix()));
        prop_assert!(key.starts_with(&codec.collection_prefix(&col)));
        prop_assert!(key.starts_with(&codec.cluster_prefix(&col, t.3)));
        // Praefix-Abgeschlossenheit: fremder Key liegt nur im Praefix, wenn die Felder gleich sind.
        let ok = enc(&other);
        prop_assert_eq!(
            ok.starts_with(&codec.cluster_prefix(&col, t.3)),
            (other.0, other.1, other.2, other.3) == (t.0, t.1, t.2, t.3)
        );
        prop_assert_eq!(
            ok.starts_with(&codec.tenant_prefix()),
            (other.0, other.1) == (t.0, t.1)
        );
    }

    #[test]
    fn cluster_range_is_contiguous(a in tuple(), b in tuple(), c in tuple()) {
        // Keys desselben Clusters bilden ein zusammenhaengendes Intervall der Sortierung.
        let codec = KeyCodecV2::with_region(a.0, TenantId(a.1));
        let p = codec.cluster_prefix(&CollectionId(a.2), a.3);
        let (ka, kb, kc) = (enc(&a), enc(&b), enc(&c));
        let mut v = [ka, kb, kc];
        v.sort();
        let flags: Vec<bool> = v.iter().map(|k| k.starts_with(&p)).collect();
        let first = flags.iter().position(|f| *f);
        let last = flags.iter().rposition(|f| *f);
        if let (Some(f), Some(l)) = (first, last) {
            prop_assert!(flags[f..=l].iter().all(|x| *x));
        }
    }

    #[test]
    fn v1_v2_coexist(t in tuple(), tid in tenant(), col in 1u64.., doc in any::<u64>()) {
        let v2 = enc(&t);
        let v1 = TenantKeyCodec::new(TenantId(tid)).encode_chunk_key(&CollectionId(col), DocId(doc));
        prop_assert!(KeyCodecV2::is_v2_key(&v2));
        prop_assert!(!KeyCodecV2::is_v2_key(&v1));
        prop_assert!(TenantKeyCodec::decode_tenant_id(&v2).is_none());
        prop_assert!(KeyCodecV2::decode_key(&v1).is_none());
        prop_assert_eq!(TenantKeyCodec::decode_tenant_id(&v1).map(TenantId::inner), Some(tid));
    }

    #[test]
    fn malformed_never_panics(bytes in proptest::collection::vec(any::<u8>(), 0..64)) {
        let _ = KeyCodecV2::decode_key(&bytes);
        let _ = KeyCodecV2::decode_tenant_id(&bytes);
        let _ = KeyCodecV2::extract_cluster(&bytes);
    }
}
