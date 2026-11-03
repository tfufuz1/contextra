//! SPEC Verification Testsuite (T-KEY-1, T-ISO-2, T-DOCREF-1, T-API-4, T-LIMIT-1, Golden Bytes).
//!
//! Alle Tests besitzen Rückverfolgbarkeits-IDs aus SPEC 15.1/15.2 (T-TRACE-1).

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::cast_possible_truncation
)]

use contextra_types::error::*;
use contextra_types::key::*;
use contextra_types::limits::*;
use contextra_types::refs::*;
use contextra_types::tags::*;
use proptest::prelude::*;

/// SPEC: T-KEY-1
#[test]
fn test_t_key_1_schluesselkodierung_feste_laengen_und_sortierung() {
    let tenant = TenantTag::new([1u8; 16]);
    let coll = CollTag::new([2u8; 16]);
    let cluster = ClusterTag::new([3u8; 16]);
    let doc = DocTag::new([4u8; 16]);

    let key = KeyBuf::new(
        Region::Data,
        tenant,
        coll,
        cluster,
        doc,
        Family::Doc,
        42,
        1000,
    );

    assert_eq!(key.as_bytes().len(), KEY_LEN);
    assert_eq!(KEY_LEN, 78);
    assert_eq!(KEY_BOUND_LEN, 65);

    // Commit seq roundtrip without DEK (D-74)
    assert_eq!(key.commit_seq(), 1000);

    // Newer commit sequence sorts earlier because of !commit_seq (TY-04)
    let key_older = KeyBuf::new(
        Region::Data,
        tenant,
        coll,
        cluster,
        doc,
        Family::Doc,
        42,
        999,
    );
    assert!(key < key_older, "Neueste Version muss zuerst sortieren");

    // TagPreimage prefix check (TY-07)
    let pre1 = TagPreimage::new(b"test").field(b"a").field(b"bc");
    let pre2 = TagPreimage::new(b"test").field(b"ab").field(b"c");
    assert_ne!(
        pre1.as_bytes(),
        pre2.as_bytes(),
        "Längenpräfixe müssen Kollisionen verhindern"
    );
}

/// SPEC: T-ISO-2
#[test]
fn test_t_iso_2_matcher_bereichs_und_exakt_treffer() {
    let tenant = TenantTag::new([1u8; 16]);
    let coll = CollTag::new([2u8; 16]);
    let cluster = ClusterTag::new([3u8; 16]);
    let doc = DocTag::new([4u8; 16]);

    let key = KeyBuf::new(
        Region::Data,
        tenant,
        coll,
        cluster,
        doc,
        Family::Doc,
        1,
        500,
    );

    let range_doc = KeyRange::doc(Region::Data, tenant, coll, cluster, doc);
    let matcher = Matcher::new(vec![MatcherItem::Range(range_doc)]);

    assert!(matcher.matches(&key));

    let other_doc = DocTag::new([5u8; 16]);
    let key_other = KeyBuf::new(
        Region::Data,
        tenant,
        coll,
        cluster,
        other_doc,
        Family::Doc,
        1,
        500,
    );
    assert!(!matcher.matches(&key_other));

    // Normalisierung: Exakter Key innerhalb von Range wird deduped
    let matcher_with_redundant = Matcher::new(vec![
        MatcherItem::Range(range_doc),
        MatcherItem::Exact(key),
    ]);
    assert_eq!(matcher_with_redundant.items().len(), 1);
}

