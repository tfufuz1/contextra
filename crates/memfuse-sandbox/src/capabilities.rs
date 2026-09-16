// FILE-CONTEXT
// STAND: 2026-09-15T16:00:00Z (SESSION: acf8fe72)
// ZWECK: Capability Whitelist & Configuration for WASM Execution Boundary
// INVARIANTEN: allow_cloud_egress defaults to false (Least Privilege)
// NICHT-OFFENSICHTLICH: Memory pages are capped at 16 pages (1MB) by default to avoid OOM
// SIEHE AUCH: AGENTS.md §4.18 WASM Execution Boundary

//! WasmCapabilities — Capability-Whitelist für WASM-Guest-Module (§4.18).

/// Whitelist für WASM-Guest-Capabilities.
///
/// # Sicherheitsmodell
/// Kein Dateisystem- und Netzwerkzugriff per Default.
/// Monotone Uhr ist erlaubt (deterministisch, kein Side-Channel).
#[derive(Debug, Clone)]
pub struct WasmCapabilities {
    /// Stdout-Ausgabe erlaubt. Default: true.
    pub allow_stdout: bool,
    /// Stderr-Ausgabe erlaubt. Default: false (kein Logging-Leak).
    pub allow_stderr: bool,
    /// Max. WASM-Memory-Pages (1 Page = 64 KB). Default: 16 = 1 MB.
    pub max_memory_pages: u32,
    /// Max. Fuel (CPU-Ticks). Default: 10_000_000.
    pub max_fuel: u64,
    /// Dateisystemzugriff. Default: false.
    pub allow_filesystem: bool,
    /// Netzwerkzugriff. Default: false.
    pub allow_network: bool,
    /// Monotone Uhr (WASI clock_time_get). Default: true.
    pub allow_clock: bool,
    /// Cloud-Egress-Zugriff (dedizierte Cloud-Query-Calls). Default: false.
    pub allow_cloud_egress: bool,
    /// Max. Wall-Clock-Timeout in Millisekunden. Default: 5_000 (5s).
    /// `0` bedeutet unbegrenztes Wall-Clock-Time-Limit (gefördert durch max_fuel / caller timeout).
    /// Orthogonal zu `max_fuel` (CPU-Limit vs. Wall-Clock-Limit, beide unabhängig zu setzen).
    pub max_wall_clock_ms: u64,
}

impl Default for WasmCapabilities {
    fn default() -> Self {
        Self {
            allow_stdout: true,
            allow_stderr: false,
            max_memory_pages: 16, // 1 MB
            max_fuel: 10_000_000,
            allow_filesystem: false,
            allow_network: false,
            allow_clock: true,
            allow_cloud_egress: false,
            max_wall_clock_ms: 5_000,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wasm_capabilities_default_cloud_egress_is_false() {
        let caps = WasmCapabilities::default();
        assert!(
            !caps.allow_cloud_egress,
            "allow_cloud_egress MUST default to false (Least Privilege)"
        );
        assert_eq!(caps.max_wall_clock_ms, 5_000);
    }
}
