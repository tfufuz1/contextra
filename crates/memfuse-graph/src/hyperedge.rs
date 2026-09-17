//! Hyperkanten-Kerndatenmodell (`HyperEdge`, `RoleBinding`, `RoleId`, `HyperEdgeId`).
//!
//! Implementiert n-äre Hyperkanten für komplexe Wissensrepräsentation (IP-20 / ADR-064).
//! Persistenz erfolgt serde/bincode-kompatibel analog zu `PersistedEdgePayload`.

use crate::csr::EdgeType;
use memfuse_core::{DocId, EntityId, MemFuseError, Result, TxId};
use serde::{Deserialize, Serialize};

/// LSM-Key-Präfix für Hyperkanten.
pub const HYPEREDGE_PREFIX: &str = "__graph:hyperedge:";

/// LSM-Key-Präfix für Hyperkanten-Index nach Entity.
pub const HYPEREDGE_BY_ENTITY_PREFIX: &str = "__graph:hyperedge_by_entity:";

/// Eindeutiger Identifikator für eine Hyperkante.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, Default,
)]
#[repr(transparent)]
pub struct HyperEdgeId(pub u64);

impl HyperEdgeId {
    /// Erstellt eine neue `HyperEdgeId`.
    #[inline]
    pub const fn new(id: u64) -> Self {
        Self(id)
    }

    /// Gibt den zugrundeliegenden `u64`-Wert zurück.
    #[inline]
    pub const fn inner(self) -> u64 {
        self.0
    }
}

impl From<u64> for HyperEdgeId {
    fn from(id: u64) -> Self {
        Self(id)
    }
}

impl std::fmt::Display for HyperEdgeId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "HyperEdgeId({})", self.0)
    }
}

/// Identifikator für die Rolle eines Teilnehmers in einer Hyperkante.
///
/// NOTE: Internierung über einen String-Interner ist als Folge-Prompt geplant.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, Default,
)]
#[repr(transparent)]
pub struct RoleId(pub u32);

impl RoleId {
    /// Erstellt eine neue `RoleId`.
    #[inline]
    pub const fn new(id: u32) -> Self {
        Self(id)
    }

    /// Gibt den zugrundeliegenden `u32`-Wert zurück.
    #[inline]
    pub const fn inner(self) -> u32 {
        self.0
    }
}

impl From<u32> for RoleId {
    fn from(id: u32) -> Self {
        Self(id)
    }
}

impl std::fmt::Display for RoleId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "RoleId({})", self.0)
    }
}

/// Bindung einer Entity an eine spezifische Rolle innerhalb einer Hyperkante.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RoleBinding {
    /// Die Rolle der Entity in der Hyperkante.
    pub role: RoleId,
    /// Die gebundene Entity.
    pub entity: EntityId,
}

impl RoleBinding {
    /// Erstellt eine neue `RoleBinding`.
    pub fn new(role: RoleId, entity: EntityId) -> Self {
        Self { role, entity }
    }
}

/// Eine n-äre Hyperkante zur Verbindung von 2 oder mehr Entitäten mit spezifischen Rollen.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HyperEdge {
    /// Eindeutige ID der Hyperkante.
    pub id: HyperEdgeId,
    /// Prädikat/Typ der Hyperkante.
    pub predicate: EdgeType,
    /// Liste der Teilnehmer mit ihren Rollen.
    pub participants: Vec<RoleBinding>,
    /// Gewichtung der Hyperkante.
    pub weight: f32,
    /// Start der Transaktionsgültigkeit (MVCC / Systemzeit).
    #[serde(default)]
    pub tx_valid_from: Option<TxId>,
    /// Ende der Transaktionsgültigkeit (MVCC / Systemzeit).
    #[serde(default)]
    pub tx_valid_to: Option<TxId>,
    /// Start der fachlichen Gültigkeit (Business-Zeit in Unix-ms).
    #[serde(default)]
    pub business_valid_from: Option<i64>,
    /// Ende der fachlichen Gültigkeit (Business-Zeit in Unix-ms).
    #[serde(default)]
    pub business_valid_to: Option<i64>,
    /// Optionales Quell-Dokument, aus dem diese Hyperkante abgeleitet wurde.
    #[serde(default)]
    pub source_doc_id: Option<DocId>,
}

