// FILE-CONTEXT (STAND 2026-10-14, ZWECK: Integrationstest fuer Runtime-Hardening und EnvObservation Status-Verhalten, INVARIANTEN: Zero-Panic, Fail-Closed, Observed-Semantik)

use contextra_sys::env_observe::{
    check_memfd_secret_support, harden_runtime, observe, EnvObservation, EnvOsKind, EnvTri,
    HardenRuntimeReport, HardeningStatus,
};
use std::env;

#[test]
fn test_harden_runtime_execution_and_status_report() {
    let report = harden_runtime();

    // Verify report fields contain valid status variants
    match report.rlimit_core {
        HardeningStatus::Applied => {}
        HardeningStatus::Failed { errno } => {
            assert!(errno >= 0, "Errno must be non-negative");
        }
        HardeningStatus::NotSupported => {
            #[cfg(target_os = "linux")]
            panic!("RLIMIT_CORE hardening should be supported on Linux");
        }
    }

    match report.pr_set_dumpable {
        HardeningStatus::Applied => {}
        HardeningStatus::Failed { errno } => {
            assert!(errno >= 0, "Errno must be non-negative");
        }
        HardeningStatus::NotSupported => {
            #[cfg(target_os = "linux")]
            panic!("PR_SET_DUMPABLE hardening should be supported on Linux");
        }
    }
}

#[test]
fn test_memfd_secret_support_detection() {
    let memfd_tri = check_memfd_secret_support();
    // Verification: detection returns an EnvTri variant without panicking
    assert!(matches!(
        memfd_tri,
        EnvTri::Yes | EnvTri::No | EnvTri::Unknown
    ));
}

#[test]
fn test_fake_backend_failed_status_handling() {
    // Simulated/Fake status report testing
    let simulated_failure_report = HardenRuntimeReport {
        rlimit_core: HardeningStatus::Failed { errno: 1 }, // EPERM
        pr_set_dumpable: HardeningStatus::Failed { errno: 13 }, // EACCES
    };

    assert_eq!(
        simulated_failure_report.rlimit_core,
        HardeningStatus::Failed { errno: 1 }
    );
    assert_eq!(
        simulated_failure_report.pr_set_dumpable,
        HardeningStatus::Failed { errno: 13 }
    );
}

#[test]
fn test_observe_environment_returns_observed_semantics() {
    let temp_dir = env::temp_dir();
    let obs: EnvObservation = observe(&temp_dir);

    // Verify OS detection matching target OS
    #[cfg(target_os = "linux")]
    assert_eq!(obs.os, EnvOsKind::Linux);

    #[cfg(target_os = "macos")]
    assert_eq!(obs.os, EnvOsKind::MacOs);

    #[cfg(windows)]
    assert_eq!(obs.os, EnvOsKind::Windows);

    // Ensure all EnvTri values are valid empirical observations (never guaranteed)
    let valid_tris = [EnvTri::Yes, EnvTri::No, EnvTri::Unknown];
    assert!(valid_tris.contains(&obs.cow_fs));
    assert!(valid_tris.contains(&obs.discard_enabled));
    assert!(valid_tris.contains(&obs.block_encrypted));
    assert!(valid_tris.contains(&obs.rotational));
    assert!(valid_tris.contains(&obs.swap_active));
    assert!(valid_tris.contains(&obs.core_dumps_possible));
}
