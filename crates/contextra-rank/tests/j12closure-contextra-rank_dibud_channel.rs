#![cfg(feature = "dibud")]

use contextra_rank::BudgetedChannel;

#[test]
fn test_j12closure_budgeted_channel_from_index() {
    assert_eq!(BudgetedChannel::from_index(0), Some(BudgetedChannel::Vector));
    assert_eq!(BudgetedChannel::from_index(1), Some(BudgetedChannel::Text));
    assert_eq!(BudgetedChannel::from_index(2), Some(BudgetedChannel::Graph));
    assert_eq!(BudgetedChannel::from_index(3), None);
    assert_eq!(BudgetedChannel::from_index(999), None);

    let all_channels: Vec<BudgetedChannel> = BudgetedChannel::all().collect();
    assert_eq!(all_channels.len(), 3);
    assert_eq!(all_channels[0], BudgetedChannel::Vector);

    for idx in 0..3 {
        let channel = BudgetedChannel::from_index(idx).unwrap();
        assert_eq!(channel.index(), idx);
    }
}
