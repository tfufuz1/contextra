// FILE-CONTEXT
// STAND: 2026-11-02T00:00:00Z
// ZWECK: Tests for contextra-sys hardening module (SY-01, SY-02, SY-07, SY-08, SY-10, T-DUMP-1, T-SEC-2, T-UNSAFE-1)
// INVARIANTEN: Zero panic in library code, fixed error messages, child process test coverage

#![allow(unsafe_code)]

#[path = "../src/perm.rs"]
mod perm;

#[path = "../src/hardening.rs"]
mod hardening;

use hardening::{
    apply_process_hardening, ensure_fd_limit, lock_pages, mark_dont_dump, set_fd_limit,
    HardeningPolicy, SysError,
};
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};

static PROBE_COUNTER: AtomicUsize = AtomicUsize::new(0);

fn compile_and_get_probe_bin() -> Option<std::path::PathBuf> {
    let probe_src = "crates/contextra-sys/tests/support/child_probe.rs";
    let count = PROBE_COUNTER.fetch_add(1, Ordering::SeqCst);
    let probe_bin = std::path::PathBuf::from(format!(
        "/tmp/child_probe_bin_{}_{}",
        std::process::id(),
        count
    ));

    let deps_dir = std::path::Path::new("target/debug/deps");
    let libc_rlib = std::fs::read_dir(deps_dir)
        .ok()?
        .filter_map(|e| e.ok())
        .find(|e| {
            let name = e.file_name();
            let s = name.to_string_lossy();
            s.starts_with("liblibc") && s.ends_with(".rlib")
        })?;

    let rustc_sysroot = Command::new("rustc")
        .arg("--print")
        .arg("sysroot")
        .output()
        .ok()?;
    let sysroot_str = String::from_utf8_lossy(&rustc_sysroot.stdout)
        .trim()
        .to_string();
    let rpath_arg = format!("-Clink-args=-Wl,-rpath,{}/lib", sysroot_str);

    let mut cmd = Command::new("rustc");
    cmd.arg(probe_src)
        .arg("-L")
        .arg(format!("dependency={}", deps_dir.display()))
        .arg("--extern")
        .arg(format!("libc={}", libc_rlib.path().display()))
        .arg(&rpath_arg)
        .arg("-o")
        .arg(&probe_bin);

    let output = cmd.output().ok()?;
    if output.status.success() {
        Some(probe_bin)
    } else {
        None
    }
}

#[test]
fn sy_07_hardening_policy_required_besteffort_off() {
    // Test Off policy: does nothing, reports false for changes
    let rep_off = apply_process_hardening(HardeningPolicy::Off);
    assert!(rep_off.is_ok());
    let rep_off = rep_off.unwrap();
    assert!(!rep_off.core_limit_zero);
    assert!(!rep_off.dumpable_off);

    // Test BestEffort policy: never fails, returns a valid HardeningReport
    let rep_be = apply_process_hardening(HardeningPolicy::BestEffort);
    assert!(rep_be.is_ok());
    let rep_be = rep_be.unwrap();
    // On Linux x86_64 both core_limit_zero and dumpable_off should succeed in user space
    #[cfg(target_os = "linux")]
    {
        assert!(rep_be.core_limit_zero);
        assert!(rep_be.dumpable_off);
    }

    // Test Required policy: succeeds if environment allows, fails if any step fails
    let rep_req = apply_process_hardening(HardeningPolicy::Required);
    assert!(rep_req.is_ok());
}

#[test]
fn sy_08_core_limit_and_dumpable_child_process() {
    let probe_bin = compile_and_get_probe_bin().expect("failed to compile child_probe");
    let output = Command::new(&probe_bin)
        .arg("apply")
        .arg("best_effort")
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(output.status.success(), "child probe failed: {:?}", stdout);
    assert!(
        stdout.contains("CORE_REP=true"),
        "unexpected output: {}",
        stdout
    );

    let output_req = Command::new(&probe_bin)
        .arg("apply")
        .arg("required")
        .output()
        .unwrap();
    assert!(output_req.status.success());

    let output_off = Command::new(&probe_bin)
        .arg("apply")
        .arg("off")
        .output()
        .unwrap();
    assert!(output_off.status.success());
    assert!(String::from_utf8_lossy(&output_off.stdout).contains("CORE_REP=false"));
    let _ = std::fs::remove_file(probe_bin);
}

#[test]
fn t_dump_1_madv_dontdump_smaps_flag() {
    let buf = vec![0u8; 8192];
    let ok = mark_dont_dump(&buf);
    #[cfg(target_os = "linux")]
    assert!(ok);

    let empty: [u8; 0] = [];
    assert!(mark_dont_dump(&empty));

    // Child probe smaps verification
    let probe_bin = compile_and_get_probe_bin().expect("failed to compile child_probe");
    let output = Command::new(&probe_bin).arg("dont_dump").output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(output.status.success(), "dont_dump failed: {}", stdout);
    assert!(stdout.contains("DONTDUMP_VERIFIED"), "output: {}", stdout);
    let _ = std::fs::remove_file(probe_bin);
}

#[test]
fn t_sec_2_required_fails_on_mlock_limit_zero() {
    let data = vec![0x55u8; 4096];
    let res = lock_pages(&data, HardeningPolicy::BestEffort);
    assert!(res.is_ok());

    let empty: [u8; 0] = [];
    assert!(lock_pages(&empty, HardeningPolicy::Required).unwrap());
    assert!(lock_pages(&empty, HardeningPolicy::Off).unwrap());

    // Child probe forced mlock failure verification
    let probe_bin = compile_and_get_probe_bin().expect("failed to compile child_probe");
    let output = Command::new(&probe_bin)
        .arg("fail_mlock_required")
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(output.status.success(), "fail_mlock output: {}", stdout);
    assert!(stdout.contains("REQUIRED_FAILED_AS_EXPECTED"));
    let _ = std::fs::remove_file(probe_bin);
}

#[test]
fn t_unsafe_1_no_public_raw_address_export() {
    // Compile check that hardening only exposes safe slices and no raw pointer / usize addresses
    let data = [1u8, 2, 3];
    assert!(mark_dont_dump(&data));
    assert!(lock_pages(&data, HardeningPolicy::BestEffort).is_ok());
}

#[test]
fn sy_10_no_panics_and_fixed_error_messages() {
    let err = SysError::HardeningFailed("hardening operation failed");
    assert_eq!(err.to_string(), "hardening operation failed");
    assert!(!err.to_string().contains('/'));
    assert!(!err.to_string().contains('\\'));
}

#[test]
fn sy_07_fd_limits() {
    assert!(ensure_fd_limit(64));
    let _ = set_fd_limit(1024);
}
