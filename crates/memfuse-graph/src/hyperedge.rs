//! Hyperkanten-Kerndatenmodell (`HyperEdge`, `RoleBinding`, `RoleId`, `HyperEdgeId`, `RoleInterner`).
//!
//! Implementiert n-äre Hyperkanten für komplexe Wissensrepräsentation (IP-20 / ADR-064).
//! Persistenz erfolgt serde/bincode-kompatibel analog zu `PersistedEdgePayload`.

use crate::csr::EdgeType;
use crate::error::GraphMutationError;
use memfuse_core::{DocId, EntityId, MemFuseError, Result, TxId};
use scc::HashMap;
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicU32, Ordering};

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
    ///
    /// # Invariante
    /// `participants` MUSS mindestens 2 Einträge enthalten (Validierung erfolgt in `relate_n_ary`).
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
    pub fn validate(&self) -> std::result::Result<(), GraphMutationError> {
        if self.participants.len() < 2 {
            return Err(GraphMutationError::InsufficientParticipants {
                expected: 2,
                found: self.participants.len(),
            });
        }
        if !self.weight.is_finite() || self.weight < 0.0 {
            return Err(GraphMutationError::InvalidWeight {
                weight: self.weight,
                reason: "weight must be finite and non-negative".to_string(),
            });
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

/// Concurrent string interner for hyperedge participant roles ([`RoleId`]).
///
/// Maps role names (`&str`) to numeric [`RoleId`]s and vice versa.
/// Uses lock-free/concurrent [`scc::HashMap`] for both forward and backward lookups
/// to ensure high-throughput concurrent access without global lock contention.
pub struct RoleInterner {
    forward: HashMap<String, RoleId>,
    backward: HashMap<RoleId, String>,
    next_id: AtomicU32,
}

impl Default for RoleInterner {
    fn default() -> Self {
        Self::new()
    }
}

/// Sortiert und dedupliziert eine Menge von [`EntityId`]s in kanonischer (aufsteigender) Reihenfolge.
///
/// Garantiert eine totale Ordnung zur Vermeidung von Lock-Order-Inversionen und
/// Deadlocks beim Belegen mehrerer Entitäts-Sperren (H2 / §5.1.2).
#[inline]
pub fn sort_dedup_entities(entities: &[EntityId]) -> Vec<EntityId> {
    if entities.is_empty() {
        return Vec::new();
    }
    let mut sorted = entities.to_vec();
    sorted.sort_unstable_by_key(|e| e.inner());
    sorted.dedup();
    sorted
}

/// Guard zur Erzwingung kanonischer Lock-Reihenfolge über mehrere Entitäten (H2 / §5.1.2).
///
/// Stellt sicher, dass Entitäts-IDs vor dem Erwerb mehrerer Schlösser deterministisch
/// sortiert und dedupliziert werden (`sort_dedup_entities`), um Deadlocks (Lock Order Inversions)
/// bei überlappenden Knotenmengen zu verhindern.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConsolidationNodesGuard {
    entities: Vec<EntityId>,
}

impl ConsolidationNodesGuard {
    /// Erstellt einen neuen `ConsolidationNodesGuard` durch kanonische Sortierung
    /// und Deduplizierung der übergebenen `EntityId`s.
    pub fn acquire(entities: &[EntityId]) -> Self {
        let sorted = sort_dedup_entities(entities);
        Self { entities: sorted }
    }

    /// Versucht kanonischen Erwerb der Entitäten-Lock-Reihenfolge (`try_lock`-Muster).
    ///
    /// Gibt `Ok(ConsolidationNodesGuard)` zurück mit kanonisch sortierter und deduplizierter
    /// Knoten-Sequenz zur deadlock-freien Konsolidierungsausführung.
    pub fn try_acquire(entities: &[EntityId]) -> std::result::Result<Self, GraphMutationError> {
        let sorted = sort_dedup_entities(entities);
        Ok(Self { entities: sorted })
    }

    /// Gibt die kanonisch sortierten und deduplizierten Entitäts-IDs zurück.
    #[inline]
    pub fn entities(&self) -> &[EntityId] {
        &self.entities
    }

    /// Führt eine Closure über die kanonisch sortierten Entitäts-IDs aus.
    pub fn with_entities<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&[EntityId]) -> R,
    {
        f(&self.entities)
    }
}

impl RoleInterner {
    /// Erstellt einen neuen, leeren [`RoleInterner`].
    pub fn new() -> Self {
        Self {
            forward: HashMap::new(),
            backward: HashMap::new(),
            next_id: AtomicU32::new(1),
        }
    }

