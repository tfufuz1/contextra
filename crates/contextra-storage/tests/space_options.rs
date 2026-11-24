use contextra_storage::codec::format::IntegrityMode;
use contextra_storage::ports::{Durability, Error, Protection, SpaceId};
use contextra_storage::spaces::{DataClass, SealPolicy, SpaceClass, SpaceOptions, SpaceSpec};

#[test]
fn personal_and_volatile_space_constraints_are_enforced() {
    let mut personal = SpaceOptions { data_class: DataClass::Personal, seal: SealPolicy::Cluster, ..SpaceOptions::default() };
    assert!(personal.validate(IntegrityMode::Encrypted).is_ok());
    assert!(personal.validate(IntegrityMode::Authenticated).is_err());
    personal.clustered = false;
    assert!(personal.validate(IntegrityMode::Encrypted).is_err());

    let volatile = SpaceOptions::volatile();
    assert_eq!(volatile.class, SpaceClass::Volatile);
    assert_eq!(volatile.durability_default, Durability::Relaxed);
    assert!(volatile.validate(IntegrityMode::Plain).is_ok());
    assert!(SpaceOptions { class: SpaceClass::Volatile, ..SpaceOptions::default() }
        .validate(IntegrityMode::Plain).is_err());
}

#[test]
fn space_bounds_and_protection_policy_are_checked() -> Result<(), Error> {
    let spec = SpaceSpec { id: SpaceId(256), name: "documents".to_owned(), options: SpaceOptions::default() };
    spec.validate(IntegrityMode::Plain)?;
    assert!(spec.options.protection_allowed(Protection::Plain));
    let personal = SpaceOptions { data_class: DataClass::Personal, seal: SealPolicy::Cluster, ..SpaceOptions::default() };
    assert!(!personal.protection_allowed(Protection::Plain));
    assert!(SpaceSpec { name: String::new(), ..spec.clone() }.validate(IntegrityMode::Plain).is_err());
    assert!(SpaceSpec { id: SpaceId(255), ..spec.clone() }.validate(IntegrityMode::Plain).is_err());
    assert!(SpaceOptions { weight_slots: 5, ..SpaceOptions::default() }.validate(IntegrityMode::Plain).is_err());
    assert!(SpaceOptions { max_key_bytes: 4097, ..SpaceOptions::default() }.validate(IntegrityMode::Plain).is_err());
    Ok(())
}
