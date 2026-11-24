// ANCHOR[contextra-mvcc:TEST-TOMBSTONE-DIFF]
#![allow(unexpected_cfgs)]
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use contextra_mvcc::tombstone::{
    SeqBitTombstone as NewTombstone, TombstoneSemanticsCheck as NewTrait,
};
use contextra_types::tombstone::{
    SeqBitTombstone as LegacyTombstone, TombstoneSemanticsCheck as LegacyTrait,
};

#[test]
fn test_tombstone_identical_behavior() {
    let test_seqs = [
        0u64,
        1,
        2,
        42,
        999,
        1_000_000,
        (1 << 62) - 1,
        (1 << 63),
        (1 << 63) | 42,
        u64::MAX,
    ];
    let test_flags = [0u8, 1, 2, 0x7F, 0x80, 0xFE, 0xFF];

    for &seq in &test_seqs {
        for &flags in &test_flags {
            assert_eq!(
                LegacyTombstone::is_tombstone(seq, flags),
                NewTombstone::is_tombstone(seq, flags)
            );
        }
        assert_eq!(
            LegacyTombstone::make_tombstone(seq),
            NewTombstone::make_tombstone(seq)
        );
    }
}
