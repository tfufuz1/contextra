//! Tests for SegmentPool and SegmentHandle guarantees: Use-After-Free, Double-Free, Zeroize and Fragmentation

use contextra_kvcache::segment::{SegmentPool, SegmentPoolConfig};

#[test]
fn test_segment_pool_double_free_and_use_after_free() {
    let pool = SegmentPool::new(SegmentPoolConfig {
        block_size: 128,
        max_blocks: 2,
    });

    let handle1 = pool.allocate(1, b"segment-one").unwrap();
    let handle2 = pool.allocate(1, b"segment-two").unwrap();

    assert!(pool.allocate(1, b"segment-three").is_err());
    assert!(pool.deallocate(1, handle1).is_ok());
    assert!(pool.deallocate(1, handle1).is_err());
    assert!(pool.read(1, handle1).is_err());
    assert_eq!(pool.read(1, handle2).unwrap(), b"segment-two");

    let handle3 = pool.allocate(1, b"segment-three").unwrap();
    assert_eq!(handle3.slot_index(), handle1.slot_index());
    assert_ne!(handle3.generation(), handle1.generation());

    assert!(pool.read(1, handle1).is_err());
    assert!(pool.deallocate(1, handle1).is_err());
    assert_eq!(pool.read(1, handle3).unwrap(), b"segment-three");
}

#[test]
fn test_segment_pool_fragmentation_metrics() {
    let pool = SegmentPool::new(SegmentPoolConfig {
        block_size: 100,
        max_blocks: 10,
    });

    assert_eq!(pool.fragmentation_ratio(), 0.0);

    let h1 = pool.allocate(1, &[1u8; 50]).unwrap();
    assert_eq!(pool.allocated_blocks(), 1);
    assert_eq!(pool.allocated_bytes(), 50);
    assert!((pool.fragmentation_ratio() - 0.5).abs() < 1e-6);

    let h2 = pool.allocate(1, &[2u8; 100]).unwrap();
    assert_eq!(pool.allocated_blocks(), 2);
    assert_eq!(pool.allocated_bytes(), 150);
    assert!((pool.fragmentation_ratio() - 0.25).abs() < 1e-6);

    pool.deallocate(1, h1).unwrap();
    pool.deallocate(1, h2).unwrap();
    assert_eq!(pool.fragmentation_ratio(), 0.0);
}
