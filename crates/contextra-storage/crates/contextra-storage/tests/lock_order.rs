use contextra_storage::sync::ranked::{RankStack, Ranked};

#[test]
fn ranked_locks_allow_only_strictly_increasing_order() -> Result<(), contextra_storage::ports::Error> {
    let rank1 = Ranked::<_, 1>::new(());
    let rank2 = Ranked::<_, 2>::new(());
    let rank3 = Ranked::<_, 3>::new(());

    let first = rank1.lock()?;
    assert_eq!(RankStack::current(), vec![1]);
    assert!(rank1.lock().is_err());
    {
        let second = rank2.lock()?;
        assert_eq!(RankStack::current(), vec![1, 2]);
        let third = rank3.lock()?;
        assert_eq!(RankStack::current(), vec![1, 2, 3]);
        drop(third);
        assert_eq!(RankStack::current(), vec![1, 2]);
        assert!(rank1.lock().is_err());
        drop(second);
    }
    assert_eq!(RankStack::current(), vec![1]);
    drop(first);
    assert!(RankStack::current().is_empty());
    Ok(())
}
