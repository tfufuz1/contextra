//! Optional HostProbe port (SPEC §3.2, §9.9, E-78).
//!
//! Adapter-only host diagnostics. The storage core never relies on HostProbe
//! for correctness; `Unknown` values are surfaced as `Declared` residual risk.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HostToggle {
    Disabled,
    Enabled,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HostReport {
    pub swap: HostToggle,
    pub core_dumps: HostToggle,
    pub memlock_limit_bytes: Option<u64>,
    pub rlimit_nofile: Option<u64>,
    pub fs_kind: &'static str,
    pub data_device_id: Option<u64>,
    pub ledger_device_id: Option<u64>,
}

impl Default for HostReport {
    fn default() -> Self {
        Self {
            swap: HostToggle::Unknown,
            core_dumps: HostToggle::Unknown,
            memlock_limit_bytes: None,
            rlimit_nofile: None,
            fs_kind: "unknown",
            data_device_id: None,
            ledger_device_id: None,
        }
    }
}

impl HostReport {
    pub fn has_unverified_host_exposure(&self) -> bool {
        self.swap != HostToggle::Disabled
            || self.core_dumps != HostToggle::Disabled
            || self.memlock_limit_bytes.is_none()
    }
}

pub trait HostProbe: Send + Sync {
    fn probe(&self) -> HostReport;
}
