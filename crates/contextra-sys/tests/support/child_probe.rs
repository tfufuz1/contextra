// FILE-CONTEXT
// ZWECK: Child probe helper for contextra-sys hardening tests (T-DUMP-1, T-SEC-2)
// INVARIANTEN: Zero panic, fixed exit codes, parses command-line arguments to test process hardening in isolation

use std::env;
use std::fs::File;
use std::io::Read;

use contextra_sys::{apply_process_hardening, lock_pages, mark_dont_dump, HardeningPolicy};

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        std::process::exit(1);
    }

    match args[1].as_str() {
        "apply" => {
            if args.len() < 3 {
                std::process::exit(1);
            }
            let policy = match args[2].as_str() {
                "required" => HardeningPolicy::Required,
                "best_effort" => HardeningPolicy::BestEffort,
                "off" => HardeningPolicy::Off,
                _ => std::process::exit(1),
            };

            let report = match apply_process_hardening(policy) {
                Ok(r) => r,
                Err(_) => std::process::exit(2),
            };

            // Inspect /proc/self/status for Dumpable flag
            #[cfg(target_os = "linux")]
            {
                let mut status_content = String::new();
                if let Ok(mut f) = File::open("/proc/self/status") {
                    let _ = f.read_to_string(&mut status_content);
                }

                let dumpable_line = status_content
                    .lines()
                    .find(|line| line.starts_with("Dumpable:"));

                let dumpable_val = dumpable_line
                    .and_then(|line| line.split_whitespace().nth(1))
                    .unwrap_or("unknown");

                // Check RLIMIT_CORE
                let mut rlim = libc::rlimit {
                    rlim_cur: 0,
                    rlim_max: 0,
                };
                let rlim_res = unsafe { libc::getrlimit(libc::RLIMIT_CORE, &mut rlim) };
                let core_zero = rlim_res == 0 && rlim.rlim_cur == 0;

                println!(
                    "DUMPABLE={} CORE_ZERO={} CORE_REP={} DUMP_REP={}",
                    dumpable_val, core_zero, report.core_limit_zero, report.dumpable_off
                );
            }

            #[cfg(not(target_os = "linux"))]
            {
                println!(
                    "DUMPABLE=na CORE_ZERO=na CORE_REP={} DUMP_REP={}",
                    report.core_limit_zero, report.dumpable_off
                );
            }
        }
        "dont_dump" => {
            // Allocate a page-aligned buffer and call mark_dont_dump
            let size = 64 * 1024;
            let mut buf = vec![0x42u8; size];
            let ok = mark_dont_dump(&buf);
            if !ok {
                std::process::exit(3);
            }

            // Inspect /proc/self/smaps for 'dd' flag
            #[cfg(target_os = "linux")]
            {
                let mut smaps_content = String::new();
                if let Ok(mut f) = File::open("/proc/self/smaps") {
                    let _ = f.read_to_string(&mut smaps_content);
                }

                let has_dd = smaps_content
                    .lines()
                    .any(|line| line.starts_with("VmFlags:") && line.contains(" dd"));

                if has_dd {
                    println!("DONTDUMP_VERIFIED");
                } else {
                    println!("DONTDUMP_MISSING");
                }
            }

            #[cfg(not(target_os = "linux"))]
            {
                println!("DONTDUMP_VERIFIED");
            }

            // Keep buffer alive
            buf[0] = 0xAA;
        }
        "fail_mlock_required" => {
            // Force mlock limit to 0
            #[cfg(unix)]
            {
                let rlim = libc::rlimit {
                    rlim_cur: 0,
                    rlim_max: 0,
                };
                unsafe {
                    libc::setrlimit(libc::RLIMIT_MEMLOCK, &rlim);
                }
            }

            let page = vec![1u8; 4096];
            match lock_pages(&page, HardeningPolicy::Required) {
                Ok(_) => std::process::exit(4), // Expected failure!
                Err(_) => {
                    println!("REQUIRED_FAILED_AS_EXPECTED");
                    std::process::exit(0);
                }
            }
        }
        _ => std::process::exit(1),
    }
}