impl HyperEdge {
    /// Erstellt eine neue `HyperEdge`.
    pub fn new(
        id: HyperEdgeId,
        predicate: EdgeType,
        participants: Vec<RoleBinding>,
        weight: f32,
    ) -> Self {
        Self {
            id,
            predicate,
            participants,
            weight,
            tx_valid_from: None,
            tx_valid_to: None,
            business_valid_from: None,
            business_valid_to: None,
            source_doc_id: None,
        }
    }

    /// Setzt die Transaktionsgültigkeit der Hyperkante.
    pub fn with_tx_validity(mut self, valid_from: Option<TxId>, valid_to: Option<TxId>) -> Self {
        self.tx_valid_from = valid_from;
        self.tx_valid_to = valid_to;
        self
    }

    /// Setzt die fachliche Gültigkeit der Hyperkante.
    pub fn with_business_validity(
        mut self,
        valid_from: Option<i64>,
        valid_to: Option<i64>,
    ) -> Self {
        self.business_valid_from = valid_from;
        self.business_valid_to = valid_to;
        self
    }

    /// Setzt die Quell-Dokument-ID der Hyperkante.
    pub fn with_source_doc_id(mut self, source_doc_id: Option<DocId>) -> Self {
        self.source_doc_id = source_doc_id;
        self
    }

    /// Validiert die Invarianten der Hyperkante.
    ///
    /// # Invarianten
    /// - Mindestens 2 Teilnehmer (`participants.len() >= 2`), sonst degeneriert zur binären Edge.
    /// - Endliches, nicht-negatives Gewicht (`0.0 <= weight`).
    pub fn validate(&self) -> Result<()> {
        if self.participants.len() < 2 {
            return Err(MemFuseError::InvalidInput(format!(
                "HyperEdge requires at least 2 participants, found {}",
                self.participants.len()
            )));
        }
        if !self.weight.is_finite() || self.weight < 0.0 {
            return Err(MemFuseError::InvalidInput(format!(
                "Invalid hyperedge weight {}: weight must be finite and non-negative",
                self.weight
            )));
        }
        Ok(())
    }

    /// Serialisiert die Hyperkante via `bincode`.
    pub fn serialize(&self) -> Result<Vec<u8>> {
        bincode::serialize(self).map_err(|e| {
            MemFuseError::Internal(format!("Failed to serialize hyperedge {}: {e}", self.id))
        })
    }

