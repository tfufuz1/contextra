#![cfg(feature = "dibud")]

use contextra_rank::{DiBudFusionState, FusionBudget};
use contextra_types::DocId;

#[test]
fn test_j12closure_dibud_provenance_of() {
    let mut state = DiBudFusionState::new(16);
    let budget = FusionBudget::default();

    let doc_id = DocId::from(42_u64);

    assert!(state.provenance_of(&doc_id).is_none());

    state
        .feed(contextra_rank::BudgetedChannel::Vector, Some(doc_id), |_| 0.5, &budget)
        .unwrap();

    let prov = state.provenance_of(&doc_id);
    assert!(prov.is_some());
    let prov_record = prov.unwrap();
    assert_eq!(prov_record.signal_ranks.len(), 1);

    let explanation = state.explain_doc(&doc_id);
    assert!(explanation.is_some());
}
