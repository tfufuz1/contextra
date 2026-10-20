use contextra_durable_fs::{
    atomic_replace, atomic_replace_with_injector, durable_remove_with_injector, FaultInjector,
};
use std::fs;
use std::io::{Error, ErrorKind, Result};
use std::path::Path;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;
use std::thread;

#[derive(Debug, Default)]
struct StepFaultInjector {
    fail_write: AtomicBool,
    fail_fsync: AtomicBool,
    fail_rename: AtomicBool,
    fail_dir_fsync: AtomicBool,
    write_count: AtomicUsize,
    fsync_count: AtomicUsize,
    rename_count: AtomicUsize,
    dir_fsync_count: AtomicUsize,
}

impl StepFaultInjector {
    fn reset(&self) {
        self.fail_write.store(false, Ordering::SeqCst);
        self.fail_fsync.store(false, Ordering::SeqCst);
        self.fail_rename.store(false, Ordering::SeqCst);
        self.fail_dir_fsync.store(false, Ordering::SeqCst);
        self.write_count.store(0, Ordering::SeqCst);
        self.fsync_count.store(0, Ordering::SeqCst);
        self.rename_count.store(0, Ordering::SeqCst);
        self.dir_fsync_count.store(0, Ordering::SeqCst);
    }
}

impl FaultInjector for StepFaultInjector {
    fn on_write(&self, _path: Option<&Path>, _bytes: usize) -> Result<()> {
        self.write_count.fetch_add(1, Ordering::SeqCst);
        if self.fail_write.load(Ordering::SeqCst) {
            Err(Error::new(ErrorKind::Other, "injected write failure"))
        } else {
            Ok(())
        }
    }

    fn on_fsync(&self, _path: Option<&Path>) -> Result<()> {
        self.fsync_count.fetch_add(1, Ordering::SeqCst);
        if self.fail_fsync.load(Ordering::SeqCst) {
            Err(Error::new(ErrorKind::Other, "injected fsync failure"))
        } else {
            Ok(())
        }
    }

    fn on_rename(&self, _from: &Path, _to: &Path) -> Result<()> {
        self.rename_count.fetch_add(1, Ordering::SeqCst);
        if self.fail_rename.load(Ordering::SeqCst) {
            Err(Error::new(ErrorKind::Other, "injected rename failure"))
        } else {
            Ok(())
        }
    }

    fn on_dir_fsync(&self, _dir: &Path) -> Result<()> {
        self.dir_fsync_count.fetch_add(1, Ordering::SeqCst);
        if self.fail_dir_fsync.load(Ordering::SeqCst) {
            Err(Error::new(ErrorKind::Other, "injected dir_fsync failure"))
        } else {
            Ok(())
        }
    }
}

#[test]
fn test_atomic_replace_fault_matrix_all_or_nothing() {
    let temp = tempfile::tempdir().expect("tempdir created");
    let target = temp.path().join("data.bin");

    let initial_data = b"INITIAL_STATE_12345";
    fs::write(&target, initial_data).expect("initial write");

    let injector = Arc::new(StepFaultInjector::default());
    let new_data = b"NEW_STATE_67890_UPDATED";

    // 1. Fail on write
    injector.reset();
    injector.fail_write.store(true, Ordering::SeqCst);
    let res = atomic_replace_with_injector(&target, new_data, &*injector);
    assert!(res.is_err(), "write failure must be propagated");
    assert_eq!(
        fs::read(&target).expect("read target"),
        initial_data,
        "target must remain untouched on write failure"
    );

    // 2. Fail on fsync
    injector.reset();
    injector.fail_fsync.store(true, Ordering::SeqCst);
    let res = atomic_replace_with_injector(&target, new_data, &*injector);
    assert!(res.is_err(), "fsync failure must be propagated");
    assert_eq!(
        fs::read(&target).expect("read target"),
        initial_data,
        "target must remain untouched on fsync failure"
    );

    // 3. Fail on rename
    injector.reset();
    injector.fail_rename.store(true, Ordering::SeqCst);
    let res = atomic_replace_with_injector(&target, new_data, &*injector);
    assert!(res.is_err(), "rename failure must be propagated");
    assert_eq!(
        fs::read(&target).expect("read target"),
        initial_data,
        "target must remain untouched on rename failure"
    );

    // 4. Fail on dir_fsync
    injector.reset();
    injector.fail_dir_fsync.store(true, Ordering::SeqCst);
    let res = atomic_replace_with_injector(&target, new_data, &*injector);
    assert!(res.is_err(), "dir_fsync failure must be propagated");
    // Note: since rename succeeded prior to dir_fsync, the file on disk was updated atomically
    assert_eq!(
        fs::read(&target).expect("read target"),
        new_data,
        "atomic replacement succeeded before dir_fsync failure"
    );

    // Ensure temp files are cleaned up
    let entries: Vec<_> = fs::read_dir(temp.path())
        .expect("read_dir")
        .filter_map(|e| e.ok())
        .collect();
    assert_eq!(
        entries.len(),
        1,
        "no leftover .tmp files must remain in directory"
    );
}

