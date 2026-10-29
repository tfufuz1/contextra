use contextra_types::{ContextraError, Result};
use reqwest::Url;
use serde::{Deserialize, Serialize};
use std::net::{IpAddr, Ipv4Addr};
use std::path::PathBuf;
use std::str::FromStr;

/// Trait for auditing remote egress access attempts.
pub trait EgressAuditSink: Send + Sync + std::fmt::Debug {
    /// Called when an egress check is performed for a remote (non-loopback) endpoint.
    fn audit_egress(&self, endpoint: &str, allowed: bool, reason: &str);
}

/// Default no-op implementation of `EgressAuditSink`.
#[derive(Debug, Default, Clone, Copy)]
pub struct NoopEgressAuditSink;

impl EgressAuditSink for NoopEgressAuditSink {
    fn audit_egress(&self, _endpoint: &str, _allowed: bool, _reason: &str) {}
}

/// Endpoint policy governing allowed egress destinations for Ollama requests.
#[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct EndpointPolicy {
    /// Whether non-loopback remote endpoints are permitted. Default: `false`.
    pub allow_remote: bool,
    /// Explicit allowlist of allowed remote hosts or host:port strings (e.g. `"ollama.internal:11434"`, `"192.168.1.50"`).
    pub allowed_remote_hosts: Vec<String>,
    /// Explicit list of allowed Unix domain socket paths.
    pub allowed_unix_sockets: Vec<PathBuf>,
}

impl EndpointPolicy {
    /// Creates a default loopback-only policy.
    pub fn loopback_only() -> Self {
        Self::default()
    }

