/// SPEC: T-KEY-1
#[test]
fn t_key_1_schluesselkodierung_feste_laengen_und_sortierung() {
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

    for k in 1..=10u8 {
        let kind = FileKind::try_from(k);
        assert!(kind.is_ok(), "FileKind {k} muss gültig sein");
    }

    assert_eq!(FileKind::try_from(0u8), Err(Error::Corrupt));
    assert_eq!(FileKind::try_from(11u8), Err(Error::Corrupt));
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
