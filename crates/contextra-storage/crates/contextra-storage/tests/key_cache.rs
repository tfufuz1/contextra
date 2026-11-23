use contextra_storage::keys::cache::KeyCache;
use contextra_storage::ports::traits::{Clock, ClockReading, ClockSkew, Entropy};
use contextra_storage::ports::{ClusterId, DestroyBehavior, Error, KeyProvider, KeyState};
use contextra_crypto_ref::mem_key_provider::MemKeyProvider;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

struct TestClock(AtomicU64, AtomicU64);

impl TestClock {
    fn new(monotonic_nanos: u64) -> Self {
        Self(AtomicU64::new(monotonic_nanos), AtomicU64::new(1_000))
    }
}

impl Clock for TestClock {
    fn read(&self) -> Result<ClockReading, Error> {
        let monotonic_nanos = self.0.load(Ordering::Acquire);
        let durable_utc_millis = self.1.load(Ordering::Acquire);
        Ok(ClockReading {
            monotonic_nanos,
            utc_millis: durable_utc_millis,
            durable_utc_millis,
            skew: ClockSkew::Certain,
        })
    }

    fn advance_utc_floor(&self, expect_millis: u64, next_millis: u64) -> Result<(), Error> {
        if next_millis < expect_millis { return Err(Error::ClockUncertain); }
        self.1.compare_exchange(expect_millis, next_millis, Ordering::AcqRel, Ordering::Acquire)
            .map(|_| ())
            .map_err(|_| Error::CasLost)
    }
}
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
    let entropy: Arc<dyn Entropy> = Arc::new(TestEntropy(AtomicU64::new(1)));
    let provider = Arc::new(MemKeyProvider::new(Arc::clone(&entropy)));
    let clusters = [ClusterId::from_wire([1; 16]), ClusterId::from_wire([2; 16])];
    for cluster in &clusters { provider.create_cluster_key(*cluster, entropy.as_ref())?; }
    let clock = Arc::new(TestClock::new(10));
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
fn clock_floor_advancement_is_monotone_and_cas_guarded() -> Result<(), Error> {
    let clock = TestClock::new(0);
    clock.advance_utc_floor(1_000, 1_100)?;
    assert_eq!(clock.read()?.deadline_utc_millis()?, 1_100);
    assert!(matches!(clock.advance_utc_floor(1_000, 1_200), Err(Error::CasLost)));
    assert!(matches!(clock.advance_utc_floor(1_100, 1_099), Err(Error::ClockUncertain)));
    Ok(())
}

#[test]
fn missing_cluster_key_is_key_lost_not_shredded() -> Result<(), Error> {
    let provider = Arc::new(MemKeyProvider::new(Arc::new(TestEntropy(AtomicU64::new(4)))));
    let cache = KeyCache::new(provider, Arc::new(TestClock::new(0)), 100, 50, 8)?;
    assert!(matches!(cache.get(ClusterId::from_wire([9; 16]), 1), Err(Error::KeyLost)));
    Ok(())
}

#[test]
fn cluster_key_creation_is_idempotent_until_all_epochs_are_destroyed() -> Result<(), Error> {
    let entropy: Arc<dyn Entropy> = Arc::new(TestEntropy(AtomicU64::new(8)));
    let provider = MemKeyProvider::new(Arc::clone(&entropy));
    let live = ClusterId::from_wire([22; 16]);
    assert_eq!(provider.create_cluster_key(live, entropy.as_ref())?, 1);
    assert_eq!(provider.create_cluster_key(live, entropy.as_ref())?, 1);
    let first = provider.cluster_key(live, 1)?.ok_or(Error::KeyLost)?;
    let second = provider.cluster_key(live, 1)?.ok_or(Error::KeyLost)?;
    assert_eq!(first.expose(), second.expose());
    assert_eq!(provider.probe(live)?, KeyState::Present);
    Ok(())
}

#[test]
fn provider_reports_scope_wide_absence_only_after_destroy() -> Result<(), Error> {
    let entropy: Arc<dyn Entropy> = Arc::new(TestEntropy(AtomicU64::new(40)));
    let provider = MemKeyProvider::new(Arc::clone(&entropy));
    let cluster = ClusterId::from_wire([41; 16]);
    assert_eq!(provider.info().destroy, DestroyBehavior::Immediate);
    assert!(!provider.info().keys_in_backups);
    assert_eq!(provider.create_cluster_key(cluster, entropy.as_ref())?, 1);
    assert_eq!(provider.probe(cluster)?, KeyState::Present);
    assert!(provider.cluster_key(cluster, 1)?.is_some());
    let receipt = provider.destroy_many(&[cluster])?;
    assert_eq!(receipt.requested_clusters, 1);
    assert_eq!(receipt.absent_clusters, 1);
    assert_eq!(provider.probe(cluster)?, KeyState::Absent);
    assert!(provider.cluster_key(cluster, 1)?.is_none());
    Ok(())
}

#[test]
fn destroy_many_removes_every_epoch_for_the_cluster() -> Result<(), Error> {
    // INV-KEY-4: provider destruction covers every stored epoch.
    let provider = MemKeyProvider::new(Arc::new(TestEntropy(AtomicU64::new(51))));
    let cluster = ClusterId::from_wire([52; 16]);
    provider.insert_epoch_for_test(cluster, 1, [1; 32])?;
    provider.insert_epoch_for_test(cluster, 2, [2; 32])?;
    provider.destroy_many(&[cluster])?;
    assert!(provider.cluster_key(cluster, 1)?.is_none());
    assert!(provider.cluster_key(cluster, 2)?.is_none());
    assert_eq!(provider.probe(cluster)?, KeyState::Absent);
    Ok(())
}
