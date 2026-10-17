use contextra_durable_fs::{atomic_replace, DirLock, DurableHandle, LockMode};
use contextra_sys::{FlushKind, FsClass};
use std::fs::File;

#[test]
fn test_durable_handle_poisoning_on_failure() -> std::io::Result<()> {
    let temp_dir = std::env::temp_dir();
    let file_path = temp_dir.join(format!(
        "contextra_durable_handle_test_{}.bin",
        std::process::id()
    ));

    let file = File::create(&file_path)?;
    let mut handle = DurableHandle::new(file);

    assert!(!handle.is_poisoned());
    assert!(handle.get_ref().is_some());

    handle.flush(FlushKind::DataOnly)?;

    handle.poison();
    assert!(handle.is_poisoned());
    assert!(handle.get_ref().is_none());

    let err = handle.flush(FlushKind::DataOnly);
    assert!(err.is_err());

    let _ = std::fs::remove_file(file_path);
    Ok(())
}

#[test]
fn test_atomic_replace_property_consistency() -> std::io::Result<()> {
    let temp_dir = std::env::temp_dir();
    let file_path = temp_dir.join(format!(
        "contextra_atomic_property_test_{}.bin",
        std::process::id()
    ));

    let initial_data = b"INITIAL_STATE_MEMBER_1";
    atomic_replace(&file_path, initial_data)?;

    for i in 0..20 {
        let new_data = format!("STATE_UPDATE_{}", i);
        atomic_replace(&file_path, new_data.as_bytes())?;

        let read_back = std::fs::read(&file_path)?;
        assert_eq!(read_back, new_data.as_bytes());
    }

    let _ = std::fs::remove_file(file_path);
    Ok(())
}

#[test]
fn test_dirlock_cloud_sync_rejection_and_memory_only_override() -> std::io::Result<()> {
    let temp_dir = std::env::temp_dir();
    let cloud_dir = temp_dir.join(format!("OneDrive_Mock_{}/data", std::process::id()));

    std::fs::create_dir_all(&cloud_dir)?;

    let class = contextra_sys::fs_class(&cloud_dir)?;
    assert_eq!(class, FsClass::CloudSync);

    let lock_res = DirLock::acquire(&cloud_dir, LockMode::Normal);
    assert!(lock_res.is_err());

    if let Err(err) = lock_res {
        let err_msg = err.to_string();
        assert!(err_msg.contains("cloud_sync"));
        assert!(!err_msg.contains("OneDrive_Mock_"));
    }

    let lock_mem = DirLock::acquire(&cloud_dir, LockMode::MemoryOnly)?;
    assert_eq!(lock_mem.mode(), LockMode::MemoryOnly);

    if let Some(parent) = cloud_dir.parent() {
        let _ = std::fs::remove_dir_all(parent);
    }
    Ok(())
}
