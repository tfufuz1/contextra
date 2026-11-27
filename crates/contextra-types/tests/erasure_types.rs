#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss
)]

use contextra_types::{
    ContextraError, ContextraErrorDto, Cursor, EraseTarget, ErasureError, ErasureException,
    ErasureId, ErasureState, ErasureStatus, Guarantee, LayerErrorKind, LayerState, LayerStatus,
    TargetKind, TenantId,
};
use proptest::prelude::*;

fn tenant(id: u64) -> TenantId {
    TenantId::try_new(id).unwrap()
}

/// TST-TYPES-01: `validate`: alle Verletzungen aus §4.2, inklusive `Collection{default}`.
#[test]
fn tst_types_01_validate_rules() {
    // Valid cases
    assert!(EraseTarget::Document {
        tenant: tenant(1),
        collection: "default".to_string(),
        key: "doc-1".to_string(),
    }
    .validate()
    .is_ok());

    assert!(EraseTarget::Collection {
        tenant: tenant(1),
        collection: "notes".to_string(),
    }
    .validate()
    .is_ok());

    assert!(EraseTarget::Tenant { tenant: tenant(1) }.validate().is_ok());

    // Empty collection
    assert!(matches!(
        EraseTarget::Document {
            tenant: tenant(1),
            collection: String::new(),
            key: "k".to_string(),
        }
        .validate(),
        Err(ErasureError::InvalidTarget(_))
    ));
    assert!(matches!(
        EraseTarget::Collection {
            tenant: tenant(1),
            collection: String::new(),
        }
        .validate(),
        Err(ErasureError::InvalidTarget(_))
    ));

    // Collection > 255 bytes
    assert!(matches!(
        EraseTarget::Collection {
            tenant: tenant(1),
            collection: "a".repeat(256),
        }
        .validate(),
        Err(ErasureError::InvalidTarget(_))
    ));
    assert!(EraseTarget::Collection {
        tenant: tenant(1),
        collection: "a".repeat(255),
    }
    .validate()
    .is_ok());

    // Collection with NUL byte
    assert!(matches!(
        EraseTarget::Collection {
            tenant: tenant(1),
            collection: "a\0b".to_string(),
        }
        .validate(),
        Err(ErasureError::InvalidTarget(_))
    ));

    // Reserved __ prefix
    assert!(matches!(
        EraseTarget::Document {
            tenant: tenant(1),
            collection: "__internal".to_string(),
            key: "k".to_string(),
        }
        .validate(),
        Err(ErasureError::InvalidTarget(_))
    ));
    assert!(matches!(
        EraseTarget::Collection {
            tenant: tenant(1),
            collection: "__sys".to_string(),
        }
        .validate(),
        Err(ErasureError::InvalidTarget(_))
    ));

    // Default collection cannot be erased as collection (K-5)
    assert!(matches!(
        EraseTarget::Collection {
            tenant: tenant(1),
            collection: "default".to_string(),
        }
        .validate(),
        Err(ErasureError::InvalidTarget(
            "default collection cannot be erased as collection"
        ))
    ));

    // Empty key
    assert!(matches!(
        EraseTarget::Document {
            tenant: tenant(1),
            collection: "c".to_string(),
            key: String::new(),
        }
        .validate(),
        Err(ErasureError::InvalidTarget(_))
    ));

    // Key > 4096 bytes
    assert!(matches!(
        EraseTarget::Document {
            tenant: tenant(1),
            collection: "c".to_string(),
            key: "x".repeat(4097),
        }
        .validate(),
        Err(ErasureError::InvalidTarget(_))
    ));
    assert!(EraseTarget::Document {
        tenant: tenant(1),
        collection: "c".to_string(),
        key: "x".repeat(4096),
    }
    .validate()
    .is_ok());

    // Key with NUL byte
    assert!(matches!(
        EraseTarget::Document {
            tenant: tenant(1),
            collection: "c".to_string(),
            key: "k\0v".to_string(),
        }
        .validate(),
        Err(ErasureError::InvalidTarget(_))
    ));
}

fn arb_target() -> impl Strategy<Value = EraseTarget> {
    let tid = (1u64..=100).prop_map(tenant);
    let s1 = "[a-z\0]{0,16}";
    let s2 = "[a-z\0]{0,16}";
    prop_oneof![
        (tid.clone(), s1, s2).prop_map(|(t, c, k)| EraseTarget::Document {
            tenant: t,
            collection: c,
            key: k,
        }),
        (tid.clone(), s1).prop_map(|(t, c)| EraseTarget::Collection {
            tenant: t,
            collection: c,
        }),
        tid.prop_map(|t| EraseTarget::Tenant { tenant: t }),
    ]
}

