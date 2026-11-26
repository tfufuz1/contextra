//! Trust, taint, grounding and belief types (SPEC-02 §2.3; SPEC §8.1).
//!
//! Inheritance rules are implemented here as pure functions, so every crate
//! applies the same rule:
//! - [`TrustTier`] of a derived record is the **minimum** of its sources.
//! - [`TaintFlags`] of a derived record is the **union** of its sources.

use core::fmt;
use core::ops::{BitOr, BitOrAssign};

/// Trust tier of a record. Ordering: `Untrusted < … < Verified`.
///
/// The SPEC lists `Untrusted/Hypothesis` as the lowest tier. `Hypothesis` is
/// modelled as a grounding level ([`GroundingLevel::Hypothesis`], G3), not as a
/// trust tier, because a hypothesis can carry any trust tier.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TrustTier {
    /// Untrusted input. Lowest tier.
    Untrusted,
    /// Inferred by an agent.
    AgentInferred,
    /// Verified by a tool.
    ToolVerified,
    /// Curated by a human.
    Curated,
    /// Verified. Highest tier.
    Verified,
}

impl TrustTier {
    /// Trust of a derived record: the minimum over its sources. `None` for no sources.
    pub fn inherit<I>(sources: I) -> Option<TrustTier>
    where
        I: IntoIterator<Item = TrustTier>,
    {
        sources.into_iter().min()
    }
}

/// Taint flags, combinable. Stored as a bit set.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct TaintFlags(u8);

impl TaintFlags {
    /// No taint.
    pub const NONE: TaintFlags = TaintFlags(0);
    /// Content originates outside the trust boundary.
    pub const EXTERNAL: TaintFlags = TaintFlags(1 << 0);
    /// Injection scan flagged this content.
    pub const INJECTION_SUSPECT: TaintFlags = TaintFlags(1 << 1);
    /// Produced by a tool whose output was not verified.
    pub const UNVERIFIED_TOOL: TaintFlags = TaintFlags(1 << 2);
    /// Contains personal data.
    pub const PII: TaintFlags = TaintFlags(1 << 3);

    const KNOWN: u8 = 0b0000_1111;

    /// Bit representation.
    #[inline]
    pub const fn bits(self) -> u8 {
        self.0
    }

    /// Builds flags from bits. `None` if any unknown bit is set.
    #[inline]
    pub const fn from_bits(bits: u8) -> Option<TaintFlags> {
        if bits & !Self::KNOWN == 0 {
            Some(TaintFlags(bits))
        } else {
            None
        }
    }

    /// `true` if no flag is set.
    #[inline]
    pub const fn is_empty(self) -> bool {
        self.0 == 0
    }

    /// `true` if every flag in `other` is set in `self`.
    #[inline]
    pub const fn contains(self, other: TaintFlags) -> bool {
        self.0 & other.0 == other.0
    }

    /// Taint of a derived record: union over its sources.
    pub fn inherit<I>(sources: I) -> TaintFlags
    where
        I: IntoIterator<Item = TaintFlags>,
    {
        sources.into_iter().fold(TaintFlags::NONE, BitOr::bitor)
    }
}

impl BitOr for TaintFlags {
    type Output = TaintFlags;
    #[inline]
    fn bitor(self, rhs: TaintFlags) -> TaintFlags {
        TaintFlags(self.0 | rhs.0)
    }
}

impl BitOrAssign for TaintFlags {
    #[inline]
    fn bitor_assign(&mut self, rhs: TaintFlags) {
        self.0 |= rhs.0;
    }
}

impl fmt::Debug for TaintFlags {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let names = [
            (Self::EXTERNAL, "EXTERNAL"),
            (Self::INJECTION_SUSPECT, "INJECTION_SUSPECT"),
            (Self::UNVERIFIED_TOOL, "UNVERIFIED_TOOL"),
            (Self::PII, "PII"),
        ];
        f.write_str("TaintFlags(")?;
        let mut first = true;
        for (flag, name) in names {
            if self.contains(flag) {
                if !first {
                    f.write_str(" | ")?;
                }
                f.write_str(name)?;
                first = false;
            }
        }
        if first {
            f.write_str("NONE")?;
        }
        f.write_str(")")
    }
}

/// Grounding level G0..G3 (SPEC-02 §2.3). Ordering: `VerifiedSpan` is the strongest.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum GroundingLevel {
    /// G0: source present, span verified.
    VerifiedSpan,
    /// G1: source present, span not verified.
    UnverifiedSource,
    /// G2: inferred by an agent from parent gids.
    Inferred,
    /// G3: no source. Quarantined and excluded from standard recall.
    Hypothesis,
}

impl GroundingLevel {
    /// Stable short code, `"G0"` to `"G3"`.
    pub const fn code(self) -> &'static str {
        match self {
            Self::VerifiedSpan => "G0",
            Self::UnverifiedSource => "G1",
            Self::Inferred => "G2",
            Self::Hypothesis => "G3",
        }
    }

    /// `true` if the level is admitted to standard recall (everything except G3).
    #[inline]
    pub const fn is_recall_eligible(self) -> bool {
        !matches!(self, Self::Hypothesis)
    }
}

