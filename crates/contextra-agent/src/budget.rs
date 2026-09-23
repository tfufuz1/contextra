// FILE-CONTEXT Header (Format v3)
// ZWECK: Token budget RAII reservation abstraction for agent workflow steps.
// INVARIANTEN: No locks held across await points; Zero-Panic-Doctrine in drop handlers; atomic drop refunds.
// NICHT-OFFENSICHTLICH: Reservation automatically refunds tokens on drop unless settle() is called.
// STAND: TS:2026-09-17T19:00:00Z (SESSION: jules-agent-budget)

//! RAII budget management and token reservation for agent workflows (§6.3.3).

pub use contextra_core::types::budget::{
    BudgetStrategy, Reservation, ResourceBudget, ResourceTracker, TokenBudget,
};

/// Helper to reserve `amount` tokens from `budget`.
///
/// Returns an RAII [`Reservation`] guard on success. If execution drops the guard
/// without calling `reservation.settle()`, the reserved tokens are automatically refunded.
pub fn reserve_tokens<'a>(
    budget: &'a TokenBudget,
    amount: usize,
) -> contextra_core::Result<Reservation<'a>> {
    budget.reserve(amount)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reserve_tokens_raii_settle() {
        let budget = TokenBudget::new(1000, 100);
        assert_eq!(budget.available(), 900);

        let reservation = reserve_tokens(&budget, 200).expect("reservation should succeed");
        assert_eq!(budget.available(), 700);

        reservation.settle();
        assert_eq!(budget.available(), 700);
        assert_eq!(budget.consumed(), 200);
    }

    #[test]
    fn test_reserve_tokens_raii_drop_refund() {
        let budget = TokenBudget::new(1000, 100);
        assert_eq!(budget.available(), 900);

        {
            let _res = reserve_tokens(&budget, 300).expect("reservation should succeed");
            assert_eq!(budget.available(), 600);
            assert_eq!(budget.consumed(), 300);
            // Drop guard without calling settle()
        }

        assert_eq!(budget.available(), 900);
        assert_eq!(budget.consumed(), 0);
    }

    #[test]
    fn test_reserve_tokens_over_budget_rejected() {
        let budget = TokenBudget::new(500, 100);
        assert_eq!(budget.available(), 400);

        let res = reserve_tokens(&budget, 500);
        assert!(
            res.is_err(),
            "Reservation exceeding available limit must fail"
        );
        assert_eq!(budget.available(), 400);
        assert_eq!(budget.consumed(), 0);
    }

    #[test]
    fn test_reserve_tokens_sequential_reservations() {
        let budget = TokenBudget::new(1000, 0);

        let res1 = reserve_tokens(&budget, 300).expect("first reservation");
        assert_eq!(budget.available(), 700);

        let res2 = reserve_tokens(&budget, 400).expect("second reservation");
        assert_eq!(budget.available(), 300);

        // res1 drops without settle -> refunds 300
        drop(res1);
        assert_eq!(budget.available(), 600);

        res2.settle();
        assert_eq!(budget.available(), 600);
        assert_eq!(budget.consumed(), 400);
    }
}
