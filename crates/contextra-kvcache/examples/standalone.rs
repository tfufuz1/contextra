#![allow(
    clippy::all,
    clippy::type_complexity,clippy::expect_used)]
//! Standalone-Beispiel für `contextra-kvcache` (<= 30 Zeilen).
use contextra_ids::TenantId;
use contextra_kvcache::contract::{KvCacheEngine, MemoryKvCache};
use contextra_kvcache::radix::KvReusePolicy;
use contextra_kvcache::segment::KvSegment;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cache = MemoryKvCache::new();
    let tenant = TenantId::try_new(1)?;

    let segment = KvSegment::new(tenant, 100, vec![0xAA, 0xBB, 0xCC, 0xDD]);
    cache.insert(tenant, segment)?;

    let data = cache.get(tenant, 100).expect("segment 100 exists");
    assert_eq!(data, vec![0xAA, 0xBB, 0xCC, 0xDD]);

    let _prefix = cache.find_longest_prefix(tenant, &[101, 102], KvReusePolicy::Always);
    println!("contextra-kvcache standalone example finished successfully.");
    Ok(())
}
