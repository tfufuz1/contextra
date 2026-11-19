//! Aufbewahrungs-Attestierung nach SPEC §5.3 (Schritt 1.1) und §5.2.

use contextra_ids::Timestamp;

use crate::types_local::OperatorRef;

/// Gründe, aus denen eine Löschung erlaubt oder verboten ist.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RetentionCode {
    /// Keine Pflicht bekannt.
    NoObligation,
    /// Pflicht bestand und ist abgelaufen.
    ObligationExpired,
    /// Betreiber-Direktive oder behördliche Anordnung.
    OperatorOverride,
    /// Gesetzliche Aufbewahrungspflicht aktiv: Löschung endet mit `Refused`.
    ObligationActive,
    /// Rechtsstreit anhängig: Löschung endet mit `Refused`.
    LegalClaimsPending,
    /// Betroffene Person verlangt Einschränkung (Art. 18 DSGVO).
    DataSubjectRequestedRestriction,
}

impl RetentionCode {
    /// Ob diese Pflicht die Löschung verbietet.
    pub const fn blocks_erasure(self) -> bool {
        matches!(self, Self::ObligationActive | Self::LegalClaimsPending)
    }

    /// Stabile Kodierung für das Journal.
    pub const fn to_u8(self) -> u8 {
        match self {
            Self::NoObligation => 1,
            Self::ObligationExpired => 2,
            Self::OperatorOverride => 3,
            Self::ObligationActive => 4,
            Self::LegalClaimsPending => 5,
            Self::DataSubjectRequestedRestriction => 6,
        }
    }

    /// Dekodiert die Journal-Kodierung.
    pub const fn from_u8(v: u8) -> Option<Self> {
        Some(match v {
            1 => Self::NoObligation,
            2 => Self::ObligationExpired,
            3 => Self::OperatorOverride,
            4 => Self::ObligationActive,
            5 => Self::LegalClaimsPending,
            6 => Self::DataSubjectRequestedRestriction,
            _ => return None,
        })
    }
}

/// Attestierung, die eine Löschung begleitet.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RetentionAttestation {
    /// Grund.
    pub code: RetentionCode,
    /// Betreiber, der die Entscheidung attestiert hat.
    pub attested_by: OperatorRef,
    /// Zeitpunkt der Attestierung.
    pub attested_at: Timestamp,
}
