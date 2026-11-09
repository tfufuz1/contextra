use contextra_durable_fs::vfs::{StdVfs, Vfs};
use tempfile::tempdir;

#[test]
fn test_vfs_conformance_basic_ops() {
    let dir = tempdir().expect("tempdir");
    let vfs = StdVfs;

    let file_path = dir.path().join("vfs_test.txt");
    assert!(!vfs.exists(&file_path));

    // write_atomic
    vfs.write_atomic(&file_path, b"hello vfs")
        .expect("write atomic");
    assert!(vfs.exists(&file_path));
    assert_eq!(vfs.read(&file_path).expect("read"), b"hello vfs");
    assert_eq!(vfs.len(&file_path).expect("len"), 9);

    // append
    vfs.append(&file_path, b" appended").expect("append");
    assert_eq!(vfs.read(&file_path).expect("read"), b"hello vfs appended");

    // list
    let list = vfs.list(dir.path()).expect("list");
    assert!(list.contains(&"vfs_test.txt".to_string()));

    // rename
    let renamed = dir.path().join("renamed.txt");
    vfs.rename(&file_path, &renamed).expect("rename");
    assert!(!vfs.exists(&file_path));
    assert!(vfs.exists(&renamed));

    // remove
    vfs.remove(&renamed).expect("remove");
    assert!(!vfs.exists(&renamed));
}
