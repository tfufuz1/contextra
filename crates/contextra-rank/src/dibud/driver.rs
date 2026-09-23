//! Synchronous and std-only asynchronous drivers for DiBud fusion state machine.

use core::future::Future;
use contextra_types::{ContextraError, DocId};

use super::state::{DiBudFusionState, DiBudStep};
use super::types::{BudgetedChannel, DiBudOutcome, FusionBudget};

/// Synchronously executes DiBud fusion using three channel iterators.
pub fn fuse_exact_prefix<I1, I2, I3>(
    mut state: DiBudFusionState,
    vector: &mut I1,
    text: &mut I2,
    graph: &mut I3,
    edge_reinforcement: impl Fn(DocId) -> f32,
    budget: &FusionBudget,
) -> Result<DiBudOutcome, ContextraError>
where
    I1: Iterator<Item = DocId>,
    I2: Iterator<Item = DocId>,
    I3: Iterator<Item = DocId>,
{
    budget.validate()?;
    loop {
        match state.next_request(budget) {
            DiBudStep::Poll(ch) => {
                let item = match ch {
                    BudgetedChannel::Vector => vector.next(),
                    BudgetedChannel::Text => text.next(),
                    BudgetedChannel::Graph => graph.next(),
                };
                state.feed(ch, item, &edge_reinforcement, budget)?;
            }
            DiBudStep::Done(outcome) => return Ok(outcome),
        }
    }
}

/// Asynchronously executes DiBud fusion using a channel poll closure returning a std `Future`.
///
/// This function relies purely on `core::future::Future` and does not require an async runtime.
pub async fn fuse_exact_prefix_async<P, Fut>(
    mut state: DiBudFusionState,
    mut poll: P,
    edge_reinforcement: impl Fn(DocId) -> f32,
    budget: &FusionBudget,
) -> Result<DiBudOutcome, ContextraError>
where
    P: FnMut(BudgetedChannel) -> Fut,
    Fut: Future<Output = Result<Option<DocId>, ContextraError>>,
{
    budget.validate()?;
    loop {
        match state.next_request(budget) {
            DiBudStep::Poll(ch) => {
                let item = poll(ch).await?;
                state.feed(ch, item, &edge_reinforcement, budget)?;
            }
            DiBudStep::Done(outcome) => return Ok(outcome),
        }
    }
}
