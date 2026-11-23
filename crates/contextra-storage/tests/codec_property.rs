#![allow(clippy::panic, clippy::unwrap_used, clippy::expect_used, clippy::indexing_slicing)]

use contextra_storage::codec::bloom::BloomFilter;
use contextra_storage::codec::comparator::EntryKey;
use contextra_storage::codec::crc32c::crc32c;
use contextra_storage::codec::format_header::decode_format_plain;
use contextra_storage::codec::tag::{InternalKey, Tag, TagKind};
use contextra_storage::codec::varint::{decode_u64, encode_u64};
use proptest::prelude::*;

proptest! {
    // 1. Property: decode(encode(x)) == x für Varint
    #[test]
    fn prop_varint_roundtrip(val: u64) {
        let mut buf = [0u8; 10];
        let len = encode_u64(val, &mut buf);
        let (decoded, consumed) = decode_u64(&buf[..len]).expect("Varint roundtrip decode should succeed");
        prop_assert_eq!(decoded, val);
        prop_assert_eq!(consumed, len);
    }

    // 2. Property: Jede Byte-Folge ergibt Ok oder Err, nie Panik (P-4 totale Dekodierung)
    #[test]
    fn prop_varint_total_decoding(bytes in proptest::collection::vec(any::<u8>(), 0..32)) {
        let _ = decode_u64(&bytes);
    }

    // 3. Property: Tag Roundtrip und Ordnung
    #[test]
    fn prop_tag_roundtrip(stamp in 0u64..(1u64 << 56), kind_idx in 0..5usize) {
        let kinds = [TagKind::Put, TagKind::Delete, TagKind::RangeDelete, TagKind::EraseRange, TagKind::PutRef];
        let kind = kinds[kind_idx];
        let tag = Tag::try_new(stamp, kind).expect("stamp < 2^56");
        prop_assert_eq!(tag.stamp(), stamp);
        prop_assert_eq!(tag.kind().expect("valid kind"), kind);

        let le = tag.to_le_bytes();
        let from_le = Tag::from_le_bytes(le);
        prop_assert_eq!(from_le, tag);
    }

    // 4. Property: InternalKey Sortierordnung
    #[test]
    fn prop_internal_key_order(key_a: Vec<u8>, key_b: Vec<u8>, stamp_a in 1u64..1000, stamp_b in 1u64..1000) {
        let tag_a = Tag::new(stamp_a, TagKind::Put);
        let tag_b = Tag::new(stamp_b, TagKind::Put);
        let ik_a = InternalKey::new(key_a.clone(), tag_a);
        let ik_b = InternalKey::new(key_b.clone(), tag_b);

        if key_a == key_b {
            if stamp_a > stamp_b {
                prop_assert!(ik_a < ik_b); // Größerer Stamp kommt ZUERST
            } else if stamp_a < stamp_b {
                prop_assert!(ik_a > ik_b);
            }
        } else if key_a < key_b {
            prop_assert!(ik_a < ik_b);
        } else {
            prop_assert!(ik_a > ik_b);
        }
    }

    // 5. Property: Bloom Roundtrip & Total Decoding
    #[test]
    fn prop_bloom_roundtrip(keys in proptest::collection::vec(proptest::collection::vec(any::<u8>(), 1..16), 1..20)) {
        let mut filter = BloomFilter::new(keys.len(), 10);
        for k in &keys {
            filter.insert(k);
        }
        for k in &keys {
            prop_assert!(filter.may_contain(k));
        }

        let encoded = filter.encode();
        let decoded = BloomFilter::decode(&encoded).expect("decode valid");
        prop_assert_eq!(filter, decoded);
    }

    #[test]
    fn prop_bloom_total_decoding(bytes in proptest::collection::vec(any::<u8>(), 0..64)) {
        let _ = BloomFilter::decode(&bytes);
    }
}

proptest! {
    // 6. Property: FORMAT-Dekodierung ist total (P-4): Ok oder Err, nie Panik.
    #[test]
    fn prop_format_decode_total(bytes in proptest::collection::vec(any::<u8>(), 0..80)) {
        let _ = decode_format_plain(&bytes, &crc32c);
    }

    // 7. Property: Comparator (§6.2) ist antisymmetrisch und folgt der Feldordnung
    //    (Cluster ↑, Schlüssel ↑ bytelexikografisch, Tag ↓).
    #[test]
    fn prop_entry_key_order(
        ca in any::<u8>(), cb in any::<u8>(),
        ka in proptest::collection::vec(any::<u8>(), 0..8),
        kb in proptest::collection::vec(any::<u8>(), 0..8),
        sa in 1u64..1000, sb in 1u64..1000,
    ) {
        use contextra_storage::codec::tag::{Tag, TagKind};
        use std::cmp::Ordering;
        let a = EntryKey { cluster: [ca; 16], user_key: &ka, tag: Tag::new(sa, TagKind::Put) };
        let b = EntryKey { cluster: [cb; 16], user_key: &kb, tag: Tag::new(sb, TagKind::Put) };
        prop_assert_eq!(a.cmp(&b), b.cmp(&a).reverse());
        let expected = ca.cmp(&cb)
            .then_with(|| ka.cmp(&kb))
            .then_with(|| sb.cmp(&sa));
        prop_assert_eq!(a.cmp(&b), expected);
        if expected == Ordering::Equal {
            prop_assert_eq!(a, b);
        }
    }
}

