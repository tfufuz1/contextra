// FILE-CONTEXT
// STAND: 2026-10-06T00:00:00Z (SESSION: p05-b-wasm-module-provenance)
// ZWECK: Capability Whitelist, Configuration & Module Provenance Policy for WASM Execution Boundary
// INVARIANTEN: allow_cloud_egress defaults to false (Least Privilege); ModulePolicy defaults to Unrestricted
// NICHT-OFFENSICHTLICH: Memory pages are capped at 16 pages (1MB) by default to avoid OOM; SHA-256 is implemented in pure safe Rust to avoid new external dependencies
// SIEHE AUCH: AGENTS.md §4.18 WASM Execution Boundary, P05 / F-02 HIGH

//! WasmCapabilities — Capability-Whitelist & Provenance Policy für WASM-Guest-Module (§4.18).

use crate::error::SandboxError;

/// WASM Module Provenance / Authorization Policy (§4.18, P05/F-02).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum ModulePolicy {
    /// Unrestricted execution (default for generic backward compatibility).
    #[default]
    Unrestricted,
    /// Allow execution only if the SHA-256 hash of the WASM binary matches an entry in the allowlist.
    HashAllowlist(Vec<[u8; 32]>),
    /// Allow execution if signed by a trusted Ed25519 public key (requires ed25519-dalek dependency).
    SignedBy(Vec<[u8; 32]>),
}

/// Constant-time comparison of two 32-byte arrays to prevent timing side-channels.
pub fn constant_time_eq_32(a: &[u8; 32], b: &[u8; 32]) -> bool {
    let mut res = 0u8;
    for i in 0..32 {
        res |= a[i] ^ b[i];
    }
    res == 0
}

/// Pure safe-Rust FIPS 180-4 SHA-256 implementation.
fn sha256_digest(data: &[u8]) -> [u8; 32] {
    let mut h: [u32; 8] = [
        0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a,
        0x510e527f, 0x9b05688c, 0x1f83d9ab, 0x5be0cd19,
    ];

    const K: [u32; 64] = [
        0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4, 0xab1c5ed5,
        0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe, 0x9bdc06a7, 0xc19bf174,
        0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f, 0x4a7484aa, 0x5cb0a9dc, 0x76f988da,
        0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7, 0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967,
        0x27b70a85, 0x2e1b2138, 0x4d2c6dfc, 0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85,
        0xa2bfe8a1, 0xa81a664b, 0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070,
        0x19a4c116, 0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
        0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7, 0xc67178f2,
    ];

    let bit_len = (data.len() as u64).wrapping_mul(8);
    let mut padded = data.to_vec();
    padded.push(0x80);
    while (padded.len() % 64) != 56 {
        padded.push(0x00);
    }
    padded.extend_from_slice(&bit_len.to_be_bytes());

    for chunk in padded.chunks_exact(64) {
        let mut w = [0u32; 64];
        for i in 0..16 {
            w[i] = u32::from_be_bytes([
                chunk[i * 4],
                chunk[i * 4 + 1],
                chunk[i * 4 + 2],
                chunk[i * 4 + 3],
            ]);
        }
        for i in 16..64 {
            let s0 = w[i - 15].rotate_right(7) ^ w[i - 15].rotate_right(18) ^ (w[i - 15] >> 3);
            let s1 = w[i - 2].rotate_right(17) ^ w[i - 2].rotate_right(19) ^ (w[i - 2] >> 10);
            w[i] = w[i - 16]
                .wrapping_add(s0)
                .wrapping_add(w[i - 7])
                .wrapping_add(s1);
        }

        let mut a = h[0];
        let mut b = h[1];
        let mut c = h[2];
        let mut d = h[3];
        let mut e = h[4];
        let mut f = h[5];
        let mut g = h[6];
        let mut h_var = h[7];

        for i in 0..64 {
            let s1 = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
            let ch = (e & f) ^ ((!e) & g);
            let temp1 = h_var
                .wrapping_add(s1)
                .wrapping_add(ch)
                .wrapping_add(K[i])
                .wrapping_add(w[i]);
            let s0 = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
            let maj = (a & b) ^ (a & c) ^ (b & c);
            let temp2 = s0.wrapping_add(maj);

            h_var = g;
            g = f;
            f = e;
            e = d.wrapping_add(temp1);
            d = c;
            c = b;
            b = a;
            a = temp1.wrapping_add(temp2);
        }

        h[0] = h[0].wrapping_add(a);
        h[1] = h[1].wrapping_add(b);
        h[2] = h[2].wrapping_add(c);
        h[3] = h[3].wrapping_add(d);
        h[4] = h[4].wrapping_add(e);
        h[5] = h[5].wrapping_add(f);
        h[6] = h[6].wrapping_add(g);
        h[7] = h[7].wrapping_add(h_var);
    }

    let mut out = [0u8; 32];
    for i in 0..8 {
        out[i * 4..i * 4 + 4].copy_from_slice(&h[i].to_be_bytes());
    }
    out
}

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
    /// Max. WASM-Modul-Größe in Bytes (INV-SBX-1). Default: 10 MB (10_485_760).
    pub max_module_size_bytes: usize,
    /// Max. WASM-Tabellen-Einträge (INV-SBX-2). Default: 10_000.
    pub max_table_entries: u32,
    /// Max. Output-Größe in Bytes je Stream (stdout/stderr). Default: 1 MB (1_048_576).
    pub max_output_bytes: usize,
    /// Max. Stdin-Input-Größe in Bytes. Default: 1 MB (1_048_576).
    pub max_stdin_bytes: usize,
    /// Seed für deterministisches `random_get`. Default: `None` (`random_get` liefert `NOSYS`).
    pub random_seed: Option<u64>,
    /// WASM Modul-Richtlinie zur Herkunfts- und Integritätsprüfung (P05/F-02).
    pub module_policy: ModulePolicy,
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
            max_module_size_bytes: 10 * 1024 * 1024, // 10 MB
            max_table_entries: 10_000,
            max_output_bytes: 1024 * 1024, // 1 MB
            max_stdin_bytes: 1024 * 1024,  // 1 MB
            random_seed: None,
            module_policy: ModulePolicy::Unrestricted,
        }
    }
}