proptest! {
    /// TST-TYPES-02: `canonical_bytes` ist injektiv über Tag, Tenant und Strings (inkl. `\0`).
    #[test]
    fn tst_types_02_canonical_bytes_injective(t1 in arb_target(), t2 in arb_target()) {
        if t1 != t2 {
            prop_assert_ne!(t1.canonical_bytes(), t2.canonical_bytes());
        } else {
            prop_assert_eq!(t1.canonical_bytes(), t2.canonical_bytes());
        }
    }
}

#[test]
fn tst_types_02_delimiter_collision_resistance() {
    let a = EraseTarget::Document {
        tenant: tenant(1),
        collection: "a\0b".to_string(),
        key: "c".to_string(),
    };
    let b = EraseTarget::Document {
        tenant: tenant(1),
        collection: "a".to_string(),
        key: "b\0c".to_string(),
    };
    assert_ne!(a.canonical_bytes(), b.canonical_bytes());
}

/// TST-TYPES-03: Golden-Test für `canonical_bytes`, `target_hash`, JSON-Feldnamen und `ContextraErrorDto`.
#[test]
fn tst_types_03_golden_bytes_and_json_contract() {
    let target = EraseTarget::Document {
        tenant: tenant(7),
        collection: "docs".to_string(),
        key: "k1".to_string(),
    };
    assert_eq!(target.kind(), TargetKind::Document);
    assert_eq!(target.tenant(), tenant(7));

    let expected_bytes: Vec<u8> = vec![
        1, // tag = Document
        7, 0, 0, 0, 0, 0, 0, 0, // tenant = 7 LE
        4, 0, 0, 0, b'd', b'o', b'c', b's', // collection = "docs"
        2, 0, 0, 0, b'k', b'1', // key = "k1"
    ];
    assert_eq!(target.canonical_bytes(), expected_bytes);

    let key = [0x42u8; 32];
    let expected_hash = *blake3::keyed_hash(&key, &expected_bytes).as_bytes();
    assert_eq!(target.target_hash(&key), expected_hash);

    // Golden JSON field names for Guarantee and ErasureStatus (§7)
    let g = Guarantee::new(true, true, false).unwrap();
    let g_json: serde_json::Value = serde_json::to_value(g).unwrap();
    assert_eq!(
        g_json,
        serde_json::json!({
            "gated": true,
            "purged": true,
            "crypto_erased": false,
        })
    );

    let status = ErasureStatus {
        id: ErasureId::from_u128(1),
        state: ErasureState::Sealed,
        reached: g,
        layers: vec![LayerStatus {
            layer: "store".to_string(),
            state: LayerState::Verified,
            attempt: 1,
            remaining: Some(0),
        }],
        exceptions: vec![ErasureException::OutOfScopeExternalCopies],
        requested_at: 100,
        gated_at: Some(200),
        sealed_at: Some(300),
        ledger_index: Some(42),
    };
    let s_json: serde_json::Value = serde_json::to_value(&status).unwrap();
    let obj = s_json.as_object().unwrap();
    for field in [
        "id",
        "state",
        "reached",
        "layers",
        "exceptions",
        "requested_at",
        "gated_at",
        "sealed_at",
        "ledger_index",
    ] {
        assert!(obj.contains_key(field), "missing field {field}");
    }

    // Golden DTO for ContextraError::Erased (§6.3)
    let dto = ContextraErrorDto::from(&ContextraError::Erased);
    assert_eq!(dto.kind, "ERASED");
    assert!(dto.details.is_none());
    assert_eq!(
        dto.message,
        "Operation rejected: target is subject to erasure"
    );
}