/// SPEC: T-DOCREF-1
#[test]
fn test_t_docref_1_dokumentidentitaet_und_cluster_tag(
) -> std::result::Result<(), Box<dyn std::error::Error>> {
    let doc_key = DocKey::try_new(b"doc-123".to_vec())?;
    let subj_a = SubjectRef::try_new(b"subject-A".to_vec())?;
    let subj_b = SubjectRef::try_new(b"subject-B".to_vec())?;

    let doc_ref_a = DocRef {
        subject: Some(subj_a.clone()),
        key: doc_key.clone(),
    };
    let doc_ref_b = DocRef {
        subject: Some(subj_b),
        key: doc_key,
    };

    assert_ne!(
        doc_ref_a, doc_ref_b,
        "Gleicher Key mit anderem Betroffenen muss verschiedene Identität haben"
    );

    let subj_tag = SubjectTag::new([9u8; 16]);
    let cluster_from_subj = ClusterTag::from_subject(subj_tag);
    assert_eq!(cluster_from_subj.as_bytes(), subj_tag.as_bytes());

    let doc_tag = DocTag::new([1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16]);
    let unassigned = ClusterTag::unassigned(doc_tag);
    assert_eq!(&unassigned.as_bytes()[0..12], &[0u8; 12]);
    assert_eq!(&unassigned.as_bytes()[12..16], &doc_tag.as_bytes()[0..4]);
    Ok(())
}

/// SPEC: T-API-4
#[test]
fn test_t_api_4_fehlertexte_eingefroren_golden() {
    let table = [
        (
            Error::Config(ConfigError::EmptyIdentifier),
            "configuration error: identifier is empty",
        ),
        (
            Error::Config(ConfigError::IdentifierTooLong),
            "configuration error: identifier exceeds maximum length",
        ),
        (
            Error::Config(ConfigError::InvalidLength),
            "configuration error: invalid byte length",
        ),
        (Error::Io, "i/o error"),
        (Error::Corrupt, "data corruption detected"),
        (Error::Auth, "authentication failed"),
        (Error::Erased, "target is erased"),
        (Error::Restricted, "target is restricted"),
        (Error::NotFound, "not found"),
        (
            Error::Conflict(Conflict::IdCollision),
            "conflict: identifier collision",
        ),
        (
            Error::Conflict(Conflict::Exists),
            "conflict: record already exists",
        ),
        (
            Error::Limit(Limit::Capacity),
            "limit exceeded: storage capacity",
        ),
        (
            Error::Limit(Limit::DiskReserve),
            "limit exceeded: disk reserve exhausted",
        ),
        (Error::Limit(Limit::Snapshots), "limit exceeded: snapshots"),
        (
            Error::Limit(Limit::QueryTooLarge),
            "limit exceeded: query too large",
        ),
        (Error::Limit(Limit::Dimension), "limit exceeded: dimension"),
        (Error::SnapshotRevoked, "snapshot revoked"),
        (Error::Cancelled, "operation cancelled"),
        (Error::Refused, "operation refused"),
        (Error::Internal, "internal system error"),
    ];

    for (err, expected_text) in table {
        assert_eq!(
            err.to_string(),
            expected_text,
            "Fehlertext weicht von gefrorener D-110-Vorgabe ab"
        );
    }
}

/// SPEC: T-LIMIT-1
#[test]
fn test_t_limit_1_typebene_kapazitaetsgrenzen() {
    assert_eq!(MAX_DOCS_PER_STORE, 50_000_000);
    assert_eq!(MAX_DOC_BYTES_DEFAULT, 4 * 1024 * 1024);
    assert_eq!(MAX_DOC_BYTES_HARD, 64 * 1024 * 1024);
    assert_eq!(MAX_SNAPSHOTS_PER_TENANT, 1024);
    assert_eq!(MAX_DIMENSION, 4096);
    assert_eq!(MAX_QUERY_TERMS, 256);
    assert_eq!(MAX_K, 1000);
}

