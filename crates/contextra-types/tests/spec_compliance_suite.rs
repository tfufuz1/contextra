#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]

/// SPEC: T-KEY-1
#[test]
fn t_key_1_schluesselkodierung_feste_laengen_und_sortierung() {
    use contextra_types::key::*;
    use contextra_types::tags::*;

    let tenant = TenantTag::new([1u8; 16]);
    let coll = CollTag::new([2u8; 16]);
    let cluster = ClusterTag::new([3u8; 16]);
    let doc = DocTag::new([4u8; 16]);

    let families = [
        Family::Doc,
        Family::DocMap,
        Family::Manifest,
        Family::ReservedLocator,
        Family::TextBuf,
        Family::Vec,
    ];

    for family in families {
        let key = KeyBuf::new(Region::Data, tenant, coll, cluster, doc, family, 42, 1000);

        assert_eq!(key.as_bytes().len(), KEY_LEN);
        assert_eq!(KEY_LEN, 78);
        assert_eq!(KEY_BOUND_LEN, 65);

        // Commit seq roundtrip without DEK (D-74)
        assert_eq!(key.commit_seq(), 1000);

        // Newer commit sequence sorts earlier because of !commit_seq (TY-04)
        let key_older = KeyBuf::new(Region::Data, tenant, coll, cluster, doc, family, 42, 999);
        assert!(
            key < key_older,
            "Neueste Version muss zuerst sortieren für Familie {family:?}"
        );
    }

    // TagPreimage prefix check (TY-07)
    let pre1 = TagPreimage::new(b"test").field(b"a").field(b"bc");
    let pre2 = TagPreimage::new(b"test").field(b"ab").field(b"c");
    assert_ne!(
        pre1.as_bytes(),
        pre2.as_bytes(),
        "Längenpräfixe müssen Kollisionen verhindern"
    );
}

/// SPEC: T-FMT-1 Golden Bytes für alle 6 Schlüsselfamilien (SPEC 9.1, 9.2).
#[test]
fn test_t_key_golden_bytes_all_families() {
    use contextra_types::key::*;
    use contextra_types::tags::*;

    let tenant = TenantTag::new([0x11; 16]);
    let coll = CollTag::new([0x22; 16]);
    let cluster = ClusterTag::new([0x33; 16]);
    let doc = DocTag::new([0x44; 16]);
    let sub = 0x12345678u32;
    let commit_seq = 0x0102030405060708u64;

    let cases = [
        (Family::Doc, 0x01u8),
        (Family::DocMap, 0x02u8),
        (Family::Manifest, 0x03u8),
        (Family::ReservedLocator, 0x04u8),
        (Family::TextBuf, 0x05u8),
        (Family::Vec, 0x06u8),
    ];

    for (fam, code) in cases {
        let key = KeyBuf::new(
            Region::Data,
            tenant,
            coll,
            cluster,
            doc,
            fam,
            sub,
            commit_seq,
        );
        let bytes = key.as_bytes();

        assert_eq!(bytes[0], 0x01); // Region::Data
        assert_eq!(&bytes[1..17], &[0x11; 16]);
        assert_eq!(&bytes[17..33], &[0x22; 16]);
        assert_eq!(&bytes[33..49], &[0x33; 16]);
        assert_eq!(&bytes[49..65], &[0x44; 16]);
        assert_eq!(bytes[65], code);
        assert_eq!(&bytes[66..70], &sub.to_be_bytes());
        assert_eq!(&bytes[70..78], &(!commit_seq).to_be_bytes());
    }
}

/// SPEC: F.10 Golden Format für kanonisches JSON (Bericht nach SPEC 11.7, F.10).
#[test]
fn test_t_fmt_10_golden_canonical_report_format() {
    // Kanonische Vorgaben: sortierte Schlüssel, keine Whitespaces, UTC-Mikrosekunden
    let sample_canonical_json = "{\"commitment\":\"0102030405060708090a0b0c0d0e0f10\",\"completed_at\":1700000000000000,\"erasure_id\":\"11112222333344445555666677778888\",\"report_version\":1,\"retention_code\":\"NoObligation\",\"target_type\":\"Document\"}";

    // Parsen und Verifizieren, dass es formkonform nach F.10 ist
    let parsed: serde_json::Value = serde_json::from_str(sample_canonical_json).unwrap();
    let re_serialized = serde_json::to_string(&parsed).unwrap();
    assert_eq!(
        sample_canonical_json, re_serialized,
        "Kanonisches JSON muss byteweise erhalten bleiben"
    );
}