    /// Deserialisiert eine Hyperkante aus einem `bincode`-Byte-Slice.
    pub fn deserialize(bytes: &[u8]) -> Result<Self> {
        bincode::deserialize(bytes)
            .map_err(|e| MemFuseError::Internal(format!("Failed to deserialize hyperedge: {e}")))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn test_hyperedge_id_methods_and_traits() {
        let id1 = HyperEdgeId::new(42);
        assert_eq!(id1.inner(), 42);
        assert_eq!(HyperEdgeId::from(42u64), id1);
        assert_eq!(format!("{id1}"), "HyperEdgeId(42)");

        let mut set = HashSet::new();
        set.insert(id1);
        assert!(set.contains(&HyperEdgeId::new(42)));
    }

    #[test]
    fn test_role_id_methods_and_traits() {
        let role1 = RoleId::new(7);
        assert_eq!(role1.inner(), 7);
        assert_eq!(RoleId::from(7u32), role1);
        assert_eq!(format!("{role1}"), "RoleId(7)");

        let mut set = HashSet::new();
        set.insert(role1);
        assert!(set.contains(&RoleId::new(7)));
    }

    #[test]
    fn test_hyperedge_bincode_serialization_roundtrip() {
        let edge = HyperEdge::new(
            HyperEdgeId::new(100),
            EdgeType::Default,
            vec![
                RoleBinding::new(RoleId::new(1), EntityId::new(10)),
                RoleBinding::new(RoleId::new(2), EntityId::new(20)),
                RoleBinding::new(RoleId::new(3), EntityId::new(30)),
            ],
            1.5,
        )
        .with_tx_validity(Some(TxId::new(1)), Some(TxId::new(10)))
        .with_business_validity(Some(1000), Some(2000))
        .with_source_doc_id(Some(DocId::new(500)));

        let serialized = edge.serialize().expect("Serialization failed");
        let deserialized = HyperEdge::deserialize(&serialized).expect("Deserialization failed");

        assert_eq!(edge, deserialized);
    }

    #[test]
    fn test_validation_insufficient_participants() {
        // 0 participants
        let edge0 = HyperEdge::new(HyperEdgeId::new(1), EdgeType::Default, vec![], 1.0);
        let err0 = edge0.validate().unwrap_err();
        assert!(
            matches!(err0, MemFuseError::InvalidInput(msg) if msg.contains("at least 2 participants"))
        );

        // 1 participant
        let edge1 = HyperEdge::new(
            HyperEdgeId::new(1),
            EdgeType::Default,
            vec![RoleBinding::new(RoleId::new(1), EntityId::new(10))],
            1.0,
        );
        let err1 = edge1.validate().unwrap_err();
        assert!(
            matches!(err1, MemFuseError::InvalidInput(msg) if msg.contains("at least 2 participants"))
        );
    }

    #[test]
    fn test_validation_valid_participants() {
        // Exactly 2 participants
        let edge2 = HyperEdge::new(
            HyperEdgeId::new(1),
            EdgeType::Default,
            vec![
                RoleBinding::new(RoleId::new(1), EntityId::new(10)),
                RoleBinding::new(RoleId::new(2), EntityId::new(20)),
            ],
            1.0,
        );
        assert!(edge2.validate().is_ok());

        // N participants (4)
        let edge_n = HyperEdge::new(
            HyperEdgeId::new(2),
            EdgeType::Default,
            vec![
                RoleBinding::new(RoleId::new(1), EntityId::new(10)),
                RoleBinding::new(RoleId::new(2), EntityId::new(20)),
                RoleBinding::new(RoleId::new(3), EntityId::new(30)),
                RoleBinding::new(RoleId::new(4), EntityId::new(40)),
            ],
            0.8,
        );
        assert!(edge_n.validate().is_ok());
    }

    #[test]
    fn test_validation_invalid_weights() {
        let valid_bindings = vec![
            RoleBinding::new(RoleId::new(1), EntityId::new(10)),
            RoleBinding::new(RoleId::new(2), EntityId::new(20)),
        ];

        // NaN
        let edge_nan = HyperEdge::new(
            HyperEdgeId::new(1),
            EdgeType::Default,
            valid_bindings.clone(),
            f32::NAN,
        );
        assert!(
            matches!(edge_nan.validate().unwrap_err(), MemFuseError::InvalidInput(msg) if msg.contains("Invalid hyperedge weight"))
        );

        // Negative
        let edge_neg = HyperEdge::new(
            HyperEdgeId::new(2),
            EdgeType::Default,
            valid_bindings.clone(),
            -0.5,
        );
        assert!(
            matches!(edge_neg.validate().unwrap_err(), MemFuseError::InvalidInput(msg) if msg.contains("Invalid hyperedge weight"))
        );

        // Infinity
        let edge_inf = HyperEdge::new(
            HyperEdgeId::new(3),
            EdgeType::Default,
            valid_bindings,
            f32::INFINITY,
        );
        assert!(
            matches!(edge_inf.validate().unwrap_err(), MemFuseError::InvalidInput(msg) if msg.contains("Invalid hyperedge weight"))
        );
    }

    #[test]
    fn test_hyperedge_prefix_constants() {
        assert_eq!(HYPEREDGE_PREFIX, "__graph:hyperedge:");
        assert_eq!(HYPEREDGE_BY_ENTITY_PREFIX, "__graph:hyperedge_by_entity:");
    }

    #[test]
    fn test_deserialize_corrupted_bytes_returns_error() {
        let corrupted_bytes = vec![0xFF, 0x00, 0xAA, 0xBB];
        let res = HyperEdge::deserialize(&corrupted_bytes);
        assert!(
            matches!(res, Err(MemFuseError::Internal(msg)) if msg.contains("Failed to deserialize hyperedge"))
        );
    }
}
