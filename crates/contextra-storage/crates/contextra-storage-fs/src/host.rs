//! HostProbe adapter using safe OS inspections (SPEC §3.2, §9.9, E-78).
#![forbid(unsafe_code)]

use contextra_storage_ports::traits::{HostProbe, HostReport, HostToggle};
use std::path::{Path, PathBuf};

pub struct RealHostProbe {
    data_dir: PathBuf,
    ledger_dir: PathBuf,
}

impl RealHostProbe {
    pub fn new(data_dir: PathBuf, ledger_dir: PathBuf) -> Self {
        Self {
            data_dir,
            ledger_dir,
        }
    }
}

impl HostProbe for RealHostProbe {
    fn probe(&self) -> HostReport {
        HostReport {
            swap: probe_swap(),
            core_dumps: probe_core_dumps(),
            memlock_limit_bytes: probe_proc_limit("Max locked memory"),
            rlimit_nofile: probe_proc_limit("Max open files"),
            fs_kind: probe_fs_kind(&self.data_dir),
            data_device_id: device_id(&self.data_dir),
            ledger_device_id: device_id(&self.ledger_dir),
        }
    }
}

fn device_id(path: &Path) -> Option<u64> {
    #[cfg(unix)]
    {
        if let Ok(st) = rustix::fs::stat(path) {
            return Some(st.st_dev);
        }
        if let Some(parent) = path.parent() {
            if let Ok(st) = rustix::fs::stat(parent) {
                return Some(st.st_dev);
            }
        }
    }
    None
}

fn probe_swap() -> HostToggle {
    #[cfg(target_os = "linux")]
    {
        if let Ok(swaps) = std::fs::read_to_string("/proc/swaps") {
            let active = swaps.lines().skip(1).any(|line| !line.trim().is_empty());
            return if active {
                HostToggle::Enabled
            } else {
                HostToggle::Disabled
            };
        }
    }
    HostToggle::Unknown
}

fn probe_core_dumps() -> HostToggle {
    if let Some(soft) = probe_proc_limit("Max core file size") {
        return if soft == 0 {
            HostToggle::Disabled
        } else {
            HostToggle::Enabled
        };
    }
    HostToggle::Unknown
}

fn probe_proc_limit(label: &str) -> Option<u64> {
    #[cfg(target_os = "linux")]
    {
        let text = std::fs::read_to_string("/proc/self/limits").ok()?;
        for line in text.lines() {
            if let Some(rest) = line.strip_prefix(label) {
                let first_col = rest.split_whitespace().next()?;
                if first_col == "unlimited" {
                    return Some(u64::MAX);
                }
                if let Ok(val) = first_col.parse::<u64>() {
                    return Some(val);
                }
            }
        }
    }
    let _ = label;
    None
}

fn probe_fs_kind(path: &Path) -> &'static str {
    #[cfg(target_os = "linux")]
    {
        if let Ok(st) = rustix::fs::statfs(path) {
            return match st.f_type {
                0xef53 => "ext4",
                0x58465342 => "xfs",
                0x9123683e => "btrfs",
                0x01021994 => "tmpfs",
                _ => "linux-fs",
            };
        }
    }
    let _ = path;
    "unknown"
}