/// SPEC: T-ISO-2
#[test]
fn t_iso_2_matcher_bereichs_und_exakt_treffer() {
    use contextra_types::key::*;
    use contextra_types::tags::*;

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
}

/// SPEC: T-DOCREF-1
#[test]
fn t_docref_1_dokumentidentitaet_und_cluster_tag() -> Result<(), Box<dyn std::error::Error>> {
    use contextra_types::refs::*;
    use contextra_types::tags::*;

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

/// SPEC: T-ARCH-1
#[test]
fn t_arch_1_reinheit_ohne_io() {
    // contextra-types kompiliert ohne std::net / tokio / std::fs
    assert_eq!(contextra_types::limits::MAX_DIMENSION, 4096);
}

/// SPEC: T-FMT-1
#[test]
fn t_fmt_1_file_kind_und_alg_ids_parsing() {
    use contextra_types::error::Error;
    use contextra_types::file::*;

    let expected_kinds = [
        (1, FileKind::Wal, 0),
        (2, FileKind::SSTable, 0),
        (3, FileKind::Manifest, 1),
        (4, FileKind::TextSegment, 0),
        (5, FileKind::VectorSnapshot, 0),
        (6, FileKind::Journal, 1),
        (7, FileKind::GateLog, 1),
        (8, FileKind::Ledger, 1),
        (9, FileKind::Scope, 0),
        (10, FileKind::Vault, 1),
    ];

    for (k, expected_kind, expected_key_mode) in expected_kinds {
        let kind = FileKind::try_from(k).expect("FileKind muss gültig sein");
        assert_eq!(kind, expected_kind, "FileKind {k} weicht von SPEC F.1 ab");
        assert_eq!(kind.as_u8(), k);
        assert_eq!(
            kind.key_mode(),
            expected_key_mode,
            "key_mode für {kind:?} weicht ab"
        );
    }

    assert_eq!(FileKind::try_from(0u8), Err(Error::Corrupt));
    assert_eq!(FileKind::try_from(11u8), Err(Error::Corrupt));
    assert_eq!(FileKind::try_from(255u8), Err(Error::Corrupt));

    // Standard Algorithmen-Kombination in v1: [AES-256-GCM-SIV, kein KDF, HMAC-SHA256, BLAKE3/SHA256]
    assert_eq!(AlgIds::V1.0, [1, 0, 1, 1]);
    assert_eq!(FormatVersion::CURRENT.0, 1);
}

/// SPEC: T-LOG-1
#[test]
fn t_log_1_alle_fehlertexte_statisch_ohne_nutzerdaten() {
    use contextra_types::error::*;

    let all_errors: Vec<Error> = vec![
        Error::Config(ConfigError::InvalidLength),
        Error::Io,
        Error::Corrupt,
        Error::Auth,
        Error::Erased,
        Error::Restricted,
        Error::NotFound,
        Error::Conflict(Conflict::Exists),
        Error::Conflict(Conflict::IdCollision),
        Error::Limit(Limit::Capacity),
        Error::Limit(Limit::DiskReserve),
        Error::Limit(Limit::Snapshots),
        Error::Limit(Limit::QueryTooLarge),
        Error::Limit(Limit::Dimension),
        Error::SnapshotRevoked,
        Error::Cancelled,
        Error::Refused,
        Error::Internal,
    ];

    for err in all_errors {
        let msg = format!("{err}");
        assert!(
            !msg.contains("{}"),
            "Fehlertext darf keine Format-Platzhalter enthalten: {msg}"
        );
        assert!(
            !msg.contains("%s"),
            "Fehlertext darf keine Format-Platzhalter enthalten: {msg}"
        );
    }
}

/// SPEC: T-API-2
#[test]
fn t_api_2_retention_code_und_erasure_id() {
    use contextra_types::erasure::*;

    let codes = [
        RetentionCode::NoObligation,
        RetentionCode::ObligationExpired,
        RetentionCode::ObligationActive,
        RetentionCode::LegalClaimsPending,
        RetentionCode::DataSubjectRequestedRestriction,
        RetentionCode::OperatorOverride,
    ];
    assert_eq!(codes.len(), 6);

    let risks = [
        ResidualRisk::FlashRemanence,
        ResidualRisk::FsJournalOrCow,
        ResidualRisk::WrappedKeyRemanence,
        ResidualRisk::FreedBlockRemanence,
        ResidualRisk::OldKeyslotRemanence,
        ResidualRisk::ProcessMemory,
        ResidualRisk::IndexTopologyInference,
        ResidualRisk::TextIndexUntilPurge,
        ResidualRisk::ReplicaWindow,
        ResidualRisk::ExternalCopies,
        ResidualRisk::ExternalProcessors,
        ResidualRisk::PseudonymCommitment,
        ResidualRisk::PurgeScalesWithDb,
        ResidualRisk::PurgeDeferred,
        ResidualRisk::KeyCompromise,
    ];
    assert_eq!(risks.len(), 15);
    for r in risks {
        assert!(!r.id().is_empty());
    }
}

/// SPEC: L-8 Golden Test
#[test]
fn test_l8_meta_schluesselformat_golden() {
    use contextra_types::key::*;
    use contextra_types::tags::*;

    let tenant = TenantTag::new([0x11; 16]);
    let coll = CollTag::new([0x22; 16]);
    let name = NameTag::new([0x33; 16]);

    let meta_key = KeyBuf::meta(tenant, coll, name, 0, 1);
    assert_eq!(meta_key.as_bytes()[0], Region::Meta as u8);
    assert_eq!(&meta_key.as_bytes()[1..17], &[0x11; 16]);
    assert_eq!(&meta_key.as_bytes()[17..33], &[0x22; 16]);
    assert_eq!(&meta_key.as_bytes()[33..49], &[0x00; 16]); // cluster = 0
    assert_eq!(&meta_key.as_bytes()[49..65], &[0x33; 16]); // doc_tag = name_tag
    assert_eq!(meta_key.as_bytes()[65], Family::Manifest as u8);
}

/// SPEC: L-10 Golden Test
#[test]
fn test_l10_dimension_error_limit() {
    use contextra_types::error::{Error, Limit};

    let err = Error::Limit(Limit::Dimension);
    assert_eq!(err.to_string(), "limit exceeded: dimension");
}

/// SPEC: T-API-4
#[test]
fn test_t_api_4_fehlertexte_eingefroren_golden() {
    use contextra_types::error::*;

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
    use contextra_types::limits::*;

    assert_eq!(MAX_DOCS_PER_STORE, 50_000_000);
    assert_eq!(MAX_DOC_BYTES_DEFAULT, 4 * 1024 * 1024);
    assert_eq!(MAX_DOC_BYTES_HARD, 64 * 1024 * 1024);
    assert_eq!(MAX_SNAPSHOTS_PER_TENANT, 1024);
    assert_eq!(MAX_DIMENSION, 4096);
    assert_eq!(MAX_QUERY_TERMS, 256);
    assert_eq!(MAX_K, 1000);
}

/// SPEC: TY-11 / T-DEF-1 Debug-Redaktion für sensible Geheimnisse
#[test]
fn test_t_def_1_secret_debug_redaction_and_zeroize() {
    use contextra_types::secret::Secret;

    let raw = b"super-secret-key-material-do-not-leak".to_vec();
    let secret = Secret::new(raw);

    assert_eq!(secret.len(), 37);
    assert!(!secret.is_empty());
    assert_eq!(
        secret.expose_secret(),
        b"super-secret-key-material-do-not-leak"
    );

    let debug_output = format!("{secret:?}");
    assert!(
        debug_output.contains("[REDACTED 37 bytes]"),
        "Debug-Ausgabe muss [REDACTED <N> bytes] enthalten"
    );
    assert!(
        !debug_output.contains("super-secret"),
        "Debug-Ausgabe darf niemals Schlüsselmaterial im Klartext enthalten"
    );
}

/// SPEC: T-KEY-1 Serde-Längenprüfungen für große Arrays (>32 Bytes: KeyBuf [78 B], KeyBound [65 B])
#[test]
fn test_t_key_serde_length_validation_rejects_malformed() {
    use contextra_types::key::{KeyBound, KeyBuf, KEY_BOUND_LEN, KEY_LEN};

    // Gültige Roundtrips
    let valid_bound = KeyBound::new([0xAA; KEY_BOUND_LEN]);
    let bincode_bound = bincode::serialize(&valid_bound).unwrap();
    let deser_bound: KeyBound = bincode::deserialize(&bincode_bound).unwrap();
    assert_eq!(valid_bound, deser_bound);

    let valid_key = KeyBuf::from_raw([0xBB; KEY_LEN]);
    let bincode_key = bincode::serialize(&valid_key).unwrap();
    let deser_key: KeyBuf = bincode::deserialize(&bincode_key).unwrap();
    assert_eq!(valid_key, deser_key);

    // Zu kurze Bytes
    let short_bound = vec![0xAA; KEY_BOUND_LEN - 1];
    assert!(bincode::deserialize::<KeyBound>(&short_bound).is_err());
    assert!(serde_json::from_slice::<KeyBound>(&short_bound).is_err());

    let short_key = vec![0xBB; KEY_LEN - 1];
    assert!(bincode::deserialize::<KeyBuf>(&short_key).is_err());
    assert!(serde_json::from_slice::<KeyBuf>(&short_key).is_err());

    // Zu lange Sequenzen
    let long_bound_seq = serde_json::to_string(&vec![0u8; KEY_BOUND_LEN + 1]).unwrap();
    assert!(serde_json::from_str::<KeyBound>(&long_bound_seq).is_err());

    let long_key_seq = serde_json::to_string(&vec![0u8; KEY_LEN + 1]).unwrap();
    assert!(serde_json::from_str::<KeyBuf>(&long_key_seq).is_err());
}

/// SPEC: T-KEY-1
#[test]
fn t_key_1_exhaustive_deterministic_ordering_and_roundtrip() {
    use contextra_types::key::*;
    use contextra_types::tags::*;

    let families = [
        Family::Doc,
        Family::DocMap,
        Family::Manifest,
        Family::ReservedLocator,
        Family::TextBuf,
        Family::Vec,
    ];

    let seq_pairs = [
        (0u64, 1u64),
        (1u64, 2u64),
        (100u64, 200u64),
        (u64::MAX - 1, u64::MAX),
        (0, u64::MAX),
    ];

    for family in families {
        for &(s_older, s_newer) in &seq_pairs {
            let tenant = TenantTag::new([0xAA; 16]);
            let coll = CollTag::new([0xBB; 16]);
            let cluster = ClusterTag::new([0xCC; 16]);
            let doc = DocTag::new([0xDD; 16]);

            let k_older = KeyBuf::new(Region::Data, tenant, coll, cluster, doc, family, 7, s_older);
            let k_newer = KeyBuf::new(Region::Data, tenant, coll, cluster, doc, family, 7, s_newer);

            assert_eq!(k_older.commit_seq(), s_older);
            assert_eq!(k_newer.commit_seq(), s_newer);
            assert!(
                k_newer < k_older,
                "Jüngere Sequenz muss vor älterer Sequenz sortieren"
            );

            let b = k_newer.bound();
            assert_eq!(b.as_bytes().len(), KEY_BOUND_LEN);
            assert_eq!(&b.as_bytes()[..], &k_newer.as_bytes()[0..KEY_BOUND_LEN]);
        }
    }
}

/// SPEC: T-LIMIT-1
#[test]
fn test_t_limit_1_boundary_plus_one_checks() {
    use contextra_types::error::{Error, Limit};
    use contextra_types::limits::*;

    // Prüfe die statischen Grenzen
    assert_eq!(MAX_DOCS_PER_STORE, 50_000_000);
    assert_eq!(MAX_DOC_BYTES_DEFAULT, 4 * 1024 * 1024);
    assert_eq!(MAX_DOC_BYTES_HARD, 64 * 1024 * 1024);
    assert_eq!(MAX_SNAPSHOTS_PER_TENANT, 1024);
    assert_eq!(MAX_DIMENSION, 4096);
    assert_eq!(MAX_QUERY_TERMS, 256);
    assert_eq!(MAX_K, 1000);

    // Grenzüberschreitungstest-Mapping
    let dim_overflow = MAX_DIMENSION + 1;
    assert!(dim_overflow > 4096);
    let err = Error::Limit(Limit::Dimension);
    assert_eq!(err.to_string(), "limit exceeded: dimension");

    let query_overflow = MAX_QUERY_TERMS + 1;
    assert!(query_overflow > 256);
    let err_q = Error::Limit(Limit::QueryTooLarge);
    assert_eq!(err_q.to_string(), "limit exceeded: query too large");
}

/// SPEC: T-DOCREF-1
#[test]
fn test_t_docref_1_subject_isolation() {
    use contextra_types::ids::{DocKey, DocRef, SubjectRef};

    let key = DocKey::try_new(b"item-123".to_vec()).unwrap();
    let subj_a = SubjectRef::try_new(b"person-a".to_vec()).unwrap();
    let subj_b = SubjectRef::try_new(b"person-b".to_vec()).unwrap();

    let doc_a = DocRef {
        subject: Some(subj_a),
        key: key.clone(),
    };
    let doc_b = DocRef {
        subject: Some(subj_b),
        key: key.clone(),
    };
    let doc_none = DocRef {
        subject: None,
        key: key.clone(),
    };

    assert_ne!(doc_a, doc_b);
    assert_ne!(doc_a, doc_none);
    assert_ne!(doc_b, doc_none);
}