#[test]
fn test_concurrent_atomic_replace_same_file() {
    let temp = tempfile::tempdir().expect("tempdir created");
    let target = Arc::new(temp.path().join("shared_target.dat"));

    let initial = b"INITIAL_0000000000000000";
    fs::write(&*target, initial).expect("write initial");

    let num_threads = 8;
    let iterations_per_thread = 20;

    let barrier = Arc::new(std::sync::Barrier::new(num_threads));
    let mut handles = Vec::new();

    for t_id in 0..num_threads {
        let target = Arc::clone(&target);
        let barrier = Arc::clone(&barrier);
        handles.push(thread::spawn(move || {
            barrier.wait();
            for i in 0..iterations_per_thread {
                // Construct a distinct fixed-size pattern for thread t_id and iteration i
                let payload = format!("THREAD_{:02}_ITER_{:04}_DATA_PAYLOAD!", t_id, i);
                atomic_replace(&target, payload.as_bytes()).expect("atomic_replace succeed");
            }
        }));
    }

    for h in handles {
        h.join().expect("thread join");
    }

    let final_bytes = fs::read(&*target).expect("read final file");
    let final_str = String::from_utf8_lossy(&final_bytes);
    assert!(
        final_str.starts_with("THREAD_"),
        "final file must contain valid thread output, got: {}",
        final_str
    );
    assert!(
        final_str.ends_with("_DATA_PAYLOAD!"),
        "final file must not be partially written or corrupted, got: {}",
        final_str
    );
}

#[test]
fn test_atomic_replace_edge_cases() {
    let temp = tempfile::tempdir().expect("tempdir created");

    // Edge case 1: 0-byte file
    let empty_target = temp.path().join("empty.dat");
    atomic_replace(&empty_target, b"").expect("atomic_replace 0 bytes");
    assert_eq!(fs::read(&empty_target).expect("read empty"), b"");

    // Edge case 2: Large file > 2MB (larger than typical OS page / block sizes)
    let large_target = temp.path().join("large.bin");
    let large_payload = vec![0xAB; 2 * 1024 * 1024 + 137];
    atomic_replace(&large_target, &large_payload).expect("atomic_replace large payload");
    assert_eq!(fs::read(&large_target).expect("read large"), large_payload);

    // Edge case 3: Unicode and spaces in path
    let unicode_target = temp
        .path()
        .join("Ordner mit spaces & Umlaute ääöü/Prüfung #1.txt");
    fs::create_dir_all(unicode_target.parent().unwrap()).expect("create parent unicode dir");
    let unicode_data = b"Unicode Path Test Payload \xF0\x9F\xA6\x80";
    atomic_replace(&unicode_target, unicode_data).expect("atomic_replace unicode");
    assert_eq!(
        fs::read(&unicode_target).expect("read unicode"),
        unicode_data
    );

    // Edge case 4: Target is a directory
    let dir_target = temp.path().join("is_a_directory");
    fs::create_dir_all(&dir_target).expect("create dir_target");
    let err = atomic_replace(&dir_target, b"some content");
    assert!(err.is_err(), "replacing a directory with a file must fail");

    // Edge case 5: Non-existent parent directory
    let invalid_parent_target = temp.path().join("non_existent_dir/file.txt");
    let err = atomic_replace(&invalid_parent_target, b"content");
    assert!(
        err.is_err(),
        "writing to non-existent parent directory must fail"
    );
}

#[test]
fn test_durable_remove_fault_injection() {
    let temp = tempfile::tempdir().expect("tempdir created");
    let file_path = temp.path().join("to_remove.txt");
    fs::write(&file_path, b"delete_me").expect("write file_path");

    let injector = StepFaultInjector::default();
    injector.fail_dir_fsync.store(true, Ordering::SeqCst);

    let res = durable_remove_with_injector(&file_path, &injector);
    assert!(
        res.is_err(),
        "durable_remove must propagate dir_fsync error"
    );
    assert!(
        !file_path.exists(),
        "file must be removed before dir_fsync attempt"
    );
}
