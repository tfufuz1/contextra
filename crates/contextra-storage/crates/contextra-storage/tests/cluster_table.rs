use contextra_storage::erase::cluster_table::{ClusterState, ClusterTable};
use contextra_storage::ports::{ClusterId, Error, Protection};

#[test]
fn lifecycle_is_monotone_and_shredding_requires_sealed_protection() -> Result<(), Error> {
    let sealed = ClusterId::from_wire([1; 16]);
    let plain = ClusterId::from_wire([2; 16]);
    let mut table = ClusterTable::new();
    table.register(sealed, Protection::Sealed)?;
    table.register(plain, Protection::Plain)?;
    let initial_generation = table.get(sealed).ok_or(Error::Corrupt { place: "test cluster" })?.generation;
    table.mark_retired(sealed, 7)?;
    assert_eq!(table.require_live(sealed).err().map(|error| error.class()), Some(Error::ClusterRetired.class()));
    table.mark_shredded(sealed)?;
    table.mark_purged(sealed)?;
    let final_entry = table.get(sealed).ok_or(Error::Corrupt { place: "test cluster" })?;
    assert_eq!(final_entry.state, ClusterState::Purged);
    assert_eq!(final_entry.generation, initial_generation + 3);
    assert!(table.mark_retired(sealed, 8).is_err());

    table.mark_retired(plain, 9)?;
    assert!(table.mark_shredded(plain).is_err());
    table.mark_purged(plain)?;
    Ok(())
}

#[test]
fn duplicate_registration_does_not_replace_existing_protection() -> Result<(), Error> {
    let cluster = ClusterId::from_wire([3; 16]);
    let mut table = ClusterTable::new();
    table.register(cluster, Protection::Sealed)?;
    assert!(table.register(cluster, Protection::Plain).is_err());
    assert_eq!(table.get(cluster).map(|entry| entry.protection), Some(Protection::Sealed));
    Ok(())
}
