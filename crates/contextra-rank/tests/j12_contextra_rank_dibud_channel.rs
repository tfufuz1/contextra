#![cfg(feature = "dibud")]

use contextra_rank::{
    fuse_exact_prefix, BudgetedChannel, DiBudFusionState, DiBudStep, FusionBudget,
};
use contextra_types::{ContextraError, DocId};

#[test]
fn test_budgeted_channel_from_index_bidirectional_and_state_polling() -> Result<(), ContextraError>
{
    // Check indexing 0 -> Vector, 1 -> Text, 2 -> Graph
    assert_eq!(
        BudgetedChannel::from_index(0),
        Some(BudgetedChannel::Vector)
    );
    assert_eq!(BudgetedChannel::from_index(1), Some(BudgetedChannel::Text));
    assert_eq!(BudgetedChannel::from_index(2), Some(BudgetedChannel::Graph));

    // Check invalid indices return None
    assert_eq!(BudgetedChannel::from_index(3), None);
    assert_eq!(BudgetedChannel::from_index(100), None);

    // Check round-trip consistency (ch.index() -> from_index(idx) -> ch)
    for ch in [
        BudgetedChannel::Vector,
        BudgetedChannel::Text,
        BudgetedChannel::Graph,
    ] {
        let idx = ch.index();
        let restored = BudgetedChannel::from_index(idx);
        assert_eq!(restored, Some(ch));
    }

    // Verify next_request uses BudgetedChannel::from_index to return Poll variants
    let state = DiBudFusionState::new(16);
    let budget = FusionBudget::default();

    let step = state.next_request(&budget);
    assert!(matches!(step, DiBudStep::Poll(_)));

    // Verify driver execution path
    let doc1 = DocId::new(1);
    let mut v = vec![doc1].into_iter();
    let mut t = Vec::<DocId>::new().into_iter();
    let mut g = Vec::<DocId>::new().into_iter();

    let outcome = fuse_exact_prefix(state, &mut v, &mut t, &mut g, |_| 0.0, &budget)?;
    assert_eq!(outcome.ranked, vec![doc1]);

    Ok(())
}