/// SPEC: T-KEY-1 Golden Bytes (Anhang C & Anhang F)
#[test]
fn test_t_key_1_golden_bytes_layout() {
    let tenant = TenantTag::new([0xAA; 16]);
    let coll = CollTag::new([0xBB; 16]);
    let cluster = ClusterTag::new([0xCC; 16]);
    let doc = DocTag::new([0xDD; 16]);

    let key = KeyBuf::new(
        Region::Data,
        tenant,
        coll,
        cluster,
        doc,
        Family::Doc,
        0x12345678,
        0x0102030405060708,
    );

    let bytes = key.as_bytes();
    assert_eq!(bytes[0], 0x01); // Region::Data
    assert_eq!(&bytes[1..17], &[0xAA; 16]); // tenant
    assert_eq!(&bytes[17..33], &[0xBB; 16]); // coll
    assert_eq!(&bytes[33..49], &[0xCC; 16]); // cluster
    assert_eq!(&bytes[49..65], &[0xDD; 16]); // doc
    assert_eq!(bytes[65], 0x01); // Family::Doc
    assert_eq!(&bytes[66..70], &[0x12, 0x34, 0x56, 0x78]); // sub (be)
                                                           // !commit_seq (bitwise not of 0x0102030405060708 = 0xFEFDFCFBFAF9F8F7)
    assert_eq!(
        &bytes[70..78],
        &[0xFE, 0xFD, 0xFC, 0xFB, 0xFA, 0xF9, 0xF8, 0xF7]
    );

    // Region codes nach SPEC 9.1
    assert_eq!(Region::Data.as_u8(), 0x01);
    assert_eq!(Region::Meta.as_u8(), 0x03);

    // Familien codes nach SPEC 9.2
    assert_eq!(Family::Doc.as_u8(), 0x01);
    assert_eq!(Family::DocMap.as_u8(), 0x02);
    assert_eq!(Family::Manifest.as_u8(), 0x03);
    assert_eq!(Family::TextBuf.as_u8(), 0x05);
}

// Proptests für alle Schlüsselfamilien und Regionen
proptest! {
    #[test]
    fn prop_key_commit_seq_ordering(
        sub in proptest::num::u32::ANY,
        seq1 in 0u64..u64::MAX - 1,
        step in 1u64..1_000_000,
    ) {
        let seq2 = seq1 + step;
        let tenant = TenantTag::new([1u8; 16]);
        let coll = CollTag::new([2u8; 16]);
        let cluster = ClusterTag::new([3u8; 16]);
        let doc = DocTag::new([4u8; 16]);

        for fam in [Family::Doc, Family::DocMap, Family::Manifest, Family::TextBuf] {
            let k1 = KeyBuf::new(Region::Data, tenant, coll, cluster, doc, fam, sub, seq1);
            let k2 = KeyBuf::new(Region::Data, tenant, coll, cluster, doc, fam, sub, seq2);

            prop_assert_eq!(k1.commit_seq(), seq1);
            prop_assert_eq!(k2.commit_seq(), seq2);
            // Higher sequence number must sort strictly LESS (newest first)
            prop_assert!(k2 < k1, "k2 (newer) must sort before k1 (older)");
        }
    }

    #[test]
    fn prop_key_bounds_and_range_containment(
        seq in proptest::num::u64::ANY,
        sub in proptest::num::u32::ANY,
        fam_idx in 0usize..4usize,
    ) {
        #[allow(clippy::indexing_slicing)]
        let families = [Family::Doc, Family::DocMap, Family::Manifest, Family::TextBuf];
        let fam = families.get(fam_idx).copied().unwrap_or(Family::Doc);

        let tenant = TenantTag::new([10u8; 16]);
        let coll = CollTag::new([20u8; 16]);
        let cluster = ClusterTag::new([30u8; 16]);
        let doc = DocTag::new([40u8; 16]);

        let key = KeyBuf::new(Region::Data, tenant, coll, cluster, doc, fam, sub, seq);
        let range_doc = KeyRange::doc(Region::Data, tenant, coll, cluster, doc);
        let range_cluster = KeyRange::cluster(Region::Data, tenant, coll, cluster);
        let range_coll = KeyRange::collection(Region::Data, tenant, coll);
        let range_tenant = KeyRange::tenant(Region::Data, tenant);

        prop_assert!(range_doc.contains_key(&key));
        prop_assert!(range_cluster.contains_key(&key));
        prop_assert!(range_coll.contains_key(&key));
        prop_assert!(range_tenant.contains_key(&key));
    }
}