/// TST-TYPES-04: `Guarantee`-Invariante; `meet` ist kommutativ, assoziativ, idempotent.
#[test]
fn tst_types_04_guarantee_invariants_and_lattice() {
    assert!(Guarantee::new(false, true, false).is_err());
    assert!(Guarantee::new(false, false, true).is_err());
    assert!(Guarantee::new(false, true, true).is_err());

    let valid = [
        (Guarantee::new(false, false, false).unwrap(), "G0"),
        (Guarantee::new(true, false, false).unwrap(), "G1"),
        (Guarantee::new(true, true, false).unwrap(), "G1+G3"),
        (Guarantee::new(true, false, true).unwrap(), "G1+G2"),
        (Guarantee::new(true, true, true).unwrap(), "G1+G2+G3"),
    ];
    for (g, lbl) in &valid {
        assert_eq!(g.label(), *lbl);
    }

    for (a, _) in &valid {
        // Idempotent
        assert_eq!(a.meet(*a), *a);
        for (b, _) in &valid {
            // Commutative
            assert_eq!(a.meet(*b), b.meet(*a));
            // Invariant preserved
            let m = a.meet(*b);
            assert!(!m.purged || m.gated);
            assert!(!m.crypto_erased || m.gated);
            for (c, _) in &valid {
                // Associative
                assert_eq!(a.meet(*b).meet(*c), a.meet(b.meet(*c)));
            }
        }
    }
}

/// TST-TYPES-05: Übergangsmatrix gleich §5 (alle 11×11 Paare).
#[test]
fn tst_types_05_state_transition_matrix() {
    let states = [
        ErasureState::Requested,
        ErasureState::Planned,
        ErasureState::Gating,
        ErasureState::Gated,
        ErasureState::KeysDestroying,
        ErasureState::Purging,
        ErasureState::Verifying,
        ErasureState::Sealed,
        ErasureState::Completed,
        ErasureState::Failed {
            layer: Some("store".to_string()),
            attempt: 1,
        },
        ErasureState::Quarantined {
            layer: "store".to_string(),
        },
    ];

    // Indices:
    // 0: Requested
    // 1: Planned
    // 2: Gating
    // 3: Gated
    // 4: KeysDestroying
    // 5: Purging
    // 6: Verifying
    // 7: Sealed
    // 8: Completed
    // 9: Failed
    // 10: Quarantined
    let expected_allowed: [[bool; 11]; 11] = [
        // To: Req, Pla, Gat, Gtd, Key, Prg, Ver, Sea, Cmp, Fai, Qua
        [false, true, false, false, false, false, false, false, false, true, false], // 0 Requested
        [false, false, true, false, false, false, false, false, false, true, false], // 1 Planned
        [false, false, false, true, false, false, false, false, false, true, false], // 2 Gating
        [false, false, false, false, true, true, false, false, false, false, false], // 3 Gated
        [false, false, false, false, false, true, false, false, false, false, false], // 4 KeysDestroying
        [false, false, false, false, false, false, true, false, false, true, false], // 5 Purging
        [false, false, false, false, false, true, false, true, false, true, false], // 6 Verifying
        [false, false, false, false, false, false, false, false, true, false, false], // 7 Sealed
        [false, false, false, false, false, false, false, false, false, false, false], // 8 Completed
        [false, false, false, false, false, true, true, false, false, false, true], // 9 Failed
        [false, false, false, false, false, true, false, false, false, false, false], // 10 Quarantined
    ];

    for (from, row) in states.iter().zip(expected_allowed.iter()) {
        for (to, &allowed) in states.iter().zip(row.iter()) {
            assert_eq!(
                from.can_transition_to(to),
                allowed,
                "mismatch for transition {:?} -> {:?}",
                from,
                to
            );
        }
    }

    assert!(!ErasureState::Requested.is_terminal());
    assert!(!ErasureState::Sealed.is_terminal());
    assert!(ErasureState::Completed.is_terminal());

    assert!(!ErasureState::Requested.gate_active());
    assert!(!ErasureState::Planned.gate_active());
    assert!(ErasureState::Gating.gate_active());
    assert!(ErasureState::Gated.gate_active());
    assert!(ErasureState::KeysDestroying.gate_active());
    assert!(ErasureState::Purging.gate_active());
    assert!(ErasureState::Verifying.gate_active());
    assert!(ErasureState::Sealed.gate_active());
    assert!(ErasureState::Quarantined {
        layer: "store".to_string()
    }
    .gate_active());
    assert!(!ErasureState::Completed.gate_active());

    assert_eq!(ErasureState::KeysDestroying.reached(), ErasureState::Gated.reached());
    assert!(ErasureState::Sealed.reached().purged);
    assert!(ErasureState::Completed.reached().purged);
}

