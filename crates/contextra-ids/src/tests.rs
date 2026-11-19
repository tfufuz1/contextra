//! Unit-Tests für `contextra-ids`: Pflichtkatalog AGENTS.md §3 (T-ID1 bis T-ID6)
//! sowie Redaktions-, Serde- und Mandanten-Invarianten.
//!
//! Die Tests sind bewusst ohne `unwrap`/`expect`/Indexierung formuliert, damit
//! der Workspace-Lint-Satz auch für Testcode ohne Ausnahmen gilt.

use super::*;
use rand::RngCore;
use zeroize::{Zeroize, ZeroizeOnDrop};

/// Zufallsquelle, die jeden Aufruf fehlschlagen lässt (simulierte Entropie-Erschöpfung).
struct FailingRng;

impl RngCore for FailingRng {
    fn next_u32(&mut self) -> u32 {
        0
    }

    fn next_u64(&mut self) -> u64 {
        0
    }

    fn fill_bytes(&mut self, dest: &mut [u8]) {
        dest.fill(0);
    }

    fn try_fill_bytes(&mut self, _dest: &mut [u8]) -> core::result::Result<(), rand::Error> {
        Err(rand::Error::new(std::io::Error::other(
            "simulated entropy failure",
        )))
    }
}

fn is_entropy_error(err: &Option<IdError>) -> bool {
    matches!(err, Some(IdError::EntropyExhaustion(msg)) if msg.contains("simulated"))
}

// ─── T-ID1: RNG-Erschöpfung ──────────────────────────────────────────────────

#[test]
fn t_id1_subject_ref_reports_entropy_exhaustion() {
    assert!(is_entropy_error(
        &SubjectRef::from_rng(&mut FailingRng).err()
    ));
}

#[test]
fn t_id1_personal_ref_reports_entropy_exhaustion() {
    assert!(is_entropy_error(
        &PersonalRef::from_rng(&mut FailingRng).err()
    ));
}

#[test]
fn t_id1_erasure_id_reports_entropy_exhaustion() {
    assert!(is_entropy_error(
        &ErasureId::from_rng(&mut FailingRng).err()
    ));
}

#[test]
fn t_id1_generate_uses_os_rng_and_yields_distinct_values() {
    let a = SubjectRef::generate().ok();
    let b = SubjectRef::generate().ok();
    assert!(a.is_some() && b.is_some());
    assert_ne!(a, b);
    assert!(PersonalRef::generate().is_ok());
    assert!(ErasureId::generate().is_ok());
}

// ─── T-ID2: Zeroize bei Drop / kein Klartext in Traces ───────────────────────

fn assert_zeroize_on_drop<T: ZeroizeOnDrop>() {}

#[test]
fn t_id2_subject_ref_is_zeroize_on_drop() {
    assert_zeroize_on_drop::<SubjectRef>();
}

#[test]
fn t_id2_subject_ref_zeroizes_contents() {
    let mut s = SubjectRef::from_bytes([0xAB; 16]);
    s.zeroize();
    assert_eq!(s.as_bytes(), &[0u8; 16]);
}

#[test]
fn t_id2_subject_ref_debug_and_display_are_redacted() {
    let s = SubjectRef::from_bytes([0x41; 16]);
    let shown = format!("{s} {s:?}");
    assert_eq!(shown, "SubjectRef([REDACTED]) SubjectRef([REDACTED])");
}

#[test]
fn t_id2_personal_ref_debug_and_display_are_redacted() {
    let p = PersonalRef::from_bytes([0x41; 16]);
    let shown = format!("{p} {p:?}");
    assert_eq!(shown, "PersonalRef([REDACTED]) PersonalRef([REDACTED])");
}

// ─── T-ID3: BLAKE3-Präfix-Determinismus ──────────────────────────────────────

#[cfg(not(feature = "docid-128"))]
#[test]
fn t_id3_doc_id_from_key_is_blake3_prefix() {
    let hash = blake3::hash(b"doc:1");
    let mut prefix = [0u8; 8];
    prefix.copy_from_slice(&hash.as_bytes()[..8]);
    let expected = DocId(u64::from_le_bytes(prefix));
    assert_eq!(DocId::from_key("doc:1").ok(), Some(expected));
}

#[cfg(feature = "docid-128")]
#[test]
fn t_id3_doc_id_from_key_is_blake3_prefix() {
    let hash = blake3::hash(b"doc:1");
    let mut prefix = [0u8; 16];
    prefix.copy_from_slice(&hash.as_bytes()[..16]);
    let expected = DocId(u128::from_le_bytes(prefix));
    assert_eq!(DocId::from_key("doc:1").ok(), Some(expected));
}

#[test]
fn t_id3_doc_id_from_key_is_deterministic_and_rejects_empty() {
    assert_eq!(DocId::from_key("same").ok(), DocId::from_key("same").ok());
    assert_eq!(DocId::from_key("").err(), Some(IdError::EmptyKey));
}

// ─── T-ID4: TenantId(0) ist für SYSTEM reserviert ────────────────────────────

#[test]
fn t_id4_tenant_zero_is_reserved() {
    assert_eq!(
        TenantId::try_new(0).err(),
        Some(IdError::TenantSystemReserved)
    );
    assert!(TenantId::SYSTEM.is_system());
    assert_eq!(TenantId::try_new(7).ok(), Some(TenantId(7)));
}

// ─── T-ID5: CollectionId(0) wird abgelehnt ───────────────────────────────────

#[test]
fn t_id5_collection_zero_is_rejected() {
    assert_eq!(
        CollectionId::try_new(0).err(),
        Some(IdError::CollectionZero)
    );
    assert_eq!(CollectionId::try_new(3).ok(), Some(CollectionId(3)));
}

// ─── T-ID6: Layout-Garantien zur Laufzeit ────────────────────────────────────

#[test]
fn t_id6_doc_id_width_matches_feature() {
    assert_eq!(core::mem::size_of::<DocId>() * 8, ID_WIDTH_BITS as usize);
    assert_eq!(
        core::mem::align_of::<DocId>(),
        core::mem::size_of::<DocId>().min(16)
    );
}

// ─── TenantScoped: geprüfte Konstruktion, Isolation, Serde ───────────────────

#[test]
fn tenant_scoped_try_new_rejects_system_tenant() {
    let rejected = TenantScoped::try_new(TenantId::SYSTEM, 1u32).err();
    assert_eq!(rejected, Some(TenantScopeViolation::InvalidZero));
}

#[test]
fn tenant_scoped_mismatch_is_reported() {
    let scoped = TenantScoped::new(TenantId(1), "payload");
    let err = scoped.into_inner_checked(&TenantId(2)).err();
    assert_eq!(
        err,
        Some(TenantScopeViolation::Mismatch {
            expected: TenantId(2),
            actual: TenantId(1),
        })
    );
}

#[test]
fn tenant_scoped_serde_roundtrip_preserves_binding() {
    let scoped = TenantScoped::new(TenantId(9), 42u32);
    let json = serde_json::to_string(&scoped).unwrap_or_default();
    let back: core::result::Result<TenantScoped<u32>, _> = serde_json::from_str(&json);
    assert_eq!(back.ok(), Some(scoped));
}

// ─── ErasureId: Hex-Darstellung ──────────────────────────────────────────────

#[test]
fn erasure_id_display_is_full_hex() {
    let shown = format!("{}", ErasureId::new([0xAB; 16]));
    assert_eq!(shown.len(), 32);
    assert!(shown.chars().all(|c| c.is_ascii_hexdigit()));
}
