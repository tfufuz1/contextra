use contextra_storage::env::memfs::MemFs;
use contextra_storage::maint::reserve::{ScratchReserve, REQUIRED_PURGE_RESERVE_BYTES};
use contextra_storage_ports::Error;
use std::path::Path;

#[test]
fn erase_budget_preallocates_two_protected_slots_and_limits_rewrite_concurrency() -> Result<(), Error> {
    let fs = MemFs::new();
    let root = Path::new("/store");
    let reserve = ScratchReserve::open(&fs, root, REQUIRED_PURGE_RESERVE_BYTES / 2)?;
    assert_eq!(reserve.slot_bytes(), REQUIRED_PURGE_RESERVE_BYTES / 2);
    let active = reserve.writer_slot()?;
    assert!(reserve.writer_slot().is_err());
    assert!(reserve.recovery_path().ends_with("PURGE-SCRATCH-1"));
    drop(active);
    assert!(reserve.writer_slot().is_ok());

    let crashed = fs.crash_image()?;
    let reopened = ScratchReserve::open(&crashed, root, REQUIRED_PURGE_RESERVE_BYTES / 2)?;
    assert_eq!(reopened.slot_bytes(), REQUIRED_PURGE_RESERVE_BYTES / 2);
    Ok(())
}

#[test]
fn erase_budget_rejects_small_or_insufficient_reserve() {
    let fs = MemFs::new();
    assert!(ScratchReserve::open(&fs, Path::new("/store"), 16 * 1024 * 1024).is_err());
    fs.set_free_space(1);
    assert!(ScratchReserve::open(&fs, Path::new("/store"), REQUIRED_PURGE_RESERVE_BYTES / 2).is_err());
}
