use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use std::process::Command;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemMetadata {
    pub commit_hash: String,
    pub cpu_model: String,
    pub kernel_version: String,
    pub os_name: String,
    pub filesystem: String,
    pub mount_options: String,
}

impl SystemMetadata {
    pub fn collect_for_path(path: &Path) -> Self {
        let commit_hash = Command::new("git")
            .args(["rev-parse", "HEAD"])
            .output()
            .ok()
            .and_then(|o| String::from_utf8(o.stdout).ok())
            .map(|s| s.trim().to_string())
            .unwrap_or_else(|| "unknown".to_string());

        let cpu_model = collect_cpu_model();
        let kernel_version = collect_kernel_version();
        let os_name = collect_os_name();
        let (filesystem, mount_options) = collect_fs_info(path);

        Self {
            commit_hash,
            cpu_model,
            kernel_version,
            os_name,
            filesystem,
            mount_options,
        }
    }
}

fn collect_cpu_model() -> String {
    if let Ok(cpuinfo) = fs::read_to_string("/proc/cpuinfo") {
        for line in cpuinfo.lines() {
            if line.starts_with("model name") {
                if let Some((_, model)) = line.split_once(':') {
                    return model.trim().to_string();
                }
            }
        }
    }

    Command::new("sysctl")
        .arg("-n")
        .arg("machdep.cpu.brand_string")
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "Unknown CPU".to_string())
}

fn collect_kernel_version() -> String {
    Command::new("uname")
        .arg("-sr")
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| "Unknown Kernel".to_string())
}

fn collect_os_name() -> String {
    if let Ok(os_release) = fs::read_to_string("/etc/os-release") {
        for line in os_release.lines() {
            if line.starts_with("PRETTY_NAME=") {
                let name = line.trim_start_matches("PRETTY_NAME=").trim_matches('"');
                return name.to_string();
            }
        }
    }
    "Unknown OS".to_string()
}

fn collect_fs_info(path: &Path) -> (String, String) {
    let canonical = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
    let mounts = fs::read_to_string("/proc/mounts").unwrap_or_default();

    let mut best_match: Option<(&str, &str, &str)> = None;

    for line in mounts.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 4 {
            let mount_point = parts[1];
            let fs_type = parts[2];
            let options = parts[3];

            if canonical.starts_with(mount_point) {
                if let Some((best_mp, _, _)) = best_match {
                    if mount_point.len() > best_mp.len() {
                        best_match = Some((mount_point, fs_type, options));
                    }
                } else {
                    best_match = Some((mount_point, fs_type, options));
                }
            }
        }
    }

    if let Some((_, fs_type, options)) = best_match {
        (fs_type.to_string(), options.to_string())
    } else {
        ("unknown".to_string(), "unknown".to_string())
    }
}

pub fn get_peak_rss_bytes() -> u64 {
    unsafe {
        let mut rusage: libc::rusage = std::mem::zeroed();
        if libc::getrusage(libc::RUSAGE_SELF, &mut rusage) == 0 {
            // Linux returns maxrss in KiB
            #[cfg(target_os = "linux")]
            return (rusage.ru_maxrss as u64) * 1024;

            #[cfg(not(target_os = "linux"))]
            return rusage.ru_maxrss as u64;
        }
    }
    0
}

pub fn get_open_fd_count() -> usize {
    if let Ok(entries) = fs::read_dir("/proc/self/fd") {
        entries.count()
    } else {
        0
    }
}

pub fn get_dir_size_bytes(path: &Path) -> u64 {
    let mut total = 0;
    if path.is_dir() {
        if let Ok(entries) = fs::read_dir(path) {
            for entry in entries.flatten() {
                let p = entry.path();
                if p.is_dir() {
                    total += get_dir_size_bytes(&p);
                } else if let Ok(meta) = entry.metadata() {
                    total += meta.len();
                }
            }
        }
    } else if let Ok(meta) = fs::metadata(path) {
        total += meta.len();
    }
    total
}
