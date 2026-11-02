// FILE-CONTEXT
// STAND: 2026-11-02T00:00:00Z
// ZWECK: Tests für SY-05, SY-06, SY-10, T-FSYNC-1, T-LOG-1 (Dauerhaftigkeit, Fehlerpropagation, statische Fehlermeldungen)

use contextra_sys::durability::{
    flush_dir, flush_file, rename_durable, write_all_durable, FlushKind,
};
use std::fs::File;
use std::io::Write;
use std::os::unix::io::FromRawFd;

// ANCHOR[contextra-sys:TEST-SY-05-FSYNC-1]
#[test]
fn sy_05_error_unveraendert_gemeldet_t_fsync_1() {
    // Fehlerinjektion über Pipe: Ein Pipe-Deskriptor unterstützt kein fsync/fdatasync -> EBADF/EINVAL/ESPIPE
    let mut fds = [0i32; 2];
    let res = unsafe { libc::pipe(fds.as_mut_ptr()) };
    assert_eq!(res, 0, "pipe creation must succeed");

    let read_file = unsafe { File::from_raw_fd(fds[0]) };
    let write_file = unsafe { File::from_raw_fd(fds[1]) };

    // fsync auf Pipe schlägt unter POSIX/Linux mit EINVAL oder EBADF fehl
    let res_data = flush_file(&read_file, FlushKind::Data);
    assert!(res_data.is_err(), "Flush auf Pipe muss fehlschlagen");
    let err_data = match res_data {
        Err(e) => e,
        Ok(()) => unreachable!(),
    };
    assert!(
        err_data.raw_os_error() == Some(libc::EINVAL)
            || err_data.raw_os_error() == Some(libc::EBADF)
            || err_data.raw_os_error() == Some(libc::ESPIPE),
        "Fehlercode muss unverändert erhalten bleiben: {:?}",
        err_data.raw_os_error()
    );

    let res_all = flush_file(&write_file, FlushKind::All);
    assert!(res_all.is_err(), "FlushAll auf Pipe muss fehlschlagen");
    let err_all = match res_all {
        Err(e) => e,
        Ok(()) => unreachable!(),
    };
    assert!(
        err_all.raw_os_error() == Some(libc::EINVAL)
            || err_all.raw_os_error() == Some(libc::EBADF)
            || err_all.raw_os_error() == Some(libc::ESPIPE),
        "Fehlercode muss unverändert erhalten bleiben: {:?}",
        err_all.raw_os_error()
    );
}

// ANCHOR[contextra-sys:TEST-SY-06-SUCCESS]
#[test]
fn sy_06_flush_file_and_flush_dir_success() {
    let temp_dir = tempfile::tempdir().expect("tempdir");
    let file_path = temp_dir.path().join("sy_06_test.dat");

    let mut file = File::create(&file_path).expect("create file");
    write_all_durable(&mut file, b"test-sy-06-payload").expect("write durable");

    flush_file(&file, FlushKind::Data).expect("flush data");
    flush_file(&file, FlushKind::All).expect("flush all");

    flush_dir(temp_dir.path()).expect("flush dir");
}

// ANCHOR[contextra-sys:TEST-SY-06-RENAME]
#[test]
fn sy_06_rename_durable() {
    let temp_dir = tempfile::tempdir().expect("tempdir");
    let src = temp_dir.path().join("src.dat");
    let dst = temp_dir.path().join("dst.dat");

    {
        let mut file = File::create(&src).expect("create src");
        file.write_all(b"durable rename content").expect("write");
        flush_file(&file, FlushKind::All).expect("flush");
    }

    rename_durable(&src, &dst).expect("rename durable");

    assert!(!src.exists());
    assert!(dst.exists());
    let content = std::fs::read(&dst).expect("read dst");
    assert_eq!(content, b"durable rename content");
}

// ANCHOR[contextra-sys:TEST-SY-10-LOG-1]
#[test]
fn sy_10_sync_failure_display_t_log_1() {
    let mut fds = [0i32; 2];
    let res = unsafe { libc::pipe(fds.as_mut_ptr()) };
    assert_eq!(res, 0);

    let f = unsafe { File::from_raw_fd(fds[0]) };
    let _writer = unsafe { File::from_raw_fd(fds[1]) };

    let err = match flush_file(&f, FlushKind::Data) {
        Err(e) => e,
        Ok(()) => unreachable!(),
    };

    let display_str = format!("{err}");
    let debug_str = format!("{err:?}");

    // T-LOG-1: Keine Pfade oder Nutzerinhalte in Fehlermeldungen
    assert!(
        !display_str.contains('/'),
        "Display darf keine Dateipfade enthalten: {display_str}"
    );
    assert!(
        !display_str.contains('\\'),
        "Display darf keine Dateipfade enthalten: {display_str}"
    );
    assert!(
        !debug_str.contains('/'),
        "Debug darf keine Dateipfade enthalten: {debug_str}"
    );
}
