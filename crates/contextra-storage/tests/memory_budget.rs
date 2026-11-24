use contextra_storage::env::budget::{Budget, BudgetLimits};
use contextra_storage::ports::traits::{MemoryBudget, ResourceClass};

#[test]
fn memory_admission_is_class_bounded_and_raii_released() {
    let limits = BudgetLimits {
        total_bytes: 100,
        class_bytes: [50, 20, 5, 10, 5, 5, 5, 0],
    };
    let budget = Budget::new(limits).expect("valid budget");
    let first = budget.try_reserve(ResourceClass::Memtable, 40).expect("first reservation");
    assert_eq!(budget.used(), 40);
    assert_eq!(budget.class_used(ResourceClass::Memtable), 40);
    assert!(budget.try_reserve(ResourceClass::Memtable, 11).is_err());
    assert!(budget.try_reserve(ResourceClass::Guard, 6).is_err());
    drop(first);
    assert_eq!(budget.used(), 0);
    let _replacement = budget.try_reserve(ResourceClass::Memtable, 50).expect("capacity released");
}

#[test]
fn memory_limits_cannot_overcommit_total_or_overflow_accounting() {
    assert!(Budget::new(BudgetLimits {
        total_bytes: 10,
        class_bytes: [2; 8],
    }).is_err());
    let budget = Budget::new(BudgetLimits {
        total_bytes: u64::MAX,
        class_bytes: [u64::MAX, 0, 0, 0, 0, 0, 0, 0],
    }).expect("max budget");
    let _permit = budget.try_reserve(ResourceClass::Memtable, u64::MAX).expect("exact limit");
    assert!(budget.try_reserve(ResourceClass::Memtable, 1).is_err());
}
