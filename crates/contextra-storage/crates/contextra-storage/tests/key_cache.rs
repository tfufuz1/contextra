use contextra_storage::keys::cache::KeyCache;
use contextra_storage::ports::traits::{Clock, Entropy};
use contextra_storage::ports::{BatchId, ClusterId, Error, KeyCreateFailure, KeyCreateResult, KeyProvider, KeyProbe};
use contextra_storage::testing::keys::MemKeyProvider;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

struct TestClock(AtomicU64);
impl Clock for TestClock { fn now_nanos(&self) -> u64 { self.0.load(Ordering::Acquire) } }
struct TestEntropy(AtomicU64);
impl Entropy for TestEntropy {
    fn fill(&self, output: &mut [u8]) -> Result<(), Error> {
        let mut next = self.0.fetch_add(1, Ordering::AcqRel);
        for byte in output { *byte = next as u8; next = next.wrapping_add(1); }
        Ok(())
    }
}

#[test]
fn key_cache_batch_prefetch_and_generation_fence() -> Result<(), Error> {
    let entropy = Arc::new(TestEntropy(AtomicU64::new(1)));
    let provider = Arc::new(MemKeyProvider::new(entropy));
    let clusters = [ClusterId::from_wire([1; 16]), ClusterId::from_wire([2; 16])];
    provider.create_cluster_keys(BatchId::from_bytes([99; 16]), &clusters)?;
    let clock = Arc::new(TestClock(AtomicU64::new(10)));
    let cache = KeyCache::new(provider, clock, 100, 50, 8)?;
    cache.prefetch(&[(clusters[0], 1), (clusters[1], 1)])?;
    let handle = cache.get(clusters[0], 1)?;
    assert!(!format!("{handle:?}").contains("ClusterId"));
    handle.with_key(|key| {
        assert_eq!(key.expose().len(), 32);
        Ok(())
    })?;
    cache.invalidate(clusters[0])?;
    assert!(matches!(handle.with_key(|_| Ok(())), Err(Error::ClusterRetired)));
    assert!(matches!(cache.get(clusters[0], 1), Err(Error::ClusterRetired)));
    Ok(())
}

#[test]
fn missing_cluster_key_is_key_lost_not_shredded() -> Result<(), Error> {
    let provider = Arc::new(MemKeyProvider::new(Arc::new(TestEntropy(AtomicU64::new(4)))));
    let cache = KeyCache::new(provider, Arc::new(TestClock(AtomicU64::new(0))), 100, 50, 8)?;
    assert!(matches!(cache.get(ClusterId::from_wire([9; 16]), 1), Err(Error::KeyLost)));
    Ok(())
}

#[test]
fn key_creation_batch_is_idempotent_and_reports_per_cluster_failure() -> Result<(), Error> {
    let provider = MemKeyProvider::new(Arc::new(TestEntropy(AtomicU64::new(8))));
    let batch = BatchId::from_bytes([21; 16]);
    let live = ClusterId::from_wire([22; 16]);
    let destroyed = ClusterId::from_wire([23; 16]);
    provider.destroy_many(&[destroyed])?;
    let first = provider.create_cluster_keys(batch, &[live, destroyed])?;
    assert_eq!(provider.create_cluster_keys(batch, &[live, destroyed])?, first);
    assert_eq!(first.get(0), Some(&KeyCreateResult::Created { cluster: live, epoch: 1 }));
    assert_eq!(first.get(1), Some(&KeyCreateResult::Failed { cluster: destroyed, error: KeyCreateFailure::Shredded }));
    assert!(matches!(provider.create_cluster_keys(batch, &[live]), Err(Error::IdemConflict)));
    assert_eq!(provider.probe(live, 1)?, KeyProbe::Present);
    Ok(())
}
