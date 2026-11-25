#![cfg(feature = "dibud")]

use contextra_rank::{fuse_exact_prefix, BudgetedChannel, DiBudFusionState, FusionBudget};
use contextra_types::{ContextraError, DocId};

#[test]
fn test_dibud_state_provenance_and_explanation() -> Result<(), ContextraError> {
    let mut state = DiBudFusionState::new(32);
    let budget = FusionBudget::default();

    let doc1 = DocId::new(101);
    let doc2 = DocId::new(102);

    // Feed items directly into DiBudFusionState
    state.feed(BudgetedChannel::Vector, Some(doc1), |_| 0.0, &budget)?;
    state.feed(BudgetedChannel::Text, Some(doc2), |_| 0.0, &budget)?;
    state.feed(BudgetedChannel::Vector, Some(doc2), |_| 0.0, &budget)?;

    // Query explanation via explain_doc (which internally calls provenance_of)
    let explanation1 = state.explain_doc(&doc1);
    assert!(explanation1.is_some());
    let exp1 = explanation1.unwrap();
    assert_eq!(exp1.entries.len(), 1);

    let explanation2 = state.explain_doc(&doc2);
    assert!(explanation2.is_some());
    let exp2 = explanation2.unwrap();
    assert_eq!(exp2.entries.len(), 2);

    // Directly inspect provenance record
    let prov1 = state.provenance_of(&doc1);
    assert!(prov1.is_some());

    // Non-existent document
    let unobserved_doc = DocId::new(999);
    assert!(state.explain_doc(&unobserved_doc).is_none());
    assert!(state.provenance_of(&unobserved_doc).is_none());

    // Run via fuse_exact_prefix driver and check explanation
    let mut v = vec![doc1, doc2].into_iter();
    let mut t = vec![doc2].into_iter();
    let mut g = Vec::<DocId>::new().into_iter();
    let driver_state = DiBudFusionState::new(16);
    let outcome = fuse_exact_prefix(driver_state, &mut v, &mut t, &mut g, |_| 0.0, &budget)?;
    assert!(!outcome.ranked.is_empty());

    Ok(())
}
