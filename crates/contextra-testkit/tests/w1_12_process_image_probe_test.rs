// FILE-CONTEXT
// STAND: 2026-10-22
// ZWECK: Integration test for ProcessImageProbe (/proc/self/mem heap scanning).
// INVARIANTEN: Zero-Panic, pure std (no unsafe), explicit unsupported on non-Linux.

use contextra_testkit::process_image_probe::{ProbeError, ProcessImageProbe};

#[test]
fn process_image_probe_finds_heap_marker() {
    let marker_vec = vec![0xDE, 0xAD, 0xBE, 0xEF, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66];
    let marker = marker_vec.as_slice();

    match ProcessImageProbe::scan_self(marker) {
        Ok(report) => {
            #[cfg(target_os = "linux")]
            {
                assert!(
                    report.hits_count > 0,
                    "ProcessImageProbe failed to find heap marker on Linux (hits_count == 0)"
                );
                assert!(report.regions_scanned > 0, "No memory regions were scanned");
            }
            #[cfg(not(target_os = "linux"))]
            {
                panic!("ProcessImageProbe should return Err(ProbeError::Unsupported) on non-Linux");
            }
        }
        Err(ProbeError::Unsupported) => {
            #[cfg(not(target_os = "linux"))]
            {
                // Expected on non-Linux
            }
            #[cfg(target_os = "linux")]
            {
                panic!("ProcessImageProbe returned Unsupported on Linux");
            }
        }
        Err(e) => {
            panic!("Unexpected error running ProcessImageProbe: {:?}", e);
        }
    }

    // Retain marker in scope so it isn't dropped before probe finishes
    let _ = marker_vec;
}
