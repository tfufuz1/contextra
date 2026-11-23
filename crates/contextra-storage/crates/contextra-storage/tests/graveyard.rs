use contextra_storage::engine::graveyard::Graveyard;
use std::sync::Arc;

#[test]
fn graveyard_waits_until_last_view_reference_is_dropped() {
    let view = Arc::new(vec![1u8, 2, 3]);
    let retained = Arc::clone(&view);
    let mut graveyard = Graveyard::new();
    graveyard.retire(view);
    assert_eq!(graveyard.pending(), 1);
    assert!(graveyard.reclaim().is_empty());
    assert_eq!(graveyard.pending(), 1);
    drop(retained);
    assert_eq!(graveyard.reclaim(), vec![vec![1, 2, 3]]);
    assert_eq!(graveyard.pending(), 0);
}
