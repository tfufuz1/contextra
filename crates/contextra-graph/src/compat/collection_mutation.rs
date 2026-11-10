//! Port-Implementierung des `GraphCollectionMutation`-Traits für den CSR-Graphen.

use contextra_ports::graph::{GraphMutationError as PortsGraphMutationError, HyperEdgeId as PortsHyperEdgeId, RoleBinding as PortsRoleBinding};
use contextra_ports::GraphCollectionMutation;
use contextra_types::DocId;

use crate::csr::graph_write::CsrGraph;
use crate::error::GraphMutationError;

impl GraphCollectionMutation for CsrGraph {
    fn relate_n_ary(
        &self,
        predicate_tag: u32,
        participants: &[PortsRoleBinding],
        doc_id: DocId,
    ) -> std::result::Result<PortsHyperEdgeId, PortsGraphMutationError> {
        let next_id = crate::hyperedge::HyperEdgeId::new(self.max_hyperedge_id() + 1);
        let mapped_participants: Vec<crate::hyperedge::RoleBinding> = participants
            .iter()
            .map(|p| {
                crate::hyperedge::RoleBinding::new(
                    crate::hyperedge::RoleId::new(p.role.inner()),
                    p.entity,
                )
            })
            .collect();

        let _ = predicate_tag;
        let hyperedge = self
            .relate_n_ary(
                next_id,
                crate::csr::EdgeType::Default,
                mapped_participants,
                1.0,
                Some(doc_id),
            )
            .map_err(mutation_error_to_ports)?;

        Ok(PortsHyperEdgeId::new(hyperedge.id.inner()))
    }
}

pub(crate) fn mutation_error_to_ports(err: GraphMutationError) -> PortsGraphMutationError {
    match err {
        GraphMutationError::LockAcquisitionTimeout(msg) => {
            PortsGraphMutationError::LockAcquisitionTimeout(msg)
        }
        GraphMutationError::RoleBindingInvalid(msg) => {
            PortsGraphMutationError::RoleBindingInvalid(msg)
        }
        GraphMutationError::EpochReclamationPending(epoch) => {
            PortsGraphMutationError::EpochReclamationPending(epoch)
        }
        GraphMutationError::InsufficientParticipants { expected, found } => {
            PortsGraphMutationError::InsufficientParticipants { expected, found }
        }
        GraphMutationError::HyperedgeNotFound(id) => {
            PortsGraphMutationError::HyperedgeNotFound(id)
        }
        GraphMutationError::InvalidWeight { weight, reason } => {
            PortsGraphMutationError::InvalidWeight { weight, reason }
        }
        GraphMutationError::Internal(msg) => PortsGraphMutationError::Internal(msg),
        GraphMutationError::DuplicateHyperEdgeId(id) => {
            PortsGraphMutationError::DuplicateHyperEdgeId(PortsHyperEdgeId::new(id))
        }
    }
}
