use contextra_storage_ports::{Error, ErrorClass};
use std::io::ErrorKind;

#[test]
fn transient_io_is_distinguished_from_hard_io() {
    let interrupted = Error::Io { role: "redo", kind: ErrorKind::Interrupted };
    let hard = Error::Io { role: "redo", kind: ErrorKind::Other };
    assert_eq!(interrupted.class(), ErrorClass::Transient);
    assert_eq!(hard.class(), ErrorClass::Internal);
}

#[test]
fn indeterminate_and_disk_full_keep_their_recovery_classes() {
    assert_eq!(
        Error::Indeterminate { first_stamp: 7, last_stamp: 9 }.class(),
        ErrorClass::Indeterminate,
    );
    assert_eq!(Error::DiskFull.class(), ErrorClass::DiskFull);
}

#[test]
fn erased_error_has_a_static_action_hint() {
    let error = Error::Erased { at: 123 };
    assert_eq!(error.class(), ErrorClass::Usage);
    assert!(!error.hint().contains("123"));
}