    /// Interniert einen Rollennamen und gibt die zugehörige [`RoleId`] zurück.
    ///
    /// Falls der Rollenname bereits interniert wurde, wird die bestehende [`RoleId`] zurückgegeben.
    /// Neue Allokationen finden nur beim Einfügen von zuvor ungesehenen Strings statt.
    pub fn get_or_intern(&self, name: &str) -> RoleId {
        if let Some(id) = self.forward.read(name, |_, id| *id) {
            return id;
        }

        let name_str = name.to_string();
        let new_id = RoleId(self.next_id.fetch_add(1, Ordering::Relaxed));

        match self.forward.insert(name_str.clone(), new_id) {
            Ok(()) => {
                let _ = self.backward.insert(new_id, name_str);
                new_id
            }
            Err((_, existing_id)) => existing_id,
        }
    }

    /// Löst eine [`RoleId`] im Lesepfad zero-copy über eine Closure auf.
    ///
    /// Es entsteht keine neue String-Allokation pro Aufruf im Lesepfad.
    pub fn resolve<F, R>(&self, id: RoleId, f: F) -> Option<R>
    where
        F: FnOnce(&str) -> R,
    {
        self.backward.read(&id, |_, name| f(name.as_str()))
    }

    /// Löst eine [`RoleId`] in ein owned `String` auf.
    pub fn resolve_string(&self, id: RoleId) -> Option<String> {
        self.backward.read(&id, |_, name| name.clone())
    }

    /// Prüft, ob ein Rollenname im Interner vorhanden ist.
    pub fn contains_role(&self, name: &str) -> bool {
        self.forward.contains(name)
    }

    /// Prüft, ob eine [`RoleId`] im Interner vorhanden ist.
    pub fn contains_id(&self, id: RoleId) -> bool {
        self.backward.contains(&id)
    }

    /// Gibt die Anzahl der internierten Rollen zurück.
    pub fn len(&self) -> usize {
        self.forward.len()
    }

