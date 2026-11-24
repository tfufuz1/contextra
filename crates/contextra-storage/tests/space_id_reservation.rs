use contextra_storage::catalog::ops::CatalogOp;
use contextra_storage::catalog::reserve::{after_reopen, next_space_id_reservation, SPACE_ID_RESERVATION};
use contextra_storage::catalog::state::CatalogState;
use contextra_storage::catalog::file::{CatalogFile, OpenMode};
use contextra_storage::catalog::record::CatalogIntegrity;
use contextra_storage::env::memfs::MemFs;
use contextra_storage::ports::{Error, Protection, SpaceId, FIRST_USER_SPACE_ID, SYSTEM_SPACE_ID_MAX};
use std::path::Path;

#[test]
fn user_space_ids_start_after_system_range_and_reserve_as_u64() {
    assert!(matches!(SpaceId::user(SYSTEM_SPACE_ID_MAX), Err(Error::ReservedSpace)));
    assert_eq!(SpaceId::user(FIRST_USER_SPACE_ID).unwrap(), SpaceId(256));

    let (op, reservation) = next_space_id_reservation(SYSTEM_SPACE_ID_MAX, SPACE_ID_RESERVATION).unwrap();
    assert_eq!(reservation.first, 256);
    assert_eq!(reservation.through, 65_791);
    let mut encoded = Vec::new();
    op.encode(&mut encoded).unwrap();
    // Catalog op header is tag u16 + payload length u32; u64 space watermark
    // follows in canonical big-endian order.
    let watermark = 65_791u64.to_be_bytes();
    assert_eq!(encoded.get(6..14), Some(watermark.as_slice()));
    let (decoded, used) = CatalogOp::decode(&encoded).unwrap();
    assert_eq!(used, encoded.len());
    assert_eq!(decoded, CatalogOp::ReserveSpaceIds { through: 65_791 });
}

#[test]
fn catalog_rejects_unreserved_user_ids_and_recovery_never_reuses_ranges() {
    let mut state = CatalogState::default();
    assert!(state.apply(&CatalogOp::SpaceAdd { space: SpaceId(256), protection: Protection::Plain }).is_err());
    let (reserve, range) = next_space_id_reservation(state.space_id_limit(), 3).unwrap();
    state.apply(&reserve).unwrap();
    state.apply(&CatalogOp::SpaceAdd { space: SpaceId(range.first), protection: Protection::Plain }).unwrap();
    assert_eq!(state.space_id_limit(), range.through);
    assert_eq!(after_reopen(8, 19, state.space_id_limit()).unwrap(), (9, 20, range.through + 1));
    assert_eq!(after_reopen(8, 19, SYSTEM_SPACE_ID_MAX).unwrap().2, FIRST_USER_SPACE_ID);
}

#[test]
fn space_id_reservation_is_monotone_and_never_wraps() {
    let (reserve, _) = next_space_id_reservation(0, 1).unwrap();
    let mut state = CatalogState::default();
    state.apply(&reserve).unwrap();
    assert!(state.apply(&CatalogOp::ReserveSpaceIds { through: SYSTEM_SPACE_ID_MAX }).is_err());
    assert!(next_space_id_reservation(u64::MAX, 1).is_err());
    assert!(after_reopen(8, 19, u64::MAX).is_err());
}

#[test]
fn catalog_returns_space_ids_only_after_durable_reservation_and_skips_tail_on_reopen() {
    let fs = MemFs::new();
    let path = Path::new("/catalog");
    let mut catalog = CatalogFile::open(
        &fs, path, OpenMode::ReadWrite, CatalogIntegrity::Plain, None, None,
    ).unwrap();
    let first_range = catalog.reserve_space_ids_durable(4, CatalogIntegrity::Plain, None, None).unwrap();
    assert_eq!((first_range.first, first_range.through), (256, 259));
    drop(catalog);

    let mut reopened = CatalogFile::open(
        &fs, path, OpenMode::ReadWrite, CatalogIntegrity::Plain, None, None,
    ).unwrap();
    let second_range = reopened.reserve_space_ids_durable(2, CatalogIntegrity::Plain, None, None).unwrap();
    assert_eq!((second_range.first, second_range.through), (260, 261));
    assert_eq!(reopened.state().space_id_limit(), 261);
}