/// Named prior of a belief distribution (SPEC-02 §2.3, rule 1).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PriorKind {
    /// System-generated fact.
    System,
    /// Verified fact.
    Verified,
    /// Curated by a human.
    Curated,
    /// Verified by a tool.
    ToolVerified,
    /// Origin unknown.
    Unknown,
    /// Inferred by an agent.
    AgentInferred,
    /// Hypothesis without source.
    Hypothesis,
}

/// Beta distribution for uncertainty and evidence accumulation (SPEC-02 §2.3).
#[derive(Clone, Debug, PartialEq)]
pub struct BeliefBeta {
    /// Success pseudo-count.
    pub alpha: f32,
    /// Failure pseudo-count.
    pub beta: f32,
    /// Deduplicated independent source roots that have already confirmed this belief.
    pub source_roots: Vec<[u8; 16]>,
}

impl BeliefBeta {
    /// Beta distribution for a named prior, with no confirming roots.
    pub fn prior(kind: PriorKind) -> Self {
        let (alpha, beta) = match kind {
            PriorKind::System | PriorKind::Verified => (4.0, 1.0),
            PriorKind::Curated | PriorKind::ToolVerified => (2.0, 1.0),
            PriorKind::Unknown => (1.0, 1.0),
            PriorKind::AgentInferred | PriorKind::Hypothesis => (1.0, 2.0),
        };
        Self {
            alpha,
            beta,
            source_roots: Vec::new(),
        }
    }

    /// Mean of the distribution, `alpha / (alpha + beta)`.
    #[inline]
    pub fn mean(&self) -> f32 {
        self.alpha / (self.alpha + self.beta)
    }

    /// Records a confirmation from `source_root`.
    ///
    /// Independence rule (SPEC-02 §2.3, rule 2): `alpha` grows only if
    /// `source_root` has not confirmed this belief before. Returns `true` if
    /// the confirmation counted.
    pub fn confirm(&mut self, source_root: [u8; 16]) -> bool {
        if self.source_roots.contains(&source_root) {
            return false;
        }
        self.source_roots.push(source_root);
        self.alpha += 1.0;
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trust_order_and_minimum_inheritance() {
        assert!(TrustTier::Verified > TrustTier::Curated);
        assert!(TrustTier::Untrusted < TrustTier::AgentInferred);
        assert_eq!(
            TrustTier::inherit([
                TrustTier::Verified,
                TrustTier::ToolVerified,
                TrustTier::Curated
            ]),
            Some(TrustTier::ToolVerified)
        );
        assert_eq!(TrustTier::inherit(std::iter::empty()), None);
    }

    #[test]
    fn taint_union_and_from_bits() {
        let a = TaintFlags::EXTERNAL;
        let b = TaintFlags::PII;
        let u = a | b;
        assert!(u.contains(a) && u.contains(b));
        assert!(!u.contains(TaintFlags::INJECTION_SUSPECT));
        assert_eq!(TaintFlags::inherit([a, b, TaintFlags::NONE]), u);
        assert_eq!(TaintFlags::from_bits(0b1_0000), None);
        assert_eq!(
            TaintFlags::from_bits(0b0101),
            Some(a | TaintFlags::UNVERIFIED_TOOL)
        );
    }

    #[test]
    fn taint_debug_lists_flags() {
        let s = format!("{:?}", TaintFlags::EXTERNAL | TaintFlags::PII);
        assert_eq!(s, "TaintFlags(EXTERNAL | PII)");
        assert_eq!(format!("{:?}", TaintFlags::NONE), "TaintFlags(NONE)");
    }

    #[test]
    fn grounding_codes_and_recall_gate() {
        assert_eq!(GroundingLevel::Inferred.code(), "G2");
        assert!(GroundingLevel::UnverifiedSource.is_recall_eligible());
        assert!(!GroundingLevel::Hypothesis.is_recall_eligible());
        assert!(GroundingLevel::VerifiedSpan < GroundingLevel::Hypothesis);
    }

    #[test]
    fn priors_match_spec() {
        assert_eq!(
            (
                BeliefBeta::prior(PriorKind::System).alpha,
                BeliefBeta::prior(PriorKind::System).beta
            ),
            (4.0, 1.0)
        );
        let h = BeliefBeta::prior(PriorKind::Hypothesis);
        assert_eq!((h.alpha, h.beta), (1.0, 2.0));
        assert_eq!(BeliefBeta::prior(PriorKind::Unknown).mean(), 0.5);
    }

    #[test]
    fn independent_roots_only_count_once() {
        let mut b = BeliefBeta::prior(PriorKind::Unknown);
        assert!(b.confirm([1; 16]));
        assert!(
            !b.confirm([1; 16]),
            "echo from the same root must not add evidence"
        );
        assert!(b.confirm([2; 16]));
        assert_eq!(b.alpha, 3.0);
        assert_eq!(b.source_roots.len(), 2);
    }
}
