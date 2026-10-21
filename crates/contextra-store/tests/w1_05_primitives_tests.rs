use contextra_durable_fs::{
    durable_write_atomic, remove_if_exists, FailpointStep, FailpointVfs,
};
use tempfile::TempDir;

#[tokio::test]
async fn durable_write_atomic_failpoint_matrix() {
    let temp_dir = TempDir::new().expect("tempdir");
    let target_path = temp_dir.path().join("atomic_target.dat");
    let payload = b"durable_atomic_write_content_payload";

    let steps = vec![
        FailpointStep::Write,
        FailpointStep::FsyncFile,
        FailpointStep::Rename,
        FailpointStep::SyncDir,
    ];

    for step in steps {
        let vfs = FailpointVfs::new();
        vfs.set_fail_step(step);

        let res = durable_write_atomic(&vfs, &target_path, payload).await;
        assert!(
            res.is_err(),
            "durable_write_atomic MUST fail when error is injected at step {:?}",
            step
        );
        assert!(
            vfs.call_count(step) > 0,
            "Failpoint step {:?} must have been invoked",
            step
        );
        if step != FailpointStep::SyncDir {
            assert!(
                !target_path.exists(),
                "Target file must NOT exist on failure at step {:?}",
                step
            );
        }
    }

    // Success path without failpoint
    let vfs = FailpointVfs::new();
    let res = durable_write_atomic(&vfs, &target_path, payload).await;
    assert!(res.is_ok(), "durable_write_atomic must succeed with no faults");
    assert!(target_path.exists(), "Target file must exist after successful write");
    let read_data = std::fs::read(&target_path).expect("read file");
    assert_eq!(read_data, payload);
}

#[tokio::test]
async fn remove_if_exists_notfound_is_ok_other_err() {
    let temp_dir = TempDir::new().expect("tempdir");
    let non_existent_path = temp_dir.path().join("non_existent.dat");
    let existent_path = temp_dir.path().join("existent.dat");

    std::fs::write(&existent_path, b"test").expect("write test file");

    let vfs = FailpointVfs::new();

    // Removing non-existent file returns Ok(())
    let res_notfound = remove_if_exists(&vfs, &non_existent_path).await;
    assert!(
        res_notfound.is_ok(),
        "remove_if_exists on non-existent file must return Ok(())"
    );

    // Removing existing file returns Ok(()) and deletes file
    let res_existent = remove_if_exists(&vfs, &existent_path).await;
    assert!(
        res_existent.is_ok(),
        "remove_if_exists on existing file must return Ok(())"
    );
    assert!(!existent_path.exists(), "Existent file must be removed");

    // Injected error during Remove step returns Err
    std::fs::write(&existent_path, b"test2").expect("write test file 2");
    vfs.set_fail_step(FailpointStep::Remove);
    let res_err = remove_if_exists(&vfs, &existent_path).await;
    assert!(
        res_err.is_err(),
        "remove_if_exists must propagate non-NotFound errors"
    );
}