/// TST-TYPES-06: `ErasureStatus`-JSON enthält keine Schlüssel-/Namensbytes.
#[test]
fn tst_types_06_status_json_no_target_bytes() {
    let status = ErasureStatus {
        id: ErasureId::from_u128(0xDEAD_BEEF),
        state: ErasureState::Purging,
        reached: Guarantee::new(true, false, false).unwrap(),
        layers: vec![LayerStatus {
            layer: "vector".to_string(),
            state: LayerState::Purging,
            attempt: 2,
            remaining: Some(10),
        }],
        exceptions: vec![],
        requested_at: 1,
        gated_at: Some(2),
        sealed_at: None,
        ledger_index: None,
    };
    let json = serde_json::to_string(&status).unwrap();
    assert!(!json.contains("secret_collection"));
    assert!(!json.contains("secret_key"));
}

/// TST-TYPES-07: `Debug`/`Display` enthalten keinen Zielinhalt.
#[test]
fn tst_types_07_debug_redacts_target_content() {
    let doc = EraseTarget::Document {
        tenant: tenant(7),
        collection: "super_secret_collection".to_string(),
        key: "super_secret_key_123".to_string(),
    };
    let dbg = format!("{doc:?}");
    assert!(!dbg.contains("super_secret_collection"));
    assert!(!dbg.contains("super_secret_key_123"));
    assert!(dbg.contains("[REDACTED]"));

    let coll = EraseTarget::Collection {
        tenant: tenant(7),
        collection: "another_secret_coll".to_string(),
    };
    let coll_dbg = format!("{coll:?}");
    assert!(!coll_dbg.contains("another_secret_coll"));
    assert!(coll_dbg.contains("[REDACTED]"));
}

/// TST-TYPES-08: `Cursor::new` lehnt 257 Byte ab.
#[test]
fn tst_types_08_cursor_max_len() {
    assert!(Cursor::new(vec![]).is_ok());
    let c256 = Cursor::new(vec![0xAB; Cursor::MAX_LEN]).unwrap();
    assert_eq!(c256.as_bytes().len(), 256);
    assert!(matches!(
        Cursor::new(vec![0xAB; Cursor::MAX_LEN + 1]),
        Err(ErasureError::InvalidCursor)
    ));
}

/// TST-TYPES-09: `ErasureError::kind` deckt alle Varianten ab (Exhaustivitätstest ohne `strum`).
#[test]
fn tst_types_09_erasure_error_kind_exhaustive() {
    let all_variants = [
        (
            ErasureError::InvalidTarget("bad"),
            LayerErrorKind::Permanent,
        ),
        (ErasureError::InvalidCursor, LayerErrorKind::Permanent),
        (ErasureError::IdempotencyConflict, LayerErrorKind::Permanent),
        (ErasureError::ScopeAmbiguous, LayerErrorKind::Permanent),
        (
            ErasureError::TenantViolation,
            LayerErrorKind::ScopeViolation,
        ),
        (ErasureError::NotFound, LayerErrorKind::Permanent),
        (ErasureError::InvalidTransition, LayerErrorKind::Permanent),
        (ErasureError::JournalPoisoned, LayerErrorKind::Poisoned),
        (ErasureError::LedgerCorrupt, LayerErrorKind::Permanent),
        (
            ErasureError::Busy { active: 4 },
            LayerErrorKind::Retryable,
        ),
        (
            ErasureError::InsufficientSpace {
                needed: 100,
                available: 10,
            },
            LayerErrorKind::Retryable,
        ),
        (
            ErasureError::Layer {
                layer: "store".to_string(),
                kind: LayerErrorKind::Retryable,
            },
            LayerErrorKind::Permanent,
        ),
        (ErasureError::Cancelled, LayerErrorKind::Cancelled),
        (
            ErasureError::Io("disk".to_string()),
            LayerErrorKind::Retryable,
        ),
    ];

    for (err, expected_kind) in all_variants {
        assert_eq!(err.kind(), expected_kind);
        let ctx_err: ContextraError = err.into();
        let _ = ctx_err.to_string();
    }
}

/// TST-TYPES-10: `ErasureId::from_u128(x).as_u128() == x`; `Display` 32 Zeichen; `Debug` <= 4 Byte.
#[test]
fn tst_types_10_erasure_id_u128_display_debug() {
    let val: u128 = 0x0123_4567_89ab_cdef_fedc_ba98_7654_3210;
    let id = ErasureId::from_u128(val);
    assert_eq!(id.as_u128(), val);

    let disp = format!("{id}");
    assert_eq!(disp.len(), 32);
    assert_eq!(disp, "0123456789abcdeffedcba9876543210");

    let dbg = format!("{id:?}");
    assert_eq!(dbg, "ErasureId(01234567..)");
    assert!(!dbg.contains("89ab"));
}