impl WasmCapabilities {
    /// Computes the SHA-256 digest of a WASM binary.
    pub fn compute_sha256(wasm_bytes: &[u8]) -> [u8; 32] {
        sha256_digest(wasm_bytes)
    }

    /// Verifies that a WASM binary complies with the configured [`ModulePolicy`].
    ///
    /// # Errors
    /// Returns `SandboxError::InvalidModule` if the module fails authorization.
    pub fn verify_module_policy(&self, wasm_bytes: &[u8]) -> Result<(), SandboxError> {
        match &self.module_policy {
            ModulePolicy::Unrestricted => Ok(()),
            ModulePolicy::HashAllowlist(allowed_hashes) => {
                let computed = sha256_digest(wasm_bytes);
                for allowed in allowed_hashes {
                    if constant_time_eq_32(&computed, allowed) {
                        return Ok(());
                    }
                }
                Err(SandboxError::InvalidModule(
                    "WASM module binary provenance check failed: binary hash is not present in the allowed module policy list".into(),
                ))
            }
            ModulePolicy::SignedBy(_trusted_keys) => Err(SandboxError::InvalidModule(
                "WASM module provenance check failed: SignedBy policy requires ed25519 signature verification (ed25519 dependency not compiled in crate)".into(),
            )),
        }
    }

    /// Returns a strict `WasmCapabilities` preset for pure merge operators.
    ///
    /// # Security Profile
    /// - `allow_stdout`: `false`
    /// - `allow_stderr`: `false`
    /// - `allow_filesystem`: `false`
    /// - `allow_network`: `false`
    /// - `allow_clock`: `false` (no `clock_time_get` WASI access)
    /// - `random_seed`: `None` (no PRNG access)
    /// - `allow_cloud_egress`: `false`
    /// - `max_memory_pages`: `16` (1 MB)
    /// - `max_fuel`: `10_000_000`
    /// - `max_wall_clock_ms`: `5_000`
    pub fn pure_merge_operator() -> Self {
        Self {
            allow_stdout: false,
            allow_stderr: false,
            max_memory_pages: 16, // 1 MB
            max_fuel: 10_000_000,
            allow_filesystem: false,
            allow_network: false,
            allow_clock: false,
            allow_cloud_egress: false,
            max_wall_clock_ms: 5_000,
            max_module_size_bytes: 10 * 1024 * 1024,
            max_table_entries: 10_000,
            max_output_bytes: 1024 * 1024,
            max_stdin_bytes: 1024 * 1024,
            random_seed: None,
            module_policy: ModulePolicy::Unrestricted,
        }
    }
}

/// Dedicated pure capabilities preset for I/O-free WASM Merge Operators (§4.18).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MergeOperatorCapabilities;

impl MergeOperatorCapabilities {
    /// Returns a strict `WasmCapabilities` preset for pure merge operators.
    pub fn pure() -> WasmCapabilities {
        WasmCapabilities::pure_merge_operator()
    }

    /// Returns a strict `WasmCapabilities` preset for merge operators with stdout enabled for returning output bytes.
    ///
    /// Differs from `pure()` strictly by `allow_stdout = true`. All other restrictions (clock, PRNG, FS, network, cloud)
    /// remain disabled.
    pub fn pure_with_result_channel() -> WasmCapabilities {
        let mut caps = Self::pure();
        caps.allow_stdout = true;
        caps
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_merge_operator_capabilities_pure_with_result_channel() {
        let caps = MergeOperatorCapabilities::pure_with_result_channel();
        let pure_caps = MergeOperatorCapabilities::pure();

        assert!(
            caps.allow_stdout,
            "stdout must be enabled for result channel"
        );
        assert!(
            !caps.allow_stderr,
            "stderr must be disabled for pure_with_result_channel"
        );
        assert!(!caps.allow_filesystem, "filesystem must be disabled");
        assert!(!caps.allow_network, "network must be disabled");
        assert!(!caps.allow_clock, "clock access must be disabled");
        assert!(!caps.allow_cloud_egress, "cloud egress must be disabled");
        assert!(caps.random_seed.is_none(), "random seed must be None");
        assert_eq!(caps.max_fuel, pure_caps.max_fuel);
        assert_eq!(caps.max_wall_clock_ms, pure_caps.max_wall_clock_ms);
        assert_eq!(caps.max_memory_pages, pure_caps.max_memory_pages);
    }

    #[test]
    fn test_merge_operator_capabilities_pure_strictness() {
        let caps = MergeOperatorCapabilities::pure();
        assert!(
            !caps.allow_stdout,
            "stdout must be disabled for pure merge operators"
        );
        assert!(
            !caps.allow_stderr,
            "stderr must be disabled for pure merge operators"
        );
        assert!(!caps.allow_filesystem, "filesystem must be disabled");
        assert!(!caps.allow_network, "network must be disabled");
        assert!(!caps.allow_clock, "clock access must be disabled");
        assert!(!caps.allow_cloud_egress, "cloud egress must be disabled");
        assert!(caps.random_seed.is_none(), "random seed must be None");
        assert_eq!(caps.max_fuel, 10_000_000);
        assert_eq!(caps.max_wall_clock_ms, 5_000);
    }

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