    /// Checks whether the given endpoint URL is permitted under this policy.
    ///
    /// Loopback endpoints (`127.0.0.0/8`, `::1`, `localhost`) and permitted Unix sockets
    /// are allowed by default.
    /// Non-loopback destinations require `allow_remote = true` AND matching an entry in `allowed_remote_hosts`.
    /// Every remote usage triggers a call to `audit_sink.audit_egress`.
    pub fn check_endpoint(&self, endpoint: &str, audit_sink: &dyn EgressAuditSink) -> Result<()> {
        let url = Url::parse(endpoint).map_err(|e| {
            ContextraError::PolicyViolation(format!("Invalid endpoint URL '{endpoint}': {e}"))
        })?;

        // 1. Userinfo check (user:pass@...)
        if !url.username().is_empty() || url.password().is_some() {
            let reason = "Userinfo in endpoint URL is forbidden";
            audit_sink.audit_egress(endpoint, false, reason);
            return Err(ContextraError::PolicyViolation(reason.to_string()));
        }

        let scheme = url.scheme().to_lowercase();

        // 2. Unix socket check
        if scheme == "unix" {
            let socket_path = PathBuf::from(url.path());
            if self.allowed_unix_sockets.contains(&socket_path) {
                return Ok(());
            } else {
                let reason = "Unix socket path is not in allowed_unix_sockets";
                audit_sink.audit_egress(endpoint, false, reason);
                return Err(ContextraError::PolicyViolation(format!(
                    "Unix socket path '{}' is not allowed by policy",
                    socket_path.display()
                )));
            }
        }

        if scheme != "http" && scheme != "https" {
            let reason = "Unsupported scheme in endpoint URL";
            audit_sink.audit_egress(endpoint, false, reason);
            return Err(ContextraError::PolicyViolation(format!(
                "Unsupported scheme '{scheme}' in endpoint URL"
            )));
        }

        let host_str = match url.host_str() {
            Some(h) => h.to_lowercase(),
            None => {
                let reason = "Missing host in endpoint URL";
                audit_sink.audit_egress(endpoint, false, reason);
                return Err(ContextraError::PolicyViolation(reason.to_string()));
            }
        };

        let raw_ip_str =
            if host_str.starts_with('[') && host_str.ends_with(']') && host_str.len() >= 2 {
                host_str.get(1..host_str.len() - 1).unwrap_or(&host_str)
            } else {
                &host_str
            };

        let port = url.port_or_known_default().unwrap_or(80);
        let host_port = format!("{host_str}:{port}");

        // 3. Determine if host is loopback
        let is_loopback = if host_str == "localhost" {
            true
        } else if let Ok(ip) = IpAddr::from_str(raw_ip_str) {
            match ip {
                IpAddr::V4(v4) => v4.is_loopback(),
                IpAddr::V6(v6) => {
                    if v6.is_loopback() {
                        true
                    } else if let [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0xff, 0xff, a, b, c, d] =
                        v6.octets()
                    {
                        let mapped_v4 = Ipv4Addr::new(a, b, c, d);
                        mapped_v4.is_loopback()
                    } else {
                        false
                    }
                }
            }
        } else {
            false
        };

        if is_loopback {
            return Ok(());
        }

        // 4. Remote egress check
        if !self.allow_remote {
            let reason = "Remote egress rejected: allow_remote is false";
            audit_sink.audit_egress(endpoint, false, reason);
            return Err(ContextraError::PolicyViolation(format!(
                "Remote egress to host '{host_str}' rejected by policy (allow_remote = false)"
            )));
        }

        let matches_allowlist = self.allowed_remote_hosts.iter().any(|entry| {
            let entry_lower = entry.to_lowercase();
            entry_lower == host_str || entry_lower == host_port || entry_lower == raw_ip_str
        });

        if !matches_allowlist {
            let reason = "Remote host is not in allowed_remote_hosts allowlist";
            audit_sink.audit_egress(endpoint, false, reason);
            return Err(ContextraError::PolicyViolation(format!(
                "Remote host '{host_str}' (or '{host_port}') is not in allowed_remote_hosts allowlist"
            )));
        }

        // Approved remote egress
        audit_sink.audit_egress(endpoint, true, "Approved by allowlist");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;

    #[derive(Debug, Default)]
    struct MockAuditSink {
        calls: Arc<AtomicUsize>,
        last_allowed: Arc<parking_lot::Mutex<Option<bool>>>,
    }

    impl EgressAuditSink for MockAuditSink {
        fn audit_egress(&self, _endpoint: &str, allowed: bool, _reason: &str) {
            self.calls.fetch_add(1, Ordering::SeqCst);
            *self.last_allowed.lock() = Some(allowed);
        }
    }

    struct TestCase {
        endpoint: &'static str,
        allow_remote: bool,
        allowed_remote_hosts: Vec<&'static str>,
        allowed_unix_sockets: Vec<&'static str>,
        expect_ok: bool,
        expect_audit_call: bool,
    }

    #[test]
    fn test_endpoint_policy_table_driven() {
        let cases = vec![
            // Loopback cases (Default policy: allow_remote = false)
            TestCase {
                endpoint: "http://127.0.0.1:11434",
                allow_remote: false,
                allowed_remote_hosts: vec![],
                allowed_unix_sockets: vec![],
                expect_ok: true,
                expect_audit_call: false,
            },
            TestCase {
                endpoint: "http://127.0.0.2:11434",
                allow_remote: false,
                allowed_remote_hosts: vec![],
                allowed_unix_sockets: vec![],
                expect_ok: true,
                expect_audit_call: false,
            },
            TestCase {
                endpoint: "http://[::1]:11434",
                allow_remote: false,
                allowed_remote_hosts: vec![],
                allowed_unix_sockets: vec![],
                expect_ok: true,
                expect_audit_call: false,
            },
            TestCase {
                endpoint: "http://localhost:11434",
                allow_remote: false,
                allowed_remote_hosts: vec![],
                allowed_unix_sockets: vec![],
                expect_ok: true,
                expect_audit_call: false,
            },
            TestCase {
                endpoint: "http://[::ffff:127.0.0.1]:11434",
                allow_remote: false,
                allowed_remote_hosts: vec![],
                allowed_unix_sockets: vec![],
                expect_ok: true,
                expect_audit_call: false,
            },
            // Evasion / Suffix tricks (MUST fail & call audit)
            TestCase {
                endpoint: "http://127.0.0.1.evil.com:11434",
                allow_remote: false,
                allowed_remote_hosts: vec![],
                allowed_unix_sockets: vec![],
                expect_ok: false,
                expect_audit_call: true,
            },
            TestCase {
                endpoint: "http://localhost.evil.com:11434",
                allow_remote: false,
                allowed_remote_hosts: vec![],
                allowed_unix_sockets: vec![],
                expect_ok: false,
                expect_audit_call: true,
            },
            TestCase {
                endpoint: "http://evil.localhost:11434",
                allow_remote: false,
                allowed_remote_hosts: vec![],
                allowed_unix_sockets: vec![],
                expect_ok: false,
                expect_audit_call: true,
            },
            // Userinfo tricks (MUST fail & call audit)
            TestCase {
                endpoint: "http://admin:secret@127.0.0.1:11434",
                allow_remote: false,
                allowed_remote_hosts: vec![],
                allowed_unix_sockets: vec![],
                expect_ok: false,
                expect_audit_call: true,
            },
            TestCase {
                endpoint: "http://user@localhost:11434",
                allow_remote: false,
                allowed_remote_hosts: vec![],
                allowed_unix_sockets: vec![],
                expect_ok: false,
                expect_audit_call: true,
            },
            // IPv6 mapped non-loopback address
            TestCase {
                endpoint: "http://[::ffff:192.168.1.1]:11434",
                allow_remote: false,
                allowed_remote_hosts: vec![],
                allowed_unix_sockets: vec![],
                expect_ok: false,
                expect_audit_call: true,
            },
            // Remote hosts without allow_remote
            TestCase {
                endpoint: "http://192.168.1.100:11434",
                allow_remote: false,
                allowed_remote_hosts: vec!["192.168.1.100"],
                allowed_unix_sockets: vec![],
                expect_ok: false,
                expect_audit_call: true,
            },
            TestCase {
                endpoint: "http://ollama.internal:11434",
                allow_remote: false,
                allowed_remote_hosts: vec!["ollama.internal"],
                allowed_unix_sockets: vec![],
                expect_ok: false,
                expect_audit_call: true,
            },
            // Remote hosts WITH allow_remote and matching allowlist
            TestCase {
                endpoint: "http://ollama.internal:11434",
                allow_remote: true,
                allowed_remote_hosts: vec!["ollama.internal"],
                allowed_unix_sockets: vec![],
                expect_ok: true,
                expect_audit_call: true,
            },
            TestCase {
                endpoint: "http://192.168.1.50:11434",
                allow_remote: true,
                allowed_remote_hosts: vec!["192.168.1.50:11434"],
                allowed_unix_sockets: vec![],
                expect_ok: true,
                expect_audit_call: true,
            },
            // Remote host WITH allow_remote but NOT in allowlist
            TestCase {
                endpoint: "http://untrusted.server.com:11434",
                allow_remote: true,
                allowed_remote_hosts: vec!["ollama.internal"],
                allowed_unix_sockets: vec![],
                expect_ok: false,
                expect_audit_call: true,
            },
            // Unix socket allowed
            TestCase {
                endpoint: "unix:///var/run/ollama.sock",
                allow_remote: false,
                allowed_remote_hosts: vec![],
                allowed_unix_sockets: vec!["/var/run/ollama.sock"],
                expect_ok: true,
                expect_audit_call: false,
            },
            // Unix socket NOT allowed
            TestCase {
                endpoint: "unix:///tmp/evil.sock",
                allow_remote: false,
                allowed_remote_hosts: vec![],
                allowed_unix_sockets: vec!["/var/run/ollama.sock"],
                expect_ok: false,
                expect_audit_call: true,
            },
        ];

        for (idx, tc) in cases.into_iter().enumerate() {
            let policy = EndpointPolicy {
                allow_remote: tc.allow_remote,
                allowed_remote_hosts: tc
                    .allowed_remote_hosts
                    .iter()
                    .map(|s| s.to_string())
                    .collect(),
                allowed_unix_sockets: tc.allowed_unix_sockets.iter().map(PathBuf::from).collect(),
            };
            let audit = MockAuditSink::default();
            let res = policy.check_endpoint(tc.endpoint, &audit);

            assert_eq!(
                res.is_ok(),
                tc.expect_ok,
                "Case #{idx} failed for endpoint '{}': expected_ok={}, got {:?}",
                tc.endpoint,
                tc.expect_ok,
                res
            );

            assert_eq!(
                audit.calls.load(Ordering::SeqCst) > 0,
                tc.expect_audit_call,
                "Case #{idx} failed for endpoint '{}': expected_audit_call={}, got_calls={}",
                tc.endpoint,
                tc.expect_audit_call,
                audit.calls.load(Ordering::SeqCst)
            );
        }
    }
}