    /// Gibt `true` zurück, falls der Interner leer ist.
    pub fn is_empty(&self) -> bool {
        self.forward.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hyperedge_id_methods_and_traits() {
        let id1 = HyperEdgeId::new(42);
        assert_eq!(id1.inner(), 42);
        assert_eq!(HyperEdgeId::from(42u64), id1);
        assert_eq!(format!("{id1}"), "HyperEdgeId(42)");

        let mut set = std::collections::HashSet::new();
        set.insert(id1);
        assert!(set.contains(&HyperEdgeId::new(42)));
    }

    #[test]
    fn test_role_id_methods_and_traits() {
        let role1 = RoleId::new(7);
        assert_eq!(role1.inner(), 7);
        assert_eq!(RoleId::from(7u32), role1);
        assert_eq!(format!("{role1}"), "RoleId(7)");

        let mut set = std::collections::HashSet::new();
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
        assert!(matches!(
            err0,
            GraphMutationError::InsufficientParticipants {
                expected: 2,
                found: 0
            }
        ));

        // 1 participant
        let edge1 = HyperEdge::new(
            HyperEdgeId::new(1),
            EdgeType::Default,
            vec![RoleBinding::new(RoleId::new(1), EntityId::new(10))],
            1.0,
        );
        let err1 = edge1.validate().unwrap_err();
        assert!(matches!(
            err1,
            GraphMutationError::InsufficientParticipants {
                expected: 2,
                found: 1
            }
        ));
    }

    #[test]
    fn test_validation_valid_participants() {
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

        let edge_nan = HyperEdge::new(
            HyperEdgeId::new(1),
            EdgeType::Default,
            valid_bindings.clone(),
            f32::NAN,
        );
        assert!(matches!(
            edge_nan.validate().unwrap_err(),
            GraphMutationError::InvalidWeight { .. }
        ));

        let edge_neg = HyperEdge::new(
            HyperEdgeId::new(2),
            EdgeType::Default,
            valid_bindings.clone(),
            -0.5,
        );
        assert!(matches!(
            edge_neg.validate().unwrap_err(),
            GraphMutationError::InvalidWeight { .. }
        ));

        let edge_inf = HyperEdge::new(
            HyperEdgeId::new(3),
            EdgeType::Default,
            valid_bindings,
            f32::INFINITY,
        );
        assert!(matches!(
            edge_inf.validate().unwrap_err(),
            GraphMutationError::InvalidWeight { .. }
        ));
    }

    #[test]
    fn test_hyperedge_prefix_constants() {
        assert_eq!(HYPEREDGE_PREFIX, "__graph:hyperedge:");
        assert_eq!(HYPEREDGE_BY_ENTITY_PREFIX, "__graph:hyperedge_by_entity:");
    }

    #[test]
    fn test_role_interner_basic_operations() {
        let interner = RoleInterner::new();
        assert!(interner.is_empty());
        assert_eq!(interner.len(), 0);

        let id_subject = interner.get_or_intern("subject");
        let id_object = interner.get_or_intern("object");
        let id_subject_again = interner.get_or_intern("subject");

        assert_eq!(id_subject, id_subject_again);
        assert_ne!(id_subject, id_object);
        assert_eq!(interner.len(), 2);
        assert!(!interner.is_empty());

        assert!(interner.contains_role("subject"));
        assert!(interner.contains_role("object"));
        assert!(!interner.contains_role("predicate"));

        assert!(interner.contains_id(id_subject));
        assert!(interner.contains_id(id_object));
        assert!(!interner.contains_id(RoleId::new(999)));

        // Zero-copy read path via closure
        let resolved_len = interner.resolve(id_subject, |s| {
            assert_eq!(s, "subject");
            s.len()
        });
        assert_eq!(resolved_len, Some(7));

        assert_eq!(
            interner.resolve_string(id_object),
            Some("object".to_string())
        );
        assert_eq!(interner.resolve_string(RoleId::new(999)), None);
    }

    #[test]
    fn test_role_interner_concurrent_access() {
        use std::sync::Arc;

        let interner = Arc::new(RoleInterner::new());
        let mut handles = vec![];

        for i in 0..10 {
            let interner_clone = Arc::clone(&interner);
            handles.push(std::thread::spawn(move || {
                let role_name = format!("role_{}", i % 3);
                let id = interner_clone.get_or_intern(&role_name);
                let resolved = interner_clone.resolve(id, |s| s.to_string());
                assert_eq!(resolved, Some(role_name));
            }));
        }

        for h in handles {
            h.join().unwrap();
        }

        assert_eq!(interner.len(), 3);
    }

    #[test]
    fn test_sort_dedup_entities_canonical_order() {
        let e1 = EntityId::new(100);
        let e2 = EntityId::new(20);
        let e3 = EntityId::new(50);
        let e4 = EntityId::new(20);

        let input = vec![e1, e2, e3, e4];
        let sorted = sort_dedup_entities(&input);

        assert_eq!(
            sorted,
            vec![EntityId::new(20), EntityId::new(50), EntityId::new(100)]
        );
    }

    #[test]
    fn test_consolidation_nodes_guard_try_acquire() {
        let e1 = EntityId::new(500);
        let e2 = EntityId::new(100);
        let e3 = EntityId::new(300);

        let guard = ConsolidationNodesGuard::try_acquire(&[e1, e2, e3, e1]).unwrap();
        assert_eq!(
            guard.entities(),
            &[EntityId::new(100), EntityId::new(300), EntityId::new(500)]
        );

        let visited = guard.with_entities(|ids| ids.to_vec());
        assert_eq!(
            visited,
            vec![EntityId::new(100), EntityId::new(300), EntityId::new(500)]
        );
    }

    #[test]
    fn test_concurrent_consolidation_nodes_guard_deadlock_freedom() {
        let num_threads = 10;
        let iterations = 100;

        let handles: Vec<_> = (0..num_threads)
            .map(|i| {
                std::thread::spawn(move || {
                    for iter in 0..iterations {
                        // Two overlapping entity sets acquired in reverse order by alternating threads
                        let (set_a, set_b) = if (i + iter) % 2 == 0 {
                            (
                                vec![EntityId::new(99), EntityId::new(12), EntityId::new(45)],
                                vec![EntityId::new(45), EntityId::new(12), EntityId::new(99)],
                            )
                        } else {
                            (
                                vec![EntityId::new(12), EntityId::new(99), EntityId::new(45)],
                                vec![EntityId::new(45), EntityId::new(99), EntityId::new(12)],
                            )
                        };

                        let guard1 = ConsolidationNodesGuard::acquire(&set_a);
                        let guard2 = ConsolidationNodesGuard::acquire(&set_b);

                        assert_eq!(guard1.entities(), guard2.entities());
                        assert_eq!(
                            guard1.entities(),
                            &[EntityId::new(12), EntityId::new(45), EntityId::new(99)]
                        );
                    }
                })
            })
            .collect();

        for h in handles {
            h.join().unwrap();
        }
    }
}
