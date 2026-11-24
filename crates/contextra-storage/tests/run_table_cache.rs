use contextra_storage::env::memfs::MemFs;
use contextra_storage::store::run::table_cache::TableCache;
use contextra_storage_ports::traits::{FileRole, Fs, IoCtx};
use std::path::Path;

const IO: IoCtx = IoCtx { role: FileRole::Run, purpose: "table_cache_test" };

fn create(fs: &dyn Fs, path: &str) {
    let file = fs.create_new(Path::new(path), IO).unwrap();
    file.preallocate(128, IO).unwrap();
    file.sync_all(IO).unwrap();
}

#[test]
fn cache_enforces_capacity_and_pins_handles_until_readers_release_them() {
    let fs = MemFs::new();
    create(&fs, "/runs/a");
    create(&fs, "/runs/b");
    let cache = TableCache::new(1).unwrap();
    let a = cache.pin(&fs, Path::new("/runs/a")).unwrap();
    assert_eq!(cache.resident().unwrap(), 1);
    assert!(cache.pin(&fs, Path::new("/runs/b")).is_err());
    drop(a);
    let b = cache.pin(&fs, Path::new("/runs/b")).unwrap();
    assert_eq!(cache.resident().unwrap(), 1);
    drop(b);
    assert_eq!(cache.evict_unpinned().unwrap(), 1);
    assert_eq!(cache.resident().unwrap(), 0);
}

#[test]
fn cache_reuses_handle_and_rejects_zero_capacity() {
    let fs = MemFs::new();
    create(&fs, "/runs/a");
    assert!(TableCache::new(0).is_err());
    let cache = TableCache::new(2).unwrap();
    let first = cache.pin(&fs, Path::new("/runs/a")).unwrap();
    let second = cache.pin(&fs, Path::new("/runs/a")).unwrap();
    assert!(std::sync::Arc::ptr_eq(&first, &second));
    assert_eq!(cache.resident().unwrap(), 1);
}
